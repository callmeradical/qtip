use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use qtip_catalog::{
    FileSystemSource, FileSystemSourceConfig, ScenarioCatalog, StandardScenarioCatalog,
    StandardScenarioCatalogConfig,
};

const DEFAULT_FILE_COUNT: usize = 1_000;
const DEFAULT_ITERATIONS: usize = 5;
const DEFAULT_WARMUP: usize = 1;
const DEFAULT_TARGET_MS: f64 = 500.0;
const DEFAULT_CONCURRENCY: &[usize] = &[1, 8, 32, 100];
const DEFAULT_TEMPLATE_PATH: &str = "crates/qtip-catalog/benchmarks/fixture/scenario-template.yaml";

#[derive(Debug)]
struct CliConfig {
    files: usize,
    iterations: usize,
    warmup: usize,
    target_ms: f64,
    concurrency: Vec<usize>,
    template_path: PathBuf,
    report_path: Option<PathBuf>,
    machine: String,
}

#[derive(Debug, Clone)]
struct Sample {
    discover: Duration,
    load: Duration,
    total: Duration,
}

#[derive(Debug)]
struct Stats {
    avg_discover: Duration,
    avg_load: Duration,
    avg_total: Duration,
    p95_total: Duration,
    throughput_docs_per_sec: f64,
}

#[derive(Debug)]
struct BenchmarkCase {
    concurrency: usize,
    samples: Vec<Sample>,
    stats: Stats,
}

#[derive(Debug)]
struct BenchmarkRun {
    file_count: usize,
    iterations: usize,
    warmup: usize,
    target_ms: f64,
    machine: String,
    results: Vec<BenchmarkCase>,
    best_avg_total_ms: f64,
    target_met: bool,
}

fn main() -> ExitCode {
    let config = match parse_args(env::args().skip(1)) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            eprintln!("\n{}", usage());
            return ExitCode::from(1);
        }
    };

    let template = match fs::read_to_string(&config.template_path) {
        Ok(template) => template,
        Err(error) => {
            eprintln!(
                "failed to read benchmark fixture template '{}': {error}",
                config.template_path.display()
            );
            return ExitCode::from(1);
        }
    };

    let fixture = match FixtureDir::new() {
        Ok(fixture) => fixture,
        Err(error) => {
            eprintln!("failed to create benchmark fixture directory: {error}");
            return ExitCode::from(1);
        }
    };

    if let Err(error) = seed_fixture(&fixture.path, &template, config.files) {
        eprintln!("failed to create benchmark fixture files: {error}");
        return ExitCode::from(1);
    }

    let runtime_threads = std::thread::available_parallelism()
        .map(|parallelism| parallelism.get())
        .unwrap_or(4)
        .max(4);

    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .worker_threads(runtime_threads)
        .enable_time()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("failed to build tokio runtime for benchmark: {error}");
            return ExitCode::from(1);
        }
    };

    let mut results = Vec::with_capacity(config.concurrency.len());
    for concurrency in &config.concurrency {
        match run_case(
            &runtime,
            &fixture.path,
            config.files,
            *concurrency,
            config.iterations,
            config.warmup,
        ) {
            Ok(result) => results.push(result),
            Err(error) => {
                eprintln!("benchmark failed for concurrency {concurrency}: {error}");
                return ExitCode::from(1);
            }
        }
    }

    let best_avg_total_ms = results
        .iter()
        .map(|result| duration_to_ms(result.stats.avg_total))
        .fold(f64::INFINITY, f64::min);
    let target_met = best_avg_total_ms <= config.target_ms;

    let report = BenchmarkRun {
        file_count: config.files,
        iterations: config.iterations,
        warmup: config.warmup,
        target_ms: config.target_ms,
        machine: config.machine,
        results,
        best_avg_total_ms,
        target_met,
    };

    let markdown = render_report(&report);
    println!("{markdown}");

    if let Some(report_path) = config.report_path {
        if let Some(parent) = report_path.parent()
            && let Err(error) = fs::create_dir_all(parent)
        {
            eprintln!(
                "failed to create report directory '{}': {error}",
                parent.display()
            );
            return ExitCode::from(1);
        }
        if let Err(error) = fs::write(&report_path, &markdown) {
            eprintln!(
                "failed to write benchmark report '{}': {error}",
                report_path.display()
            );
            return ExitCode::from(1);
        }
    }

    if report.target_met {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(2)
    }
}

