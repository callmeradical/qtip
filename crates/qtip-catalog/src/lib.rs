#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::time::SystemTime;

/// Returns the crate name used by downstream integration tests and wiring checks.
pub fn crate_id() -> &'static str {
    "qtip-catalog"
}

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioRef {
    pub id: String,
    pub path: String,
    pub source: String,
    pub fingerprint: Option<String>,
    pub discovered_at: SystemTime,
}

impl ScenarioRef {
    fn validate_contract(&self, expected_source: Option<&str>) -> Result<(), CatalogError> {
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

    fn validate_contract(&self) -> Result<(), CatalogError> {
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
    pub action: String,
    pub payload: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioAssertion {
    pub path: String,
    pub equals: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogErrorCode {
    InvalidSourceRecord,
    DiscoveryFailure,
    LoadFailure,
    ParseFailure,
    ValidationFailure,
    Conflict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogStage {
    Contract,
    Discover,
    Load,
    Parse,
    Validate,
    Catalog,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogError {
    pub code: CatalogErrorCode,
    pub message: String,
    pub path: Option<String>,
    pub source: Option<String>,
    pub stage: CatalogStage,
    pub details: BTreeMap<String, String>,
}

impl std::fmt::Display for CatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for CatalogError {}

impl CatalogError {
    pub fn invalid_source_record(
        message: impl Into<String>,
        field: Option<&str>,
        path: Option<String>,
        source: Option<String>,
    ) -> Self {
        let mut details = BTreeMap::new();
        if let Some(field) = field {
            details.insert("field".to_string(), field.to_string());
        }

        Self {
            code: CatalogErrorCode::InvalidSourceRecord,
            message: message.into(),
            path,
            source,
            stage: CatalogStage::Contract,
            details,
        }
    }

    pub fn validation(
        message: impl Into<String>,
        stage: CatalogStage,
        field: Option<&str>,
        path: Option<String>,
        source: Option<String>,
    ) -> Self {
        let mut details = BTreeMap::new();
        if let Some(field) = field {
            details.insert("field".to_string(), field.to_string());
        }

        Self {
            code: CatalogErrorCode::ValidationFailure,
            message: message.into(),
            path,
            source,
            stage,
            details,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogEnvelope<T> {
    pub items: Vec<T>,
    pub errors: Vec<CatalogError>,
}

impl<T> CatalogEnvelope<T> {
    pub fn new(items: Vec<T>, errors: Vec<CatalogError>) -> Self {
        Self { items, errors }
    }

    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

pub type ScenarioRefEnvelope = CatalogEnvelope<ScenarioRef>;
pub type ScenarioDocumentEnvelope = CatalogEnvelope<ScenarioDocument>;
pub type ScenarioEnvelope = CatalogEnvelope<Scenario>;

pub trait ScenarioSource: Send + Sync {
    fn source_id(&self) -> &str;

    /// Deep discovery boundary: filesystem/network traversal can stay synchronous.
    fn discover_refs(&self) -> Result<Vec<ScenarioRef>, CatalogError>;

    /// Loading boundary: reading records can be asynchronous.
    fn load_document<'a>(
        &'a self,
        scenario_ref: &'a ScenarioRef,
    ) -> BoxFuture<'a, Result<ScenarioDocument, CatalogError>>;

    fn discover(&self) -> ScenarioRefEnvelope {
        let source_id = self.source_id().trim();
        if source_id.is_empty() {
            return ScenarioRefEnvelope::new(
                Vec::new(),
                vec![CatalogError::invalid_source_record(
                    "Scenario source adapter is missing source_id",
                    Some("source_id"),
                    None,
                    None,
                )],
            );
        }

        match self.discover_refs() {
            Ok(refs) => {
                let mut items = Vec::with_capacity(refs.len());
                let mut errors = Vec::new();

                for scenario_ref in refs {
                    match scenario_ref.validate_contract(Some(source_id)) {
                        Ok(()) => items.push(scenario_ref),
                        Err(error) => errors.push(error),
                    }
                }

                ScenarioRefEnvelope::new(items, errors)
            }
            Err(error) => ScenarioRefEnvelope::new(Vec::new(), vec![error]),
        }
    }

    fn load_documents<'a>(
        &'a self,
        refs: &'a [ScenarioRef],
    ) -> BoxFuture<'a, ScenarioDocumentEnvelope> {
        Box::pin(async move {
            let mut items = Vec::with_capacity(refs.len());
            let mut errors = Vec::new();

            for scenario_ref in refs {
                if let Err(error) = scenario_ref.validate_contract(Some(self.source_id().trim())) {
                    errors.push(error);
                    continue;
                }

                match self.load_document(scenario_ref).await {
                    Ok(document) => {
                        if let Err(error) = document.validate_contract() {
                            errors.push(error);
                            continue;
                        }

                        items.push(document);
                    }
                    Err(error) => errors.push(error),
                }
            }

            ScenarioDocumentEnvelope::new(items, errors)
        })
    }
}

pub trait ScenarioCatalog: Send + Sync {
    fn discover(&self) -> ScenarioRefEnvelope;
    fn load<'a>(&'a self, refs: &'a [ScenarioRef]) -> BoxFuture<'a, ScenarioDocumentEnvelope>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::task::{Context, Poll, Wake, Waker};

    #[derive(Debug)]
    struct MockSource {
        source_id: String,
        refs: Vec<ScenarioRef>,
        docs: HashMap<String, String>,
    }

    impl ScenarioSource for MockSource {
        fn source_id(&self) -> &str {
            &self.source_id
        }

        fn discover_refs(&self) -> Result<Vec<ScenarioRef>, CatalogError> {
            Ok(self.refs.clone())
        }

        fn load_document<'a>(
            &'a self,
            scenario_ref: &'a ScenarioRef,
        ) -> BoxFuture<'a, Result<ScenarioDocument, CatalogError>> {
            Box::pin(async move {
                let raw = self.docs.get(&scenario_ref.path).ok_or_else(|| {
                    CatalogError::validation(
                        "No in-memory document found for scenario path",
                        CatalogStage::Load,
                        Some("path"),
                        Some(scenario_ref.path.clone()),
                        Some(self.source_id.clone()),
                    )
                })?;

                Ok(ScenarioDocument::new(
                    scenario_ref.id.clone(),
                    raw.clone(),
                    ScenarioDocumentFormat::Yaml,
                ))
            })
        }
    }

    #[test]
    fn crate_id_is_stable() {
        assert_eq!(crate_id(), "qtip-catalog");
    }

    #[test]
    fn mock_source_feeds_two_in_memory_files_through_trait_methods() {
        let source = MockSource {
            source_id: "memory".to_string(),
            refs: vec![
                ScenarioRef {
                    id: "login-success".to_string(),
                    path: "scenarios/auth/login-success.yaml".to_string(),
                    source: "memory".to_string(),
                    fingerprint: Some("fp1".to_string()),
                    discovered_at: SystemTime::UNIX_EPOCH,
                },
                ScenarioRef {
                    id: "logout-success".to_string(),
                    path: "scenarios/auth/logout-success.yaml".to_string(),
                    source: "memory".to_string(),
                    fingerprint: Some("fp2".to_string()),
                    discovered_at: SystemTime::UNIX_EPOCH,
                },
            ],
            docs: HashMap::from([
                (
                    "scenarios/auth/login-success.yaml".to_string(),
                    "id: login-success".to_string(),
                ),
                (
                    "scenarios/auth/logout-success.yaml".to_string(),
                    "id: logout-success".to_string(),
                ),
            ]),
        };

        let discovered = source.discover();
        assert!(discovered.errors.is_empty());
        assert_eq!(discovered.items.len(), 2);

        let loaded = block_on(source.load_documents(&discovered.items));
        assert!(loaded.errors.is_empty());
        assert_eq!(loaded.items.len(), 2);
        assert_eq!(loaded.items[0].ref_id, "login-success");
        assert_eq!(loaded.items[1].ref_id, "logout-success");
    }

    #[test]
    fn trait_contract_rejects_empty_path_and_missing_source_id() {
        let missing_source_id = MockSource {
            source_id: "".to_string(),
            refs: vec![ScenarioRef {
                id: "login-success".to_string(),
                path: "scenarios/auth/login-success.yaml".to_string(),
                source: "memory".to_string(),
                fingerprint: None,
                discovered_at: SystemTime::UNIX_EPOCH,
            }],
            docs: HashMap::new(),
        };

        let missing_source_result = missing_source_id.discover();
        assert!(missing_source_result.items.is_empty());
        assert_eq!(missing_source_result.errors.len(), 1);
        assert_eq!(
            missing_source_result.errors[0].code,
            CatalogErrorCode::InvalidSourceRecord
        );
        assert_eq!(
            missing_source_result.errors[0].stage,
            CatalogStage::Contract
        );
        assert_eq!(
            missing_source_result.errors[0]
                .details
                .get("field")
                .map(String::as_str),
            Some("source_id")
        );

        let empty_path_source = MockSource {
            source_id: "memory".to_string(),
            refs: vec![ScenarioRef {
                id: "login-success".to_string(),
                path: "".to_string(),
                source: "memory".to_string(),
                fingerprint: None,
                discovered_at: SystemTime::UNIX_EPOCH,
            }],
            docs: HashMap::new(),
        };

        let empty_path_result = empty_path_source.discover();
        assert!(empty_path_result.items.is_empty());
        assert_eq!(empty_path_result.errors.len(), 1);
        assert_eq!(
            empty_path_result.errors[0].code,
            CatalogErrorCode::InvalidSourceRecord
        );
        assert_eq!(empty_path_result.errors[0].stage, CatalogStage::Contract);
        assert_eq!(
            empty_path_result.errors[0]
                .details
                .get("field")
                .map(String::as_str),
            Some("path")
        );
    }

    fn block_on<F>(future: F) -> F::Output
    where
        F: Future,
    {
        let mut future = std::pin::pin!(future);
        let waker = noop_waker();
        let mut context = Context::from_waker(&waker);

        loop {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(output) => return output,
                Poll::Pending => std::thread::yield_now(),
            }
        }
    }

    #[derive(Debug)]
    struct NoopWake;

    impl Wake for NoopWake {
        fn wake(self: Arc<Self>) {}
    }

    fn noop_waker() -> Waker {
        Waker::from(Arc::new(NoopWake))
    }
}
