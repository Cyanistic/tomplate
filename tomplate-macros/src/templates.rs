use std::collections::HashMap;
use std::sync::LazyLock;
use tomplate_build::types::Template;

// Cache for parsed templates - loaded once from OUT_DIR
static TEMPLATES: LazyLock<HashMap<String, Template>> = LazyLock::new(|| {
    // Get the OUT_DIR from the environment at macro expansion time
    let out_dir = std::env::var("OUT_DIR")
        .expect("OUT_DIR not set. Make sure you have a build.rs that uses tomplate-build");
    
    // Read the amalgamated TOML file
    let toml_path = std::path::Path::new(&out_dir).join("tomplate_amalgamated.toml");
    let toml_content = std::fs::read_to_string(&toml_path)
        .unwrap_or_else(|_| String::new());
    
    // Parse the TOML content
    if toml_content.is_empty() {
        HashMap::new()
    } else {
        toml::from_str(&toml_content)
            .expect("Failed to parse amalgamated templates TOML")
    }
});

/// Get a clone of all templates
pub fn load_templates() -> HashMap<String, Template> {
    TEMPLATES.clone()
}