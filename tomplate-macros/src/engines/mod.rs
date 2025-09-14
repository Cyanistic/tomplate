pub mod simple;

use std::collections::HashMap;

/// Process a template with the specified engine
pub fn process(
    engine: &str,
    template: &str,
    params: &HashMap<String, String>,
) -> Result<String, String> {
    match engine {
        "simple" | "" => simple::process(template, params),
        _ => Err(format!("Unknown template engine: {}", engine)),
    }
}