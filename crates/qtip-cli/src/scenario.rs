use serde::Deserialize;
use serde_json::Value;

/// Full scenario as defined in YAML files — bridges catalog discovery
/// to the resolver and executor.
#[derive(Debug, Clone, Deserialize)]
pub struct ScenarioFile {
    pub id: String,
    pub name: String,
    pub applies_to: AppliesTo,
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    pub interaction: Interaction,
    pub checks: Vec<Check>,
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
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectManifest {
    pub project_id: String,
    pub repo_url: Option<String>,
    pub commit: Option<String>,
    pub environment: String,
    pub interfaces: Vec<ManifestInterface>,
    pub observability: Option<Observability>,
    pub capabilities: Vec<String>,
    pub scenarios: Option<ScenariosConfig>,
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
#[serde(rename_all = "camelCase")]
pub struct ManifestInterface {
    #[serde(rename = "type")]
    pub interface_type: String,
    pub name: Option<String>,
    pub base_url: Option<String>,
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
