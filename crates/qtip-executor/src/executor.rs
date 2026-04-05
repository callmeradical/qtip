use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use serde::Deserialize;

use crate::check::{Check, Evidence, evaluate_checks};

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvaluationStatus {
    Passed,
    Failed,
    Error,
}

#[derive(Debug, Clone)]
pub struct EvaluationResult {
    pub scenario_id: String,
    pub status: EvaluationStatus,
    pub evidence: Evidence,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Interaction {
    #[serde(rename = "type")]
    pub interaction_type: String,
    #[serde(flatten)]
    pub params: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExecutableScenario {
    pub id: String,
    pub name: String,
    pub interaction: Interaction,
    pub checks: Vec<Check>,
}

pub trait Adapter: Send + Sync {
    fn interaction_type(&self) -> &str;

    fn execute<'a>(
        &'a self,
        interaction: &'a Interaction,
    ) -> BoxFuture<'a, Result<Evidence, String>>;
}

pub struct ScenarioExecutor {
    adapters: HashMap<String, Box<dyn Adapter>>,
}

impl Default for ScenarioExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl ScenarioExecutor {
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    pub fn register_adapter(&mut self, adapter: Box<dyn Adapter>) {
        let key = adapter.interaction_type().to_string();
        self.adapters.insert(key, adapter);
    }

    pub async fn execute(&self, scenario: &ExecutableScenario) -> EvaluationResult {
        let adapter = self.adapters.get(&scenario.interaction.interaction_type);

        let Some(adapter) = adapter else {
            return EvaluationResult {
                scenario_id: scenario.id.clone(),
                status: EvaluationStatus::Error,
                evidence: Evidence::cli(1, "", ""),
                failures: vec![format!(
                    "Unsupported interaction type: {}",
                    scenario.interaction.interaction_type
                )],
            };
        };

        match adapter.execute(&scenario.interaction).await {
            Ok(evidence) => {
                let failures = evaluate_checks(&scenario.checks, &evidence);
                let status = if failures.is_empty() {
                    EvaluationStatus::Passed
                } else {
                    EvaluationStatus::Failed
                };
                EvaluationResult {
                    scenario_id: scenario.id.clone(),
                    status,
                    evidence,
                    failures,
                }
            }
            Err(message) => EvaluationResult {
                scenario_id: scenario.id.clone(),
                status: EvaluationStatus::Error,
                evidence: Evidence::cli(1, "", ""),
                failures: vec![message],
            },
        }
    }
}
