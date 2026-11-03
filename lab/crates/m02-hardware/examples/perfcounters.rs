//! Article: "Measuring hardware with perf counters".
//! Opens counters with the perf_event_open system call directly (what the `perf` tool does
//! underneath). On this cloud VM the hypervisor exposes no hardware PMU, so hardware events
//! fail; software events still work, and page faults tell a story of their own.
//!
//!     cargo run --release -p m02-hardware --example perfcounters

use std::time::Instant;

// perf_event_attr fields we set, at their offsets in the kernel's struct (PERF_ATTR_SIZE_VER5 = 112).
const ATTR_SIZE: usize = 112;
const PERF_TYPE_HARDWARE: u32 = 0;
const PERF_TYPE_SOFTWARE: u32 = 1;
// Hardware event ids.
const HW_CPU_CYCLES: u64 = 0;
const HW_INSTRUCTIONS: u64 = 1;
const HW_CACHE_MISSES: u64 = 3;
const HW_BRANCH_MISSES: u64 = 5;
// Software event ids.
const SW_TASK_CLOCK: u64 = 1;
const SW_PAGE_FAULTS: u64 = 2;
const SW_CONTEXT_SWITCHES: u64 = 3;
// ioctl requests (_IO('$', n)).
const IOC_ENABLE: libc::c_ulong = 0x2400;
const IOC_DISABLE: libc::c_ulong = 0x2401;
const IOC_RESET: libc::c_ulong = 0x2403;

/// Opens one counter for this thread, user space only, starting disabled.
fn open(kind: u32, config: u64) -> Result<i32, std::io::Error> {
    open_with(kind, config, true)
}

/// Like `open`, optionally counting events that happen while the kernel runs on our behalf.
/// Context switches happen inside the kernel, so a user-only counter never sees them.
fn open_with(kind: u32, config: u64, user_only: bool) -> Result<i32, std::io::Error> {
    let mut attr = [0u8; ATTR_SIZE];
    attr[0..4].copy_from_slice(&kind.to_ne_bytes());
    attr[4..8].copy_from_slice(&(ATTR_SIZE as u32).to_ne_bytes());
    attr[8..16].copy_from_slice(&config.to_ne_bytes());
    // flags: disabled (bit 0), exclude_kernel (bit 5), exclude_hv (bit 6)
    let flags: u64 = if user_only {
        1 | (1 << 5) | (1 << 6)
    } else {
        1
    };
    attr[40..48].copy_from_slice(&flags.to_ne_bytes());
    // SAFETY: perf_event_open(attr, pid = 0 (this thread), cpu = -1 (any), group = -1, flags = 0).
    let fd = unsafe { libc::syscall(libc::SYS_perf_event_open, attr.as_ptr(), 0, -1, -1, 0) };
    if fd < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(fd as i32)
    }
}

struct Counter(i32);
impl Counter {
    fn start(&self) {
        // SAFETY: ioctl on a perf fd we opened.
        unsafe {
            libc::ioctl(self.0, IOC_RESET, 0);
            libc::ioctl(self.0, IOC_ENABLE, 0);
        }
    }
    fn stop(&self) -> u64 {
        let mut v = 0u64;
        // SAFETY: ioctl and an 8-byte read on a perf fd we opened.
        unsafe {
            libc::ioctl(self.0, IOC_DISABLE, 0);
            libc::read(self.0, &mut v as *mut u64 as *mut libc::c_void, 8);
        }
        v
    }
}
impl Drop for Counter {
    fn drop(&mut self) {
        // SAFETY: closing our fd.
        unsafe { libc::close(self.0) };
    }
}

/// Maps `bytes` of fresh anonymous memory, optionally advised for huge pages, writes one byte
/// per 4 KiB, and returns (page faults, seconds).
fn first_touch(bytes: usize, huge: bool, faults: &Counter) -> (u64, f64) {
    // SAFETY: fresh private anonymous mapping, checked, unmapped at the end.
    unsafe {
        let len = bytes + (2 << 20);
        let base = libc::mmap(
            std::ptr::null_mut(),
            len,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        );
        assert_ne!(base, libc::MAP_FAILED);
        let p = ((base as usize).div_ceil(2 << 20) * (2 << 20)) as *mut u8;
        let advice = if huge {
            libc::MADV_HUGEPAGE
        } else {
            libc::MADV_NOHUGEPAGE
        };
        libc::madvise(p as *mut libc::c_void, bytes, advice);
        faults.start();
        let t0 = Instant::now();
        let mut off = 0;
        while off < bytes {
            std::ptr::write_volatile(p.add(off), 1);
            off += 4096;
        }
        let secs = t0.elapsed().as_secs_f64();
        let n = faults.stop();
        libc::munmap(base, len);
        (n, secs)
    }
}

