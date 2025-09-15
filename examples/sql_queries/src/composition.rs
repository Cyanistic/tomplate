use tomplate::tomplate;

// Example of composition blocks
pub fn composition_example() {
    // Define reusable template fragments and compose them
    tomplate! {
        // Using registered templates from queries.tomplate.toml
        const USER_SELECT_QUERY = tomplate!(
            "select_user",
            fields = "id, name, email",
            condition = "active = true"
        );
        
        const POST_SELECT_QUERY = tomplate!(
            "select_posts",
            fields = "id, title, content",
            condition = "published = true"
        );

    }
    
    // The constants are now available
    println!("User query: {}", USER_SELECT_QUERY);
    println!("Post query: {}", POST_SELECT_QUERY);
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_composition_block() {
        tomplate! {
            const TEST_QUERY = tomplate!(
                "select_user",
                fields = "id, name",
                condition = "test = true"
            );
        }
        
        assert_eq!(TEST_QUERY, "SELECT id, name FROM users WHERE test = true");
    }
    
    #[test]
    fn test_composition_with_let() {
        tomplate! {
            let my_fields = tomplate!("user_fields");
            const RESULT = tomplate!(
                "select_user",
                fields = my_fields,
                condition = "active = true"
            );
        }
        
        assert_eq!(RESULT, "SELECT id, name, email FROM users WHERE active = true");
    }
}
