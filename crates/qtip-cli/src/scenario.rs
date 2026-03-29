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
