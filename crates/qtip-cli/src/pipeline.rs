use std::collections::HashMap;
use std::path::Path;

use qtip_executor::check::{Check, CheckType, Evidence};
use qtip_executor::executor::{
    EvaluationResult, EvaluationStatus, ExecutableScenario, Interaction as ExecInteraction,
    ScenarioExecutor,
};
use qtip_resolver::{
    AppliesTo, ScenarioManifest, ScenarioResolver, SubjectInterface, SubjectQuery,
};

use crate::scenario::{
    Check as ScenarioCheck, Interaction as ScenarioInteraction, ScenarioFile, SubjectManifest,
};
use crate::variable_resolver::{ResolveContext, resolve_json_value};

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
                interface_type: i.interface_type().to_string(),
                name: i.name().map(|s| s.to_string()),
            })
            .collect(),
        environment: manifest.environment.name.clone(),
    };

    let matched = resolver.resolve(&query);
    let matched_ids: Vec<&str> = matched.iter().map(|m| m.id.as_str()).collect();

    scenarios
        .iter()
        .filter(|s| matched_ids.contains(&s.id.as_str()))
        .collect()
}

/// Convert a ScenarioFile + manifest context into an ExecutableScenario.
fn to_executable(
    scenario: &ScenarioFile,
    interaction: &ScenarioInteraction,
    checks: &[ScenarioCheck],
    manifest: &SubjectManifest,
) -> Result<ExecutableScenario, String> {
    let environment_values = build_environment_values(manifest);
    let step_outputs = HashMap::new();
    to_executable_with_context(
        scenario,
        interaction,
        checks,
        manifest,
        &environment_values,
        &step_outputs,
    )
}

fn to_executable_with_context(
    scenario: &ScenarioFile,
    interaction: &ScenarioInteraction,
    checks: &[ScenarioCheck],
    manifest: &SubjectManifest,
    environment_values: &HashMap<String, String>,
    step_outputs: &HashMap<String, String>,
) -> Result<ExecutableScenario, String> {
    let context = ResolveContext::new(environment_values, step_outputs);
    let resolved_params_value = resolve_json_value(
        &serde_json::Value::Object(interaction.params.clone()),
        &context,
    )
    .map_err(|error| {
        format!(
            "Scenario `{}` interaction template resolution failed: {error}",
            scenario.id
        )
    })?;
    let resolved_params = resolved_params_value
        .as_object()
        .ok_or_else(|| "Resolved interaction params must be a JSON object".to_string())?;

    let mut params: HashMap<String, serde_json::Value> = resolved_params
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    // For API interactions, resolve the full URL from manifest
    if interaction.interaction_type == "api"
        && let Some(request) = resolved_params.get("request")
        && let Some(path) = request.get("path").and_then(|p| p.as_str())
    {
        let service = resolved_params.get("service").and_then(|s| s.as_str());

        let base_url = manifest
            .interfaces
            .iter()
            .find(|i| i.interface_type() == "api" && (service.is_none() || i.name() == service))
            .and_then(|i| i.base_url())
            .or_else(|| manifest.environment.get("api_base_url"))
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

    // For log interactions, inject log_path from manifest
    if interaction.interaction_type == "logs" {
        let log_path = manifest
            .observability
            .as_ref()
            .and_then(|obs| obs.logs.as_ref())
            .and_then(|logs| logs.path.as_deref())
            .or_else(|| manifest.environment.get("log_path"));

        if let Some(path) = log_path {
            params.insert(
                "log_path".to_string(),
                serde_json::Value::String(path.to_string()),
            );
        }
    }

    let checks = checks
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

    Ok(ExecutableScenario {
        id: scenario.id.clone(),
        name: scenario.name.clone(),
        interaction: ExecInteraction {
            interaction_type: interaction.interaction_type.clone(),
            params,
        },
        checks,
    })
}

fn build_environment_values(manifest: &SubjectManifest) -> HashMap<String, String> {
    let mut values = HashMap::new();

    for (key, value) in &manifest.environment.fields {
        if let Some(scalar) = scalar_value_to_string(value) {
            values.insert(key.clone(), scalar);
        }
    }

    for (key, value) in std::env::vars() {
        values.insert(key, value);
    }

    values
}

fn scalar_value_to_string(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(v) => Some(v.clone()),
        serde_json::Value::Number(v) => Some(v.to_string()),
        serde_json::Value::Bool(v) => Some(v.to_string()),
        _ => None,
    }
}