fn parse_args<I>(args: I) -> Result<CliConfig, String>
where
    I: Iterator<Item = String>,
{
    let mut files = DEFAULT_FILE_COUNT;
    let mut iterations = DEFAULT_ITERATIONS;
    let mut warmup = DEFAULT_WARMUP;
    let mut target_ms = DEFAULT_TARGET_MS;
    let mut concurrency = DEFAULT_CONCURRENCY.to_vec();
    let mut template_path = PathBuf::from(DEFAULT_TEMPLATE_PATH);
    let mut report_path = None;
    let mut machine = "unspecified baseline machine".to_string();

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Err("help requested".to_string()),
            "--files" => {
                files = parse_required_usize(args.next(), "--files")?;
            }
            "--iterations" => {
                iterations = parse_required_usize(args.next(), "--iterations")?;
            }
            "--warmup" => {
                warmup = parse_required_usize(args.next(), "--warmup")?;
            }
            "--target-ms" => {
                target_ms = parse_required_f64(args.next(), "--target-ms")?;
            }
            "--concurrency" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --concurrency".to_string())?;
                concurrency = parse_concurrency_list(&value)?;
            }
            "--fixture-template" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --fixture-template".to_string())?;
                template_path = PathBuf::from(value);
            }
            "--report" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value for --report".to_string())?;
                report_path = Some(PathBuf::from(value));
            }
            "--machine" => {
                machine = args
                    .next()
                    .ok_or_else(|| "missing value for --machine".to_string())?;
            }
            _ => return Err(format!("unsupported argument: {arg}")),
        }
    }

    if files == 0 {
        return Err("--files must be greater than zero".to_string());
    }
    if iterations == 0 {
        return Err("--iterations must be greater than zero".to_string());
    }
    if concurrency.is_empty() {
        return Err("--concurrency must include at least one value".to_string());
    }

    Ok(CliConfig {
        files,
        iterations,
        warmup,
        target_ms,
        concurrency,
        template_path,
        report_path,
        machine,
    })
}

fn parse_required_usize(value: Option<String>, flag: &str) -> Result<usize, String> {
    let value = value.ok_or_else(|| format!("missing value for {flag}"))?;
    value
        .parse::<usize>()
        .map_err(|error| format!("invalid value for {flag}: {error}"))
}

fn parse_required_f64(value: Option<String>, flag: &str) -> Result<f64, String> {
    let value = value.ok_or_else(|| format!("missing value for {flag}"))?;
    value
        .parse::<f64>()
        .map_err(|error| format!("invalid value for {flag}: {error}"))
}

fn parse_concurrency_list(value: &str) -> Result<Vec<usize>, String> {
    let mut parsed = Vec::new();
    for entry in value.split(',') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        let number = entry
            .parse::<usize>()
            .map_err(|error| format!("invalid concurrency value '{entry}': {error}"))?;
        if number == 0 {
            return Err("concurrency values must be greater than zero".to_string());
        }
        parsed.push(number);
    }

    if parsed.is_empty() {
        return Err("--concurrency must include at least one positive integer".to_string());
    }

    parsed.sort_unstable();
    parsed.dedup();
    Ok(parsed)
}

