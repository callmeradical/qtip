#[allow(dead_code)]
mod scenario;
mod cache;
mod pipeline;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use qtip_executor::executor::EvaluationStatus;

#[derive(Parser)]
#[command(name = "qtip", about = "Scenario evaluation platform")]
struct Cli {
    /// Path to the subject manifest JSON file (default: auto-discover in current directory)
    manifest: Option<PathBuf>,

    /// Local directory containing scenario YAML files (overrides manifest)
    #[arg(long)]
    scenarios: Option<PathBuf>,

    /// GitHub repo containing scenarios (overrides manifest)
    #[arg(long)]
    repo: Option<String>,

    /// Subdirectory within the repo (overrides manifest)
    #[arg(long)]
    path: Option<String>,
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

    // Discover manifest
    let manifest_path = match cli.manifest.or_else(discover_manifest) {
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
