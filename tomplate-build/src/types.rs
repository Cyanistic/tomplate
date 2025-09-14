use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// A template definition from a .stencil.toml file
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Template {
    /// The template string or file path
    pub template: String,
    
    /// Optional template engine to use (e.g., "handlebars", "tera", "simple")
    /// Defaults to "simple" if not specified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    
    /// Additional metadata for the template
    /// Can include schema validation, description, etc.
    #[serde(flatten)]
    pub metadata: HashMap<String, toml::Value>,
}

/// Error types for mosaic operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),
    
    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
    
    #[error("Glob pattern error: {0}")]
    Glob(#[from] glob::PatternError),
    
    #[error("Template file not found: {0}")]
    FileNotFound(PathBuf),
    
    #[error("Duplicate template name: {0}")]
    DuplicateTemplate(String),
    
    #[error("Invalid template definition: {0}")]
    InvalidTemplate(String),
    
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
    
    #[error("Template engine error: {0}")]
    EngineError(String),
    
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

/// Result type alias for mosaic operations
pub type Result<T> = std::result::Result<T, Error>;