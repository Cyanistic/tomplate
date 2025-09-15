use crate::types::{Engine, Error, Result, Template};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn amalgamate_templates(
    template_files: &[impl AsRef<Path>], 
    default_engine: Option<Engine>
) -> Result<String> {
    let mut all_templates: HashMap<String, Template> = HashMap::new();
    
    for file_path in template_files {
        let file_path = file_path.as_ref();
        let content = fs::read_to_string(file_path)?;
        
        // Parse the TOML file
        let mut templates: HashMap<String, Template> = toml::from_str(&content)
            .map_err(|e| {
                eprintln!("Error parsing {}: {}", file_path.display(), e);
                e
            })?;
        
        // Apply default engine if not specified
        if let Some(default) = default_engine {
            for template in templates.values_mut() {
                if template.engine.is_none() {
                    template.engine = Some(default.to_string());
                }
            }
        }
        
        // Merge templates, checking for duplicates
        for (name, template) in templates {
            if all_templates.contains_key(&name) {
                return Err(Error::DuplicateTemplate(name));
            }
            all_templates.insert(name, template);
        }
    }
    
    // Serialize back to TOML
    let amalgamated = toml::to_string_pretty(&all_templates)?;
    Ok(amalgamated)
}