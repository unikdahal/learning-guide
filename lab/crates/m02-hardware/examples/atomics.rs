//! Article: "Atomics and memory ordering for engine developers".
//! What atomic operations cost with and without contention, what the memory orderings compile to
//! on x86-64, and a litmus test that shows the processor reordering a store and a later load
//! unless the code asks for sequential consistency.
//!
//!     cargo run --release -p m02-hardware --example atomics

use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::thread;

use labkit::timing::{Summary, repeat};
use m02_hardware::keep;

const N: usize = 100_000_000;

fn per_op(name: &str, f: impl FnMut() -> u64) {
    let s = Summary::of(&repeat(1, 5, f));
    println!(
        "{name:<40} {:>6.2} ns per op",
        s.median.as_secs_f64() * 1e9 / N as f64
    );
}

/// Store-buffering litmus test. Thread A: x = 1; ra = y. Thread B: y = 1; rb = x.
/// Under sequential consistency at least one thread must see the other's store, so
/// ra == rb == 0 is impossible. Returns how many of `trials` ended with both zero.
fn store_buffering(trials: usize, store: Ordering, load: Ordering) -> usize {
    let x = AtomicU32::new(0);
    let y = AtomicU32::new(0);
    let turn = AtomicUsize::new(0); // a spin barrier: both threads start each trial together
    let both_zero = AtomicUsize::new(0);
    let results = [AtomicU32::new(0), AtomicU32::new(0)];
    thread::scope(|s| {
        for me in 0..2 {
            let (x, y, turn, results, both_zero) = (&x, &y, &turn, &results, &both_zero);
            s.spawn(move || {
                let (mine, theirs) = if me == 0 { (x, y) } else { (y, x) };
                for t in 0..trials {
                    // Barrier 1: wait until both threads have arrived for trial t.
                    turn.fetch_add(1, Ordering::SeqCst);
                    while turn.load(Ordering::SeqCst) < 4 * t + 2 {
                        std::hint::spin_loop();
                    }
                    mine.store(1, store);
                    let r = theirs.load(load);
                    results[me].store(r, Ordering::SeqCst);
                    // Barrier 2: both done; thread 0 tallies and resets.
                    turn.fetch_add(1, Ordering::SeqCst);
                    while turn.load(Ordering::SeqCst) < 4 * t + 4 {
                        std::hint::spin_loop();
                    }
                    if me == 0 {
                        if results[0].load(Ordering::SeqCst) == 0
                            && results[1].load(Ordering::SeqCst) == 0
                        {
                            both_zero.fetch_add(1, Ordering::Relaxed);
                        }
                        x.store(0, Ordering::SeqCst);
                        y.store(0, Ordering::SeqCst);
                    }
                    // Barrier 3 is folded into the next trial's barrier 1: thread 1 can't
                    // start trial t+1 until thread 0 arrives, which is after the reset.
                    if me == 1 {
                        while turn.load(Ordering::SeqCst) < 4 * t + 4 {
                            std::hint::spin_loop();
                        }
                    }
                }
            });
        }
    });
    both_zero.load(Ordering::Relaxed)
}

fn main() {
    println!("== One thread, no contention ==\n");
    let mut plain = 0u64;
    per_op("plain increment (volatile)", || {
        for _ in 0..N {
            // SAFETY: a local variable.
            unsafe { std::ptr::write_volatile(&mut plain, std::ptr::read_volatile(&plain) + 1) };
        }
        plain
    });
    let a = AtomicU64::new(0);
    per_op("fetch_add, Relaxed", || {
        for _ in 0..N {
            a.fetch_add(1, Ordering::Relaxed);
        }
        a.load(Ordering::Relaxed)
    });
    per_op("fetch_add, SeqCst", || {
        for _ in 0..N {
            a.fetch_add(1, Ordering::SeqCst);
        }
        a.load(Ordering::Relaxed)
    });
    per_op("load + store, Relaxed (not atomic as a pair)", || {
        for _ in 0..N {
            a.store(a.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
        }
        a.load(Ordering::Relaxed)
    });
    per_op("store, Release", || {
        for i in 0..N {
            a.store(i as u64, Ordering::Release);
        }
        a.load(Ordering::Relaxed)
    });
    per_op("store, SeqCst", || {
        for i in 0..N {
            a.store(i as u64, Ordering::SeqCst);
        }
        a.load(Ordering::Relaxed)
    });
    per_op("compare_exchange loop (increment)", || {
        for _ in 0..N {
            let mut cur = a.load(Ordering::Relaxed);
            while let Err(now) =
                a.compare_exchange_weak(cur, cur + 1, Ordering::AcqRel, Ordering::Relaxed)
            {
                cur = now;
            }
        }
        a.load(Ordering::Relaxed)
    });

    println!("\n== Two threads incrementing one atomic ==\n");
    let shared = AtomicU64::new(0);
    let s = Summary::of(&repeat(1, 5, || {
        thread::scope(|sc| {
            for _ in 0..2 {
                sc.spawn(|| {
                    for _ in 0..N / 2 {
                        shared.fetch_add(1, Ordering::Relaxed);
                    }
                });
            }
        });
        keep(shared.load(Ordering::Relaxed))
    }));
    println!(
        "{:<40} {:>6.2} ns per op (total, both threads)",
        "fetch_add, Relaxed, contended",
        s.median.as_secs_f64() * 1e9 / N as f64
    );

    println!("\n== Store-buffering litmus test, 1,000,000 trials each ==\n");
    let trials = 1_000_000;
    for (name, st, ld) in [
        (
            "Relaxed store, Relaxed load",
            Ordering::Relaxed,
            Ordering::Relaxed,
        ),
        (
            "Release store, Acquire load",
            Ordering::Release,
            Ordering::Acquire,
        ),
        (
            "SeqCst store, SeqCst load",
            Ordering::SeqCst,
            Ordering::SeqCst,
        ),
    ] {
        let z = store_buffering(trials, st, ld);
        println!("{name:<30} both loads saw 0 in {z:>7} of {trials} trials");
    }
}
