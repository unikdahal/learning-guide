//! Article: "Networks for data movement".
//! TCP over the loopback interface: round-trip time by message size, streaming throughput by
//! write size, and the classic Nagle + delayed-ACK stall when a small request is sent as two
//! writes. Loopback never touches a network card, so these numbers are the cost of the kernel's
//! network stack and of system calls, a floor under any real network's costs.
//!
//!     cargo run --release -p m02-hardware --example network

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

fn pct(v: &mut [Duration], p: f64) -> Duration {
    v.sort();
    v[((v.len() as f64 * p) as usize).min(v.len() - 1)]
}

/// An echo server: reads exactly `size` bytes, writes them back, repeatedly.
fn echo_server(size: usize) -> (u16, thread::JoinHandle<()>) {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = l.local_addr().unwrap().port();
    let h = thread::spawn(move || {
        let (mut s, _) = l.accept().unwrap();
        s.set_nodelay(true).unwrap();
        let mut buf = vec![0u8; size];
        while s.read_exact(&mut buf).is_ok() {
            if s.write_all(&buf).is_err() {
                break;
            }
        }
    });
    (port, h)
}

fn main() {
    println!("== Round trip over loopback TCP, by message size ==\n");
    println!("{:>10}  {:>12}  {:>12}", "message", "median RTT", "p99 RTT");
    let mut csv = Vec::new();
    for size in [64usize, 1024, 16 << 10, 256 << 10, 1 << 20] {
        let (port, h) = echo_server(size);
        let mut c = TcpStream::connect(("127.0.0.1", port)).unwrap();
        c.set_nodelay(true).unwrap();
        let msg = vec![7u8; size];
        let mut back = vec![0u8; size];
        let n = if size >= 256 << 10 { 300 } else { 3000 };
        let mut lat = Vec::new();
        for i in 0..n + 100 {
            let t0 = Instant::now();
            c.write_all(&msg).unwrap();
            c.read_exact(&mut back).unwrap();
            if i >= 100 {
                lat.push(t0.elapsed());
            }
        }
        drop(c);
        h.join().unwrap();
        let med = pct(&mut lat, 0.5);
        let p99 = pct(&mut lat, 0.99);
        let label = if size >= 1024 {
            format!("{} KiB", size >> 10)
        } else {
            format!("{size} B")
        };
        println!(
            "{label:>10}  {:>9.1} µs  {:>9.1} µs",
            med.as_secs_f64() * 1e6,
            p99.as_secs_f64() * 1e6
        );
        csv.push(format!("{size},{:.2}", med.as_secs_f64() * 1e6));
    }
    println!("\nrtt csv: {}\n", csv.join(";"));

    println!("== Streaming 2 GiB one way, by write size ==\n");
    let mut csv = Vec::new();
    for wsize in [512usize, 4 << 10, 64 << 10, 1 << 20, 4 << 20] {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        let total: usize = 2 << 30;
        let rx = thread::spawn(move || {
            let (mut s, _) = l.accept().unwrap();
            let mut buf = vec![0u8; 1 << 20];
            let mut got = 0;
            while got < total {
                let n = s.read(&mut buf).unwrap();
                if n == 0 {
                    break;
                }
                got += n;
            }
            got
        });
        let mut c = TcpStream::connect(("127.0.0.1", port)).unwrap();
        let chunk = vec![1u8; wsize];
        let t0 = Instant::now();
        let mut sent = 0;
        while sent < total {
            c.write_all(&chunk).unwrap();
            sent += wsize;
        }
        drop(c);
        let got = rx.join().unwrap();
        let secs = t0.elapsed().as_secs_f64();
        let gbs = got as f64 / secs / 1e9;
        let label = if wsize >= 1024 {
            format!("{} KiB", wsize >> 10)
        } else {
            format!("{wsize} B")
        };
        println!(
            "{label:>8} writes: {gbs:>6.2} GB/s  ({:.0} writes/s)",
            sent as f64 / wsize as f64 / secs
        );
        csv.push(format!("{wsize},{gbs:.3}"));
    }
    println!("\nstream csv: {}\n", csv.join(";"));

    println!("== A request sent as two small writes, then wait for a reply: Nagle on vs off ==\n");
    for nodelay in [false, true] {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        let h = thread::spawn(move || {
            let (mut s, _) = l.accept().unwrap();
            let mut req = [0u8; 40];
            while s.read_exact(&mut req).is_ok() {
                if s.write_all(&[1u8; 8]).is_err() {
                    break;
                }
            }
        });
        let mut c = TcpStream::connect(("127.0.0.1", port)).unwrap();
        c.set_nodelay(nodelay).unwrap();
        let mut lat = Vec::new();
        let mut reply = [0u8; 8];
        for _ in 0..50 {
            let t0 = Instant::now();
            c.write_all(&[2u8; 8]).unwrap(); // a header ...
            c.write_all(&[3u8; 32]).unwrap(); // ... then the body, in a second write
            c.read_exact(&mut reply).unwrap();
            lat.push(t0.elapsed());
        }
        drop(c);
        h.join().unwrap();
        println!(
            "TCP_NODELAY = {nodelay:<5}  median {:>9.1} µs   max {:>9.1} µs",
            pct(&mut lat, 0.5).as_secs_f64() * 1e6,
            pct(&mut lat, 1.0).as_secs_f64() * 1e6
        );
    }
}
