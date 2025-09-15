use std::collections::HashMap;

pub fn process(
    template: &str,
    params: &HashMap<String, String>,
) -> Result<String, String> {
    let mut env = minijinja::Environment::new();
    
    // Add the template
    env.add_template("template", template)
        .map_err(|e| format!("MiniJinja template error: {}", e))?;
    
    // Get the template
    let tmpl = env.get_template("template")
        .map_err(|e| format!("MiniJinja get template error: {}", e))?;
    
    // Convert params to minijinja::Value using from_iter
    let context = minijinja::Value::from_iter(
        params.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    );
    
    tmpl.render(context)
        .map_err(|e| format!("MiniJinja render error: {}", e))
}