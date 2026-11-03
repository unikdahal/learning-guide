//! Article: "Disks and SSDs".
//! On this machine's disk: random 4 KiB reads at increasing queue depth, sequential reads at
//! increasing request size, buffered reads cold and warm in the page cache, and the cost of
//! making a small write durable. O_DIRECT bypasses the page cache, so those numbers are the
//! device's (here, a cloud VM's virtual disk).
//!
//!     cargo run --release -p m02-hardware --example disk
//!
//! Writes a 1 GiB file to lab/data/ and deletes it at the end.

use std::ffi::CString;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use labkit::rng::Rng;

const FILE_SIZE: usize = 1 << 30;
const BLOCK: usize = 4096;

struct Aligned(*mut u8, usize);
// SAFETY: each thread gets its own buffer; the pointer is only used by its owner.
unsafe impl Send for Aligned {}
impl Aligned {
    fn new(len: usize) -> Aligned {
        let mut p = std::ptr::null_mut();
        // SAFETY: checked below.
        let rc = unsafe { libc::posix_memalign(&mut p, BLOCK, len) };
        assert_eq!(rc, 0);
        Aligned(p as *mut u8, len)
    }
}
impl Drop for Aligned {
    fn drop(&mut self) {
        // SAFETY: allocated by posix_memalign.
        unsafe { libc::free(self.0 as *mut libc::c_void) };
    }
}

fn pread(fd: i32, buf: &Aligned, len: usize, off: usize) {
    assert!(len <= buf.1);
    // SAFETY: buf holds len bytes; fd is open.
    let n = unsafe { libc::pread(fd, buf.0 as *mut libc::c_void, len, off as libc::off_t) };
    assert_eq!(n as usize, len, "short read");
}

fn open(path: &CString, flags: i32) -> i32 {
    // SAFETY: plain open(2).
    let fd = unsafe { libc::open(path.as_ptr(), flags) };
    assert!(fd >= 0, "open failed");
    fd
}

fn pct(v: &mut [Duration], p: f64) -> Duration {
    v.sort();
    v[((v.len() as f64 * p) as usize).min(v.len() - 1)]
}

