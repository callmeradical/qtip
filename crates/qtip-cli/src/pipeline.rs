use std::collections::HashMap;
use std::path::Path;

use qtip_executor::check::{Check, CheckType, Evidence};
use qtip_executor::executor::{
    Adapter, EvaluationResult, EvaluationStatus, ExecutableScenario,
    Interaction as ExecInteraction, ScenarioExecutor,
};
use qtip_resolver::{
    AppliesTo, ScenarioManifest, ScenarioResolver, SubjectInterface, SubjectQuery,
};

use crate::scenario::{
    Check as ScenarioCheck, Interaction as ScenarioInteraction, ScenarioFile, ScenarioKind,
    ScenarioStep, SubjectManifest,
};
use crate::step_output::{extract_step_outputs, merge_step_outputs};
use crate::variable_resolver::{ResolveContext, resolve_json_value};

#[derive(Debug)]
pub struct SubjectResult {
    pub project_id: String,
    pub results: Vec<EvaluationResult>,
    pub passed: usize,
    pub failed: usize,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepPhase {
    Setup,
    Main,
    Teardown,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    Passed,
    Failed,
    Error,
    Skipped,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone)]
pub struct StepResult {
    pub name: String,
    pub phase: StepPhase,
    pub status: StepStatus,
    pub evidence: Option<Evidence>,
    pub failures: Vec<String>,
    pub outputs_captured: HashMap<String, String>,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone)]
pub struct WorkflowResult {
    #[allow(dead_code)]
    pub scenario_id: String,
    pub status: EvaluationStatus,
    pub setup: Vec<StepResult>,
    pub steps: Vec<StepResult>,
    pub teardown: Vec<StepResult>,
    pub teardown_failures: Vec<String>,
}

#[cfg_attr(not(test), allow(dead_code))]
pub struct WorkflowExecutor {
    executor: ScenarioExecutor,
}

impl Default for WorkflowExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(not(test), allow(dead_code))]
impl WorkflowExecutor {
    pub fn new() -> Self {
        Self {
            executor: ScenarioExecutor::new(),
        }
    }

    pub fn register_adapter(&mut self, adapter: Box<dyn Adapter>) {
        self.executor.register_adapter(adapter);
    }

    pub async fn execute_workflow(
        &self,
        scenario: &ScenarioFile,
        manifest: &SubjectManifest,
    ) -> Result<WorkflowResult, String> {
        let (setup_steps, main_steps, teardown_steps) = match &scenario.kind {
            ScenarioKind::Workflow {
                setup,
                steps,
                teardown,
            } => (setup.as_slice(), steps.as_slice(), teardown.as_slice()),
            ScenarioKind::Single { .. } => {
                return Err(format!("Scenario `{}` is not a workflow", scenario.id));
            }
        };

        let environment_values = build_environment_values(manifest);
        let mut output_context = HashMap::new();

        let mut setup_results = Vec::with_capacity(setup_steps.len());
        let mut setup_blocked = false;
        for step in setup_steps {
            if setup_blocked {
                setup_results.push(skipped_step_result(
                    step,
                    StepPhase::Setup,
                    "Skipped because a previous setup step failed".to_string(),
                ));
                continue;
            }

            let result = self
                .execute_step(
                    scenario,
                    step,
                    StepPhase::Setup,
                    manifest,
                    &environment_values,
                    &mut output_context,
                )
                .await;
            setup_blocked = is_blocking_status(result.status);
            setup_results.push(result);
        }

        let mut step_results = Vec::with_capacity(main_steps.len());
        let mut main_blocked = setup_blocked;
        for step in main_steps {
            if main_blocked {
                let reason = if setup_blocked {
                    "Skipped because setup did not complete successfully".to_string()
                } else {
                    "Skipped because a previous main step failed".to_string()
                };
                step_results.push(skipped_step_result(step, StepPhase::Main, reason));
                continue;
            }

            let result = self
                .execute_step(
                    scenario,
                    step,
                    StepPhase::Main,
                    manifest,
                    &environment_values,
                    &mut output_context,
                )
                .await;
            if is_blocking_status(result.status) {
                main_blocked = true;
            }
            step_results.push(result);
        }

        let mut teardown_results = Vec::with_capacity(teardown_steps.len());
        for step in teardown_steps {
            teardown_results.push(
                self.execute_step(
                    scenario,
                    step,
                    StepPhase::Teardown,
                    manifest,
                    &environment_values,
                    &mut output_context,
                )
                .await,
            );
        }

        let status = derive_main_status(&setup_results, &step_results);
        let teardown_failures = collect_teardown_failures(&teardown_results);

        Ok(WorkflowResult {
            scenario_id: scenario.id.clone(),
            status,
            setup: setup_results,
            steps: step_results,
            teardown: teardown_results,
            teardown_failures,
        })
    }

