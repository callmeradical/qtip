#[allow(dead_code)]
mod scenario;
mod cache;
mod init;
mod install;
mod pipeline;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use qtip_executor::executor::EvaluationStatus;
use serde::Serialize;

#[derive(Parser)]
#[command(name = "qtip", about = "Scenario evaluation platform")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to the subject manifest JSON file (default: auto-discover)
    #[arg(global = true)]
    manifest: Option<PathBuf>,

    /// Local directory containing scenario YAML files (overrides manifest)
    #[arg(long, global = true)]
    scenarios: Option<PathBuf>,

    /// GitHub repo containing scenarios (overrides manifest)
    #[arg(long, global = true)]
    repo: Option<String>,

    /// Subdirectory within the repo (overrides manifest)
    #[arg(long, global = true)]
    path: Option<String>,

    /// Output format
    #[arg(long, global = true, default_value = "text")]
    output: OutputFormat,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Serialize)]
struct JsonReport {
    project_id: String,
    status: String,
    passed: usize,
    failed: usize,
    total: usize,
    results: Vec<JsonResult>,
}

#[derive(Serialize)]
struct JsonResult {
    id: String,
    name: String,
    status: String,
    failures: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a qtip-manifest.json in the current directory
    Init,
    /// Install qtip skills for your AI coding agent
    Install {
        #[command(subcommand)]
        what: InstallCommands,
    },
}

#[derive(Subcommand)]
enum InstallCommands {
    /// Install the qtip-scenarios skill for generating scenarios from PRDs
    Skill,
}

/// Look for a manifest file in the current directory.
fn discover_manifest() -> Option<PathBuf> {
    let candidates = [
        "qtip-manifest.json",
        "qtip.json",
        "manifest.json",
    ];
    for name in candidates {
        let path = Path::new(name);
        if path.exists() {
            return Some(path.to_path_buf());
        }
    }
    None
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    // Handle subcommands
    if let Some(command) = &cli.command {
        return match command {
            Commands::Init => {
                match init::run_init() {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("Error: {e}");
                        ExitCode::FAILURE
                    }
                }
            }
            Commands::Install { what } => match what {
                InstallCommands::Skill => {
                    match install::install_skill() {
                        Ok(()) => ExitCode::SUCCESS,
                        Err(e) => {
                            eprintln!("Error: {e}");
                            ExitCode::FAILURE
                        }
                    }
                }
            },
        };
    }

    // Default: run evaluation
    run_evaluate(&cli).await
}

async fn run_evaluate(cli: &Cli) -> ExitCode {
    // Discover manifest
    let manifest_path = match cli.manifest.clone().or_else(discover_manifest) {
        Some(p) => p,
        None => {
            eprintln!("No manifest found. Looked for qtip-manifest.json, qtip.json, manifest.json");
            eprintln!("Usage: qtip [manifest.json]");
            return ExitCode::FAILURE;
        }
    };

    // Load manifest
    let manifest_content = match std::fs::read_to_string(&manifest_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read manifest {}: {e}", manifest_path.display());
            return ExitCode::FAILURE;
        }
    };
    let manifest: scenario::SubjectManifest = match serde_json::from_str(&manifest_content) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to parse manifest: {e}");
            return ExitCode::FAILURE;
        }
    };

    // Resolve scenarios directory: CLI flags > manifest config > error
    let scenarios_dir = if let Some(dir) = &cli.scenarios {
        dir.clone()
    } else if let Some(repo) = cli.repo.as_deref().or_else(|| {
        manifest.scenarios.as_ref().and_then(|s| s.repo.as_deref())
    }) {
        let path = cli.path.as_deref().or_else(|| {
            manifest.scenarios.as_ref().and_then(|s| s.path.as_deref())
        }).unwrap_or(".");

        match cache::sync_repo(repo) {
            Ok(cached) => cached.join(path),
            Err(e) => {
                eprintln!("Failed to sync scenarios repo: {e}");
                return ExitCode::FAILURE;
            }
        }
    } else if let Some(local) = manifest.scenarios.as_ref().and_then(|s| s.local.as_deref()) {
        PathBuf::from(local)
    } else {
        eprintln!("No scenarios source configured. Add to your manifest:");
        eprintln!("  \"scenarios\": {{ \"repo\": \"owner/repo\", \"path\": \"my-project\" }}");
        return ExitCode::FAILURE;
    };

    // Load scenarios
    let scenarios = match pipeline::load_scenarios(&scenarios_dir) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to load scenarios: {e}");
            return ExitCode::FAILURE;
        }
    };

    // Resolve
    let resolved = pipeline::resolve_scenarios(&manifest, &scenarios);

    let is_json = matches!(cli.output, OutputFormat::Json);

    if !is_json {
        println!("Evaluating Subject: {}", manifest.project_id);
        println!(
            "{}: Resolved {} scenarios for project {}",
            manifest.project_id,
            resolved.len(),
            manifest.project_id
        );
        println!();
    }

    // Execute
    let subject_result = pipeline::execute_subject(&manifest, &resolved).await;

    if is_json {
        let report = JsonReport {
            project_id: subject_result.project_id.clone(),
            status: if subject_result.failed == 0 {
                "passed".to_string()
            } else {
                "failed".to_string()
            },
            passed: subject_result.passed,
            failed: subject_result.failed,
            total: subject_result.results.len(),
            results: subject_result
                .results
                .iter()
                .map(|r| JsonResult {
                    id: r.scenario_id.clone(),
                    name: r.scenario_id.clone(),
                    status: match r.status {
                        EvaluationStatus::Passed => "passed",
                        EvaluationStatus::Failed => "failed",
                        EvaluationStatus::Error => "error",
                    }
                    .to_string(),
                    failures: r.failures.clone(),
                })
                .collect(),
        };
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        for result in &subject_result.results {
            let icon = match result.status {
                EvaluationStatus::Passed => "PASSED",
                EvaluationStatus::Failed => "FAILED",
                EvaluationStatus::Error => "ERROR",
            };
            println!("  - {}: {} ... {icon}", result.scenario_id, result.scenario_id);

            for failure in &result.failures {
                println!("      - {failure}");
            }
        }

        println!();
        println!(
            "Summary ({}): {} passed, {} failed, {} total.",
            subject_result.project_id,
            subject_result.passed,
            subject_result.failed,
            subject_result.results.len(),
        );
    }

    if subject_result.failed > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
