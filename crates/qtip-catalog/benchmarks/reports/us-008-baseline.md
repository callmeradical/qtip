# Rust Catalog Benchmark Report

- Generated: unix:1774761702
- Story: US-008
- Path measured: deep discovery (`FileSystemSource::discover`) + load/parse/validate (`StandardScenarioCatalog::load_scenarios`)
- Build profile: `--release`
- Fixture size: 1000 valid YAML files
- Warmup runs per concurrency: 1
- Measured runs per concurrency: 5
- Baseline machine assumptions: OS Darwin 25.1.0 arm64; CPU Apple M1 Pro; Cores 10; Memory 16.0 GiB; Local SSD assumed
- Target: <= 500.0ms for 1000 valid files

## Observed Results

| Concurrency | Samples | Avg Discover (ms) | Avg Load (ms) | Avg Total (ms) | P95 Total (ms) | Throughput (docs/sec) |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 5 | 6.54 | 23.54 | 30.09 | 30.58 | 33239 |
| 8 | 5 | 6.35 | 12.15 | 18.51 | 18.65 | 54034 |
| 32 | 5 | 6.21 | 12.65 | 18.86 | 19.00 | 53032 |
| 100 | 5 | 6.43 | 13.61 | 20.04 | 22.62 | 49894 |

**Result:** PASS (best average total latency 18.51ms vs target 500.0ms).

Target met on baseline hardware assumptions.
