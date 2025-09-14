use crate::types::Result;
use glob::glob;
use std::path::PathBuf;

pub fn discover_templates(patterns: &[String]) -> Result<Vec<PathBuf>> {
    let mut template_files = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();
    
    for pattern in patterns {
        for entry in glob(pattern)? {
            match entry {
                Ok(path) => {
                    // Only include files, not directories
                    if path.is_file() {
                        // Deduplicate paths
                        if seen_paths.insert(path.clone()) {
                            template_files.push(path);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Error reading path matching pattern '{}': {}", pattern, e);
                }
            }
        }
    }
    
    // Sort for consistent ordering
    template_files.sort();
    
    Ok(template_files)
}