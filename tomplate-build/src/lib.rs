//! # Build-time utilities for Tomplate
//!
//! This crate provides build-time template discovery and amalgamation for the Tomplate
//! template engine. It's designed to be used in `build.rs` scripts to discover template
//! files in your project and prepare them for compile-time processing.
//!
//! ## Overview
//!
//! The build process:
//! 1. Discovers `.tomplate.toml` files using glob patterns
//! 2. Parses and validates template definitions
//! 3. Amalgamates all templates into a single TOML file
//! 4. Places the amalgamated file in `OUT_DIR` for the macro to read
//!
//! ## Quick Start
//!
//! In your `build.rs`:
//!
//! ```rust,ignore
//! fn main() {
//!     tomplate_build::Builder::new()
//!         .add_patterns([
//!             "**/*.tomplate.toml",
//!             "templates/*.toml"
//!         ])
//!         .build()
//!         .expect("Failed to build templates");
//! }
//! ```
//!
//! ## Template File Format
//!
//! Templates are defined in TOML files with the following structure:
//!
//! ```toml
//! [template_name]
//! template = "The template string with {placeholders}"
//! engine = "simple"  # Optional: "simple", "handlebars", "tera", or "minijinja"
//!
//! [another_template]
//! template = """
//! Multi-line templates
//! are also supported
//! """
//! ```
//!
//! ## Advanced Configuration
//!
//! ```rust,ignore
//! use tomplate_build::{Builder, Engine};
//!
//! fn main() {
//!     Builder::new()
//!         // Add patterns one by one
//!         .add_pattern("templates/*.toml")
//!         .add_pattern("sql/*.toml")
//!         // Or add multiple at once
//!         .add_patterns(vec!["config/*.toml", "queries/*.toml"])
//!         // Set a default engine for templates without explicit engine
//!         .default_engine(Engine::Handlebars)
//!         // Build and generate the amalgamated file
//!         .build()
//!         .expect("Failed to build templates");
//! }
//! ```
//!
//! ## Error Handling
//!
//! The builder will fail if:
//! - Template files have invalid TOML syntax
//! - Duplicate template names are found across files
//! - File I/O errors occur
//!
//! ## Integration with Cargo
//!
//! The builder automatically:
//! - Sets up `cargo:rerun-if-changed` for template files
//! - Outputs to `OUT_DIR` (respecting Cargo's build system)
//! - Provides clear error messages for debugging

mod amalgamator;
mod builder;
mod discovery;

/// Types used throughout the build system.
///
/// This module contains the core types used by the build system including
/// template definitions, error types, and engine specifications.
pub mod types;

/// The main builder for discovering and processing templates.
///
/// See [`Builder`] for detailed documentation and examples.
pub use builder::Builder;

/// Build mode for template amalgamation.
///
/// See [`BuildMode`] for available modes.
pub use builder::BuildMode;

/// Template engine specifications.
///
/// See [`Engine`] for available engines.
pub use types::Engine;

/// Error type for build operations.
///
/// See [`Error`] for possible error variants.
pub use types::Error;

/// Result type alias for build operations.
pub use types::Result;

/// Template definition structure.
///
/// See [`Template`] for template structure details.
pub use types::Template;