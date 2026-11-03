//! Article: "The memory hierarchy, from registers to object storage".
//! Reads from this machine's disk with O_DIRECT, bypassing the operating system's page cache,
//! so the times are the device's (here, whatever virtual disk the cloud provides), and compares
//! a cached read of the same file.
//!
//!     cargo run --release -p m02-hardware --example storage

use std::ffi::CString;
use std::io::Write;
use std::time::{Duration, Instant};

use labkit::rng::Rng;
use labkit::timing::Summary;

const FILE_SIZE: usize = 1 << 30; // 1 GiB
const BLOCK: usize = 4096;

struct Aligned {
    ptr: *mut u8,
    len: usize,
}

impl Aligned {
    fn new(len: usize) -> Aligned {
        let mut ptr = std::ptr::null_mut();
        // SAFETY: posix_memalign writes a valid pointer on success, checked below.
        let rc = unsafe { libc::posix_memalign(&mut ptr, BLOCK, len) };
        assert_eq!(rc, 0, "posix_memalign failed");
        Aligned {
            ptr: ptr as *mut u8,
            len,
        }
    }
}

impl Drop for Aligned {
    fn drop(&mut self) {
        // SAFETY: allocated by posix_memalign in new().
        unsafe { libc::free(self.ptr as *mut libc::c_void) };
    }
}

fn pread(fd: i32, buf: &Aligned, len: usize, offset: usize) {
    assert!(len <= buf.len);
    // SAFETY: buf has at least len bytes; fd is open for reading.
    let n = unsafe { libc::pread(fd, buf.ptr as *mut libc::c_void, len, offset as libc::off_t) };
    assert_eq!(n as usize, len, "short read");
}

fn report(name: &str, samples: &[Duration]) {
    let s = Summary::of(samples);
    let mut sorted = samples.to_vec();
    sorted.sort();
    let p99 = sorted[(sorted.len() * 99 / 100).min(sorted.len() - 1)];
    println!(
        "{name:<34} median {:>9.1} µs   p99 {:>9.1} µs   ({} reads)",
        s.median.as_secs_f64() * 1e6,
        p99.as_secs_f64() * 1e6,
        samples.len()
    );
}

fn main() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("m02-storage.bin");
    if std::fs::metadata(&path).ok().map(|m| m.len() as usize) != Some(FILE_SIZE) {
        println!("writing a 1 GiB test file to {} ...", path.display());
        let mut f = std::fs::File::create(&path).unwrap();
        let chunk: Vec<u8> = (0..1 << 20).map(|i| (i * 31) as u8).collect();
        for _ in 0..FILE_SIZE >> 20 {
            f.write_all(&chunk).unwrap();
        }
        f.sync_all().unwrap();
    }
    let cpath = CString::new(path.to_str().unwrap()).unwrap();

    println!("\n== Random 4 KiB reads, one at a time ==\n");
    // SAFETY: plain open(2) calls on a path we just wrote.
    let direct = unsafe { libc::open(cpath.as_ptr(), libc::O_RDONLY | libc::O_DIRECT) };
    assert!(direct >= 0, "open with O_DIRECT failed");
    let buf = Aligned::new(1 << 20);
    let mut rng = Rng::new(5);
    let blocks = FILE_SIZE / BLOCK;
    let mut samples = Vec::new();
    for i in 0..2100 {
        let off = rng.below(blocks as u64) as usize * BLOCK;
        let t = Instant::now();
        pread(direct, &buf, BLOCK, off);
        if i >= 100 {
            samples.push(t.elapsed()); // skip the first 100 as warm-up
        }
    }
    report("disk, O_DIRECT (no page cache)", &samples);

    // SAFETY: as above.
    let cached = unsafe { libc::open(cpath.as_ptr(), libc::O_RDONLY) };
    assert!(cached >= 0);
    // Pull a 64 MiB region into the page cache, then read random blocks from it.
    for off in (0..64 << 20).step_by(1 << 20) {
        pread(cached, &buf, 1 << 20, off);
    }
    let mut samples = Vec::new();
    for _ in 0..20000 {
        let off = rng.below(((64 << 20) / BLOCK) as u64) as usize * BLOCK;
        let t = Instant::now();
        pread(cached, &buf, BLOCK, off);
        samples.push(t.elapsed());
    }
    report("page cache (file data in RAM)", &samples);

    println!("\n== Sequential 1 MiB reads, O_DIRECT ==\n");
    let t = Instant::now();
    let mut off = 0;
    while off < FILE_SIZE {
        pread(direct, &buf, 1 << 20, off);
        off += 1 << 20;
    }
    let secs = t.elapsed().as_secs_f64();
    println!(
        "read 1 GiB in {:.2} s: {:.0} MB/s",
        secs,
        FILE_SIZE as f64 / secs / 1e6
    );
    // SAFETY: closing descriptors we opened.
    unsafe {
        libc::close(direct);
        libc::close(cached);
    }
}
