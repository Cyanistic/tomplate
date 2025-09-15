use std::collections::HashMap;

pub fn process(
    template: &str,
    params: &HashMap<String, String>,
) -> Result<String, String> {
    let mut handlebars = handlebars::Handlebars::new();
    
    // Disable HTML escaping for SQL and other non-HTML templates
    handlebars.register_escape_fn(handlebars::no_escape);
    
    // Convert params to serde_json::Value for Handlebars
    let mut data = serde_json::Map::new();
    for (key, value) in params {
        data.insert(key.clone(), serde_json::Value::String(value.clone()));
    }
    let json_data = serde_json::Value::Object(data);
    
    handlebars
        .render_template(template, &json_data)
        .map_err(|e| format!("Handlebars error: {}", e))
}