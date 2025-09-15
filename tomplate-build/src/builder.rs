use crate::{amalgamator, discovery, types::{Engine, Result}};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Build mode for template amalgamation.
///
/// Determines how the builder handles existing template files in the output directory.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum BuildMode {
    /// Overwrite existing templates (default).
    /// 
    /// This mode will completely replace any existing amalgamated template file
    /// with the newly discovered templates.
    #[default]
    Overwrite,
    
    /// Append to existing templates, merging with what's already there.
    /// 
    /// This mode will merge newly discovered templates with any existing
    /// amalgamated file. Note: Duplicate template names will cause an error.
    Append,
}

/// Builder for discovering and processing template files.
///
/// The `Builder` is the main entry point for the build-time template discovery system.
/// It finds template files matching specified patterns, validates them, and amalgamates
/// them into a single TOML file for the macro system to use at compile time.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust,ignore
/// fn main() {
///     tomplate_build::Builder::new()
///         .add_pattern("**/*.tomplate.toml")
///         .build()
///         .expect("Failed to build templates");
/// }
/// ```
///
/// ## Advanced Configuration
///
/// ```rust,ignore
/// use tomplate_build::{Builder, Engine};
///
/// fn main() {
///     Builder::new()
///         .add_patterns([
///             "templates/*.toml",
///             "sql/**/*.tomplate.toml",
///             "config/*.toml"
///         ])
///         .default_engine(Engine::Handlebars)
///         .build()
///         .expect("Failed to build templates");
/// }
/// ```
#[derive(Default)]
pub struct Builder {
    patterns: Vec<String>,
    output_dir: Option<PathBuf>,
    mode: BuildMode,
    default_engine: Option<Engine>,
}

impl Builder {
    /// Creates a new `Builder` with default settings.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use tomplate_build::Builder;
    ///
    /// let builder = Builder::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a single glob pattern for discovering template files.
    ///
    /// The pattern follows standard glob syntax:
    /// - `*` matches any sequence of characters within a path segment
    /// - `**` matches zero or more path segments
    /// - `?` matches any single character
    /// - `[...]` matches any character within the brackets
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// Builder::new()
    ///     .add_pattern("templates/*.toml")
    ///     .add_pattern("**/*.tomplate.toml")
    ///     .build()?;
    /// ```
    pub fn add_pattern<S: AsRef<str>>(mut self, pattern: S) -> Self {
        self.patterns.push(pattern.as_ref().to_string());
        self
    }
    
    /// Adds multiple glob patterns for discovering template files.
    ///
    /// This is a convenience method for adding multiple patterns at once.
    /// It accepts any iterator of string-like items.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Using an array
    /// Builder::new()
    ///     .add_patterns(["*.toml", "templates/*.toml"])
    ///     .build()?;
    ///
    /// // Using a vector
    /// let patterns = vec!["sql/*.toml", "queries/*.toml"];
    /// Builder::new()
    ///     .add_patterns(patterns)
    ///     .build()?;
    ///
    /// // Using an iterator
    /// let patterns = (1..=3).map(|i| format!("v{}/**.toml", i));
    /// Builder::new()
    ///     .add_patterns(patterns)
    ///     .build()?;
    /// ```
    pub fn add_patterns<I, S>(mut self, patterns: I) -> Self 
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.patterns.extend(patterns.into_iter().map(|s| s.as_ref().to_string()));
        self
    }

    /// Sets a custom output directory for the amalgamated template file.
    ///
    /// By default, the builder uses the `OUT_DIR` environment variable set by Cargo.
    /// This method allows overriding that behavior for special use cases.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// Builder::new()
    ///     .add_pattern("**/*.toml")
    ///     .output_dir("target/templates")
    ///     .build()?;
    /// ```
    pub fn output_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.output_dir = Some(dir.as_ref().to_path_buf());
        self
    }

    /// Sets the build mode for template amalgamation.
    ///
    /// See [`BuildMode`] for available modes and their behavior.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use tomplate_build::{Builder, BuildMode};
    ///
    /// Builder::new()
    ///     .add_pattern("**/*.toml")
    ///     .mode(BuildMode::Append)
    ///     .build()?;
    /// ```
    pub fn mode(mut self, mode: BuildMode) -> Self {
        self.mode = mode;
        self
    }
    
    /// Sets a default template engine for templates without an explicit engine.
    ///
    /// When a template in a TOML file doesn't specify an `engine` field,
    /// this default will be used. If no default is set, "simple" is used.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use tomplate_build::{Builder, Engine};
    ///
    /// Builder::new()
    ///     .add_pattern("**/*.toml")
    ///     .default_engine(Engine::Handlebars)
    ///     .build()?;
    /// ```
    ///
    /// With this configuration, any template like:
    /// ```toml
    /// [my_template]
    /// template = "Hello {{name}}"
    /// # No engine specified
    /// ```
    /// Will use Handlebars instead of the default simple engine.
    pub fn default_engine(mut self, engine: Engine) -> Self {
        self.default_engine = Some(engine);
        self
    }

    /// Builds and processes all discovered templates.
    ///
    /// This method:
    /// 1. Discovers all template files matching the configured patterns
    /// 2. Parses and validates the TOML files
    /// 3. Applies the default engine if configured
    /// 4. Checks for duplicate template names
    /// 5. Amalgamates all templates into a single TOML file
    /// 6. Writes the result to `OUT_DIR/tomplate_amalgamated.toml`
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No output directory is configured and `OUT_DIR` is not set
    /// - Template files contain invalid TOML
    /// - Duplicate template names are found
    /// - File I/O operations fail
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn main() {
    ///     if let Err(e) = Builder::new()
    ///         .add_pattern("**/*.tomplate.toml")
    ///         .build()
    ///     {
    ///         eprintln!("Build failed: {}", e);
    ///         std::process::exit(1);
    ///     }
    /// }
    /// ```
    pub fn build(self) -> Result<()> {
        let out_dir = self
            .output_dir
            .or_else(|| env::var_os("OUT_DIR").map(PathBuf::from))
            .expect("OUT_DIR not set and no output_dir specified");

        // Tell Cargo to rerun if any tomplate files change
        for pattern in &self.patterns {
            println!("cargo:rerun-if-changed={}", pattern);
        }

        // Discover all template files
        let template_files = discovery::discover_templates(&self.patterns)?;

        if template_files.is_empty() {
            // No templates found, create empty constants
            Self::write_empty_templates(&out_dir)?;
            return Ok(());
        }

        // Amalgamate all templates into a single TOML structure
        let amalgamated = amalgamator::amalgamate_templates(&template_files, self.default_engine)?;

        // Write the amalgamated TOML file
        let toml_path = out_dir.join("tomplate_amalgamated.toml");
        fs::write(&toml_path, &amalgamated)?;

        println!(
            "cargo:rustc-env=TOMPLATE_TEMPLATES_PATH={}",
            toml_path.display()
        );

        Ok(())
    }

    fn write_empty_templates(out_dir: &Path) -> Result<()> {
        // Write empty TOML file
        let toml_path = out_dir.join("tomplate_amalgamated.toml");
        fs::write(&toml_path, "")?;

        Ok(())
    }
}
