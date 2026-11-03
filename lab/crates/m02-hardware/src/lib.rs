//! Module 02 lab: small kernels that expose the machine.
//!
//! Each function does one memory or branch pattern and nothing else, so its running time is a
//! measurement of the hardware rather than of the code. The examples time them over a range of
//! sizes; the tests check that they compute the right answer.

use labkit::rng::Rng;
use std::hint::black_box;

/// Bytes per cache line on every x86-64 and most ARM server CPUs.
pub const LINE: usize = 64;

/// One cache line holding the index of the next line to visit. Padding keeps each node on its
/// own line, so every hop of a chase touches a different line.
#[repr(C, align(64))]
#[derive(Clone, Copy)]
pub struct Node {
    pub next: usize,
    _pad: [usize; 7],
}

/// Builds `n` nodes linked into a single random cycle (Sattolo's algorithm), so a chase visits
/// every node exactly once per lap, in an order the prefetcher cannot predict.
pub fn random_cycle(n: usize, seed: u64) -> Vec<Node> {
    let mut nodes = vec![
        Node {
            next: 0,
            _pad: [0; 7]
        };
        n
    ];
    link_random_cycle(&mut nodes, seed);
    nodes
}

/// Links existing nodes into a single random cycle, in place. Lets the caller choose where the
/// memory comes from, for example a region backed by huge pages.
pub fn link_random_cycle(nodes: &mut [Node], seed: u64) {
    let n = nodes.len();
    let mut order: Vec<usize> = (0..n).collect();
    let mut rng = Rng::new(seed);
    for i in (1..n).rev() {
        let j = rng.below(i as u64) as usize; // j < i: a single cycle
        order.swap(i, j);
    }
    for i in 0..n {
        nodes[order[i]].next = order[(i + 1) % n];
    }
}

/// Anonymous memory for `n` nodes, aligned to 2 MiB and advised to use transparent huge pages.
/// The kernel may still decline; [`anon_huge_kib`] reports how much it actually backed.
pub struct HugeNodes {
    base: *mut libc::c_void,
    len: usize,
    nodes: *mut Node,
    n: usize,
}

impl HugeNodes {
    pub const HUGE: usize = 2 << 20;

    pub fn new(n: usize) -> HugeNodes {
        let bytes = (n * LINE).div_ceil(Self::HUGE) * Self::HUGE;
        let len = bytes + Self::HUGE;
        // SAFETY: a fresh private anonymous mapping; checked for failure below.
        let base = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        };
        assert_ne!(base, libc::MAP_FAILED, "mmap failed");
        let aligned = (base as usize).div_ceil(Self::HUGE) * Self::HUGE;
        // SAFETY: [aligned, aligned + bytes) lies inside the mapping.
        let rc = unsafe { libc::madvise(aligned as *mut libc::c_void, bytes, libc::MADV_HUGEPAGE) };
        assert_eq!(rc, 0, "madvise(MADV_HUGEPAGE) failed");
        HugeNodes {
            base,
            len,
            nodes: aligned as *mut Node,
            n,
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [Node] {
        // SAFETY: the region is mapped, zero-filled (a valid Node is all integers), aligned to
        // 64 bytes, and owned by self.
        unsafe { std::slice::from_raw_parts_mut(self.nodes, self.n) }
    }

    pub fn as_slice(&self) -> &[Node] {
        // SAFETY: as above.
        unsafe { std::slice::from_raw_parts(self.nodes, self.n) }
    }
}

impl Drop for HugeNodes {
    fn drop(&mut self) {
        // SAFETY: unmapping exactly the mapping created in new().
        unsafe { libc::munmap(self.base, self.len) };
    }
}

/// Kilobytes of anonymous memory this process has backed with transparent huge pages.
pub fn anon_huge_kib() -> u64 {
    std::fs::read_to_string("/proc/self/smaps_rollup")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("AnonHugePages:"))
                .and_then(|l| l.split_whitespace().nth(1)?.parse().ok())
        })
        .unwrap_or(0)
}

/// Follows `steps` links from node 0. Each load's address depends on the previous load's value,
/// so the CPU cannot overlap them: the time per step is the latency of one memory access.
pub fn chase(nodes: &[Node], steps: usize) -> usize {
    let mut i = 0;
    for _ in 0..steps {
        i = nodes[i].next;
    }
    i
}

/// Sums every element in order. Independent loads in address order: the access pattern that
/// prefetchers and wide loads handle best, so the time measures bandwidth.
pub fn sum_sequential(data: &[u64]) -> u64 {
    // Four independent accumulators so the additions themselves never become the bottleneck.
    let mut acc = [0u64; 4];
    let chunks = data.chunks_exact(4);
    let rest = chunks.remainder();
    for c in chunks {
        acc[0] = acc[0].wrapping_add(c[0]);
        acc[1] = acc[1].wrapping_add(c[1]);
        acc[2] = acc[2].wrapping_add(c[2]);
        acc[3] = acc[3].wrapping_add(c[3]);
    }
    let tail: u64 = rest.iter().fold(0u64, |a, &x| a.wrapping_add(x));
    acc.iter().fold(tail, |a, &x| a.wrapping_add(x))
}

