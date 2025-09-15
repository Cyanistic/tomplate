//! Type definitions for the Tomplate build system.
//!
//! This module contains the core types used throughout the build system,
//! including template definitions, error handling, and engine specifications.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

/// A template definition from a `.tomplate.toml` file.
///
/// Templates are defined in TOML format with a required `template` field
/// and optional `engine` and metadata fields.
///
/// # Examples
///
/// A typical template definition in TOML:
///
/// ```toml
/// [my_template]
/// template = "Hello {name}, welcome to {place}!"
/// engine = "simple"  # Optional
///
/// [complex_template]
/// template = """
/// {{#if logged_in}}
///   Welcome back, {{username}}!
/// {{else}}
///   Please log in.
/// {{/if}}
/// """
/// engine = "handlebars"
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Template {
    /// The template string containing the template pattern.
    ///
    /// This can use various syntaxes depending on the engine:
    /// - Simple: `{variable}` placeholders
    /// - Handlebars: `{{variable}}` with full Handlebars features
    /// - Tera: `{{ variable }}` with Tera/Jinja2 syntax
    /// - MiniJinja: Similar to Tera with Jinja2 syntax
    pub template: String,
    
    /// Optional template engine to use.
    ///
    /// If not specified, defaults to "simple" or the builder's default engine.
    /// Valid values: "simple", "handlebars", "tera", "minijinja"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    
    /// Additional metadata for the template.
    ///
    /// This can include custom fields for documentation, validation schemas,
    /// or any other template-specific information. These fields are preserved
    /// but not used by the core template system.
    #[serde(flatten)]
    pub metadata: HashMap<String, toml::Value>,
}

/// Error types for Tomplate build operations.
///
/// This enum represents all possible errors that can occur during
/// template discovery, parsing, and amalgamation.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O error occurred during file operations.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Failed to parse TOML template file.
    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),
    
    /// Failed to serialize templates to TOML.
    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
    
    /// Invalid glob pattern provided.
    #[error("Glob pattern error: {0}")]
    Glob(#[from] glob::PatternError),
    
    /// Referenced template file does not exist.
    #[error("Template file not found: {0}")]
    FileNotFound(PathBuf),
    
    /// Found duplicate template names across files.
    ///
    /// Each template name must be unique across all discovered files.
    #[error("Duplicate template name: {0}")]
    DuplicateTemplate(String),
    
    /// Template definition is invalid or malformed.
    #[error("Invalid template definition: {0}")]
    InvalidTemplate(String),
    
    /// Referenced template not found in registry.
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
    
    /// Template engine error during processing.
    #[error("Template engine error: {0}")]
    EngineError(String),
    
    /// Invalid parameter provided to template.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

/// Result type alias for Tomplate build operations.
///
/// A convenience type alias for `Result<T, tomplate_build::Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// Supported template engines for processing templates.
///
/// Each engine provides different features and syntax:
///
/// - **Simple**: Basic `{variable}` substitution
/// - **Handlebars**: Full Handlebars templating with helpers and conditionals
/// - **Tera**: Jinja2-like templating with filters and inheritance
/// - **MiniJinja**: Lightweight Jinja2 implementation
///
/// # Examples
///
/// ```rust,ignore
/// use tomplate_build::Engine;
///
/// // Use in Builder
/// Builder::new()
///     .default_engine(Engine::Handlebars)
///     .build()?;
/// ```
///
/// # Feature Flags
///
/// Some engines require feature flags to be enabled:
/// - `handlebars`: Enables the Handlebars engine
/// - `tera`: Enables the Tera engine
/// - `minijinja`: Enables the MiniJinja engine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Engine {
    /// Simple variable substitution engine.
    ///
    /// Uses `{variable}` syntax for placeholders.
    /// This is the default engine and requires no additional features.
    ///
    /// # Example Template
    /// ```text
    /// Hello {name}, you have {count} messages.
    /// ```
    Simple,
    
    /// Handlebars template engine.
    ///
    /// Provides full Handlebars functionality including:
    /// - Conditionals: `{{#if}}`, `{{#unless}}`
    /// - Loops: `{{#each}}`
    /// - Helpers and partials
    /// - HTML escaping (disabled by default in Tomplate)
    ///
    /// Requires the `handlebars` feature.
    ///
    /// # Example Template
    /// ```text
    /// {{#if logged_in}}
    ///   Welcome {{username}}!
    /// {{else}}
    ///   Please log in.
    /// {{/if}}
    /// ```
    #[cfg(feature = "handlebars")]
    #[cfg_attr(docsrs, doc(cfg(feature = "handlebars")))]
    Handlebars,
    
    /// Tera template engine.
    ///
    /// Provides Jinja2-like syntax with:
    /// - Variables: `{{ variable }}`
    /// - Filters: `{{ value | upper }}`
    /// - Control structures: `{% if %}`, `{% for %}`
    /// - Template inheritance
    ///
    /// Requires the `tera` feature.
    ///
    /// # Example Template
    /// ```text
    /// {% for user in users %}
    ///   {{ user.name | upper }}
    /// {% endfor %}
    /// ```
    #[cfg(feature = "tera")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tera")))]
    Tera,
    
    /// MiniJinja template engine.
    ///
    /// A lightweight Jinja2 implementation with:
    /// - Similar syntax to Tera
    /// - Good performance
    /// - Smaller dependency footprint
    ///
    /// Requires the `minijinja` feature.
    ///
    /// # Example Template
    /// ```text
    /// {% if items %}
    ///   {% for item in items %}
    ///     - {{ item }}
    ///   {% endfor %}
    /// {% endif %}
    /// ```
    #[cfg(feature = "minijinja")]
    #[cfg_attr(docsrs, doc(cfg(feature = "minijinja")))]
    MiniJinja,
}

impl Engine {
    /// Returns the string representation of the engine.
    ///
    /// This is the value used in TOML files for the `engine` field.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// assert_eq!(Engine::Simple.as_str(), "simple");
    /// assert_eq!(Engine::Handlebars.as_str(), "handlebars");
    /// ```
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
    /// Returns the default engine (Simple).
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
    
    /// Parses an engine name from a string.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use std::str::FromStr;
    /// use tomplate_build::Engine;
    ///
    /// let engine = Engine::from_str("handlebars")?;
    /// assert_eq!(engine, Engine::Handlebars);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the engine name is unknown or the required
    /// feature is not enabled.
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