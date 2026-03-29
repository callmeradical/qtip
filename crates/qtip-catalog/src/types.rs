use std::collections::BTreeMap;
use std::time::SystemTime;

use crate::error::{CatalogError, CatalogEnvelope, CatalogStage};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioRef {
    pub id: String,
    pub path: String,
    pub source: String,
    pub fingerprint: Option<String>,
    pub discovered_at: SystemTime,
}

impl ScenarioRef {
    pub(crate) fn validate_contract(&self, expected_source: Option<&str>) -> Result<(), CatalogError> {
        if self.id.trim().is_empty() {
            return Err(CatalogError::invalid_source_record(
                "Scenario reference is missing an id",
                Some("id"),
                Some(self.path.clone()),
                Some(self.source.clone()),
            ));
        }

        if self.path.trim().is_empty() {
            return Err(CatalogError::invalid_source_record(
                "Scenario reference is missing a path",
                Some("path"),
                None,
                Some(self.source.clone()),
            ));
        }

        if self.source.trim().is_empty() {
            return Err(CatalogError::invalid_source_record(
                "Scenario reference is missing a source",
                Some("source"),
                Some(self.path.clone()),
                None,
            ));
        }

        if let Some(expected_source) = expected_source
            && self.source != expected_source
        {
            return Err(CatalogError::invalid_source_record(
                "Scenario reference source does not match source adapter id",
                Some("source"),
                Some(self.path.clone()),
                Some(self.source.clone()),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScenarioDocumentFormat {
    Yaml,
    Json,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioDocument {
    pub ref_id: String,
    pub raw: String,
    pub format: ScenarioDocumentFormat,
    pub bytes: usize,
}

impl ScenarioDocument {
    pub fn new(
        ref_id: impl Into<String>,
        raw: impl Into<String>,
        format: ScenarioDocumentFormat,
    ) -> Self {
        let raw = raw.into();
        Self {
            ref_id: ref_id.into(),
            bytes: raw.len(),
            raw,
            format,
        }
    }

    pub(crate) fn validate_contract(&self) -> Result<(), CatalogError> {
        if self.ref_id.trim().is_empty() {
            return Err(CatalogError::validation(
                "Scenario document is missing ref_id",
                CatalogStage::Load,
                Some("ref_id"),
                None,
                None,
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scenario {
    pub id: String,
    pub service: String,
    pub steps: Vec<ScenarioStep>,
    pub assertions: Vec<ScenarioAssertion>,
    pub tags: Vec<String>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioStep {
    pub order: usize,
    pub action: String,
    pub payload: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioAssertion {
    pub path: String,
    pub equals: String,
}

pub type ScenarioRefEnvelope = CatalogEnvelope<ScenarioRef>;
pub type ScenarioDocumentEnvelope = CatalogEnvelope<ScenarioDocument>;
pub type ScenarioEnvelope = CatalogEnvelope<Scenario>;
