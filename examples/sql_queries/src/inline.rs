use tomplate::tomplate;

/// Demonstrates inline template strings (not from registry)
pub fn inline_examples() {
    // Direct inline template with simple substitution
    const GREETING: &str = tomplate!(
        "Hello {name}, welcome to {place}!",
        name = "Alice",
        place = "Wonderland"
    );
    println!("Inline greeting: {}", GREETING);
    
    // Inline SQL query template
    const QUERY: &str = tomplate!(
        "SELECT {fields} FROM {table} WHERE {condition}",
        fields = "id, name, email",
        table = "users",
        condition = "active = true"
    );
    println!("Inline query: {}", QUERY);
    
    // Inline with nested tomplate calls
    const NESTED: &str = tomplate!(
        "User: {name}, Query: {query}",
        name = "Bob",
        query = tomplate!(
            "SELECT * FROM posts WHERE user_id = {id}",
            id = "123"
        )
    );
    println!("Nested inline: {}", NESTED);
}

/// Demonstrates inline templates in composition blocks
pub fn inline_composition() {
    tomplate! {
        // Inline templates as let bindings
        let header = tomplate!("=== {title} ===", title = "Report");
        let separator = tomplate!("{char}{char}{char}", char = "-");
        
        // Use inline templates in const exports
        const REPORT_HEADER = tomplate!(
            "{header}\n{sep}\n",
            header = header,
            sep = separator
        );
        
        // Mix inline and registry templates
        const MIXED = tomplate!(
            "{inline} | {registry}",
            inline = tomplate!("Inline: {value}", value = "test"),
            registry = tomplate!("simple_greeting", name = "User", place = "Here")
        );
    }
    
    println!("Report header:\n{}", REPORT_HEADER);
    println!("Mixed: {}", MIXED);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_inline_template() {
        const RESULT: &str = tomplate!(
            "Hello {name}!",
            name = "Test"
        );
        assert_eq!(RESULT, "Hello Test!");
    }
    
    #[test]
    fn test_inline_with_multiple_params() {
        const RESULT: &str = tomplate!(
            "{a} + {b} = {c}",
            a = "1",
            b = "2", 
            c = "3"
        );
        assert_eq!(RESULT, "1 + 2 = 3");
    }
    
    #[test]
    fn test_nested_inline() {
        const RESULT: &str = tomplate!(
            "Outer: {inner}",
            inner = tomplate!("Inner: {value}", value = "nested")
        );
        assert_eq!(RESULT, "Outer: Inner: nested");
    }
}