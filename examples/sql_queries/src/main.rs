use tomplate::tomplate;

mod cfg_example;
mod composition;
mod eager;
mod engines;
mod inline;

fn main() {
    println!("=== Direct Template Calls ===");
    direct_template_examples();
    
    println!("\n=== Composition Blocks ===");
    composition::composition_example();
    
    println!("\n=== Inline Templates ===");
    inline::inline_examples();
    inline::inline_composition();
    
    println!("\n=== Template Engines ===");
    engines::simple_example();
    
    #[cfg(feature = "handlebars")]
    engines::handlebars_example();
    
    #[cfg(feature = "tera")]
    engines::tera_example();
    
    #[cfg(feature = "minijinja")]
    engines::minijinja_example();
    
    println!("\n=== Eager Evaluation ===");
    eager::macro_order_problem();
    eager::eager_evaluation_solution();
    eager::simulated_sqlx_example();
    eager::nested_composition_example();
    eager::multiple_macros_example();
    
    println!("\n=== Cfg Attributes ===");
    cfg_example::setup_queries();
}

fn direct_template_examples() {
    // Simple template usage
    const SIMPLE_QUERY: &str = tomplate!("select_user",
        fields = "id, name",
        condition = "active = true"
    );
    println!("Simple query: {}", SIMPLE_QUERY);
    
    // Using nested tomplate! calls for recursive expansion
    const NESTED_QUERY: &str = tomplate!("select_user",
        fields = tomplate!("user_fields"),
        condition = "created_at > '2024-01-01'"
    );
    println!("Nested query: {}", NESTED_QUERY);
    
    // Complex join query with multiple nested templates
    const JOIN_QUERY: &str = tomplate!("join_query",
        fields = "u.name, p.title",
        table1 = tomplate!("table_name"),
        table2 = "posts p",
        join_condition = "u.id = p.user_id",
        where_condition = "p.published = true"
    );
    println!("Join query: {}", JOIN_QUERY);
    
    // Double nesting - template within template within template
    const DOUBLE_NESTED: &str = tomplate!("select_posts",
        fields = tomplate!("post_fields"),
        condition = "user_id IN (SELECT id FROM users WHERE active = true)"
    );
    println!("Double nested: {}", DOUBLE_NESTED);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_template() {
        const QUERY: &str = tomplate!("select_user",
            fields = "id",
            condition = "id = 1"
        );
        assert_eq!(QUERY, "SELECT id FROM users WHERE id = 1");
    }
    
    #[test]
    fn test_nested_template() {
        const QUERY: &str = tomplate!("select_user",
            fields = tomplate!("user_fields"),
            condition = "active = true"
        );
        assert_eq!(QUERY, "SELECT id, name, email FROM users WHERE active = true");
    }
}