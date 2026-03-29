#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::future::Future;
use std::path::{Component, Path, PathBuf};
use std::pin::Pin;
use std::time::SystemTime;

use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;

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
    use std::collections::HashMap;
    use std::path::Path;
    use std::sync::Arc;
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
