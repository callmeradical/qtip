use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;

use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;

use crate::error::CatalogError;
use crate::types::{
    ScenarioDocument, ScenarioDocumentEnvelope, ScenarioDocumentFormat, ScenarioRef,
    ScenarioRefEnvelope,
};
use crate::BoxFuture;

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
