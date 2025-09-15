use std::collections::HashMap;
use std::sync::LazyLock;
use tomplate_build::types::Template;

// Cache for parsed templates - loaded once from OUT_DIR
static TEMPLATES: LazyLock<HashMap<String, Template>> = LazyLock::new(|| {
    // Get the OUT_DIR from the environment at macro expansion time
    let tomplate_path = std::env::var("TOMPLATE_TEMPLATES_PATH").expect(
        "TOMPLATE_TEMPLATES_PATH not set. Make sure you have a build.rs that uses tomplate-build",
    );
    let toml_content = std::fs::read_to_string(&tomplate_path).unwrap_or_else(|_| String::new());

    // Parse the TOML content
    if toml_content.is_empty() {
        HashMap::new()
    } else {
        toml::from_str(&toml_content).expect("Failed to parse amalgamated templates TOML")
    }
});

/// Get a clone of all templates
pub fn load_templates() -> HashMap<String, Template> {
    TEMPLATES.clone()
}

