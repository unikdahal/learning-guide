# Reproducible performance evidence

Correctness is deterministic; speed depends on the compiler, data, hardware, cache state and competing
work. Run measurements in release mode, one at a time. Keep raw samples and exact answer checks separate
from the timed interval. Record whether parsing/planning, decoding, allocation and result collection are included.

```sh
python3 scripts/capture-benchmark.py --question 'row versus column sum' --cache-state warm \
  --output benchmarks/runs/layout.json -- cargo bench --manifest-path lab/Cargo.toml -p m00-orientation --bench sum_layout
```

The recorder stores the exact argument vector, commit/dirty state, compiler version, OS/CPU/memory,
dataset manifest, stdout/stderr, exit status and elapsed time. Helpers using `repeat`/`repeat_async`
also export raw nanosecond samples when the recorder sets `QE_BENCH_SAMPLES`. Programs with manual
sampling must call `record_samples` after the timed loop or include their own raw data artifact.
For Criterion, retain its `target/criterion` data alongside the record. Generated runs are ignored by Git;
review and add selected evidence deliberately. Do not capture credentials or sensitive query data.

Use at least 20 measured warm runs or Criterion's sampling procedure. Interleave/randomize old and new
versions, repeat comparisons, and state practical effect size and uncertainty. A coefficient of variation
is not a significance threshold. A first run is not proven cold-cache unless cache state was controlled.
Never assert historical milliseconds as a CI pass condition. Never compare debug-build results to release numbers.

## Historical evidence boundary

Tables already in the published guide are historical observations. Their example programs and pinned
compiler are present, but original raw samples and complete host metadata were not committed for every
measurement. Where the original text says only “2-core cloud VM”, the exact CPU/OS/memory remain unknown.
Do not fill those fields from a different machine or relabel a new run as the old measurement.

The original join comparison used 15 warm runs; current code uses 20. The original row/column comparison
and Criterion excerpts use different input sizes. Orientation's simulated trial percentages are historical
unseeded runs and cannot be reproduced exactly; the interactive simulation illustrates uncertainty rather
than guaranteeing a particular percentage. Record seeded offline experiments before publishing new exact
simulation-frequency claims. New hardware/representation measurements should link a complete record when
available; absent a record, describe them as historical observations rather than fully reproducible evidence.

Generator version 2 corrects trips fares to round(100 × (2.50 + 1.80 × distance_km)) / 100. Customers/orders
and scale-1 revenue totals are unchanged. `lab/data/generator.json` records version, scale, counts and chunk
size. Logical rows are repeatable for that contract; encoded bytes can change with Arrow/Parquet upgrades.
