use std::io::{self, Write};

fn prompt(label: &str, default: &str) -> String {
    if default.is_empty() {
        print!("{label}: ");
    } else {
        print!("{label} [{default}]: ");
    }
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    let input = input.trim().to_string();
    if input.is_empty() { default.to_string() } else { input }
}

fn prompt_list(label: &str, default: &str) -> Vec<String> {
    let raw = prompt(label, default);
    raw.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn run_init() -> Result<(), String> {
    let cwd = std::env::current_dir()
        .map_err(|e| format!("Could not get current directory: {e}"))?;
    let dir_name = cwd
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-project");

    // Check if manifest already exists
    for name in ["qtip-manifest.json", "qtip.json"] {
        if cwd.join(name).exists() {
            return Err(format!("{name} already exists in this directory"));
        }
    }

    println!("Initializing qtip manifest in {}", cwd.display());
    println!();

    let project_id = prompt("Project name", dir_name);
    let environment = prompt("Environment", "ci");
    let interfaces = prompt_list("Interfaces (comma-separated)", "cli");
    let capabilities = prompt_list("Capabilities (comma-separated)", "build");
    let scenarios_repo = prompt("Scenarios repo (e.g. owner/scenarios)", "");
    let scenarios_path = if !scenarios_repo.is_empty() {
        prompt("Scenarios path within repo", &project_id)
    } else {
        String::new()
    };

    // Build manifest
    let interfaces_json: Vec<serde_json::Value> = interfaces
        .iter()
        .map(|i| serde_json::Value::String(i.clone()))
        .collect();

    let mut manifest = serde_json::Map::new();
    manifest.insert("projectId".to_string(), serde_json::json!(project_id));
    manifest.insert("environment".to_string(), serde_json::json!(environment));
    manifest.insert("interfaces".to_string(), serde_json::json!(interfaces_json));
    manifest.insert("capabilities".to_string(), serde_json::json!(capabilities));

    if !scenarios_repo.is_empty() {
        let mut scenarios = serde_json::Map::new();
        scenarios.insert("repo".to_string(), serde_json::json!(scenarios_repo));
        if !scenarios_path.is_empty() {
            scenarios.insert("path".to_string(), serde_json::json!(scenarios_path));
        }
        manifest.insert("scenarios".to_string(), serde_json::Value::Object(scenarios));
    }

    let json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("Failed to serialize manifest: {e}"))?;

    let output_path = cwd.join("qtip-manifest.json");
    std::fs::write(&output_path, format!("{json}\n"))
        .map_err(|e| format!("Failed to write {}: {e}", output_path.display()))?;

    println!();
    println!("Wrote {}", output_path.display());
    println!();
    println!("{json}");

    if scenarios_repo.is_empty() {
        println!();
        println!("Tip: add a scenarios repo later:");
        println!("  \"scenarios\": {{ \"repo\": \"owner/scenarios\", \"path\": \"{}\" }}", project_id);
    }

    Ok(())
}
