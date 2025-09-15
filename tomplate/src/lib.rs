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
//! ## Getting Started
//!
//! ### Step 1: Add Dependencies
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! tomplate = "0.1"
//!
//! [build-dependencies]
//! tomplate-build = "0.1"
//!
//! # Optional: Enable additional template engines
//! # [dependencies.tomplate]
//! # version = "0.1"
//! # features = ["handlebars", "tera"]  # Add the engines you need
//! ```
//!
//! ### Step 2: Create a Build Script
//!
//! Create `build.rs` in your project root:
//!
//! ```rust,ignore
//! fn main() {
//!     tomplate_build::Builder::new()
//!         .add_patterns([
//!             "**/*.tomplate.toml",  // Recursively find .tomplate.toml files
//!             "templates/*.toml",     // Also check templates directory
//!         ])
//!         .build()
//!         .expect("Failed to build templates");
//! }
//! ```
//!
//! ### Step 3: Create Template Files
//!
//! Create a file like `templates/queries.tomplate.toml`:
//!
//! ```toml
//! # Simple variable substitution (default engine)
//! [user_query]
//! template = "SELECT {fields} FROM users WHERE {condition}"
//!
//! # Using Handlebars for logic
//! [conditional_query]
//! template = """
//! SELECT * FROM users
//! {{#if status}}
//! WHERE status = '{{status}}'
//! {{/if}}
//! """
//! engine = "handlebars"
//!
//! # Template with default values
//! [paginated_query]
//! template = "SELECT * FROM {table} LIMIT {limit} OFFSET {offset}"
//! ```
//!
//! ### Step 4: Use Templates in Your Code
//!
//! ```rust,ignore
//! # use tomplate::tomplate;
//! # fn main() {
//! // Using templates from files
//! const USER_QUERY: &str = tomplate!("user_query",
//!     fields = "id, name, email",
//!     condition = "active = true"
//! );
//! // Result: "SELECT id, name, email FROM users WHERE active = true"
//!
//! // Using inline templates (when not found in registry)
//! const GREETING: &str = tomplate!(
//!     "Hello {name}, welcome to {place}!",
//!     name = "Alice",
//!     place = "Wonderland"
//! );
//! // Result: "Hello Alice, welcome to Wonderland!"
//! # }
//! ```
//!
//! ## Major Features Examples
//!
//! ### 1. File-Based vs Inline Templates
//!
//! ```rust,ignore
//! use tomplate::tomplate;
//!
//! // File-based: Looks for "user_query" in your .tomplate.toml files
//! const FROM_FILE: &str = tomplate!("user_query", 
//!     fields = "id, name",
//!     condition = "active = true"
//! );
//!
//! // Inline: If "Hello {name}!" isn't found in files, treats it as template
//! const INLINE: &str = tomplate!(
//!     "Hello {name}!",
//!     name = "World"
//! );
//!
//! // How it works:
//! // 1. First checks if the string matches a template name in registry
//! // 2. If not found, uses the string itself as an inline template
//! ```
//!
//! ### 2. Nested Template Composition
//!
//! ```rust,ignore
//! # use tomplate::tomplate;
//! # fn main() {
//! // Templates can use other templates as parameters
//! const NESTED: &str = tomplate!("wrapper_template",
//!     header = tomplate!("header_template", title = "My App"),
//!     body = tomplate!("SELECT * FROM {table}", table = "users"),
//!     footer = tomplate!("footer_template", year = "2024")
//! );
//!
//! // This enables building complex templates from simple parts
//! # }
//! ```
//!
//! ### 3. Composition Blocks with Scoped Variables
//!
//! ```rust,ignore
//! use tomplate::tomplate;
//!
//! tomplate! {
//!     // Local variables - reusable within the block
//!     let base_fields = tomplate!("id, name, email");
//!     let active_condition = tomplate!("status = 'active'");
//!     let pagination = tomplate!("LIMIT {limit} OFFSET {offset}",
//!         limit = "10",
//!         offset = "0"
//!     );
//!     
//!     // Export constants - available outside the block
//!     const GET_ACTIVE_USERS = tomplate!(
//!         "SELECT {fields} FROM users WHERE {condition} {page}",
//!         fields = base_fields,
//!         condition = active_condition,
//!         page = pagination
//!     );
//!     
//!     const COUNT_ACTIVE = tomplate!(
//!         "SELECT COUNT(*) FROM users WHERE {condition}",
//!         condition = active_condition
//!     );
//!     
//!     // Can use both file templates and inline templates
//!     const MIXED = tomplate!("wrapper_template",
//!         content = tomplate!("Inline: {value}", value = base_fields)
//!     );
//! }
//!
//! // The constants are now available for use
//! fn main(){
//!     println!("{}", GET_ACTIVE_USERS);
//! }
//! // Output: "SELECT id, name, email FROM users WHERE status = 'active' LIMIT 10 OFFSET 0"
//! ```
//!
//! ### 4. Multiple Template Engines
//!
//! ```rust,ignore
//! // In your .tomplate.toml file:
//! // [simple_template]
//! // template = "Hello {name}"
//! // engine = "simple"  # Default - basic {var} substitution
//! //
//! // [handlebars_template]
//! // template = "{{#if logged_in}}Welcome {{user}}{{else}}Please login{{/if}}"
//! // engine = "handlebars"
//! //
//! // [tera_template]
//! // template = "{% for item in items %}{{ item|upper }}{% endfor %}"
//! // engine = "tera"
//!
//! // Use them the same way
//! const SIMPLE: &str = tomplate!("simple_template", name = "Alice");
//! const LOGIC: &str = tomplate!("handlebars_template", 
//!     logged_in = "true",
//!     user = "Bob"
//! );
//! ```
//!
//! ### 5. Eager Evaluation for Nested Macros
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
/// # use tomplate::{tomplate, tomplate_eager};
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let pool = ();
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
/// # Ok(())
/// # }
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
