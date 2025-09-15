// Example build.rs demonstrating default_engine
fn main() {
    // Set Handlebars as default for all templates without explicit engine
    tomplate_build::Builder::new()
        .add_patterns([
            "**/*.tomplate.toml",
            "templates/*.toml"
        ])
        .default_engine(tomplate_build::Engine::Simple)  // All templates default to simple
        .build()
        .expect("Failed to build templates");
    
    // Or if you have Handlebars feature enabled:
    #[cfg(feature = "handlebars")]
    {
        tomplate_build::Builder::new()
            .add_patterns(["templates/*.toml"])
            .default_engine(tomplate_build::Engine::Handlebars)
            .build()
            .expect("Failed to build templates");
    }
}