fn main() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("m02-disk.bin");
    {
        let mut f = std::fs::File::create(&path).unwrap();
        let chunk: Vec<u8> = (0..1 << 20).map(|i| (i * 7) as u8).collect();
        for _ in 0..FILE_SIZE >> 20 {
            f.write_all(&chunk).unwrap();
        }
        f.sync_all().unwrap();
    }
    let cpath = CString::new(path.to_str().unwrap()).unwrap();

    println!("== Random 4 KiB reads (O_DIRECT) at increasing queue depth ==\n");
    println!(
        "{:>11}  {:>10}  {:>12}  {:>10}",
        "queue depth", "IOPS", "median lat", "p99 lat"
    );
    let mut csv = Vec::new();
    for qd in [1usize, 2, 4, 8, 16, 32, 64] {
        let stop = AtomicBool::new(false);
        let done = AtomicU64::new(0);
        let window = Duration::from_secs(3);
        let lats: Vec<Vec<Duration>> = std::thread::scope(|s| {
            let hs: Vec<_> = (0..qd)
                .map(|t| {
                    let (stop, done, cpath) = (&stop, &done, &cpath);
                    s.spawn(move || {
                        let fd = open(cpath, libc::O_RDONLY | libc::O_DIRECT);
                        let buf = Aligned::new(BLOCK);
                        let mut rng = Rng::new(100 + t as u64);
                        let mut lat = Vec::new();
                        while !stop.load(Ordering::Relaxed) {
                            let off = rng.below((FILE_SIZE / BLOCK) as u64) as usize * BLOCK;
                            let t0 = Instant::now();
                            pread(fd, &buf, BLOCK, off);
                            lat.push(t0.elapsed());
                            done.fetch_add(1, Ordering::Relaxed);
                        }
                        // SAFETY: closing our descriptor.
                        unsafe { libc::close(fd) };
                        lat
                    })
                })
                .collect();
            std::thread::sleep(Duration::from_millis(300)); // warm-up
            let start = done.load(Ordering::Relaxed);
            let t0 = Instant::now();
            std::thread::sleep(window);
            let n = done.load(Ordering::Relaxed) - start;
            let secs = t0.elapsed().as_secs_f64();
            stop.store(true, Ordering::Relaxed);
            let iops = n as f64 / secs;
            let mut v: Vec<Vec<Duration>> = hs.into_iter().map(|h| h.join().unwrap()).collect();
            v.push(vec![Duration::from_secs_f64(iops)]); // smuggle IOPS out
            v
        });
        let iops = lats.last().unwrap()[0].as_secs_f64();
        let mut all: Vec<Duration> = lats[..lats.len() - 1].iter().flatten().copied().collect();
        let med = pct(&mut all, 0.5);
        let p99 = pct(&mut all, 0.99);
        println!(
            "{qd:>11}  {:>10.0}  {:>9.1} µs  {:>7.1} µs",
            iops,
            med.as_secs_f64() * 1e6,
            p99.as_secs_f64() * 1e6
        );
        csv.push(format!("{qd},{iops:.0},{:.1}", med.as_secs_f64() * 1e6));
    }
    println!("\nqd csv: {}\n", csv.join(";"));

    println!("== Sequential reads (O_DIRECT, one thread) at increasing request size ==\n");
    let fd = open(&cpath, libc::O_RDONLY | libc::O_DIRECT);
    let buf = Aligned::new(4 << 20);
    let mut csv = Vec::new();
    for size in [
        4usize << 10,
        16 << 10,
        64 << 10,
        256 << 10,
        1 << 20,
        4 << 20,
    ] {
        let total = (256usize << 20).min(FILE_SIZE);
        let mut best = f64::MAX;
        for rep in 0..3 {
            let start = rep * total % FILE_SIZE;
            let t0 = Instant::now();
            let mut off = 0;
            while off < total {
                pread(fd, &buf, size, (start + off) % FILE_SIZE);
                off += size;
            }
            best = best.min(t0.elapsed().as_secs_f64());
        }
        let mbs = total as f64 / best / 1e6;
        println!("{:>6} KiB requests: {mbs:>7.0} MB/s", size >> 10);
        csv.push(format!("{size},{mbs:.0}"));
    }
    println!("\nseq csv: {}\n", csv.join(";"));
    // SAFETY: closing our descriptor.
    unsafe { libc::close(fd) };

    println!("== Buffered reads of 256 MiB: cold vs in the page cache ==\n");
    let fd = open(&cpath, libc::O_RDONLY);
    let buf = Aligned::new(1 << 20);
    let region = 256usize << 20;
    // SAFETY: advisory call on our own descriptor: drop this file's pages from the cache.
    unsafe { libc::posix_fadvise(fd, 0, 0, libc::POSIX_FADV_DONTNEED) };
    for label in ["cold (evicted with fadvise)", "warm (second read)"] {
        let t0 = Instant::now();
        let mut off = 0;
        while off < region {
            pread(fd, &buf, 1 << 20, off);
            off += 1 << 20;
        }
        let s = t0.elapsed().as_secs_f64();
        println!("{label:<30} {:>7.0} MB/s", region as f64 / s / 1e6);
    }
    // SAFETY: closing our descriptor.
    unsafe { libc::close(fd) };

    println!("\n== Making a 4 KiB write durable: pwrite + fdatasync, 300 times ==\n");
    let wpath = dir.join("m02-wal.bin");
    let wc = CString::new(wpath.to_str().unwrap()).unwrap();
    // SAFETY: open(2) with create flags and a mode.
    let wfd = unsafe {
        libc::open(
            wc.as_ptr(),
            libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
            0o644,
        )
    };
    assert!(wfd >= 0);
    let page = vec![42u8; BLOCK];
    let (mut w, mut wd) = (Vec::new(), Vec::new());
    for i in 0..300 {
        let t0 = Instant::now();
        // SAFETY: page holds BLOCK bytes.
        let n = unsafe {
            libc::pwrite(
                wfd,
                page.as_ptr() as *const libc::c_void,
                BLOCK,
                (i * BLOCK) as libc::off_t,
            )
        };
        assert_eq!(n as usize, BLOCK);
        w.push(t0.elapsed());
        // SAFETY: fdatasync on our descriptor.
        assert_eq!(unsafe { libc::fdatasync(wfd) }, 0);
        wd.push(t0.elapsed());
    }
    println!(
        "pwrite alone (into the page cache)  median {:>8.1} µs",
        pct(&mut w, 0.5).as_secs_f64() * 1e6
    );
    println!(
        "pwrite + fdatasync (on the device)  median {:>8.1} µs   p99 {:>8.1} µs",
        pct(&mut wd, 0.5).as_secs_f64() * 1e6,
        pct(&mut wd, 0.99).as_secs_f64() * 1e6
    );
    // SAFETY: closing our descriptor.
    unsafe { libc::close(wfd) };
    std::fs::remove_file(&wpath).ok();
    std::fs::remove_file(&path).ok();
}
