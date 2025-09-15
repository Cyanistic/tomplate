pub mod simple;

#[cfg(feature = "handlebars")]
pub mod handlebars;

#[cfg(feature = "tera")]
pub mod tera;

#[cfg(feature = "minijinja")]
pub mod minijinja;

use std::collections::HashMap;

/// Supported template engines
pub enum Engine {
    Simple,
    #[cfg(feature = "handlebars")]
    Handlebars,
    #[cfg(feature = "tera")]
    Tera,
    #[cfg(feature = "minijinja")]
    MiniJinja,
}

impl Engine {
    /// Parse engine from string
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "simple" | "" => Ok(Engine::Simple),
            #[cfg(feature = "handlebars")]
            "handlebars" => Ok(Engine::Handlebars),
            #[cfg(feature = "tera")]
            "tera" => Ok(Engine::Tera),
            #[cfg(feature = "minijinja")]
            "minijinja" => Ok(Engine::MiniJinja),
            _ => Err(format!("Unknown or disabled template engine: {}", s)),
        }
    }
    
    /// Process a template with this engine
    pub fn process(
        &self,
        template: &str,
        params: &HashMap<String, String>,
    ) -> Result<String, String> {
        match self {
            Engine::Simple => simple::process(template, params),
            #[cfg(feature = "handlebars")]
            Engine::Handlebars => handlebars::process(template, params),
            #[cfg(feature = "tera")]
            Engine::Tera => tera::process(template, params),
            #[cfg(feature = "minijinja")]
            Engine::MiniJinja => minijinja::process(template, params),
        }
    }
}

/// Process a template with the specified engine
pub fn process(
    engine: &str,
    template: &str,
    params: &HashMap<String, String>,
) -> Result<String, String> {
    let engine = Engine::from_str(engine)?;
    engine.process(template, params)
}