fn run_case(
    runtime: &tokio::runtime::Runtime,
    fixture_root: &Path,
    file_count: usize,
    concurrency: usize,
    iterations: usize,
    warmup: usize,
) -> Result<BenchmarkCase, String> {
    let total_runs = warmup + iterations;
    let mut samples = Vec::with_capacity(iterations);

    for run_index in 0..total_runs {
        let source = build_source(fixture_root)?;
        let catalog = StandardScenarioCatalog::new(
            source,
            StandardScenarioCatalogConfig {
                io_concurrency_limit: concurrency,
            },
        )
        .map_err(|error| format!("catalog config rejected benchmark case: {error}"))?;

        let discover_start = Instant::now();
        let discovered = catalog.discover();
        let discover_elapsed = discover_start.elapsed();
        if !discovered.errors.is_empty() {
            return Err(format!(
                "discovery returned {} errors",
                discovered.errors.len()
            ));
        }
        if discovered.items.len() != file_count {
            return Err(format!(
                "discovery returned {} files, expected {file_count}",
                discovered.items.len()
            ));
        }

        let load_start = Instant::now();
        let loaded = runtime.block_on(catalog.load_scenarios(&discovered.items));
        let load_elapsed = load_start.elapsed();

        if !loaded.errors.is_empty() {
            return Err(format!("load returned {} errors", loaded.errors.len()));
        }
        if loaded.items.len() != file_count {
            return Err(format!(
                "load returned {} scenarios, expected {file_count}",
                loaded.items.len()
            ));
        }

        if run_index >= warmup {
            samples.push(Sample {
                discover: discover_elapsed,
                load: load_elapsed,
                total: discover_elapsed + load_elapsed,
            });
        }
    }

    let stats = compute_stats(&samples, file_count);
    Ok(BenchmarkCase {
        concurrency,
        samples,
        stats,
    })
}

fn build_source(fixture_root: &Path) -> Result<FileSystemSource, String> {
    let mut config = FileSystemSourceConfig::new("bench-filesystem");
    config.roots = vec![fixture_root.to_path_buf()];
    config.include_patterns = vec!["**/*.yaml".to_string()];
    config.exclude_patterns = Vec::new();
    config.ignore_patterns = Vec::new();
    config.respect_gitignore = false;

    FileSystemSource::new(config).map_err(|error| format!("invalid source config: {error}"))
}

fn compute_stats(samples: &[Sample], file_count: usize) -> Stats {
    let discover = samples
        .iter()
        .map(|sample| sample.discover)
        .collect::<Vec<_>>();
    let load = samples.iter().map(|sample| sample.load).collect::<Vec<_>>();
    let total = samples
        .iter()
        .map(|sample| sample.total)
        .collect::<Vec<_>>();

    let avg_total = average_duration(&total);
    let throughput_docs_per_sec = if avg_total.is_zero() {
        0.0
    } else {
        file_count as f64 / avg_total.as_secs_f64()
    };

    Stats {
        avg_discover: average_duration(&discover),
        avg_load: average_duration(&load),
        avg_total,
        p95_total: percentile_duration(&total, 95),
        throughput_docs_per_sec,
    }
}

fn average_duration(durations: &[Duration]) -> Duration {
    if durations.is_empty() {
        return Duration::ZERO;
    }

    let total_nanos = durations
        .iter()
        .map(|duration| duration.as_nanos())
        .sum::<u128>();
    let average_nanos = total_nanos / durations.len() as u128;
    let clamped = average_nanos.min(u64::MAX as u128) as u64;
    Duration::from_nanos(clamped)
}

fn percentile_duration(durations: &[Duration], percentile: usize) -> Duration {
    if durations.is_empty() {
        return Duration::ZERO;
    }

    let mut sorted = durations.to_vec();
    sorted.sort_unstable();

    let numerator = percentile.saturating_mul(sorted.len());
    let mut index = numerator.div_ceil(100);
    index = index.saturating_sub(1);
    sorted[index.min(sorted.len() - 1)]
}

fn duration_to_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}

