use crate::{amalgamator, discovery, types::{Engine, Result}};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Build mode for template amalgamation
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum BuildMode {
    /// Overwrite existing templates (default)
    #[default]
    Overwrite,
    /// Append to existing templates, merging with what's already there
    Append,
}

#[derive(Default)]
pub struct Builder {
    patterns: Vec<String>,
    output_dir: Option<PathBuf>,
    mode: BuildMode,
    default_engine: Option<Engine>,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_pattern<S: AsRef<str>>(mut self, pattern: S) -> Self {
        self.patterns.push(pattern.as_ref().to_string());
        self
    }
    
    pub fn add_patterns<I, S>(mut self, patterns: I) -> Self 
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.patterns.extend(patterns.into_iter().map(|s| s.as_ref().to_string()));
        self
    }

    pub fn output_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.output_dir = Some(dir.as_ref().to_path_buf());
        self
    }

    pub fn mode(mut self, mode: BuildMode) -> Self {
        self.mode = mode;
        self
    }
    
    pub fn default_engine(mut self, engine: Engine) -> Self {
        self.default_engine = Some(engine);
        self
    }

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
