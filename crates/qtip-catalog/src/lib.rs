#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::path::{Component, Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::time::SystemTime;

use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use rayon::prelude::*;
use serde::Deserialize;
use serde_yaml::Value as YamlValue;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

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
    pub order: usize,
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

    pub fn discovery(
        message: impl Into<String>,
        path: Option<String>,
        source: Option<String>,
    ) -> Self {
        Self {
            code: CatalogErrorCode::DiscoveryFailure,
            message: message.into(),
            path,
            source,
            stage: CatalogStage::Discover,
            details: BTreeMap::new(),
        }
    }

    pub fn load(message: impl Into<String>, path: Option<String>, source: Option<String>) -> Self {
        Self {
            code: CatalogErrorCode::LoadFailure,
            message: message.into(),
            path,
            source,
            stage: CatalogStage::Load,
            details: BTreeMap::new(),
        }
    }

    pub fn parse(message: impl Into<String>, path: Option<String>, source: Option<String>) -> Self {
        Self {
            code: CatalogErrorCode::ParseFailure,
            message: message.into(),
            path,
            source,
            stage: CatalogStage::Parse,
            details: BTreeMap::new(),
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

#[derive(Debug, Deserialize)]
struct RawScenario {
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

#[derive(Debug, Clone)]
struct LoadedScenarioDocument {
    index: usize,
    scenario_ref: ScenarioRef,
    document: ScenarioDocument,
}

impl RawScenario {
    fn into_scenario(self, scenario_ref: &ScenarioRef) -> Result<Scenario, CatalogError> {
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
    fn load_scenarios<'a>(&'a self, refs: &'a [ScenarioRef]) -> BoxFuture<'a, ScenarioEnvelope>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandardScenarioCatalogConfig {
    pub io_concurrency_limit: usize,
}

impl Default for StandardScenarioCatalogConfig {
    fn default() -> Self {
        Self {
            io_concurrency_limit: 100,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StandardScenarioCatalog<S>
where
    S: ScenarioSource + 'static,
{
    source: Arc<S>,
    io_concurrency_limit: usize,
}

impl<S> StandardScenarioCatalog<S>
where
    S: ScenarioSource + 'static,
{
    pub fn new(source: S, config: StandardScenarioCatalogConfig) -> Result<Self, CatalogError> {
        Self::from_arc(Arc::new(source), config)
    }

    pub fn from_arc(
        source: Arc<S>,
        config: StandardScenarioCatalogConfig,
    ) -> Result<Self, CatalogError> {
        if config.io_concurrency_limit == 0 {
            return Err(CatalogError::validation(
                "StandardScenarioCatalog requires io_concurrency_limit greater than zero",
                CatalogStage::Catalog,
                Some("io_concurrency_limit"),
                None,
                Some(source.source_id().to_string()),
            ));
        }

        Ok(Self {
            source,
            io_concurrency_limit: config.io_concurrency_limit,
        })
    }

    async fn load_with_bounded_io(&self, refs: &[ScenarioRef]) -> ScenarioDocumentEnvelope {
        let source_id = self.source.source_id().trim().to_string();
        if source_id.is_empty() {
            return ScenarioDocumentEnvelope::new(
                Vec::new(),
                vec![CatalogError::invalid_source_record(
                    "Scenario source adapter is missing source_id",
                    Some("source_id"),
                    None,
                    None,
                )],
            );
        }

        let mut validated_refs = Vec::with_capacity(refs.len());
        let mut errors = Vec::new();

        for (index, scenario_ref) in refs.iter().enumerate() {
            match scenario_ref.validate_contract(Some(source_id.as_str())) {
                Ok(()) => validated_refs.push((index, scenario_ref.clone())),
                Err(error) => errors.push(error),
            }
        }

        if validated_refs.is_empty() {
            return ScenarioDocumentEnvelope::new(Vec::new(), errors);
        }

        let loaded = self.read_documents_async(validated_refs).await;
        errors.extend(loaded.errors);

        let validated = Self::validate_loaded_documents(loaded.items);
        errors.extend(validated.errors);
        let items = validated
            .items
            .into_iter()
            .map(|loaded| loaded.document)
            .collect::<Vec<ScenarioDocument>>();

        ScenarioDocumentEnvelope::new(items, errors)
    }

    async fn load_scenarios_with_bounded_io(&self, refs: &[ScenarioRef]) -> ScenarioEnvelope {
        let source_id = self.source.source_id().trim().to_string();
        if source_id.is_empty() {
            return ScenarioEnvelope::new(
                Vec::new(),
                vec![CatalogError::invalid_source_record(
                    "Scenario source adapter is missing source_id",
                    Some("source_id"),
                    None,
                    None,
                )],
            );
        }

        let mut validated_refs = Vec::with_capacity(refs.len());
        let mut errors = Vec::new();

        for (index, scenario_ref) in refs.iter().enumerate() {
            match scenario_ref.validate_contract(Some(source_id.as_str())) {
                Ok(()) => validated_refs.push((index, scenario_ref.clone())),
                Err(error) => errors.push(error),
            }
        }

        if validated_refs.is_empty() {
            return ScenarioEnvelope::new(Vec::new(), errors);
        }

        let loaded = self.read_documents_async(validated_refs).await;
        errors.extend(loaded.errors);

        let validated_documents = Self::validate_loaded_documents(loaded.items);
        errors.extend(validated_documents.errors);

        let parsed = Self::parse_and_validate_scenarios(validated_documents.items);
        errors.extend(parsed.errors);

        ScenarioEnvelope::new(parsed.items, errors)
    }

    async fn read_documents_async(
        &self,
        refs: Vec<(usize, ScenarioRef)>,
    ) -> CatalogEnvelope<LoadedScenarioDocument> {
        let semaphore = Arc::new(Semaphore::new(self.io_concurrency_limit));
        let mut tasks = JoinSet::new();

        for (index, scenario_ref) in refs {
            let source = Arc::clone(&self.source);
            let source_id = source.source_id().to_string();
            let semaphore = Arc::clone(&semaphore);
            tasks.spawn(async move {
                let permit = semaphore.acquire_owned().await.map_err(|_| {
                    CatalogError::load(
                        "Loader semaphore closed before acquiring permit",
                        Some(scenario_ref.path.clone()),
                        Some(source_id.clone()),
                    )
                })?;

                let loaded = source.load_document(&scenario_ref).await;
                drop(permit);

                loaded.map(|document| LoadedScenarioDocument {
                    index,
                    scenario_ref,
                    document,
                })
            });
        }

        let mut items = Vec::new();
        let mut errors = Vec::new();

        while let Some(joined) = tasks.join_next().await {
            match joined {
                Ok(Ok(item)) => items.push(item),
                Ok(Err(error)) => errors.push(error),
                Err(join_error) => errors.push(CatalogError::load(
                    format!("Loader task failed before completion: {join_error}"),
                    None,
                    Some(self.source.source_id().to_string()),
                )),
            }
        }

        CatalogEnvelope::new(items, errors)
    }

    fn validate_loaded_documents(
        mut documents: Vec<LoadedScenarioDocument>,
    ) -> CatalogEnvelope<LoadedScenarioDocument> {
        documents.sort_by_key(|loaded| loaded.index);

        let validated = documents
            .into_par_iter()
            .map(|loaded| {
                if let Err(mut error) = loaded.document.validate_contract() {
                    if error.path.is_none() {
                        error.path = Some(loaded.scenario_ref.path.clone());
                    }
                    if error.source.is_none() {
                        error.source = Some(loaded.scenario_ref.source.clone());
                    }
                    return Err(error);
                }

                Ok(loaded)
            })
            .collect::<Vec<Result<LoadedScenarioDocument, CatalogError>>>();

        let mut items = Vec::new();
        let mut errors = Vec::new();

        for result in validated {
            match result {
                Ok(loaded) => items.push(loaded),
                Err(error) => errors.push(error),
            }
        }

        CatalogEnvelope::new(items, errors)
    }

    fn parse_and_validate_scenarios(documents: Vec<LoadedScenarioDocument>) -> ScenarioEnvelope {
        let parsed = documents
            .into_par_iter()
            .map(Self::parse_and_validate_scenario_document)
            .collect::<Vec<Result<Scenario, CatalogError>>>();

        let mut items = Vec::new();
        let mut errors = Vec::new();

        for result in parsed {
            match result {
                Ok(scenario) => items.push(scenario),
                Err(error) => errors.push(error),
            }
        }

        ScenarioEnvelope::new(items, errors)
    }

    fn parse_and_validate_scenario_document(
        loaded: LoadedScenarioDocument,
    ) -> Result<Scenario, CatalogError> {
        if !matches!(loaded.document.format, ScenarioDocumentFormat::Yaml) {
            return Err(CatalogError::parse(
                format!(
                    "Unsupported scenario document format `{:?}`; only YAML is supported",
                    loaded.document.format
                ),
                Some(loaded.scenario_ref.path.clone()),
                Some(loaded.scenario_ref.source.clone()),
            ));
        }

        let parsed =
            serde_yaml::from_str::<RawScenario>(&loaded.document.raw).map_err(|error| {
                CatalogError::parse(
                    format!("Failed to parse scenario YAML: {error}"),
                    Some(loaded.scenario_ref.path.clone()),
                    Some(loaded.scenario_ref.source.clone()),
                )
            })?;

        parsed.into_scenario(&loaded.scenario_ref)
    }
}

impl<S> ScenarioCatalog for StandardScenarioCatalog<S>
where
    S: ScenarioSource + 'static,
{
    fn discover(&self) -> ScenarioRefEnvelope {
        self.source.discover()
    }

    fn load<'a>(&'a self, refs: &'a [ScenarioRef]) -> BoxFuture<'a, ScenarioDocumentEnvelope> {
        Box::pin(async move { self.load_with_bounded_io(refs).await })
    }

    fn load_scenarios<'a>(&'a self, refs: &'a [ScenarioRef]) -> BoxFuture<'a, ScenarioEnvelope> {
        Box::pin(async move { self.load_scenarios_with_bounded_io(refs).await })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileSystemSourceConfig {
    pub source_id: String,
    pub roots: Vec<PathBuf>,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub ignore_patterns: Vec<String>,
    pub respect_gitignore: bool,
}

impl FileSystemSourceConfig {
    pub fn new(source_id: impl Into<String>) -> Self {
        Self {
            source_id: source_id.into(),
            roots: Vec::new(),
            include_patterns: vec!["**/*.yaml".to_string()],
            exclude_patterns: Vec::new(),
            ignore_patterns: Vec::new(),
            respect_gitignore: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileSystemSource {
    source_id: String,
    roots: Vec<PathBuf>,
    include_patterns: GlobSet,
    exclude_patterns: GlobSet,
    ignore_patterns: GlobSet,
    respect_gitignore: bool,
}

impl FileSystemSource {
    pub fn new(config: FileSystemSourceConfig) -> Result<Self, CatalogError> {
        if config.source_id.trim().is_empty() {
            return Err(CatalogError::invalid_source_record(
                "FileSystemSource is missing source_id",
                Some("source_id"),
                None,
                None,
            ));
        }

        if config.roots.is_empty() {
            return Err(CatalogError::invalid_source_record(
                "FileSystemSource requires at least one root",
                Some("roots"),
                None,
                Some(config.source_id.clone()),
            ));
        }

        if config.include_patterns.is_empty() {
            return Err(CatalogError::invalid_source_record(
                "FileSystemSource requires at least one include pattern",
                Some("include_patterns"),
                None,
                Some(config.source_id.clone()),
            ));
        }

        let include_patterns = build_glob_set(
            &config.include_patterns,
            "include_patterns",
            &config.source_id,
        )?;
        let exclude_patterns = build_glob_set(
            &config.exclude_patterns,
            "exclude_patterns",
            &config.source_id,
        )?;
        let ignore_patterns = build_glob_set(
            &config.ignore_patterns,
            "ignore_patterns",
            &config.source_id,
        )?;

        Ok(Self {
            source_id: config.source_id,
            roots: config.roots,
            include_patterns,
            exclude_patterns,
            ignore_patterns,
            respect_gitignore: config.respect_gitignore,
        })
    }

    fn discover_with_errors(&self) -> ScenarioRefEnvelope {
        let mut items = Vec::new();
        let mut errors = Vec::new();

        for root in &self.roots {
            let mut walker = WalkBuilder::new(root);
            walker.git_ignore(self.respect_gitignore);
            walker.git_global(self.respect_gitignore);
            walker.git_exclude(self.respect_gitignore);
            walker.require_git(false);
            let root_for_filter = root.clone();
            let exclude_patterns = self.exclude_patterns.clone();
            let ignore_patterns = self.ignore_patterns.clone();
            walker.filter_entry(move |entry| {
                let relative_path = entry
                    .path()
                    .strip_prefix(&root_for_filter)
                    .unwrap_or_else(|_| entry.path());
                let normalized_path = normalize_glob_path(relative_path);
                !exclude_patterns.is_match(&normalized_path)
                    && !ignore_patterns.is_match(&normalized_path)
            });

            for entry in walker.build() {
                match entry {
                    Ok(entry) => {
                        if !entry
                            .file_type()
                            .is_some_and(|file_type| file_type.is_file())
                        {
                            continue;
                        }

                        let relative_path = entry
                            .path()
                            .strip_prefix(root)
                            .unwrap_or_else(|_| entry.path());
                        let normalized_path = normalize_glob_path(relative_path);

                        if !self.include_patterns.is_match(&normalized_path) {
                            continue;
                        }

                        if self.exclude_patterns.is_match(&normalized_path)
                            || self.ignore_patterns.is_match(&normalized_path)
                        {
                            continue;
                        }

                        items.push(ScenarioRef {
                            id: derive_scenario_id(entry.path()),
                            path: normalized_path,
                            source: self.source_id.clone(),
                            fingerprint: None,
                            discovered_at: SystemTime::now(),
                        });
                    }
                    Err(error) => {
                        let path = ignore_error_path(&error);
                        errors.push(CatalogError::discovery(
                            format!("Filesystem discovery failed: {error}"),
                            path,
                            Some(self.source_id.clone()),
                        ));
                    }
                }
            }
        }

        items.sort_by(|left, right| left.path.cmp(&right.path).then(left.id.cmp(&right.id)));
        ScenarioRefEnvelope::new(items, errors)
    }
}

impl ScenarioSource for FileSystemSource {
    fn source_id(&self) -> &str {
        &self.source_id
    }

    fn discover_refs(&self) -> Result<Vec<ScenarioRef>, CatalogError> {
        let envelope = self.discover_with_errors();
        if envelope.items.is_empty() && !envelope.errors.is_empty() {
            return Err(envelope.errors[0].clone());
        }

        Ok(envelope.items)
    }

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

        let discovered = self.discover_with_errors();
        let mut items = Vec::with_capacity(discovered.items.len());
        let mut errors = discovered.errors;

        for scenario_ref in discovered.items {
            match scenario_ref.validate_contract(Some(source_id)) {
                Ok(()) => items.push(scenario_ref),
                Err(error) => errors.push(error),
            }
        }

        ScenarioRefEnvelope::new(items, errors)
    }

    fn load_document<'a>(
        &'a self,
        scenario_ref: &'a ScenarioRef,
    ) -> BoxFuture<'a, Result<ScenarioDocument, CatalogError>> {
        Box::pin(async move {
            let normalized_path = scenario_ref.path.replace('\\', "/");
            let requested_path = Path::new(&scenario_ref.path);

            if requested_path.is_absolute()
                || requested_path
                    .components()
                    .any(|component| matches!(component, Component::ParentDir))
            {
                return Err(CatalogError::load(
                    "Scenario document path must be relative and cannot contain '..'",
                    Some(normalized_path),
                    Some(self.source_id.clone()),
                ));
            }

            for root in &self.roots {
                let absolute_path = root.join(&scenario_ref.path);
                match std::fs::read_to_string(&absolute_path) {
                    Ok(raw) => {
                        return Ok(ScenarioDocument::new(
                            scenario_ref.id.clone(),
                            raw,
                            detect_document_format(&absolute_path),
                        ));
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                    Err(error) => {
                        return Err(CatalogError::load(
                            format!("Failed to read scenario document: {error}"),
                            Some(normalized_path),
                            Some(self.source_id.clone()),
                        ));
                    }
                }
            }

            Err(CatalogError::load(
                "Scenario document was not found in configured roots",
                Some(normalized_path),
                Some(self.source_id.clone()),
            ))
        })
    }
}

fn build_glob_set(
    patterns: &[String],
    field: &'static str,
    source: &str,
) -> Result<GlobSet, CatalogError> {
    let mut builder = GlobSetBuilder::new();

    for pattern in patterns {
        let glob = Glob::new(pattern).map_err(|error| {
            CatalogError::invalid_source_record(
                format!("Invalid glob pattern '{pattern}': {error}"),
                Some(field),
                None,
                Some(source.to_string()),
            )
        })?;
        builder.add(glob);
    }

    builder.build().map_err(|error| {
        CatalogError::invalid_source_record(
            format!("Failed to compile glob patterns for {field}: {error}"),
            Some(field),
            None,
            Some(source.to_string()),
        )
    })
}

fn normalize_glob_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn ignore_error_path(error: &ignore::Error) -> Option<String> {
    match error {
        ignore::Error::Partial(errors) => errors.iter().find_map(ignore_error_path),
        ignore::Error::WithLineNumber { err, .. } => ignore_error_path(err),
        ignore::Error::WithPath { path, .. } => Some(normalize_glob_path(path)),
        ignore::Error::WithDepth { err, .. } => ignore_error_path(err),
        ignore::Error::Loop { child, .. } => Some(normalize_glob_path(child)),
        ignore::Error::Io(_) => None,
        ignore::Error::Glob { .. } => None,
        ignore::Error::UnrecognizedFileType(_) => None,
        ignore::Error::InvalidDefinition => None,
    }
}

fn derive_scenario_id(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map_or_else(String::new, ToString::to_string)
}

fn detect_document_format(path: &Path) -> ScenarioDocumentFormat {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .as_deref()
    {
        Some("yaml") | Some("yml") => ScenarioDocumentFormat::Yaml,
        Some("json") => ScenarioDocumentFormat::Json,
        Some(other) => ScenarioDocumentFormat::Other(other.to_string()),
        None => ScenarioDocumentFormat::Other("unknown".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};
    use std::path::Path;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::task::{Context, Poll, Wake, Waker};
    use std::time::{Duration, UNIX_EPOCH};

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

    #[derive(Debug)]
    struct ConcurrencyTrackingSource {
        source_id: String,
        docs: HashMap<String, String>,
        failed_paths: HashSet<String>,
        active_loads: Arc<AtomicUsize>,
        peak_loads: Arc<AtomicUsize>,
        io_delay: Duration,
    }

    impl ScenarioSource for ConcurrencyTrackingSource {
        fn source_id(&self) -> &str {
            &self.source_id
        }

        fn discover_refs(&self) -> Result<Vec<ScenarioRef>, CatalogError> {
            Ok(Vec::new())
        }

        fn load_document<'a>(
            &'a self,
            scenario_ref: &'a ScenarioRef,
        ) -> BoxFuture<'a, Result<ScenarioDocument, CatalogError>> {
            Box::pin(async move {
                let active = self.active_loads.fetch_add(1, Ordering::SeqCst) + 1;
                update_peak(&self.peak_loads, active);

                std::thread::sleep(self.io_delay);

                self.active_loads.fetch_sub(1, Ordering::SeqCst);

                if self.failed_paths.contains(&scenario_ref.path) {
                    return Err(CatalogError::load(
                        "Simulated source read failure",
                        Some(scenario_ref.path.clone()),
                        Some(self.source_id.clone()),
                    ));
                }

                let raw = self.docs.get(&scenario_ref.path).ok_or_else(|| {
                    CatalogError::load(
                        "No in-memory document found for scenario path",
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

    #[test]
    fn filesystem_source_discovers_nested_yaml_with_recursive_include() {
        let test_dir = TestDir::new("discover-recursive");
        write_file(
            &test_dir.path.join("scenarios/auth/login-success.yaml"),
            "id: login-success",
        );
        write_file(&test_dir.path.join("scenarios/auth/readme.txt"), "ignored");

        let mut config = FileSystemSourceConfig::new("filesystem");
        config.roots.push(test_dir.path.clone());
        config.include_patterns = vec!["**/*.yaml".to_string()];

        let source = FileSystemSource::new(config).expect("filesystem source config should build");
        let discovered = source.discover();

        assert!(discovered.errors.is_empty());
        assert_eq!(discovered.items.len(), 1);
        assert_eq!(discovered.items[0].id, "login-success");
        assert_eq!(
            discovered.items[0].path,
            "scenarios/auth/login-success.yaml"
        );
    }

    #[test]
    fn filesystem_source_applies_explicit_ignore_patterns() {
        let test_dir = TestDir::new("discover-explicit-ignore");
        write_file(
            &test_dir.path.join("scenarios/auth/login-success.yaml"),
            "id: login-success",
        );
        write_file(
            &test_dir.path.join("tmp/experimental.yaml"),
            "id: experimental",
        );

        let mut config = FileSystemSourceConfig::new("filesystem");
        config.roots.push(test_dir.path.clone());
        config.include_patterns = vec!["**/*.yaml".to_string()];
        config.ignore_patterns = vec!["tmp/**".to_string()];

        let source = FileSystemSource::new(config).expect("filesystem source config should build");
        let discovered = source.discover();

        assert!(discovered.errors.is_empty());
        assert_eq!(discovered.items.len(), 1);
        assert_eq!(
            discovered.items[0].path,
            "scenarios/auth/login-success.yaml"
        );
    }

    #[test]
    fn filesystem_source_respects_gitignore_rules() {
        let test_dir = TestDir::new("discover-gitignore");
        write_file(&test_dir.path.join(".gitignore"), "tmp/\n");
        write_file(
            &test_dir.path.join("scenarios/auth/login-success.yaml"),
            "id: login-success",
        );
        write_file(
            &test_dir.path.join("tmp/experimental.yaml"),
            "id: experimental",
        );

        let mut config = FileSystemSourceConfig::new("filesystem");
        config.roots.push(test_dir.path.clone());
        config.include_patterns = vec!["**/*.yaml".to_string()];
        config.respect_gitignore = true;

        let source = FileSystemSource::new(config).expect("filesystem source config should build");
        let discovered = source.discover();

        assert!(discovered.errors.is_empty());
        assert_eq!(discovered.items.len(), 1);
        assert_eq!(
            discovered.items[0].path,
            "scenarios/auth/login-success.yaml"
        );
    }

    #[cfg(unix)]
    #[test]
    fn filesystem_source_reports_unreadable_root_as_non_fatal_error() {
        use std::os::unix::fs::PermissionsExt;

        let unreadable_root = TestDir::new("discover-unreadable");
        let readable_root = TestDir::new("discover-readable");
        write_file(
            &readable_root.path.join("scenarios/auth/login-success.yaml"),
            "id: login-success",
        );

        std::fs::set_permissions(
            &unreadable_root.path,
            std::fs::Permissions::from_mode(0o000),
        )
        .expect("set unreadable permissions");

        let mut config = FileSystemSourceConfig::new("filesystem");
        config.roots = vec![unreadable_root.path.clone(), readable_root.path.clone()];
        config.include_patterns = vec!["**/*.yaml".to_string()];

        let source = FileSystemSource::new(config).expect("filesystem source config should build");
        let discovered = source.discover();

        std::fs::set_permissions(
            &unreadable_root.path,
            std::fs::Permissions::from_mode(0o755),
        )
        .expect("restore unreadable permissions for cleanup");

        assert_eq!(discovered.items.len(), 1);
        assert_eq!(
            discovered.items[0].path,
            "scenarios/auth/login-success.yaml"
        );
        assert!(
            discovered
                .errors
                .iter()
                .any(|error| error.code == CatalogErrorCode::DiscoveryFailure)
        );
        assert!(
            discovered
                .errors
                .iter()
                .any(|error| error.stage == CatalogStage::Discover)
        );
    }

    #[test]
    fn standard_catalog_loads_500_files_with_concurrency_limit_100() {
        let source_id = "memory".to_string();
        let (refs, docs) = build_refs_and_docs(500, &source_id);
        let peak_loads = Arc::new(AtomicUsize::new(0));

        let source = ConcurrencyTrackingSource {
            source_id: source_id.clone(),
            docs,
            failed_paths: HashSet::new(),
            active_loads: Arc::new(AtomicUsize::new(0)),
            peak_loads: Arc::clone(&peak_loads),
            io_delay: Duration::from_millis(1),
        };
        let catalog = StandardScenarioCatalog::new(
            source,
            StandardScenarioCatalogConfig {
                io_concurrency_limit: 100,
            },
        )
        .expect("standard catalog should build with positive concurrency limit");
        let runtime = tokio_runtime();

        let loaded = runtime.block_on(catalog.load(&refs));

        assert!(loaded.errors.is_empty());
        assert_eq!(loaded.items.len(), 500);
        assert!(peak_loads.load(Ordering::SeqCst) <= 100);
    }

    #[test]
    fn standard_catalog_limit_one_remains_correct_without_deadlock() {
        let source_id = "memory".to_string();
        let (refs, docs) = build_refs_and_docs(25, &source_id);
        let peak_loads = Arc::new(AtomicUsize::new(0));

        let source = ConcurrencyTrackingSource {
            source_id,
            docs,
            failed_paths: HashSet::new(),
            active_loads: Arc::new(AtomicUsize::new(0)),
            peak_loads: Arc::clone(&peak_loads),
            io_delay: Duration::from_millis(2),
        };
        let catalog = StandardScenarioCatalog::new(
            source,
            StandardScenarioCatalogConfig {
                io_concurrency_limit: 1,
            },
        )
        .expect("standard catalog should build with limit one");
        let runtime = tokio_runtime();

        let loaded = runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(5), catalog.load(&refs))
                .await
                .expect("load should complete without deadlock")
        });

        assert!(loaded.errors.is_empty());
        assert_eq!(loaded.items.len(), 25);
        assert_eq!(peak_loads.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn standard_catalog_collects_source_read_failures_without_aborting_remaining_loads() {
        let source_id = "memory".to_string();
        let (refs, docs) = build_refs_and_docs(12, &source_id);
        let failed_paths = HashSet::from([
            "scenarios/auth/scenario-2.yaml".to_string(),
            "scenarios/auth/scenario-6.yaml".to_string(),
            "scenarios/auth/scenario-9.yaml".to_string(),
        ]);

        let source = ConcurrencyTrackingSource {
            source_id,
            docs,
            failed_paths: failed_paths.clone(),
            active_loads: Arc::new(AtomicUsize::new(0)),
            peak_loads: Arc::new(AtomicUsize::new(0)),
            io_delay: Duration::from_millis(1),
        };
        let catalog = StandardScenarioCatalog::new(
            source,
            StandardScenarioCatalogConfig {
                io_concurrency_limit: 4,
            },
        )
        .expect("standard catalog should build");
        let runtime = tokio_runtime();

        let loaded = runtime.block_on(catalog.load(&refs));

        assert_eq!(loaded.items.len(), 9);
        assert_eq!(loaded.errors.len(), 3);
        assert!(
            loaded
                .errors
                .iter()
                .all(|error| error.code == CatalogErrorCode::LoadFailure)
        );
        let error_paths = loaded
            .errors
            .iter()
            .filter_map(|error| error.path.clone())
            .collect::<HashSet<String>>();
        assert_eq!(error_paths, failed_paths);
    }

    #[test]
    fn standard_catalog_parses_valid_login_success_yaml_into_canonical_scenario() {
        let path = "scenarios/auth/login-success.yaml".to_string();
        let source = MockSource {
            source_id: "memory".to_string(),
            refs: vec![ScenarioRef {
                id: "login-success".to_string(),
                path: path.clone(),
                source: "memory".to_string(),
                fingerprint: None,
                discovered_at: SystemTime::UNIX_EPOCH,
            }],
            docs: HashMap::from([(
                path.clone(),
                r#"
id: login-success
service: auth
steps:
  - order: 1
    action: POST /login
    payload:
      username: demo
      password: secret
assertions:
  - path: $.status
    equals: 200
tags: [smoke]
metadata:
  owner: auth-team
"#
                .to_string(),
            )]),
        };
        let catalog =
            StandardScenarioCatalog::new(source, StandardScenarioCatalogConfig::default())
                .expect("catalog should build");
        let runtime = tokio_runtime();

        let discovered = catalog.discover();
        assert!(discovered.errors.is_empty());
        let parsed = runtime.block_on(catalog.load_scenarios(&discovered.items));

        assert!(parsed.errors.is_empty());
        assert_eq!(
            parsed.items,
            vec![Scenario {
                id: "login-success".to_string(),
                service: "auth".to_string(),
                steps: vec![ScenarioStep {
                    order: 1,
                    action: "POST /login".to_string(),
                    payload: BTreeMap::from([
                        ("password".to_string(), "secret".to_string()),
                        ("username".to_string(), "demo".to_string()),
                    ]),
                }],
                assertions: vec![ScenarioAssertion {
                    path: "$.status".to_string(),
                    equals: "200".to_string(),
                }],
                tags: vec!["smoke".to_string()],
                metadata: BTreeMap::from([("owner".to_string(), "auth-team".to_string())]),
            }]
        );
    }

    #[test]
    fn malformed_yaml_returns_parse_error_with_path_and_parse_stage() {
        let path = "scenarios/auth/malformed.yaml".to_string();
        let source = MockSource {
            source_id: "memory".to_string(),
            refs: vec![ScenarioRef {
                id: "malformed".to_string(),
                path: path.clone(),
                source: "memory".to_string(),
                fingerprint: None,
                discovered_at: SystemTime::UNIX_EPOCH,
            }],
            docs: HashMap::from([(
                path.clone(),
                "id: malformed\nservice: auth\nsteps: [\nassertions:\n  - path: $.status\n    equals: 200\n"
                    .to_string(),
            )]),
        };
        let catalog =
            StandardScenarioCatalog::new(source, StandardScenarioCatalogConfig::default())
                .expect("catalog should build");
        let runtime = tokio_runtime();

        let parsed = runtime.block_on(catalog.load_scenarios(&catalog.discover().items));

        assert!(parsed.items.is_empty());
        assert_eq!(parsed.errors.len(), 1);
        assert_eq!(parsed.errors[0].code, CatalogErrorCode::ParseFailure);
        assert_eq!(parsed.errors[0].stage, CatalogStage::Parse);
        assert_eq!(parsed.errors[0].path.as_deref(), Some(path.as_str()));
    }

    #[test]
    fn schema_violation_missing_steps_returns_validation_error_with_validate_stage() {
        let path = "scenarios/auth/missing-steps.yaml".to_string();
        let source = MockSource {
            source_id: "memory".to_string(),
            refs: vec![ScenarioRef {
                id: "missing-steps".to_string(),
                path: path.clone(),
                source: "memory".to_string(),
                fingerprint: None,
                discovered_at: SystemTime::UNIX_EPOCH,
            }],
            docs: HashMap::from([(
                path.clone(),
                "id: missing-steps\nservice: auth\nassertions:\n  - path: $.status\n    equals: 200\n"
                    .to_string(),
            )]),
        };
        let catalog =
            StandardScenarioCatalog::new(source, StandardScenarioCatalogConfig::default())
                .expect("catalog should build");
        let runtime = tokio_runtime();

        let parsed = runtime.block_on(catalog.load_scenarios(&catalog.discover().items));

        assert!(parsed.items.is_empty());
        assert_eq!(parsed.errors.len(), 1);
        assert_eq!(parsed.errors[0].code, CatalogErrorCode::ValidationFailure);
        assert_eq!(parsed.errors[0].stage, CatalogStage::Validate);
        assert_eq!(parsed.errors[0].path.as_deref(), Some(path.as_str()));
        assert_eq!(
            parsed.errors[0].details.get("field").map(String::as_str),
            Some("steps")
        );
    }

    #[test]
    fn duplicate_step_order_returns_validation_error() {
        let path = "scenarios/auth/duplicate-step-order.yaml".to_string();
        let source = MockSource {
            source_id: "memory".to_string(),
            refs: vec![ScenarioRef {
                id: "duplicate-step-order".to_string(),
                path: path.clone(),
                source: "memory".to_string(),
                fingerprint: None,
                discovered_at: SystemTime::UNIX_EPOCH,
            }],
            docs: HashMap::from([(
                path.clone(),
                r#"
id: duplicate-step-order
service: auth
steps:
  - order: 1
    action: POST /login
  - order: 1
    action: GET /profile
assertions:
  - path: $.status
    equals: 200
"#
                .to_string(),
            )]),
        };
        let catalog =
            StandardScenarioCatalog::new(source, StandardScenarioCatalogConfig::default())
                .expect("catalog should build");
        let runtime = tokio_runtime();

        let parsed = runtime.block_on(catalog.load_scenarios(&catalog.discover().items));

        assert!(parsed.items.is_empty());
        assert_eq!(parsed.errors.len(), 1);
        assert_eq!(parsed.errors[0].code, CatalogErrorCode::ValidationFailure);
        assert_eq!(parsed.errors[0].stage, CatalogStage::Validate);
        assert_eq!(
            parsed.errors[0].details.get("field").map(String::as_str),
            Some("steps.order")
        );
    }

    #[test]
    fn assertion_missing_equals_returns_validation_error() {
        let path = "scenarios/auth/missing-assertion-equals.yaml".to_string();
        let source = MockSource {
            source_id: "memory".to_string(),
            refs: vec![ScenarioRef {
                id: "missing-assertion-equals".to_string(),
                path: path.clone(),
                source: "memory".to_string(),
                fingerprint: None,
                discovered_at: SystemTime::UNIX_EPOCH,
            }],
            docs: HashMap::from([(
                path.clone(),
                r#"
id: missing-assertion-equals
service: auth
steps:
  - order: 1
    action: POST /login
assertions:
  - path: $.status
"#
                .to_string(),
            )]),
        };
        let catalog =
            StandardScenarioCatalog::new(source, StandardScenarioCatalogConfig::default())
                .expect("catalog should build");
        let runtime = tokio_runtime();

        let parsed = runtime.block_on(catalog.load_scenarios(&catalog.discover().items));

        assert!(parsed.items.is_empty());
        assert_eq!(parsed.errors.len(), 1);
        assert_eq!(parsed.errors[0].code, CatalogErrorCode::ValidationFailure);
        assert_eq!(parsed.errors[0].stage, CatalogStage::Validate);
        assert_eq!(
            parsed.errors[0].details.get("field").map(String::as_str),
            Some("assertions.equals")
        );
    }

    fn build_refs_and_docs(
        total: usize,
        source_id: &str,
    ) -> (Vec<ScenarioRef>, HashMap<String, String>) {
        let mut refs = Vec::with_capacity(total);
        let mut docs = HashMap::with_capacity(total);

        for index in 0..total {
            let id = format!("scenario-{index}");
            let path = format!("scenarios/auth/{id}.yaml");
            refs.push(ScenarioRef {
                id: id.clone(),
                path: path.clone(),
                source: source_id.to_string(),
                fingerprint: None,
                discovered_at: SystemTime::UNIX_EPOCH,
            });
            docs.insert(path, format!("id: {id}"));
        }

        (refs, docs)
    }

    fn update_peak(peak: &AtomicUsize, candidate: usize) {
        let mut current = peak.load(Ordering::SeqCst);
        while candidate > current {
            match peak.compare_exchange(current, candidate, Ordering::SeqCst, Ordering::SeqCst) {
                Ok(_) => break,
                Err(observed) => current = observed,
            }
        }
    }

    fn tokio_runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_time()
            .build()
            .expect("tokio runtime should build")
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

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new(prefix: &str) -> Self {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "qtip-catalog-{prefix}-{}-{timestamp}",
                std::process::id()
            ));
            std::fs::create_dir_all(&path).expect("create temp test directory");
            Self { path }
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            if self.path.exists() {
                let _ = std::fs::remove_dir_all(&self.path);
            }
        }
    }

    fn write_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create parent directories");
        }
        std::fs::write(path, content).expect("write test fixture");
    }
}
