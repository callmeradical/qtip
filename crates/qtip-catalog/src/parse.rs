use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use serde_yaml::Value as YamlValue;

use crate::error::{CatalogError, CatalogStage};
use crate::types::{
    Scenario, ScenarioAssertion, ScenarioDocument, ScenarioRef, ScenarioStep,
};

#[derive(Debug, Clone)]
pub(crate) struct LoadedScenarioDocument {
    pub(crate) index: usize,
    pub(crate) scenario_ref: ScenarioRef,
    pub(crate) document: ScenarioDocument,
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedScenario {
    pub(crate) scenario_ref: ScenarioRef,
    pub(crate) scenario: Scenario,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawScenario {
    id: Option<String>,
    service: Option<String>,
    steps: Option<Vec<RawScenarioStep>>,
    assertions: Option<Vec<RawScenarioAssertion>>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    metadata: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct RawScenarioStep {
    order: Option<usize>,
    action: Option<String>,
    #[serde(default)]
    payload: BTreeMap<String, YamlValue>,
}

#[derive(Debug, Deserialize)]
struct RawScenarioAssertion {
    path: Option<String>,
    equals: Option<YamlValue>,
}

impl RawScenario {
    pub(crate) fn into_scenario(self, scenario_ref: &ScenarioRef) -> Result<Scenario, CatalogError> {
        let id = required_non_empty(self.id, "id", scenario_ref)?;
        let service = required_non_empty(self.service, "service", scenario_ref)?;

        let raw_steps = self.steps.ok_or_else(|| {
            CatalogError::validation(
                "Scenario is missing required field `steps`",
                CatalogStage::Validate,
                Some("steps"),
                Some(scenario_ref.path.clone()),
                Some(scenario_ref.source.clone()),
            )
        })?;
        if raw_steps.is_empty() {
            return Err(CatalogError::validation(
                "Scenario field `steps` must contain at least one step",
                CatalogStage::Validate,
                Some("steps"),
                Some(scenario_ref.path.clone()),
                Some(scenario_ref.source.clone()),
            ));
        }

        let mut steps = Vec::with_capacity(raw_steps.len());
        let mut seen_orders = BTreeSet::new();
        for (index, raw_step) in raw_steps.into_iter().enumerate() {
            let order = raw_step.order.ok_or_else(|| {
                CatalogError::validation(
                    format!("Scenario step at index {index} is missing required field `order`"),
                    CatalogStage::Validate,
                    Some("steps.order"),
                    Some(scenario_ref.path.clone()),
                    Some(scenario_ref.source.clone()),
                )
            })?;
            if order == 0 {
                return Err(CatalogError::validation(
                    format!("Scenario step at index {index} has non-positive `order`"),
                    CatalogStage::Validate,
                    Some("steps.order"),
                    Some(scenario_ref.path.clone()),
                    Some(scenario_ref.source.clone()),
                ));
            }
            if !seen_orders.insert(order) {
                return Err(CatalogError::validation(
                    format!("Scenario contains duplicate step order value `{order}`"),
                    CatalogStage::Validate,
                    Some("steps.order"),
                    Some(scenario_ref.path.clone()),
                    Some(scenario_ref.source.clone()),
                ));
            }

            let action = required_non_empty(raw_step.action, "steps.action", scenario_ref)?;

            let mut payload = BTreeMap::new();
            for (key, value) in raw_step.payload {
                let value = yaml_scalar_to_string(value).ok_or_else(|| {
                    CatalogError::validation(
                        format!(
                            "Scenario step payload value for key `{key}` must be a scalar value"
                        ),
                        CatalogStage::Validate,
                        Some("steps.payload"),
                        Some(scenario_ref.path.clone()),
                        Some(scenario_ref.source.clone()),
                    )
                })?;
                payload.insert(key, value);
            }

            steps.push(ScenarioStep {
                order,
                action,
                payload,
            });
        }

        let raw_assertions = self.assertions.ok_or_else(|| {
            CatalogError::validation(
                "Scenario is missing required field `assertions`",
                CatalogStage::Validate,
                Some("assertions"),
                Some(scenario_ref.path.clone()),
                Some(scenario_ref.source.clone()),
            )
        })?;
        if raw_assertions.is_empty() {
            return Err(CatalogError::validation(
                "Scenario field `assertions` must contain at least one assertion",
                CatalogStage::Validate,
                Some("assertions"),
                Some(scenario_ref.path.clone()),
                Some(scenario_ref.source.clone()),
            ));
        }

        let mut assertions = Vec::with_capacity(raw_assertions.len());
        for (index, raw_assertion) in raw_assertions.into_iter().enumerate() {
            let path = required_non_empty(raw_assertion.path, "assertions.path", scenario_ref)?;
            let equals = raw_assertion.equals.ok_or_else(|| {
                CatalogError::validation(
                    format!(
                        "Scenario assertion at index {index} is missing required field `equals`"
                    ),
                    CatalogStage::Validate,
                    Some("assertions.equals"),
                    Some(scenario_ref.path.clone()),
                    Some(scenario_ref.source.clone()),
                )
            })?;
            let equals = yaml_scalar_to_string(equals).ok_or_else(|| {
                CatalogError::validation(
                    format!(
                        "Scenario assertion at index {index} field `equals` must be a scalar value"
                    ),
                    CatalogStage::Validate,
                    Some("assertions.equals"),
                    Some(scenario_ref.path.clone()),
                    Some(scenario_ref.source.clone()),
                )
            })?;

            assertions.push(ScenarioAssertion { path, equals });
        }

        Ok(Scenario {
            id,
            service,
            steps,
            assertions,
            tags: self.tags,
            metadata: self.metadata,
        })
    }
}

fn required_non_empty(
    value: Option<String>,
    field: &'static str,
    scenario_ref: &ScenarioRef,
) -> Result<String, CatalogError> {
    let value = value.ok_or_else(|| {
        CatalogError::validation(
            format!("Scenario is missing required field `{field}`"),
            CatalogStage::Validate,
            Some(field),
            Some(scenario_ref.path.clone()),
            Some(scenario_ref.source.clone()),
        )
    })?;

    if value.trim().is_empty() {
        return Err(CatalogError::validation(
            format!("Scenario field `{field}` cannot be empty"),
            CatalogStage::Validate,
            Some(field),
            Some(scenario_ref.path.clone()),
            Some(scenario_ref.source.clone()),
        ));
    }

    Ok(value)
}

fn yaml_scalar_to_string(value: YamlValue) -> Option<String> {
    match value {
        YamlValue::Null => None,
        YamlValue::Bool(value) => Some(value.to_string()),
        YamlValue::Number(value) => Some(value.to_string()),
        YamlValue::String(value) => Some(value),
        _ => None,
    }
}