fn main() {
    println!("== Can this machine count hardware events? ==\n");
    for (name, cfg) in [
        ("cpu-cycles", HW_CPU_CYCLES),
        ("instructions", HW_INSTRUCTIONS),
        ("cache-misses", HW_CACHE_MISSES),
        ("branch-misses", HW_BRANCH_MISSES),
    ] {
        match open(PERF_TYPE_HARDWARE, cfg) {
            Ok(fd) => {
                let c = Counter(fd);
                c.start();
                let mut x = 0u64;
                for i in 0..10_000_000u64 {
                    x = std::hint::black_box(x.wrapping_add(i));
                }
                println!("{name:<14} works: {} during a 10M-iteration loop", c.stop());
            }
            Err(e) => println!("{name:<14} unavailable: {e}"),
        }
    }

    println!("\n== Software events ==\n");
    let faults = Counter(open(PERF_TYPE_SOFTWARE, SW_PAGE_FAULTS).expect("page-fault counter"));
    let clock = Counter(open(PERF_TYPE_SOFTWARE, SW_TASK_CLOCK).expect("task-clock counter"));
    let switches_user =
        Counter(open(PERF_TYPE_SOFTWARE, SW_CONTEXT_SWITCHES).expect("context-switch counter"));
    let switches_all = open_with(PERF_TYPE_SOFTWARE, SW_CONTEXT_SWITCHES, false).map(Counter);

    println!("First touch of 1 GiB of fresh memory, one write per 4 KiB:\n");
    for huge in [false, true] {
        let (n, secs) = first_touch(1 << 30, huge, &faults);
        println!(
            "{:<18} {n:>8} page faults  {:>7.1} ms  {:>6.2} µs per fault",
            if huge {
                "2 MiB huge pages"
            } else {
                "4 KiB pages"
            },
            secs * 1e3,
            secs * 1e6 / n.max(1) as f64
        );
    }
    println!("Touching the same 1 GiB a second time:\n");
    // SAFETY: as in first_touch, without unmapping between the two passes.
    unsafe {
        let bytes = 1usize << 30;
        let base = libc::mmap(
            std::ptr::null_mut(),
            bytes,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        ) as *mut u8;
        libc::madvise(base as *mut libc::c_void, bytes, libc::MADV_NOHUGEPAGE);
        for pass in ["first pass", "second pass"] {
            faults.start();
            let t0 = Instant::now();
            let mut off = 0;
            while off < bytes {
                std::ptr::write_volatile(base.add(off), 2);
                off += 4096;
            }
            let secs = t0.elapsed().as_secs_f64();
            println!(
                "{pass:<18} {:>8} page faults  {:>7.1} ms",
                faults.stop(),
                secs * 1e3
            );
        }
        libc::munmap(base as *mut libc::c_void, bytes);
    }

    println!(
        "\nTask clock and context switches: 200 ms of spinning vs 200 ms of sleeping in 1 ms steps:\n"
    );
    for (name, sleep) in [("spinning", false), ("sleeping", true)] {
        clock.start();
        switches_user.start();
        if let Ok(c) = &switches_all {
            c.start();
        }
        let t0 = Instant::now();
        while t0.elapsed().as_millis() < 200 {
            if sleep {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
        let cpu_ns = clock.stop();
        let sw_user = switches_user.stop();
        let sw_all = match &switches_all {
            Ok(c) => c.stop().to_string(),
            Err(e) => format!("unavailable ({e})"),
        };
        println!(
            "{name:<10} wall 200 ms   on-CPU {:>6.1} ms   context switches: user-only counter {sw_user}, including kernel {sw_all}",
            cpu_ns as f64 / 1e6
        );
    }
}
