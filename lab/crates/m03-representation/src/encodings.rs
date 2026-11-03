//! Lightweight encodings, written plainly so the article can show them: run-length encoding,
//! bit-packing, frame of reference and delta. Real formats (Parquet's RLE/bit-packing hybrid and
//! DELTA_BINARY_PACKED, for example) add block structure and headers, but the ideas are these.

/// Bits needed to store `v`: 0 for 0, otherwise the position of the highest set bit plus one.
pub fn bit_width(v: u64) -> u32 {
    64 - v.leading_zeros()
}

/// Bit-packed unsigned integers: each value takes exactly `width` bits, back to back.
pub struct BitPacked {
    pub width: u32,
    pub len: usize,
    pub words: Vec<u64>,
}

impl BitPacked {
    pub fn pack(values: &[u64]) -> BitPacked {
        let width = values.iter().map(|&v| bit_width(v)).max().unwrap_or(0);
        let mut words = vec![0u64; (values.len() * width as usize).div_ceil(64) + 1];
        if width > 0 {
            for (i, &v) in values.iter().enumerate() {
                let bit = i * width as usize;
                let (w, o) = (bit / 64, bit % 64);
                words[w] |= v << o;
                if o + width as usize > 64 {
                    words[w + 1] |= v >> (64 - o);
                }
            }
        }
        BitPacked {
            width,
            len: values.len(),
            words,
        }
    }

    pub fn unpack_into(&self, out: &mut [u64]) {
        let w = self.width as usize;
        if w == 0 {
            out.fill(0);
            return;
        }
        let mask = if w == 64 { u64::MAX } else { (1u64 << w) - 1 };
        for (i, slot) in out.iter_mut().enumerate().take(self.len) {
            let bit = i * w;
            let (k, o) = (bit / 64, bit % 64);
            // Read two words as one 128-bit window, so values that straddle a boundary need no branch.
            let window = (self.words[k] as u128) | ((self.words[k + 1] as u128) << 64);
            *slot = ((window >> o) as u64) & mask;
        }
    }

    pub fn bytes(&self) -> usize {
        (self.len * self.width as usize).div_ceil(8)
    }
}

/// Frame of reference: subtract the minimum, then bit-pack what's left.
pub struct ForPacked {
    pub min: i64,
    pub packed: BitPacked,
}

impl ForPacked {
    pub fn encode(values: &[i64]) -> ForPacked {
        let min = values.iter().copied().min().unwrap_or(0);
        let shifted: Vec<u64> = values.iter().map(|&v| v.wrapping_sub(min) as u64).collect();
        ForPacked {
            min,
            packed: BitPacked::pack(&shifted),
        }
    }

    pub fn decode_into(&self, out: &mut [i64]) {
        let tmp: &mut [u64] = bytemuck_i64_as_u64(out);
        self.packed.unpack_into(tmp);
        for v in out.iter_mut() {
            *v = v.wrapping_add(self.min);
        }
    }

    pub fn bytes(&self) -> usize {
        8 + self.packed.bytes()
    }
}

/// Frame of reference applied per block of `BLOCK` values, each block with its own minimum and
/// bit width, so a few outliers only widen their own block.
pub struct BlockFor {
    pub blocks: Vec<ForPacked>,
}

pub const BLOCK: usize = 1024;

impl BlockFor {
    pub fn encode(values: &[i64]) -> BlockFor {
        BlockFor {
            blocks: values.chunks(BLOCK).map(ForPacked::encode).collect(),
        }
    }

    pub fn decode_into(&self, out: &mut [i64]) {
        for (b, chunk) in self.blocks.iter().zip(out.chunks_mut(BLOCK)) {
            b.decode_into(chunk);
        }
    }

    pub fn bytes(&self) -> usize {
        // Per block: an 8-byte minimum and a 1-byte width, then the packed values.
        self.blocks.iter().map(|b| 9 + b.packed.bytes()).sum()
    }
}

/// Delta encoding: keep the first value, then store differences, frame-of-reference packed in
/// blocks (the shape of Parquet's DELTA_BINARY_PACKED, minus its miniblocks and headers).
pub struct Delta {
    pub first: i64,
    pub deltas: BlockFor,
}

impl Delta {
    pub fn encode(values: &[i64]) -> Delta {
        let first = values.first().copied().unwrap_or(0);
        let mut d = Vec::with_capacity(values.len());
        let mut prev = first;
        for &v in values {
            d.push(v.wrapping_sub(prev));
            prev = v;
        }
        Delta {
            first,
            deltas: BlockFor::encode(&d),
        }
    }

    pub fn decode_into(&self, out: &mut [i64]) {
        self.deltas.decode_into(out);
        let mut acc = self.first;
        for v in out.iter_mut() {
            acc = acc.wrapping_add(*v);
            *v = acc;
        }
    }

    pub fn bytes(&self) -> usize {
        8 + self.deltas.bytes()
    }
}

/// Run-length encoding: (value, run length) pairs.
pub struct Rle {
    pub runs: Vec<(i64, u32)>,
}

impl Rle {
    pub fn encode(values: &[i64]) -> Rle {
        let mut runs: Vec<(i64, u32)> = Vec::new();
        for &v in values {
            match runs.last_mut() {
                Some((x, n)) if *x == v && *n < u32::MAX => *n += 1,
                _ => runs.push((v, 1)),
            }
        }
        Rle { runs }
    }

    pub fn decode_into(&self, out: &mut [i64]) {
        let mut i = 0;
        for &(v, n) in &self.runs {
            out[i..i + n as usize].fill(v);
            i += n as usize;
        }
    }

    pub fn bytes(&self) -> usize {
        self.runs.len() * 12
    }
}

fn bytemuck_i64_as_u64(s: &mut [i64]) -> &mut [u64] {
    // SAFETY: i64 and u64 have the same size, alignment and validity (every bit pattern is valid).
    unsafe { std::slice::from_raw_parts_mut(s.as_mut_ptr() as *mut u64, s.len()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(v: &[i64]) {
        let mut out = vec![0i64; v.len()];
        ForPacked::encode(v).decode_into(&mut out);
        assert_eq!(out, v);
        BlockFor::encode(v).decode_into(&mut out);
        assert_eq!(out, v);
        Delta::encode(v).decode_into(&mut out);
        assert_eq!(out, v);
        Rle::encode(v).decode_into(&mut out);
        assert_eq!(out, v);
    }

    #[test]
    fn encodings_roundtrip() {
        roundtrip(&[5, 5, 5, 9, 9, 1, 1, 1, 1]);
        roundtrip(&(0..5000).map(|i| i * 37 % 1001 - 500).collect::<Vec<_>>());
        roundtrip(&[i64::MIN, i64::MAX, 0, -1, 1]);
        roundtrip(
            &(0..3000)
                .map(|i| 1_700_000_000_000 + i * 3)
                .collect::<Vec<_>>(),
        );
    }

    #[test]
    fn bit_widths() {
        assert_eq!(bit_width(0), 0);
        assert_eq!(bit_width(1), 1);
        assert_eq!(bit_width(100), 7);
        assert_eq!(bit_width(u64::MAX), 64);
    }
}
