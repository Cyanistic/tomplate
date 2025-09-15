//! # Tomplate: TOML-Based Compile-Time Template Composition
//!
//! Tomplate is a powerful compile-time template engine for Rust that processes templates
//! at compile time, resulting in zero runtime overhead. Templates are defined in TOML files
//! and can use various template engines including Handlebars, Tera, and MiniJinja.
//!
//! ## Features
//!
//! - **Zero Runtime Overhead**: All template processing happens at compile time
//! - **Multiple Template Engines**: Choose from Simple, Handlebars, Tera, or MiniJinja
//! - **Composition Blocks**: Build complex templates from reusable parts
//! - **Inline Templates**: Use template strings directly without registry
//! - **Eager Evaluation**: Solve macro expansion order issues with `tomplate_eager!`
//! - **File-Based Organization**: Store templates in `.tomplate.toml` files
//!
//! ## Quick Start
//!
//! ### Basic Usage
//!
//! ```rust,ignore
//! use tomplate::tomplate;
//!
//! // Using a template from the registry
//! const QUERY: &str = tomplate!("user_query", 
//!     table = "users",
//!     condition = "active = true"
//! );
//!
//! // Using an inline template
//! const GREETING: &str = tomplate!(
//!     "Hello {name}, welcome to {place}!",
//!     name = "Alice",
//!     place = "Wonderland"
//! );
//! ```
//!
//! ### Composition Blocks
//!
//! ```rust,ignore
//! use tomplate::tomplate;
//!
//! tomplate! {
//!     // Local variables (only available in this block)
//!     let base_fields = tomplate!("id, name, email");
//!     let condition = tomplate!("active = true");
//!     
//!     // Exported constants (available outside the block)
//!     const USER_QUERY = tomplate!(
//!         "SELECT {fields} FROM users WHERE {condition}",
//!         fields = base_fields,
//!         condition = condition
//!     );
//!     
//!     const COUNT_QUERY = tomplate!(
//!         "SELECT COUNT(*) FROM users WHERE {condition}",
//!         condition = condition
//!     );
//! }
//!
//! // Use the generated constants
//! assert_eq!(USER_QUERY, "SELECT id, name, email FROM users WHERE active = true");
//! ```
//!
//! ### Eager Evaluation for Nested Macros
//!
//! ```rust,ignore
//! use tomplate::{tomplate, tomplate_eager};
//!
//! // Problem: This won't work with macros that expect string literals
//! // sqlx::query!(tomplate!("select_user", id = "5"))  // ❌ Fails
//!
//! // Solution: Use tomplate_eager! to expand inner macros first
//! tomplate_eager! {
//!     sqlx::query!(tomplate!("select_user", id = "5"))  // ✅ Works
//!         .fetch_one(&pool)
//!         .await?
//! }
//! ```
//!
//! ## Build Configuration
//!
//! In your `build.rs`:
//!
//! ```rust,ignore
//! fn main() {
//!     tomplate::Builder::new()
//!         .add_patterns([
//!             "**/*.tomplate.toml",
//!             "templates/*.toml"
//!         ])
//!         .default_engine(tomplate::Engine::Simple)
//!         .build()
//!         .expect("Failed to build templates");
//! }
//! ```
//!
//! ## Template Files
//!
//! Create `.tomplate.toml` files in your project:
//!
//! ```toml
//! [user_query]
//! template = "SELECT {fields} FROM {table} WHERE {condition}"
//! engine = "simple"  # Optional, defaults to "simple"
//!
//! [complex_template]
//! template = "{{#if condition}}{{value}}{{else}}default{{/if}}"
//! engine = "handlebars"
//! ```
//!
//! ## Feature Flags
//!
//! - `build`: Enables the build-time template discovery (enabled by default)
//! - `handlebars`: Enables Handlebars template engine
//! - `tera`: Enables Tera template engine
//! - `minijinja`: Enables MiniJinja template engine

/// The main template macro for compile-time template processing.
///
/// This macro can be used in two ways:
/// 1. **Direct template invocation**: Process a single template
/// 2. **Composition block**: Define multiple templates with local variables
///
/// # Direct Template Invocation
///
/// ```rust,ignore
/// use tomplate::tomplate;
///
/// // From registry
/// const QUERY: &str = tomplate!("user_query", 
///     fields = "id, name",
///     table = "users"
/// );
///
/// // Inline template
/// const MSG: &str = tomplate!("Hello {name}!", name = "World");
/// ```
///
/// # Composition Block
///
/// ```rust,ignore
/// tomplate! {
///     let helper = tomplate!("template_name");
///     const OUTPUT = tomplate!("main_template", param = helper);
/// }
/// ```
///
/// # Parameters
///
/// Parameters can be:
/// - String literals: `"value"`
/// - Numbers: `42`, `3.14`
/// - Booleans: `true`, `false`
/// - Nested `tomplate!` calls for composition
pub use tomplate_macros::tomplate;

/// Eagerly evaluates `tomplate!` and `concat!` macros within a token stream.
///
/// This macro solves the problem where outer macros (like `sqlx::query!`) expect
/// string literals but receive unexpanded macro calls. `tomplate_eager!` walks
/// the token tree and expands inner macros first, allowing tomplate to work
/// seamlessly with other macros.
///
/// # Examples
///
/// ```rust,ignore
/// use tomplate::{tomplate, tomplate_eager};
///
/// // Without tomplate_eager: ❌ Fails
/// // sqlx::query!(tomplate!("select_user", id = "5"))
/// //     .fetch_one(&pool)
/// //     .await?;
///
/// // With tomplate_eager: ✅ Works
/// tomplate_eager! {
///     sqlx::query!(tomplate!("select_user", id = "5"))
///         .fetch_one(&pool)
///         .await?
/// }
///
/// // Also works with concat!
/// tomplate_eager! {
///     const QUERY: &str = concat!(
///         tomplate!("select_part1"),
///         " UNION ALL ",
///         tomplate!("select_part2")
///     );
/// }
/// ```
///
/// # How It Works
///
/// The macro recursively walks through the provided token stream, finds any
/// `tomplate!` or `concat!` invocations, evaluates them at compile time,
/// and replaces them with their resulting string literals before passing
/// the modified token stream to the compiler.
pub use tomplate_macros::tomplate_eager;

// Re-export builder utilities for use in build scripts
#[cfg(feature = "build")]
#[doc(cfg(feature = "build"))]
pub use tomplate_build::Builder;

// Re-export types for convenience
#[cfg(feature = "build")]
#[doc(cfg(feature = "build"))]
pub use tomplate_build::{BuildMode, Engine, Error, Result, Template};