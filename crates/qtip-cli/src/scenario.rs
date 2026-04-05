use std::collections::HashMap;

use serde::{Deserialize, Deserializer, de::Error};
use serde_json::Value;

/// Full scenario as defined in YAML files — bridges catalog discovery
/// to the resolver and executor.
#[derive(Debug, Clone, Deserialize)]
pub struct ScenarioFile {
    pub id: String,
    pub name: String,
    pub applies_to: AppliesTo,
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    #[serde(flatten)]
    pub kind: ScenarioKind,
}

impl ScenarioFile {
    pub fn as_single(&self) -> Option<(&Interaction, &[Check])> {
        match &self.kind {
            ScenarioKind::Single {
                interaction,
                checks,
            } => Some((interaction, checks.as_slice())),
            ScenarioKind::Workflow { .. } => None,
        }
    }

    pub fn kind_label(&self) -> &'static str {
        match &self.kind {
            ScenarioKind::Single { .. } => "single",
            ScenarioKind::Workflow { .. } => "workflow",
        }
    }
}

#[derive(Debug, Clone)]
pub enum ScenarioKind {
    Single {
        interaction: Interaction,
        checks: Vec<Check>,
    },
    Workflow {
        setup: Vec<ScenarioStep>,
        steps: Vec<ScenarioStep>,
        teardown: Vec<ScenarioStep>,
    },
}