/// Reads one byte every `stride` bytes, covering the whole buffer. Returns the bytes read, so
/// the caller can report time per access.
pub fn touch_strided(buf: &[u8], stride: usize) -> (u64, usize) {
    let mut sum = 0u64;
    let mut n = 0;
    let mut i = 0;
    while i < buf.len() {
        sum = sum.wrapping_add(buf[i] as u64);
        n += 1;
        i += stride;
    }
    (sum, n)
}

/// Sums a `rows` × `cols` matrix stored row by row, visiting it row by row.
pub fn sum_row_major(m: &[u32], rows: usize, cols: usize) -> u64 {
    let mut s = 0u64;
    for r in 0..rows {
        for c in 0..cols {
            s += m[r * cols + c] as u64;
        }
    }
    s
}

/// Sums the same matrix column by column: consecutive reads are `cols * 4` bytes apart.
pub fn sum_col_major(m: &[u32], rows: usize, cols: usize) -> u64 {
    let mut s = 0u64;
    for c in 0..cols {
        for r in 0..rows {
            s += m[r * cols + c] as u64;
        }
    }
    s
}

/// Sums `data[idx[i]]` for every i: a gather, the access pattern of a hash join probe or of
/// reading rows by a list of row ids.
pub fn gather(data: &[u32], idx: &[u32]) -> u64 {
    let mut s = 0u64;
    for &i in idx {
        s += data[i as usize] as u64;
    }
    s
}

/// Keeps the values below `threshold` with an `if`: one conditional branch per row.
#[inline(never)]
pub fn filter_branchy(data: &[u32], threshold: u32, out: &mut Vec<u32>) {
    out.clear();
    for &x in data {
        if x < threshold {
            out.push(x);
        }
    }
}

/// Keeps the same values without a branch on the data: always write, then advance the output
/// position by 0 or 1. The cost no longer depends on how predictable the condition is.
#[inline(never)]
pub fn filter_branchless(data: &[u32], threshold: u32, out: &mut Vec<u32>) {
    out.clear();
    out.resize(data.len() + 1, 0);
    let mut j = 0;
    for &x in data {
        // j never exceeds the number of rows read so far, so out[j] is always in bounds.
        out[j] = x;
        j += (x < threshold) as usize;
    }
    out.truncate(j);
}

/// `n` values uniform in 0..100, so `threshold = s` keeps about s% of them.
pub fn percent_data(n: usize, seed: u64) -> Vec<u32> {
    let mut rng = Rng::new(seed);
    (0..n).map(|_| rng.below(100) as u32).collect()
}

/// Shuffles `v` in place (Fisher-Yates).
pub fn shuffle<T>(v: &mut [T], seed: u64) {
    let mut rng = Rng::new(seed);
    for i in (1..v.len()).rev() {
        let j = rng.below(i as u64 + 1) as usize;
        v.swap(i, j);
    }
}

/// Keeps the optimizer from deleting a computation whose result we only time.
pub fn keep<T>(x: T) -> T {
    black_box(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_cycle_visits_every_node_once_per_lap() {
        let n = 1000;
        let nodes = random_cycle(n, 7);
        let mut seen = vec![false; n];
        let mut i = 0;
        for _ in 0..n {
            assert!(!seen[i], "node {i} visited twice in one lap");
            seen[i] = true;
            i = nodes[i].next;
        }
        assert_eq!(i, 0, "a lap returns to the start");
        assert!(seen.iter().all(|&s| s));
    }

    #[test]
    fn node_is_one_cache_line() {
        assert_eq!(std::mem::size_of::<Node>(), LINE);
        assert_eq!(std::mem::align_of::<Node>(), LINE);
    }

    #[test]
    fn both_filters_keep_the_same_rows() {
        let data = percent_data(10_000, 3);
        for t in [0, 1, 37, 50, 99, 100] {
            let (mut a, mut b) = (Vec::new(), Vec::new());
            filter_branchy(&data, t, &mut a);
            filter_branchless(&data, t, &mut b);
            assert_eq!(a, b, "threshold {t}");
        }
    }

    #[test]
    fn traversal_orders_agree() {
        let (rows, cols) = (37, 53);
        let m: Vec<u32> = (0..(rows * cols) as u32).collect();
        assert_eq!(sum_row_major(&m, rows, cols), sum_col_major(&m, rows, cols));
        let idx: Vec<u32> = (0..m.len() as u32).rev().collect();
        assert_eq!(gather(&m, &idx), sum_row_major(&m, rows, cols));
    }

    #[test]
    fn sum_handles_any_length() {
        for n in 0..10 {
            let v: Vec<u64> = (1..=n).collect();
            assert_eq!(sum_sequential(&v), n * (n + 1) / 2);
        }
    }
}
