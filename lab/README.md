# Query Execution lab

The Rust code behind the guide at <https://unikdahal.github.io/learning-guide/>. Every number and every
query output quoted in an article comes from one of the programs here, so you can rerun it and check.

The full setup walkthrough is the article
[Setting up the Rust lab](https://unikdahal.github.io/learning-guide/orientation/setting-up-the-rust-lab/).
The short version:

```sh
cd lab
cargo run --release -p labkit --bin gen-data                 # writes data/ (customer, orders, trips)
cargo run --release -p m01-relational --example nulls        # run any example below
cargo test                                                   # the law tests of the tiny engines
```

`rust-toolchain.toml` pins the compiler, and rustup installs it on first use. The workspace pins one
version of DataFusion (55.2) for every crate, so plans and outputs match the articles.

## Layout

| Crate | What it holds |
| --- | --- |
| `labkit` | Shared helpers: a deterministic RNG, the data generator, timing with summary statistics |
| `m00-orientation` | Module 00: embedding DataFusion, query stages, row vs column, honest measurement |
| `m01-relational` | Module 01: a tiny relational algebra engine (`Relation`), a bag engine (`Bag`), three-valued logic (`Truth`), and DataFusion comparisons |
| `m02-hardware` | Module 02: kernels that expose the machine: pointer chasing, scans, strides, gathers, branchy and branch-free filters, huge pages, direct disk reads, loopback TCP, perf_event_open, a roofline sweep |
| `m03-representation` | Module 03: row and column layouts of the same table, Arrow buffers and IPC, and a pyarrow reader in `python/` |

Each article's code is an example in its module crate's `examples/` folder. Shared code lives in the crate's
`src/`. Benchmarks use Criterion and live in `benches/`.

## Examples by article

| Article | Command |
| --- | --- |
| What a query engine actually does | `cargo run --release -p m00-orientation --example query_stages` |
| OLTP vs OLAP workloads | `cargo run --release -p m00-orientation --example row_vs_column` |
| A tour of real engines | `cargo run --release -p m00-orientation --example embed_datafusion` |
| Measuring things honestly | `cargo run --release -p m00-orientation --example measure_query` and `cargo bench -p m00-orientation` |
| Relations, tuples and attributes | `cargo run --release -p m01-relational --example relations` |
| Relational algebra: selection, projection, rename | `cargo run --release -p m01-relational --example algebra_ops` |
| Joins in relational algebra | `cargo run --release -p m01-relational --example joins` |
| Set operations and bag semantics | `cargo run --release -p m01-relational --example set_ops` |
| Grouping and aggregation semantics | `cargo run --release -p m01-relational --example grouping` |
| NULL and three-valued logic | `cargo run --release -p m01-relational --example nulls` |
| Logical processing order of a query | `cargo run --release -p m01-relational --example clause_order` |
| ORDER BY, LIMIT and determinism | `cargo run --release -p m01-relational --example order_limit` |
| Data types and their traps | `cargo run --release -p m01-relational --example types` |
| Subqueries | `cargo run --release -p m01-relational --example subqueries` |
| Window function semantics | `cargo run --release -p m01-relational --example windows` |
| CTEs and recursive queries | `cargo run --release -p m01-relational --example recursive` |
| From SQL to relational algebra by hand | `cargo run --release -p m01-relational --example sql_to_algebra` |
| Equivalence rules of relational algebra | `cargo run --release -p m01-relational --example equivalences` |
| Duplicate-sensitive and order-sensitive operators | `cargo run --release -p m01-relational --example dup_order` |
| The memory hierarchy, from registers to object storage | `cargo run --release -p m02-hardware --example latency`, then `--example tlb` and `--example storage` |
| CPU caches, cache lines and locality | `cargo run --release -p m02-hardware --example caches` |
| Branch prediction and why branches hurt | `cargo run --release -p m02-hardware --example branches` |
| Instruction-level parallelism | `cargo run --release -p m02-hardware --example ilp` |
| SIMD: one instruction, many values | `cargo run --release -p m02-hardware --example simd` |
| SIMD in Rust | `cargo run --release -p m02-hardware --example simd`, then `objdump -d --no-show-raw-insn target/release/examples/simd` |
| TLBs, pages and huge pages | `cargo run --release -p m02-hardware --example pages` (and `--example tlb`) |
| Prefetching and memory bandwidth | `cargo run --release -p m02-hardware --example prefetch` |
| Multicore, threads and false sharing | `cargo run --release -p m02-hardware --example multicore` |
| Atomics and memory ordering for engine developers | `cargo run --release -p m02-hardware --example atomics` |
| Disks and SSDs | `cargo run --release -p m02-hardware --example disk` (writes and deletes a 1 GiB file) |
| Networks for data movement | `cargo run --release -p m02-hardware --example network` (loopback TCP only) |
| Measuring hardware with perf counters | `cargo run --release -p m02-hardware --example perfcounters` (Linux; hardware events need a PMU) |
| The roofline model for query operators | `cargo run --release -p m02-hardware --example roofline` (x86-64 with AVX-512 and FMA) |
| Row vs column layout | `cargo run --release -p m03-representation --example layout` |
| Apache Arrow: the columnar in-memory standard | `cargo run --release -p m03-representation --example arrow_ipc`, then `pip install pyarrow` and `python3 crates/m03-representation/python/read_arrow.py` |

The Module 02 examples measure the hardware, so run them one at a time on an otherwise idle machine; `storage` writes a 1 GiB file under `data/`. Timings depend on your machine. The articles say what hardware produced theirs; compare shapes and
ratios, not absolute numbers.