/// Run all resolved scenarios against a manifest.
pub async fn execute_subject(
    manifest: &SubjectManifest,
    scenarios: &[&ScenarioFile],
    verbose: bool,
) -> SubjectResult {
    let mut executor = ScenarioExecutor::new();

    executor.register_adapter(Box::new(qtip_executor::adapters::cli::CliAdapter::new()));
    executor.register_adapter(Box::new(qtip_executor::adapters::log::LogAdapter::new()));
    executor.register_adapter(Box::new(qtip_executor::adapters::api::ApiAdapter::new()));

    let mut results = Vec::new();
    for scenario in scenarios {
        let Some((interaction, checks)) = scenario.as_single() else {
            let message = format!(
                "Scenario `{}` is a workflow and cannot be executed yet",
                scenario.id
            );
            if verbose {
                eprintln!("[verbose] {message}");
            }
            results.push(EvaluationResult {
                scenario_id: scenario.id.clone(),
                status: EvaluationStatus::Error,
                evidence: Evidence::cli(1, "", ""),
                failures: vec![message],
            });
            continue;
        };

        if verbose {
            eprintln!(
                "[verbose] Executing scenario: {} ({}, {})",
                scenario.id,
                interaction.interaction_type,
                scenario.kind_label()
            );
        }
        let executable = match to_executable(scenario, interaction, checks, manifest) {
            Ok(executable) => executable,
            Err(message) => {
                if verbose {
                    eprintln!("[verbose] {message}");
                }
                results.push(EvaluationResult {
                    scenario_id: scenario.id.clone(),
                    status: EvaluationStatus::Error,
                    evidence: Evidence::cli(1, "", ""),
                    failures: vec![message],
                });
                continue;
            }
        };
        if verbose {
            eprintln!("[verbose]   params: {:?}", executable.interaction.params);
            eprintln!("[verbose]   checks: {}", executable.checks.len());
        }
        let result = executor.execute(&executable).await;
        if verbose {
            eprintln!("[verbose]   result: {:?}", result.status);
            for failure in &result.failures {
                eprintln!("[verbose]   failure: {}", failure);
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use qtip_executor::executor::EvaluationStatus;
    use std::collections::HashMap;
    use std::path::Path;

    fn read_fixture(path: &str) -> String {
        let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join(path);
        std::fs::read_to_string(&fixture_path).unwrap_or_else(|error| {
            panic!("failed to read fixture {}: {error}", fixture_path.display())
        })
    }

    #[tokio::test]
    async fn legacy_single_fixture_parses_and_executes_as_single() {
        let yaml = read_fixture("scenarios/cli/test-hello.yaml");
        let scenario: ScenarioFile =
            serde_yaml::from_str(&yaml).expect("legacy fixture should parse");

        let (interaction, checks) = scenario
            .as_single()
            .expect("legacy fixture should remain single");
        assert_eq!(interaction.interaction_type, "cli");
        assert_eq!(checks.len(), 2);
        assert_eq!(checks[0].check_type, "status_code");
        assert_eq!(checks[1].check_type, "stdout");

        let manifest: SubjectManifest = serde_json::from_str(
            r#"{
  "projectId": "legacy-single-fixture",
  "environment": "local",
  "interfaces": ["cli"],
  "capabilities": ["test"]
}"#,
        )
        .expect("manifest should parse");

        let resolved = resolve_scenarios(&manifest, std::slice::from_ref(&scenario));
        assert_eq!(resolved.len(), 1);

        let result = execute_subject(&manifest, &resolved, false).await;
        assert_eq!(result.passed, 1);
        assert_eq!(result.failed, 0);
        assert_eq!(result.results.len(), 1);
        assert_eq!(result.results[0].scenario_id, "TEST-CLI-HELLO");
        assert_eq!(result.results[0].status, EvaluationStatus::Passed);
        assert!(result.results[0].failures.is_empty());
    }

    #[test]
    fn to_executable_resolves_cli_command_with_env_precedence_and_escape() {
        let scenario_yaml = r#"
id: TEST-CLI-RESOLVE-001
name: Resolve CLI vars
applies_to:
  capabilities: [test]
  interfaces: [cli]
acceptance_criteria:
  - id: AC-1
    description: command resolves
interaction:
  type: cli
  command: smith --repo $TEST_REPO && echo $$HOME
checks:
  - type: status_code
    expected: 0
    acceptance_criteria: AC-1
"#;
        let scenario: ScenarioFile = serde_yaml::from_str(scenario_yaml).unwrap();
        let (interaction, checks) = scenario.as_single().unwrap();

        let manifest: SubjectManifest = serde_json::from_str(
            r#"{
  "projectId": "resolve-cli",
  "environment": "local",
  "interfaces": ["cli"],
  "capabilities": ["test"]
}"#,
        )
        .unwrap();

        let env = HashMap::from([("TEST_REPO".to_string(), "org/repo".to_string())]);
        let outputs = HashMap::from([("TEST_REPO".to_string(), "from-output".to_string())]);

        let executable =
            to_executable_with_context(&scenario, interaction, checks, &manifest, &env, &outputs)
                .unwrap();

        assert_eq!(
            executable.interaction.params["command"],
            "smith --repo org/repo && echo $HOME"
        );
    }

    #[test]
    fn to_executable_resolves_api_and_log_string_fields() {
        let api_yaml = r#"
id: TEST-API-RESOLVE-001
name: Resolve API vars
applies_to:
  capabilities: [test]
  interfaces: [api]
acceptance_criteria:
  - id: AC-1
    description: api resolves
interaction:
  type: api
  service: repo
  request:
    method: POST
    path: /repos/$TEST_REPO/pulls
    body:
      title: Sync $PR_ID
    headers:
      Authorization: Bearer $TOKEN
checks:
  - type: status_code
    expected: 200
    acceptance_criteria: AC-1
"#;
        let api_scenario: ScenarioFile = serde_yaml::from_str(api_yaml).unwrap();
        let (api_interaction, api_checks) = api_scenario.as_single().unwrap();

        let manifest: SubjectManifest = serde_json::from_str(
            r#"{
  "projectId": "resolve-api",
  "environment": "local",
  "interfaces": [
    { "type": "api", "name": "repo", "baseUrl": "https://api.example.com" }
  ],
  "capabilities": ["test"]
}"#,
        )
        .unwrap();

        let env = HashMap::from([
            ("TEST_REPO".to_string(), "org/repo".to_string()),
            ("PR_ID".to_string(), "42".to_string()),
            ("TOKEN".to_string(), "abc123".to_string()),
        ]);
        let outputs = HashMap::new();

        let executable = to_executable_with_context(
            &api_scenario,
            api_interaction,
            api_checks,
            &manifest,
            &env,
            &outputs,
        )
        .unwrap();

        assert_eq!(
            executable.interaction.params["url"],
            "https://api.example.com/repos/org/repo/pulls"
        );
        assert_eq!(executable.interaction.params["method"], "POST");
        assert_eq!(executable.interaction.params["body"]["title"], "Sync 42");
        assert_eq!(
            executable.interaction.params["headers"]["Authorization"],
            "Bearer abc123"
        );

        let log_yaml = r#"
id: TEST-LOG-RESOLVE-001
name: Resolve log vars
applies_to:
  capabilities: [test]
  interfaces: [logs]
acceptance_criteria:
  - id: AC-1
    description: log query resolves
interaction:
  type: logs
  query: loop:$loop_id
checks:
  - type: log_contains
    acceptance_criteria: AC-1
"#;
        let log_scenario: ScenarioFile = serde_yaml::from_str(log_yaml).unwrap();
        let (log_interaction, log_checks) = log_scenario.as_single().unwrap();
        let log_outputs = HashMap::from([("loop_id".to_string(), "smi-abc123".to_string())]);

        let log_executable = to_executable_with_context(
            &log_scenario,
            log_interaction,
            log_checks,
            &manifest,
            &HashMap::new(),
            &log_outputs,
        )
        .unwrap();
        assert_eq!(
            log_executable.interaction.params["query"],
            "loop:smi-abc123"
        );
    }

    #[test]
    fn to_executable_fails_when_templates_have_unresolved_variables() {
        let scenario_yaml = r#"
id: TEST-CLI-RESOLVE-ERR
name: Missing variable
applies_to:
  capabilities: [test]
  interfaces: [cli]
acceptance_criteria:
  - id: AC-1
    description: command fails when unresolved
interaction:
  type: cli
  command: echo $loop_id
checks:
  - type: status_code
    expected: 0
    acceptance_criteria: AC-1
"#;
        let scenario: ScenarioFile = serde_yaml::from_str(scenario_yaml).unwrap();
        let (interaction, checks) = scenario.as_single().unwrap();

        let manifest: SubjectManifest = serde_json::from_str(
            r#"{
  "projectId": "resolve-cli-error",
  "environment": "local",
  "interfaces": ["cli"],
  "capabilities": ["test"]
}"#,
        )
        .unwrap();

        let error = to_executable_with_context(
            &scenario,
            interaction,
            checks,
            &manifest,
            &HashMap::new(),
            &HashMap::new(),
        )
        .unwrap_err();

        assert!(error.contains("Unresolved variables: loop_id"));
    }
}
