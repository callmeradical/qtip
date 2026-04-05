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
    setup: Option<Vec<ScenarioStep>>,
    steps: Option<Vec<ScenarioStep>>,
    teardown: Option<Vec<ScenarioStep>>,
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

                Ok(ScenarioKind::Workflow {
                    setup: self.setup.unwrap_or_default(),
                    steps,
                    teardown: self.teardown.unwrap_or_default(),
                })
            }
            (false, false) => {
                Err("Scenario must define either `interaction` or `steps`".to_string())
            }
        }
    }
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
    fn parse_workflow_scenario_preserves_step_order() {
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
}