fn render_report(run: &BenchmarkRun) -> String {
    let timestamp = timestamp_string();
    let status_label = if run.target_met { "PASS" } else { "REGRESSION" };

    let mut report = String::new();
    report.push_str("# Rust Catalog Benchmark Report\n\n");
    report.push_str(&format!("- Generated: {timestamp}\n"));
    report.push_str("- Story: US-008\n");
    report.push_str("- Path measured: deep discovery (`FileSystemSource::discover`) + load/parse/validate (`StandardScenarioCatalog::load_scenarios`)\n");
    report.push_str("- Build profile: `--release`\n");
    report.push_str(&format!(
        "- Fixture size: {} valid YAML files\n",
        run.file_count
    ));
    report.push_str(&format!("- Warmup runs per concurrency: {}\n", run.warmup));
    report.push_str(&format!(
        "- Measured runs per concurrency: {}\n",
        run.iterations
    ));
    report.push_str(&format!(
        "- Baseline machine assumptions: {}\n",
        run.machine
    ));
    report.push_str(&format!(
        "- Target: <= {:.1}ms for {} valid files\n\n",
        run.target_ms, run.file_count
    ));

    report.push_str("## Observed Results\n\n");
    report.push_str("| Concurrency | Samples | Avg Discover (ms) | Avg Load (ms) | Avg Total (ms) | P95 Total (ms) | Throughput (docs/sec) |\n");
    report.push_str("| ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");

    for result in &run.results {
        report.push_str(&format!(
            "| {} | {} | {:.2} | {:.2} | {:.2} | {:.2} | {:.0} |\n",
            result.concurrency,
            result.samples.len(),
            duration_to_ms(result.stats.avg_discover),
            duration_to_ms(result.stats.avg_load),
            duration_to_ms(result.stats.avg_total),
            duration_to_ms(result.stats.p95_total),
            result.stats.throughput_docs_per_sec,
        ));
    }

    report.push('\n');
    report.push_str(&format!(
        "**Result:** {status_label} (best average total latency {:.2}ms vs target {:.1}ms).\n\n",
        run.best_avg_total_ms, run.target_ms
    ));

    if run.target_met {
        report.push_str("Target met on baseline hardware assumptions.\n");
    } else {
        report.push_str("Target missed. Regression is flagged for this run.\n\n");
        report.push_str("## Profile Hints\n\n");
        report.push_str("- Re-run per-concurrency sweep to isolate whether discovery or load stage dominates (`scripts/benchmark-rust-catalog.sh`).\n");
        report.push_str("- Capture CPU profile for the slowest case and compare with fastest case to identify hot spots in parsing/validation.\n");
        report.push_str("- Verify benchmark runs on local SSD and without background indexing, then repeat to rule out environmental noise.\n");
    }

    report
}

fn timestamp_string() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => format!("unix:{}", duration.as_secs()),
        Err(_) => "unix:0".to_string(),
    }
}

fn seed_fixture(root: &Path, template: &str, files: usize) -> Result<(), String> {
    for index in 0..files {
        let scenario_id = format!("scenario-{index:04}");
        let group = index % 20;
        let service = (index / 20) % 10;
        let relative = PathBuf::from(format!(
            "domain-{group}/service-{service}/case-{index:04}.yaml"
        ));
        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("create fixture directory failed: {error}"))?;
        }

        let contents = template
            .replace("{{id}}", &scenario_id)
            .replace("{{index}}", &index.to_string());
        fs::write(path, contents)
            .map_err(|error| format!("write benchmark fixture file failed: {error}"))?;
    }

    Ok(())
}

fn usage() -> String {
    format!(
        "benchmark_catalog options:\n\
  --files <count>            Number of fixture files (default: {DEFAULT_FILE_COUNT})\n\
  --iterations <count>       Measured runs per concurrency (default: {DEFAULT_ITERATIONS})\n\
  --warmup <count>           Warmup runs per concurrency (default: {DEFAULT_WARMUP})\n\
  --target-ms <ms>           Latency target in milliseconds (default: {DEFAULT_TARGET_MS})\n\
  --concurrency <list>       Comma-separated io_concurrency_limit values (default: 1,8,32,100)\n\
  --fixture-template <path>  Fixture template YAML path\n\
  --report <path>            Optional report output path\n\
  --machine <text>           Baseline machine assumption text\n\
  --help                     Show this message"
    )
}

#[derive(Debug)]
struct FixtureDir {
    path: PathBuf,
}

impl FixtureDir {
    fn new() -> Result<Self, String> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let path = env::temp_dir().join(format!(
            "qtip-catalog-bench-{}-{timestamp}",
            std::process::id()
        ));
        fs::create_dir_all(&path).map_err(|error| {
            format!(
                "failed to create fixture temp dir '{}': {error}",
                path.display()
            )
        })?;
        Ok(Self { path })
    }
}

impl Drop for FixtureDir {
    fn drop(&mut self) {
        if self.path.exists() {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
