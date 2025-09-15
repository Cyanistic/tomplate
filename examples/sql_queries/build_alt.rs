// Alternative build.rs demonstrating add_patterns
fn main() {
    // Using add_patterns with a vector
    tomplate_build::Builder::new()
        .add_patterns(vec![
            "**/*.tomplate.toml",
            "templates/*.toml",
            "config/*.tomplate.toml"
        ])
        .build()
        .expect("Failed to build templates");
    
    // Or with an array
    tomplate_build::Builder::new()
        .add_patterns([
            "**/*.tomplate.toml",
            "templates/*.toml"
        ])
        .build()
        .expect("Failed to build templates");
    
    // Or collect from some logic
    let patterns: Vec<String> = (1..=3)
        .map(|i| format!("templates/v{}/**.toml", i))
        .collect();
    
    tomplate_build::Builder::new()
        .add_patterns(patterns)
        .build()
        .expect("Failed to build templates");
}