#[derive(Debug, Clone, Deserialize)]
struct RawScenarioKind {
    interaction: Option<Interaction>,
    checks: Option<Vec<Check>>,
    setup: Option<Vec<RawScenarioStep>>,
    steps: Option<Vec<RawScenarioStep>>,
    teardown: Option<Vec<RawScenarioStep>>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawScenarioStep {
    name: Option<String>,
    interaction: Option<Interaction>,
    #[serde(default)]
    checks: Vec<Check>,
    #[serde(default)]
    outputs: HashMap<String, RawStepOutput>,
    timeout: Option<i64>,
    #[serde(default)]
    warn_only: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct RawStepOutput {
    from: Option<String>,
    path: Option<String>,
    pattern: Option<String>,
}

impl RawScenarioKind {
    fn into_scenario_kind(self) -> Result<ScenarioKind, String> {
        let has_interaction = self.interaction.is_some();
        let has_steps = self.steps.is_some();

        match (has_interaction, has_steps) {
            (true, true) => Err(
                "Scenario cannot define both `interaction` and `steps`; choose one mode"
                    .to_string(),
            ),
            (true, false) => {
                if self.setup.is_some() || self.teardown.is_some() {
                    return Err(
                        "Single scenarios cannot define `setup` or `teardown` sections".to_string(),
                    );
                }

                let interaction = self
                    .interaction
                    .expect("interaction is guaranteed present by match branch");
                let checks = self
                    .checks
                    .ok_or_else(|| "Single scenarios must define `checks`".to_string())?;

                Ok(ScenarioKind::Single {
                    interaction,
                    checks,
                })
            }
            (false, true) => {
                if self.checks.is_some() {
                    return Err(
                        "Workflow scenarios cannot define top-level `checks`; move checks into each step"
                            .to_string(),
                    );
                }

                let steps = self
                    .steps
                    .expect("steps is guaranteed present by match branch");
                if steps.is_empty() {
                    return Err(
                        "Workflow scenarios must include at least one entry in `steps`".to_string(),
                    );
                }

                let setup = parse_workflow_steps(self.setup.unwrap_or_default(), "setup")?;
                let steps = parse_workflow_steps(steps, "steps")?;
                let teardown = parse_workflow_steps(self.teardown.unwrap_or_default(), "teardown")?;

                Ok(ScenarioKind::Workflow {
                    setup,
                    steps,
                    teardown,
                })
            }
            (false, false) => {
                Err("Scenario must define either `interaction` or `steps`".to_string())
            }
        }
    }
}

impl RawScenarioStep {
    fn into_scenario_step(self, section: &str, index: usize) -> Result<ScenarioStep, String> {
        let path_prefix = format!("{section}[{index}]");
        let name = required_non_empty(self.name, &format!("{path_prefix}.name"))?;
        let interaction = self
            .interaction
            .ok_or_else(|| format!("Missing required field `{path_prefix}.interaction`"))?;
        let timeout = match self.timeout {
            Some(value) if value <= 0 => {
                return Err(format!(
                    "Invalid value for `{path_prefix}.timeout`: expected a positive integer, got `{value}`"
                ));
            }
            Some(value) => Some(value as u64),
            None => None,
        };

        let mut outputs = HashMap::with_capacity(self.outputs.len());
        for (output_name, output) in self.outputs {
            let parsed = output.into_step_output(&path_prefix, &output_name)?;
            outputs.insert(output_name, parsed);
        }

        Ok(ScenarioStep {
            name,
            interaction,
            checks: self.checks,
            outputs,
            timeout,
            warn_only: self.warn_only,
        })
    }
}

impl RawStepOutput {
    fn into_step_output(self, path_prefix: &str, output_name: &str) -> Result<StepOutput, String> {
        let field_path = format!("{path_prefix}.outputs.{output_name}.from");
        let from = required_non_empty(self.from, &field_path)?;
        let normalized = from.to_ascii_lowercase();

        if !matches!(normalized.as_str(), "json" | "stdout" | "stderr") {
            return Err(format!(
                "Invalid value for `{field_path}`: expected one of `json`, `stdout`, `stderr`, got `{from}`"
            ));
        }

        Ok(StepOutput {
            from: normalized,
            path: self.path,
            pattern: self.pattern,
        })
    }
}

fn parse_workflow_steps(
    raw_steps: Vec<RawScenarioStep>,
    section: &str,
) -> Result<Vec<ScenarioStep>, String> {
    let mut steps = Vec::with_capacity(raw_steps.len());
    for (index, step) in raw_steps.into_iter().enumerate() {
        steps.push(step.into_scenario_step(section, index)?);
    }
    Ok(steps)
}

fn required_non_empty(value: Option<String>, field_path: &str) -> Result<String, String> {
    let value = value.ok_or_else(|| format!("Missing required field `{field_path}`"))?;
    if value.trim().is_empty() {
        return Err(format!("Field `{field_path}` cannot be empty"));
    }
    Ok(value)
}

impl<'de> Deserialize<'de> for ScenarioKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawScenarioKind::deserialize(deserializer)?;
        raw.into_scenario_kind().map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScenarioStep {
    pub name: String,
    pub interaction: Interaction,
    #[serde(default)]
    pub checks: Vec<Check>,
    #[serde(default)]
    pub outputs: HashMap<String, StepOutput>,
    pub timeout: Option<u64>,
    #[serde(default)]
    pub warn_only: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StepOutput {
    pub from: String,
    pub path: Option<String>,
    pub pattern: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppliesTo {
    pub capabilities: Vec<String>,
    pub interfaces: Vec<String>,
    #[serde(default)]
    pub environments: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AcceptanceCriterion {
    pub id: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Interaction {
    #[serde(rename = "type")]
    pub interaction_type: String,
    #[serde(flatten)]
    pub params: serde_json::Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Check {
    #[serde(rename = "type")]
    pub check_type: String,
    pub expected: Option<Value>,
    pub path: Option<String>,
    pub exists: Option<bool>,
    pub exact: Option<bool>,
    pub pattern: Option<String>,
    pub head_ref_pattern: Option<String>,
    pub base_ref: Option<String>,
    pub title_pattern: Option<String>,
    pub state: Option<String>,
    pub acceptance_criteria: String,
}

/// Subject manifest as defined in JSON files.
/// Accepts both camelCase and snake_case field names, and flexible types
/// for environment (string or object) and project ID (projectId or project).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectManifest {
    #[serde(alias = "project")]
    pub project_id: String,
    #[serde(alias = "repo_url")]
    pub repo_url: Option<String>,
    pub commit: Option<String>,
    #[serde(deserialize_with = "deserialize_environment")]
    pub environment: EnvironmentConfig,
    pub interfaces: Vec<ManifestInterface>,
    pub observability: Option<Observability>,
    pub capabilities: Vec<String>,
    pub scenarios: Option<ScenariosConfig>,
    // Allow extra fields without failing
    #[serde(flatten)]
    pub extra: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Clone)]
pub struct EnvironmentConfig {
    /// The environment name used for scenario resolution
    pub name: String,
    /// Extra fields from environment object (api_base_url, log_path, project_root, etc.)
    pub fields: serde_json::Map<String, serde_json::Value>,
}

impl EnvironmentConfig {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.fields.get(key).and_then(|v| v.as_str())
    }
}

fn deserialize_environment<'de, D>(deserializer: D) -> Result<EnvironmentConfig, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::String(s) => Ok(EnvironmentConfig {
            name: s,
            fields: serde_json::Map::new(),
        }),
        serde_json::Value::Object(map) => {
            let name = map
                .get("name")
                .and_then(|v| v.as_str())
                .or_else(|| map.get("runtime").and_then(|v| v.as_str()))
                .unwrap_or("default")
                .to_string();
            Ok(EnvironmentConfig { name, fields: map })
        }
        other => Ok(EnvironmentConfig {
            name: other.to_string(),
            fields: serde_json::Map::new(),
        }),
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenariosConfig {
    /// GitHub repo (e.g. "callmeradical/scenarios")
    pub repo: Option<String>,
    /// Subdirectory within the repo
    pub path: Option<String>,
    /// Local directory (alternative to repo)
    pub local: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ManifestInterface {
    /// Full object: { "type": "api", "baseUrl": "..." }
    Full {
        #[serde(rename = "type")]
        interface_type: String,
        #[serde(default, rename = "baseUrl")]
        base_url: Option<String>,
        #[serde(default)]
        name: Option<String>,
    },
    /// Short form: just "api"
    Short(String),
}

impl ManifestInterface {
    pub fn interface_type(&self) -> &str {
        match self {
            ManifestInterface::Full { interface_type, .. } => interface_type,
            ManifestInterface::Short(s) => s,
        }
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            ManifestInterface::Full { name, .. } => name.as_deref(),
            ManifestInterface::Short(_) => None,
        }
    }

    pub fn base_url(&self) -> Option<&str> {
        match self {
            ManifestInterface::Full { base_url, .. } => base_url.as_deref(),
            ManifestInterface::Short(_) => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Observability {
    pub logs: Option<LogSource>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub path: Option<String>,
    pub url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{ScenarioFile, ScenarioKind};
    use std::path::Path;

    fn read_legacy_fixture(path: &str) -> String {
        let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join(path);
        std::fs::read_to_string(&fixture_path).unwrap_or_else(|error| {
            panic!(
                "failed to read legacy fixture {}: {error}",
                fixture_path.display()
            )
        })
    }

    #[test]
    fn parse_single_scenario_uses_single_kind() {
        let yaml = r#"
id: TEST-SINGLE-001
name: Single scenario
applies_to:
  capabilities: [auth]
  interfaces: [api]
acceptance_criteria:
  - id: AC-1
    description: Request succeeds
interaction:
  type: api
  request:
    method: GET
    path: /health
checks:
  - type: status_code
    expected: 200
    acceptance_criteria: AC-1
"#;

        let scenario: ScenarioFile =
            serde_yaml::from_str(yaml).expect("single scenario should parse");

        assert!(matches!(scenario.kind, ScenarioKind::Single { .. }));
    }

    #[test]
    fn parse_workflow_scenario_preserves_step_order_and_outputs() {
        let yaml = r#"
id: TEST-WORKFLOW-001
name: Workflow scenario
applies_to:
  capabilities: [auth]
  interfaces: [cli]
acceptance_criteria:
  - id: AC-1
    description: Workflow runs
steps:
  - name: First step
    interaction:
      type: cli
      command: echo first
    outputs:
      loop_id:
        from: json
        path: $.loop_id
  - name: Second step
    interaction:
      type: cli
      command: echo second
"#;

        let scenario: ScenarioFile =
            serde_yaml::from_str(yaml).expect("workflow scenario should parse");

        match scenario.kind {
            ScenarioKind::Workflow { steps, .. } => {
                let names: Vec<String> = steps.into_iter().map(|step| step.name).collect();
                assert_eq!(names, vec!["First step", "Second step"]);
            }
            ScenarioKind::Single { .. } => panic!("expected workflow scenario kind"),
        }
    }

    #[test]
    fn parse_scenario_rejects_interaction_and_steps() {
        let yaml = r#"
id: TEST-INVALID-001
name: Invalid scenario
applies_to:
  capabilities: [auth]
  interfaces: [api]
acceptance_criteria:
  - id: AC-1
    description: Should fail parsing
interaction:
  type: api
  request:
    method: GET
    path: /health
steps:
  - name: Step one
    interaction:
      type: cli
      command: echo hi
checks:
  - type: status_code
    expected: 200
    acceptance_criteria: AC-1
"#;

        let err = serde_yaml::from_str::<ScenarioFile>(yaml)
            .expect_err("scenario should fail when both interaction and steps are present");

        assert!(
            err.to_string().contains("both `interaction` and `steps`"),
            "unexpected parse error: {err}"
        );
    }

    #[test]
    fn parse_workflow_rejects_invalid_output_source_with_field_path() {
        let yaml = r#"
id: TEST-WORKFLOW-INVALID-OUTPUT
name: Invalid workflow output source
applies_to:
  capabilities: [auth]
  interfaces: [cli]
acceptance_criteria:
  - id: AC-1
    description: Should fail parsing
steps:
  - name: Create loop
    interaction:
      type: cli
      command: smith loop create --output json
    outputs:
      loop_id:
        from: body
        path: $.loop_id
"#;

        let err = serde_yaml::from_str::<ScenarioFile>(yaml)
            .expect_err("scenario should fail with unsupported output source");
        let err_text = err.to_string();

        assert!(
            err_text.contains("steps[0].outputs.loop_id.from"),
            "unexpected parse error: {err_text}"
        );
        assert!(
            err_text.contains("expected one of `json`, `stdout`, `stderr`"),
            "unexpected parse error: {err_text}"
        );
    }

    #[test]
    fn parse_workflow_rejects_missing_step_interaction_with_field_path() {
        let yaml = r#"
id: TEST-WORKFLOW-MISSING-INTERACTION
name: Missing interaction
applies_to:
  capabilities: [auth]
  interfaces: [cli]
acceptance_criteria:
  - id: AC-1
    description: Should fail parsing
steps:
  - name: Create loop
"#;

        let err = serde_yaml::from_str::<ScenarioFile>(yaml)
            .expect_err("scenario should fail when a step is missing interaction");
        let err_text = err.to_string();

        assert!(
            err_text.contains("steps[0].interaction"),
            "unexpected parse error: {err_text}"
        );
    }

    #[test]
    fn parse_workflow_rejects_zero_timeout_with_step_context() {
        let yaml = r#"
id: TEST-WORKFLOW-ZERO-TIMEOUT
name: Invalid timeout
applies_to:
  capabilities: [auth]
  interfaces: [cli]
acceptance_criteria:
  - id: AC-1
    description: Should fail parsing
steps:
  - name: Wait for completion
    timeout: 0
    interaction:
      type: cli
      command: smith loop wait
"#;

        let err = serde_yaml::from_str::<ScenarioFile>(yaml)
            .expect_err("scenario should fail when timeout is zero");
        let err_text = err.to_string();

        assert!(
            err_text.contains("steps[0].timeout"),
            "unexpected parse error: {err_text}"
        );
        assert!(
            err_text.contains("positive integer"),
            "unexpected parse error: {err_text}"
        );
    }

    #[test]
    fn parse_workflow_rejects_negative_timeout_with_step_context() {
        let yaml = r#"
id: TEST-WORKFLOW-NEGATIVE-TIMEOUT
name: Invalid timeout
applies_to:
  capabilities: [auth]
  interfaces: [cli]
acceptance_criteria:
  - id: AC-1
    description: Should fail parsing
steps:
  - name: Wait for completion
    timeout: -5
    interaction:
      type: cli
      command: smith loop wait
"#;

        let err = serde_yaml::from_str::<ScenarioFile>(yaml)
            .expect_err("scenario should fail when timeout is negative");
        let err_text = err.to_string();

        assert!(
            err_text.contains("steps[0].timeout"),
            "unexpected parse error: {err_text}"
        );
        assert!(
            err_text.contains("got `-5`"),
            "unexpected parse error: {err_text}"
        );
    }

    #[test]
    fn legacy_single_fixture_parses_as_single_with_unchanged_checks() {
        let yaml = read_legacy_fixture("scenarios/cli/test-hello.yaml");
        let scenario: ScenarioFile =
            serde_yaml::from_str(&yaml).expect("legacy single-step fixture should parse");

        let (interaction, checks) = scenario
            .as_single()
            .expect("legacy fixture should remain single scenario");
        assert_eq!(scenario.kind_label(), "single");
        assert_eq!(interaction.interaction_type, "cli");
        assert_eq!(checks.len(), 2);
        assert_eq!(checks[0].check_type, "status_code");
        assert_eq!(checks[1].check_type, "stdout");
        assert_eq!(checks[0].acceptance_criteria, "AC-1");
        assert_eq!(checks[1].acceptance_criteria, "AC-1");
    }
}
