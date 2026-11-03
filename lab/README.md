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

Timings depend on your machine. The articles say what hardware produced theirs; compare shapes and
ratios, not absolute numbers.
