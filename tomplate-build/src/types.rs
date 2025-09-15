use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

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

/// Supported template engines
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Engine {
    /// Simple {variable} substitution
    Simple,
    /// Handlebars template engine
    #[cfg(feature = "handlebars")]
    Handlebars,
    /// Tera template engine
    #[cfg(feature = "tera")]
    Tera,
    /// MiniJinja template engine
    #[cfg(feature = "minijinja")]
    MiniJinja,
}

impl Engine {
    /// Get the string representation of the engine
    pub fn as_str(&self) -> &'static str {
        match self {
            Engine::Simple => "simple",
            #[cfg(feature = "handlebars")]
            Engine::Handlebars => "handlebars",
            #[cfg(feature = "tera")]
            Engine::Tera => "tera",
            #[cfg(feature = "minijinja")]
            Engine::MiniJinja => "minijinja",
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Engine::Simple
    }
}

impl fmt::Display for Engine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Engine {
    type Err = Error;
    
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "simple" | "" => Ok(Engine::Simple),
            #[cfg(feature = "handlebars")]
            "handlebars" => Ok(Engine::Handlebars),
            #[cfg(feature = "tera")]
            "tera" => Ok(Engine::Tera),
            #[cfg(feature = "minijinja")]
            "minijinja" => Ok(Engine::MiniJinja),
            _ => Err(Error::EngineError(format!("Unknown or disabled template engine: {}", s))),
        }
    }
}