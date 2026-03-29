# Rust Catalog Performance Envelope

This document tracks repeatable benchmark evidence for the Rust catalog deep discovery and load path.

## Benchmark Fixture + Measurement Script

- Fixture template: `crates/qtip-catalog/benchmarks/fixture/scenario-template.yaml`
- Measurement script: `scripts/benchmark-rust-catalog.sh`
- Baseline report artifact: `crates/qtip-catalog/benchmarks/reports/us-008-baseline.md`

The script generates a deep directory fixture, measures:

1. `FileSystemSource::discover`
2. `StandardScenarioCatalog::load_scenarios`

and emits a markdown report with per-concurrency latency/throughput and pass/fail status against the target.

## Baseline Machine Assumptions

Baseline run used:

- OS: Darwin 25.1.0 arm64
- CPU: Apple M1 Pro
- Logical cores: 10
- Memory: 16.0 GiB
- Storage assumption: local SSD
- Build profile: `--release`

## Observed Throughput/Latency (1000 valid files)

From `crates/qtip-catalog/benchmarks/reports/us-008-baseline.md`:

| Concurrency | Avg Discover (ms) | Avg Load (ms) | Avg Total (ms) | P95 Total (ms) | Throughput (docs/sec) |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 6.54 | 23.54 | 30.09 | 30.58 | 33239 |
| 8 | 6.35 | 12.15 | 18.51 | 18.65 | 54034 |
| 32 | 6.21 | 12.65 | 18.86 | 19.00 | 53032 |
| 100 | 6.43 | 13.61 | 20.04 | 22.62 | 49894 |

Result: PASS, best average total latency `18.51ms`, under the `<=500ms` target for 1000 valid files.

## Regression Handling

`scripts/benchmark-rust-catalog.sh` exits with code `2` when the target is missed.

When that happens, the generated report is flagged as `REGRESSION` and includes profile hints:

- Separate discovery vs load bottleneck by comparing per-stage timings across concurrency values.
- Capture CPU profile for slow cases and compare to fastest case.
- Re-run on stable local SSD conditions to rule out environmental noise.

## Run It

```bash
scripts/benchmark-rust-catalog.sh
```

Optional overrides are available via environment variables:

- `QTIP_CATALOG_BENCH_FILES`
- `QTIP_CATALOG_BENCH_ITERATIONS`
- `QTIP_CATALOG_BENCH_WARMUP`
- `QTIP_CATALOG_BENCH_TARGET_MS`
- `QTIP_CATALOG_BENCH_CONCURRENCY`
- `QTIP_CATALOG_BENCH_REPORT`
