use std::path::Path;

use qtip_executor::check::{Check, CheckType};
use qtip_executor::executor::{
    EvaluationResult, EvaluationStatus, ExecutableScenario,
    Interaction as ExecInteraction, ScenarioExecutor,
};
use qtip_resolver::{AppliesTo, ScenarioManifest, ScenarioResolver, SubjectInterface, SubjectQuery};

use crate::scenario::{ScenarioFile, SubjectManifest};

#[derive(Debug)]
pub struct SubjectResult {
    pub project_id: String,
    pub results: Vec<EvaluationResult>,
    pub passed: usize,
    pub failed: usize,
}

/// Load all YAML scenario files from a directory.
pub fn load_scenarios(dir: &Path) -> Result<Vec<ScenarioFile>, String> {
    let mut scenarios = Vec::new();
    load_scenarios_recursive(dir, &mut scenarios)?;
    scenarios.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(scenarios)
}

fn load_scenarios_recursive(dir: &Path, out: &mut Vec<ScenarioFile>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {e}", dir.display()))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {e}"))?;
        let path = entry.path();

        if path.is_dir() {
            load_scenarios_recursive(&path, out)?;
        } else if path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
            let scenario: ScenarioFile = serde_yaml::from_str(&content)
                .map_err(|e| format!("Failed to parse {}: {e}", path.display()))?;
            out.push(scenario);
        }
    }

    Ok(())
}

/// Resolve which scenarios apply to this manifest.
pub fn resolve_scenarios<'a>(
    manifest: &SubjectManifest,
    scenarios: &'a [ScenarioFile],
) -> Vec<&'a ScenarioFile> {
    let resolver_scenarios: Vec<ScenarioManifest> = scenarios
        .iter()
        .map(|s| ScenarioManifest {
            id: s.id.clone(),
            name: s.name.clone(),
            applies_to: AppliesTo {
                capabilities: s.applies_to.capabilities.clone(),
                interfaces: s.applies_to.interfaces.clone(),
                environments: s.applies_to.environments.clone(),
            },
        })
        .collect();

    let resolver = ScenarioResolver::new(resolver_scenarios);
    let query = SubjectQuery {
        capabilities: manifest.capabilities.clone(),
        interfaces: manifest
            .interfaces
            .iter()
            .map(|i| SubjectInterface {
                interface_type: i.interface_type.clone(),
                name: i.name.clone(),
            })
            .collect(),
        environment: manifest.environment.clone(),
    };

    let matched = resolver.resolve(&query);
    let matched_ids: Vec<&str> = matched.iter().map(|m| m.id.as_str()).collect();

    scenarios
        .iter()
        .filter(|s| matched_ids.contains(&s.id.as_str()))
        .collect()
}

/// Convert a ScenarioFile + manifest context into an ExecutableScenario.
fn to_executable(scenario: &ScenarioFile, manifest: &SubjectManifest) -> ExecutableScenario {
    let mut params: std::collections::HashMap<String, serde_json::Value> = scenario
        .interaction
        .params
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    // For API interactions, resolve the full URL from manifest
    if scenario.interaction.interaction_type == "api" {
        if let Some(request) = scenario.interaction.params.get("request") {
            if let Some(path) = request.get("path").and_then(|p| p.as_str()) {
                let service = scenario
                    .interaction
                    .params
                    .get("service")
                    .and_then(|s| s.as_str());

                let base_url = manifest
                    .interfaces
                    .iter()
                    .find(|i| {
                        i.interface_type == "api"
                            && (service.is_none() || i.name.as_deref() == service)
                    })
                    .and_then(|i| i.base_url.as_deref())
                    .unwrap_or("http://localhost");

                let url = format!("{base_url}{path}");
                params.insert("url".to_string(), serde_json::Value::String(url));

                if let Some(method) = request.get("method") {
                    params.insert("method".to_string(), method.clone());
                }
                if let Some(body) = request.get("body") {
                    params.insert("body".to_string(), body.clone());
                }
                if let Some(headers) = request.get("headers") {
                    params.insert("headers".to_string(), headers.clone());
                }
            }
        }
    }

    // For log interactions, inject log_path from manifest
    if scenario.interaction.interaction_type == "logs" {
        if let Some(obs) = &manifest.observability {
            if let Some(logs) = &obs.logs {
                if let Some(path) = &logs.path {
                    params.insert(
                        "log_path".to_string(),
                        serde_json::Value::String(path.clone()),
                    );
                }
            }
        }
    }

    let checks = scenario
        .checks
        .iter()
        .map(|c| {
            let check_type = match c.check_type.as_str() {
                "status_code" => CheckType::StatusCode,
                "json_path" => CheckType::JsonPath,
                "stdout" => CheckType::Stdout,
                "stderr" => CheckType::Stderr,
                "log_contains" => CheckType::LogContains,
                "log_not_contains" => CheckType::LogNotContains,
                _ => CheckType::StatusCode, // fallback
            };
            Check {
                check_type,
                expected: c.expected.clone(),
                path: c.path.clone(),
                exists: c.exists,
                acceptance_criteria: c.acceptance_criteria.clone(),
            }
        })
        .collect();

    ExecutableScenario {
        id: scenario.id.clone(),
        name: scenario.name.clone(),
        interaction: ExecInteraction {
            interaction_type: scenario.interaction.interaction_type.clone(),
            params,
        },
        checks,
    }
}

/// Run all resolved scenarios against a manifest.
pub async fn execute_subject(
    manifest: &SubjectManifest,
    scenarios: &[&ScenarioFile],
) -> SubjectResult {
    let mut executor = ScenarioExecutor::new();

    executor.register_adapter(Box::new(
        qtip_executor::adapters::cli::CliAdapter::new(),
    ));
    executor.register_adapter(Box::new(
        qtip_executor::adapters::log::LogAdapter::new(),
    ));
    executor.register_adapter(Box::new(
        qtip_executor::adapters::api::ApiAdapter::new(),
    ));

    let mut results = Vec::new();
    for scenario in scenarios {
        let executable = to_executable(scenario, manifest);
        let result = executor.execute(&executable).await;
        results.push(result);
    }

    let passed = results
        .iter()
        .filter(|r| r.status == EvaluationStatus::Passed)
        .count();
    let failed = results.len() - passed;

    SubjectResult {
        project_id: manifest.project_id.clone(),
        results,
        passed,
        failed,
    }
}
