//! Generates the Module 00 data set into `lab/data/`.
//!
//!     cargo run --release -p labkit --bin gen-data            # TPC-H scale factor 1 sizes
//!     cargo run --release -p labkit --bin gen-data -- 0.1     # a tenth of that

use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scale: f64 = std::env::args()
        .nth(1)
        .map(|s| s.parse())
        .transpose()?
        .unwrap_or(1.0);
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let t0 = std::time::Instant::now();
    labkit::data::generate(&dir, scale)?;
    println!(
        "wrote customer.parquet, orders.parquet and trips.csv (scale {scale}) to {} in {:.1?}",
        dir.canonicalize()?.display(),
        t0.elapsed()
    );
    Ok(())
}
