use std::collections::BTreeMap;

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

    pub fn conflict(
        message: impl Into<String>,
        path: Option<String>,
        source: Option<String>,
        details: BTreeMap<String, String>,
    ) -> Self {
        Self {
            code: CatalogErrorCode::Conflict,
            message: message.into(),
            path,
            source,
            stage: CatalogStage::Catalog,
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
