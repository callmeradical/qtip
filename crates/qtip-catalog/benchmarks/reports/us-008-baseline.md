# Rust Catalog Benchmark Report

- Generated: unix:1775368762
- Story: US-008
- Path measured: deep discovery (`FileSystemSource::discover`) + load/parse/validate (`StandardScenarioCatalog::load_scenarios`)
- Build profile: `--release`
- Fixture size: 1000 valid YAML files
- Warmup runs per concurrency: 1
- Measured runs per concurrency: 5
- Baseline machine assumptions: OS Darwin 25.4.0 arm64; CPU Apple M1 Pro; Cores 10; Memory 16.0 GiB; Local SSD assumed
- Target: <= 500.0ms for 1000 valid files

## Observed Results

| Concurrency | Samples | Avg Discover (ms) | Avg Load (ms) | Avg Total (ms) | P95 Total (ms) | Throughput (docs/sec) |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 5 | 6.38 | 22.81 | 29.19 | 29.67 | 34258 |
| 8 | 5 | 6.52 | 13.03 | 19.55 | 19.96 | 51148 |
| 32 | 5 | 6.50 | 13.45 | 19.94 | 20.25 | 50139 |
| 100 | 5 | 6.54 | 13.37 | 19.91 | 20.14 | 50235 |

**Result:** PASS (best average total latency 19.55ms vs target 500.0ms).

Target met on baseline hardware assumptions.
