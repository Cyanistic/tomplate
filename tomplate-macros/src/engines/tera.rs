use std::collections::HashMap;

pub fn process(
    template: &str,
    params: &HashMap<String, String>,
) -> Result<String, String> {
    let mut tera = tera::Tera::default();
    
    // Disable auto-escaping for non-HTML templates
    tera.autoescape_on(vec![]);
    
    // Add the template
    tera.add_raw_template("template", template)
        .map_err(|e| format!("Tera template error: {}", e))?;
    
    // Convert params to tera::Context
    let mut context = tera::Context::new();
    for (key, value) in params {
        context.insert(key, value);
    }
    
    tera.render("template", &context)
        .map_err(|e| format!("Tera render error: {}", e))
}