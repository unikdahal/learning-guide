//! Article: "TLBs, pages and huge pages".
//! Asks the CPU how big its TLBs are (CPUID leaf 0x18 on Intel), then runs a random gather, the
//! access pattern of a hash table probe, on 4 KiB pages and on 2 MiB transparent huge pages.
//!
//!     cargo run --release -p m02-hardware --example pages

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::__cpuid_count;

use labkit::timing::{Summary, repeat};
use m02_hardware::{anon_huge_kib, gather, keep, shuffle};

/// Decodes Intel's "deterministic address translation parameters" (CPUID leaf 0x18).
fn print_tlbs() {
    // cpuid is always available on x86-64; unsupported leaves return zeros.
    let max_leaf = __cpuid_count(0, 0).eax;
    if max_leaf < 0x18 {
        println!("CPUID leaf 0x18 not available (max leaf {max_leaf:#x})");
        return;
    }
    let max_sub = __cpuid_count(0x18, 0).eax;
    println!(
        "{:<10} {:>5} {:<22} {:>7} {:>5}",
        "TLB", "level", "page sizes", "entries", "ways"
    );
    for sub in 0..=max_sub {
        let r = __cpuid_count(0x18, sub);
        let kind = match r.edx & 0x1f {
            0 => continue,
            1 => "data",
            2 => "instr",
            3 => "unified",
            4 => "load",
            5 => "store",
            _ => "other",
        };
        let level = (r.edx >> 5) & 0x7;
        let mut sizes = Vec::new();
        for (bit, s) in [(0, "4K"), (1, "2M"), (2, "4M"), (3, "1G")] {
            if r.ebx & (1 << bit) != 0 {
                sizes.push(s);
            }
        }
        let ways = r.ebx >> 16;
        let sets = r.ecx;
        let fully = (r.edx >> 8) & 1 == 1;
        println!(
            "{kind:<10} {level:>5} {:<22} {:>7} {:>5}",
            sizes.join(" "),
            ways * sets,
            if fully {
                "full".to_string()
            } else {
                ways.to_string()
            }
        );
    }
}

/// A buffer of u32 in anonymous memory, 2 MiB aligned, optionally advised to use huge pages.
struct Buf {
    base: *mut libc::c_void,
    len: usize,
    ptr: *mut u32,
    n: usize,
}

impl Buf {
    fn new(n: usize, huge: bool) -> Buf {
        const HUGE: usize = 2 << 20;
        let bytes = (n * 4).div_ceil(HUGE) * HUGE;
        let len = bytes + HUGE;
        // SAFETY: fresh private anonymous mapping, checked below.
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
        assert_ne!(base, libc::MAP_FAILED);
        let aligned = (base as usize).div_ceil(HUGE) * HUGE;
        let advice = if huge {
            libc::MADV_HUGEPAGE
        } else {
            libc::MADV_NOHUGEPAGE
        };
        // SAFETY: the range lies inside the mapping.
        unsafe { libc::madvise(aligned as *mut libc::c_void, bytes, advice) };
        let mut b = Buf {
            base,
            len,
            ptr: aligned as *mut u32,
            n,
        };
        for (i, x) in b.slice_mut().iter_mut().enumerate() {
            *x = i as u32;
        }
        b
    }
    fn slice(&self) -> &[u32] {
        // SAFETY: mapped, initialized above, owned by self.
        unsafe { std::slice::from_raw_parts(self.ptr, self.n) }
    }
    fn slice_mut(&mut self) -> &mut [u32] {
        // SAFETY: as above.
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.n) }
    }
}

impl Drop for Buf {
    fn drop(&mut self) {
        // SAFETY: unmapping the mapping created in new().
        unsafe { libc::munmap(self.base, self.len) };
    }
}

fn main() {
    println!("== What the CPU reports about its TLBs (CPUID 0x18) ==\n");
    print_tlbs();

    println!("\n== Random gather of 16M values: 4 KiB pages vs 2 MiB pages ==\n");
    println!(
        "{:>12}  {:>12}  {:>12}  {:>8}",
        "table size", "4 KiB pages", "2 MiB pages", "speedup"
    );
    let probes = 1usize << 24;
    let mut csv = Vec::new();
    for p in [20usize, 22, 24, 26, 28, 30] {
        let size = 1usize << p; // bytes
        let n = size / 4;
        let mut idx: Vec<u32> = (0..probes).map(|i| (i % n) as u32).collect();
        shuffle(&mut idx, 5);
        let small = Buf::new(n, false);
        let a = Summary::of(&repeat(1, 5, || gather(keep(small.slice()), keep(&idx))));
        drop(small);
        let before = anon_huge_kib();
        let huge = Buf::new(n, true);
        let backed = anon_huge_kib().saturating_sub(before) as f64 * 1024.0 / size as f64;
        let b = Summary::of(&repeat(1, 5, || gather(keep(huge.slice()), keep(&idx))));
        drop(huge);
        let na = a.median.as_secs_f64() * 1e9 / probes as f64;
        let nb = b.median.as_secs_f64() * 1e9 / probes as f64;
        let label = if size >= 1 << 30 {
            format!("{} GiB", size >> 30)
        } else {
            format!("{} MiB", size >> 20)
        };
        println!(
            "{label:>12}  {na:>9.2} ns  {nb:>9.2} ns  {:>7.2}x   (huge-backed {:.0}%)",
            na / nb,
            backed * 100.0
        );
        csv.push(format!("{size},{na:.3},{nb:.3}"));
    }
    println!("\ngather csv: {}", csv.join(";"));
}
