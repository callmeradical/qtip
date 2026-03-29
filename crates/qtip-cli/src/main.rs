#[allow(dead_code)]
mod scenario;
mod pipeline;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use qtip_executor::executor::EvaluationStatus;

#[derive(Parser)]
#[command(name = "qtip", about = "Scenario evaluation platform")]
struct Cli {
    /// Path to the subject manifest JSON file
    manifest: PathBuf,

    /// Directory containing scenario YAML files
    #[arg(long, default_value = "./scenarios")]
    scenarios: PathBuf,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    // Load manifest
    let manifest_content = match std::fs::read_to_string(&cli.manifest) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read manifest {}: {e}", cli.manifest.display());
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

    // Load scenarios
    let scenarios = match pipeline::load_scenarios(&cli.scenarios) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to load scenarios: {e}");
            return ExitCode::FAILURE;
        }
    };

    // Resolve
    let resolved = pipeline::resolve_scenarios(&manifest, &scenarios);
    println!(
        "Evaluating Subject: {}",
        manifest.project_id
    );
    println!(
        "{}: Resolved {} scenarios for project {}",
        manifest.project_id,
        resolved.len(),
        manifest.project_id
    );
    println!();

    // Execute
    let subject_result = pipeline::execute_subject(&manifest, &resolved).await;

    // Report
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

    if subject_result.failed > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
