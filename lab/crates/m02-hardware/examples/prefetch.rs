//! Article: "Prefetching and memory bandwidth".
//! Memory-level parallelism (several independent pointer chases at once), software prefetching
//! in a random gather, and how much bandwidth one and two threads can draw.
//!
//!     cargo run --release -p m02-hardware --example prefetch

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{_MM_HINT_T0, _mm_prefetch};

use labkit::timing::{Summary, repeat};
use m02_hardware::{LINE, Node, keep, random_cycle, shuffle, sum_sequential};

/// Follows `K` independent random cycles in lockstep. Each chain's loads still depend on each
/// other, but the chains don't, so the core can have up to K misses in flight at once.
#[inline(never)]
fn chase_k<const K: usize>(nodes: &[Node], steps: usize) -> usize {
    let n = nodes.len();
    let mut pos = [0usize; K];
    for (k, p) in pos.iter_mut().enumerate() {
        *p = (k * (n / K)) % n; // start the chains far apart
    }
    for _ in 0..steps {
        for p in pos.iter_mut() {
            *p = nodes[*p].next;
        }
    }
    pos.iter().sum()
}

/// Sums data[idx[i]], optionally asking for data[idx[i + dist]] ahead of time.
#[inline(never)]
fn gather_prefetch(data: &[u32], idx: &[u32], dist: usize) -> u64 {
    let mut s = 0u64;
    let n = idx.len();
    for i in 0..n {
        if dist > 0 && i + dist < n {
            let p = data.as_ptr().wrapping_add(idx[i + dist] as usize) as *const i8;
            // SAFETY: prefetch is a hint; it never faults, even for a bad address.
            unsafe { _mm_prefetch::<_MM_HINT_T0>(p) };
        }
        s += data[idx[i] as usize] as u64;
    }
    s
}

fn main() {
    println!("== Memory-level parallelism: K independent chases over 1 GiB ==\n");
    let n = (1usize << 30) / LINE;
    let nodes = random_cycle(n, 42);
    let steps = 4_000_000;
    println!(
        "{:>7}  {:>14}  {:>16}",
        "chains", "ns per step", "ns per access"
    );
    let mut csv = Vec::new();
    macro_rules! k {
        ($k:literal) => {{
            let s = Summary::of(&repeat(1, 3, || keep(chase_k::<$k>(&nodes, steps))));
            let per_step = s.median.as_secs_f64() * 1e9 / steps as f64;
            let per_access = per_step / $k as f64;
            println!("{:>7}  {per_step:>14.1}  {per_access:>16.2}", $k);
            csv.push(format!("{},{per_access:.3}", $k));
        }};
    }
    k!(1);
    k!(2);
    k!(4);
    k!(8);
    k!(12);
    k!(16);
    k!(24);
    k!(32);
    println!("\nmlp csv: {}\n", csv.join(";"));
    drop(nodes);

    println!("== Software prefetch in a random gather over 256 MiB ==\n");
    let len = 1usize << 26; // 64M u32
    let data: Vec<u32> = (0..len as u32).collect();
    let mut idx: Vec<u32> = (0..len as u32).collect();
    shuffle(&mut idx, 11);
    for dist in [0usize, 4, 8, 16, 32, 64, 128] {
        let s = Summary::of(&repeat(1, 5, || {
            gather_prefetch(keep(&data), keep(&idx), dist)
        }));
        println!(
            "prefetch distance {dist:>3}: {:>6.2} ns per value   noise ±{:.1}%",
            s.median.as_secs_f64() * 1e9 / len as f64,
            s.cv * 100.0
        );
    }
    drop(data);
    drop(idx);

    println!("\n== Bandwidth from main memory: one thread vs two ==\n");
    let words = (1usize << 30) / 8;
    let buf: Vec<u64> = (0..words as u64).collect();
    let one = Summary::of(&repeat(1, 5, || sum_sequential(keep(&buf))));
    let gb = (words * 8) as f64 / 1e9;
    println!("1 thread : {:>5.1} GB/s", gb / one.median.as_secs_f64());
    let (a, b) = buf.split_at(words / 2);
    let two = Summary::of(&repeat(1, 5, || {
        std::thread::scope(|s| {
            let h = s.spawn(|| sum_sequential(keep(a)));
            let y = sum_sequential(keep(b));
            h.join().unwrap().wrapping_add(y)
        })
    }));
    println!("2 threads: {:>5.1} GB/s", gb / two.median.as_secs_f64());
}