    async fn execute_step(
        &self,
        scenario: &ScenarioFile,
        step: &ScenarioStep,
        phase: StepPhase,
        manifest: &SubjectManifest,
        environment_values: &HashMap<String, String>,
        output_context: &mut HashMap<String, String>,
    ) -> StepResult {
        let executable = match to_executable_with_context(
            scenario,
            &step.interaction,
            &step.checks,
            manifest,
            environment_values,
            output_context,
        ) {
            Ok(executable) => executable,
            Err(message) => {
                return StepResult {
                    name: step.name.clone(),
                    phase,
                    status: StepStatus::Error,
                    evidence: None,
                    failures: vec![format!(
                        "Step `{}` preparation failed: {message}",
                        step.name
                    )],
                    outputs_captured: HashMap::new(),
                };
            }
        };

        let evaluation = self.executor.execute(&executable).await;
        let mut result = StepResult {
            name: step.name.clone(),
            phase,
            status: step_status_from_evaluation(evaluation.status),
            evidence: Some(evaluation.evidence),
            failures: evaluation.failures,
            outputs_captured: HashMap::new(),
        };

        if matches!(result.status, StepStatus::Passed) && !step.outputs.is_empty() {
            match extract_step_outputs(&step.outputs, result.evidence.as_ref().expect("present")) {
                Ok(extracted) => {
                    result.outputs_captured = extracted.clone();
                    merge_step_outputs(output_context, extracted);
                }
                Err(failures) => {
                    result.status = StepStatus::Failed;
                    result.failures.extend(failures.into_iter().map(|failure| {
                        format!(
                            "Output `{}` extraction failed: {}",
                            failure.key, failure.message
                        )
                    }));
                }
            }
        }

        result
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn step_status_from_evaluation(status: EvaluationStatus) -> StepStatus {
    match status {
        EvaluationStatus::Passed => StepStatus::Passed,
        EvaluationStatus::Failed => StepStatus::Failed,
        EvaluationStatus::Error => StepStatus::Error,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn is_blocking_status(status: StepStatus) -> bool {
    matches!(status, StepStatus::Failed | StepStatus::Error)
}

#[cfg_attr(not(test), allow(dead_code))]
fn skipped_step_result(step: &ScenarioStep, phase: StepPhase, reason: String) -> StepResult {
    StepResult {
        name: step.name.clone(),
        phase,
        status: StepStatus::Skipped,
        evidence: None,
        failures: vec![reason],
        outputs_captured: HashMap::new(),
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn derive_main_status(
    setup_results: &[StepResult],
    step_results: &[StepResult],
) -> EvaluationStatus {
    if setup_results
        .iter()
        .chain(step_results.iter())
        .any(|result| result.status == StepStatus::Error)
    {
        EvaluationStatus::Error
    } else if setup_results
        .iter()
        .chain(step_results.iter())
        .any(|result| result.status == StepStatus::Failed)
    {
        EvaluationStatus::Failed
    } else {
        EvaluationStatus::Passed
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn collect_teardown_failures(teardown_results: &[StepResult]) -> Vec<String> {
    teardown_results
        .iter()
        .filter(|result| is_blocking_status(result.status))
        .map(|result| {
            if result.failures.is_empty() {
                format!(
                    "Teardown step `{}` finished with {:?}",
                    result.name, result.status
                )
            } else {
                format!(
                    "Teardown step `{}` failed: {}",
                    result.name,
                    result.failures.join("; ")
                )
            }
        })
        .collect()
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
    use qtip_executor::executor::{Adapter, EvaluationStatus};
    use std::collections::HashMap;
    use std::path::Path;
    use std::sync::{Arc, Mutex};

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

    #[derive(Clone)]
    struct RecordingCliAdapter {
        responses: HashMap<String, Result<Evidence, String>>,
        executed_commands: Arc<Mutex<Vec<String>>>,
    }

    impl Adapter for RecordingCliAdapter {
        fn interaction_type(&self) -> &str {
            "cli"
        }

        fn execute<'a>(
            &'a self,
            interaction: &'a ExecInteraction,
        ) -> qtip_executor::executor::BoxFuture<'a, Result<Evidence, String>> {
            let command = interaction
                .params
                .get("command")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .to_string();
            self.executed_commands
                .lock()
                .expect("lock commands")
                .push(command.clone());
            let response = self.responses.get(&command).cloned().unwrap_or_else(|| {
                Err(format!(
                    "No stub response configured for command `{command}`"
                ))
            });
            Box::pin(async move { response })
        }
    }

    fn test_manifest() -> SubjectManifest {
        serde_json::from_str(
            r#"{
  "projectId": "workflow-test-subject",
  "environment": "local",
  "interfaces": ["cli"],
  "capabilities": ["test"]
}"#,
        )
        .expect("manifest should parse")
    }

    fn workflow_executor_with_cli(
        responses: HashMap<String, Result<Evidence, String>>,
    ) -> (WorkflowExecutor, Arc<Mutex<Vec<String>>>) {
        let executed_commands = Arc::new(Mutex::new(Vec::new()));
        let adapter = RecordingCliAdapter {
            responses,
            executed_commands: Arc::clone(&executed_commands),
        };

        let mut executor = WorkflowExecutor::new();
        executor.register_adapter(Box::new(adapter));
        (executor, executed_commands)
    }

    #[tokio::test]
    async fn workflow_executor_marks_remaining_main_steps_skipped_and_runs_teardown() {
        let scenario_yaml = r#"
id: TEST-WORKFLOW-LIFECYCLE-001
name: Workflow main fail-fast
applies_to:
  capabilities: [test]
  interfaces: [cli]
acceptance_criteria:
  - id: AC-1
    description: Workflow runs in deterministic order
steps:
  - name: Step 1
    interaction:
      type: cli
      command: step-1
    checks:
      - type: status_code
        expected: 0
        acceptance_criteria: AC-1
  - name: Step 2
    interaction:
      type: cli
      command: step-2
    checks:
      - type: status_code
        expected: 0
        acceptance_criteria: AC-1
  - name: Step 3
    interaction:
      type: cli
      command: step-3
    checks:
      - type: status_code
        expected: 0
        acceptance_criteria: AC-1
teardown:
  - name: Cleanup
    interaction:
      type: cli
      command: cleanup
    checks:
      - type: status_code
        expected: 0
        acceptance_criteria: AC-1
"#;
        let scenario: ScenarioFile = serde_yaml::from_str(scenario_yaml).expect("parse workflow");
        let (executor, executed_commands) = workflow_executor_with_cli(HashMap::from([
            ("step-1".to_string(), Ok(Evidence::cli(0, "", ""))),
            ("step-2".to_string(), Ok(Evidence::cli(1, "", ""))),
            ("cleanup".to_string(), Ok(Evidence::cli(0, "", ""))),
        ]));

        let result = executor
            .execute_workflow(&scenario, &test_manifest())
            .await
            .expect("workflow execution succeeds");

        assert_eq!(result.status, EvaluationStatus::Failed);
        assert_eq!(result.steps.len(), 3);
        assert_eq!(result.steps[0].status, StepStatus::Passed);
        assert_eq!(result.steps[1].status, StepStatus::Failed);
        assert_eq!(result.steps[2].status, StepStatus::Skipped);
        assert_eq!(result.steps[2].phase, StepPhase::Main);
        assert_eq!(result.teardown.len(), 1);
        assert_eq!(result.teardown[0].status, StepStatus::Passed);
        assert!(result.teardown_failures.is_empty());
        assert_eq!(
            executed_commands.lock().expect("lock commands").clone(),
            vec!["step-1", "step-2", "cleanup"]
        );
    }

    #[tokio::test]
    async fn workflow_executor_aborts_main_steps_when_setup_fails_and_runs_teardown() {
        let scenario_yaml = r#"
id: TEST-WORKFLOW-LIFECYCLE-002
name: Workflow setup failure
applies_to:
  capabilities: [test]
  interfaces: [cli]
acceptance_criteria:
  - id: AC-1
    description: setup controls main execution
setup:
  - name: Setup 1
    interaction:
      type: cli
      command: setup-1
    checks:
      - type: status_code
        expected: 0
        acceptance_criteria: AC-1
  - name: Setup 2
    interaction:
      type: cli
      command: setup-2
    checks:
      - type: status_code
        expected: 0
        acceptance_criteria: AC-1
steps:
  - name: Step 1
    interaction:
      type: cli
      command: step-1
    checks:
      - type: status_code
        expected: 0
        acceptance_criteria: AC-1
teardown:
  - name: Cleanup
    interaction:
      type: cli
      command: cleanup
    checks:
      - type: status_code
        expected: 0
        acceptance_criteria: AC-1
"#;
        let scenario: ScenarioFile = serde_yaml::from_str(scenario_yaml).expect("parse workflow");
        let (executor, executed_commands) = workflow_executor_with_cli(HashMap::from([
            ("setup-1".to_string(), Ok(Evidence::cli(1, "", ""))),
            ("cleanup".to_string(), Ok(Evidence::cli(0, "", ""))),
        ]));

        let result = executor
            .execute_workflow(&scenario, &test_manifest())
            .await
            .expect("workflow execution succeeds");

        assert_eq!(result.status, EvaluationStatus::Failed);
        assert_eq!(result.setup.len(), 2);
        assert_eq!(result.setup[0].status, StepStatus::Failed);
        assert_eq!(result.setup[1].status, StepStatus::Skipped);
        assert_eq!(result.steps.len(), 1);
        assert_eq!(result.steps[0].status, StepStatus::Skipped);
        assert_eq!(result.teardown[0].status, StepStatus::Passed);
        assert_eq!(
            executed_commands.lock().expect("lock commands").clone(),
            vec!["setup-1", "cleanup"]
        );
    }

    #[tokio::test]
    async fn workflow_executor_reports_teardown_failures_without_overriding_main_status() {
        let scenario_yaml = r#"
id: TEST-WORKFLOW-LIFECYCLE-003
name: Workflow teardown failure
applies_to:
  capabilities: [test]
  interfaces: [cli]
acceptance_criteria:
  - id: AC-1
    description: teardown failures are distinct
steps:
  - name: Main step
    interaction:
      type: cli
      command: main-step
    checks:
      - type: status_code
        expected: 0
        acceptance_criteria: AC-1
teardown:
  - name: Cleanup
    interaction:
      type: cli
      command: cleanup
    checks:
      - type: status_code
        expected: 0
        acceptance_criteria: AC-1
"#;
        let scenario: ScenarioFile = serde_yaml::from_str(scenario_yaml).expect("parse workflow");
        let (executor, executed_commands) = workflow_executor_with_cli(HashMap::from([
            ("main-step".to_string(), Ok(Evidence::cli(0, "", ""))),
            ("cleanup".to_string(), Ok(Evidence::cli(1, "", ""))),
        ]));

        let result = executor
            .execute_workflow(&scenario, &test_manifest())
            .await
            .expect("workflow execution succeeds");

        assert_eq!(result.status, EvaluationStatus::Passed);
        assert_eq!(result.steps[0].status, StepStatus::Passed);
        assert_eq!(result.teardown[0].status, StepStatus::Failed);
        assert_eq!(result.teardown_failures.len(), 1);
        assert!(result.teardown_failures[0].contains("Teardown step `Cleanup`"));
        assert_eq!(
            executed_commands.lock().expect("lock commands").clone(),
            vec!["main-step", "cleanup"]
        );
    }
}
