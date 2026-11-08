//! Shared helpers for the Query Execution lab.
//!
//! - [`rng`]: a tiny deterministic random number generator, so generated data is identical on
//!   every machine and every run.
//! - `data` (with the datafusion feature): generators for the `orders`, `customer` and `trips` tables used in Module 00.
//! - [`timing`]: wall-clock and CPU-time measurement with summary statistics.

#[cfg(feature = "datafusion")]
pub mod data;
pub mod rng;
pub mod timing;
