use tomplate::tomplate_eager;
#[allow(unused_imports)]
use tomplate::tomplate;

/// Demonstrates the problem with macro expansion order
pub fn macro_order_problem() {
    // This would NOT work with sqlx::query! because it expects a string literal
    // and sees the unexpanded tomplate! macro instead:
    //
    // sqlx::query!(tomplate!("select_user", fields = "id, name", condition = "id = $1"))
    //     .bind(user_id)
    //     .fetch_one(&pool)
    //     .await?;
    //
    // The compiler would error with something like:
    // "expected string literal, found macro invocation"
    
    println!("Without tomplate_eager!, nested macros fail at compile time");
}

/// Demonstrates the solution with tomplate_eager!
pub fn eager_evaluation_solution() {
    // tomplate_eager! walks the token tree and expands tomplate! calls first
    // So the outer macro sees the result, not the macro invocation
    
    // Example 1: Simple eager evaluation
    tomplate_eager! {
        const QUERY: &str = tomplate!("select_user", 
            fields = "id, name, email",
            condition = "active = true"
        );
    }
    
    println!("Eager evaluated query: {}", QUERY);
    
    // Example 2: With concat! macro
    tomplate_eager! {
        const COMBINED: &str = concat!(
            tomplate!("select_user", fields = "id", condition = "1=1"),
            " UNION ALL ",
            tomplate!("select_posts", fields = "id", condition = "1=1")
        );
    }
    
    println!("Combined query: {}", COMBINED);
}

/// Simulated sqlx::query! usage (since we don't have sqlx as dependency)
#[allow(unused_macros)]
macro_rules! fake_query {
    ($query:expr) => {{
        // This macro expects a string literal
        struct Query { sql: &'static str }
        Query { sql: $query }
    }};
}

pub fn simulated_sqlx_example() {
    // This demonstrates how it would work with sqlx::query!
    tomplate_eager! {
        let query = fake_query!(tomplate!("select_user",
            fields = "id, name, email",
            condition = "id = 1"
        ));
    }
    
    println!("Query would be: {:?}", query.sql);
}

/// Example with nested template composition
pub fn nested_composition_example() {
    tomplate_eager! {
        const COMPLEX_QUERY: &str = tomplate!("select_user",
            fields = "id, name, email",
            condition = "active = true"
        );
    }
    
    println!("Complex nested query: {}", COMPLEX_QUERY);
}

/// Example showing multiple macros in one eager block
pub fn multiple_macros_example() {
    tomplate_eager! {
        // Multiple independent macro expansions
        const USERS_QUERY: &str = tomplate!("select_user", 
            fields = "*", 
            condition = "1=1"
        );
        
        const POSTS_QUERY: &str = tomplate!("select_posts",
            fields = "*",
            condition = "published = true"
        );
        
        // Using concat! with tomplate!
        const UNION_QUERY: &str = concat!(
            tomplate!("select_user", fields = "id, 'user' as type", condition = "1=1"),
            " UNION ALL ",
            tomplate!("select_posts", fields = "id, 'post' as type", condition = "1=1")
        );
    }
    
    println!("Users: {}", USERS_QUERY);
    println!("Posts: {}", POSTS_QUERY);
    println!("Union: {}", UNION_QUERY);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_eager_evaluation() {
        tomplate_eager! {
            const TEST: &str = tomplate!("simple_greeting",
                name = "Test",
                place = "Testing"
            );
        }
        
        assert_eq!(TEST, "Hello Test, welcome to Testing!");
    }
    
    #[test]
    fn test_concat_in_eager() {
        tomplate_eager! {
            const COMBINED: &str = concat!("Hello", " ", "World");
        }
        
        assert_eq!(COMBINED, "Hello World");
    }
    
    #[test]
    fn test_nested_templates_in_eager() {
        tomplate_eager! {
            const NESTED: &str = tomplate!("select_user",
                fields = tomplate!("user_fields"),
                condition = "active = true"
            );
        }
        
        assert_eq!(NESTED, "SELECT id, name, email FROM users WHERE active = true");
    }
}