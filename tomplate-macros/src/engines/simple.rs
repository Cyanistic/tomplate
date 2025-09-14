use std::collections::HashMap;

/// Process a template using simple {variable} substitution
pub fn process(template: &str, params: &HashMap<String, String>) -> Result<String, String> {
    let mut result = template.to_string();
    
    // Replace all {key} patterns with their values
    for (key, value) in params {
        let pattern = format!("{{{}}}", key);
        result = result.replace(&pattern, value);
    }
    
    // Check for any remaining unsubstituted variables
    if result.contains('{') && result.contains('}') {
        // Extract unsubstituted variable names for error message
        let mut unsubstituted = Vec::new();
        let mut chars = result.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '{' {
                let mut var_name = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '}' {
                        chars.next();
                        if !var_name.is_empty() && !params.contains_key(&var_name) {
                            unsubstituted.push(var_name);
                        }
                        break;
                    }
                    var_name.push(chars.next().unwrap());
                }
            }
        }
        
        if !unsubstituted.is_empty() {
            return Err(format!(
                "Template contains unsubstituted variables: {}",
                unsubstituted.join(", ")
            ));
        }
    }
    
    Ok(result)
}