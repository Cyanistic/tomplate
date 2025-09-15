use tomplate::tomplate;

#[cfg(feature = "handlebars")]
pub fn handlebars_example() {
    tomplate! {
        const USER_QUERY = tomplate!(
            "handlebars_user_query",
            fields = "id, name, email",
            condition = "active = true",
            order = "created_at DESC"
        );
    }
    
    println!("Handlebars query:\n{}", USER_QUERY);
}

#[cfg(feature = "tera")]
pub fn tera_example() {
    tomplate! {
        const CONFIG = tomplate!(
            "tera_config",
            host = "localhost",
            port = "3000",
            debug = "true"
        );
    }
    
    println!("Tera config:\n{}", CONFIG);
}

#[cfg(feature = "minijinja")]
pub fn minijinja_example() {
    // Note: MiniJinja example would need array support
    // For now, just show a simple example
    tomplate! {
        const REPORT = tomplate!(
            "minijinja_report",
            name = "Monthly Sales"
        );
    }
    
    println!("MiniJinja report:\n{}", REPORT);
}

pub fn simple_example() {
    tomplate! {
        const GREETING = tomplate!(
            "simple_greeting",
            name = "Alice",
            place = "Wonderland"
        );
    }
    
    println!("Simple greeting: {}", GREETING);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_engine() {
        tomplate! {
            const RESULT = tomplate!(
                "simple_greeting",
                name = "Bob",
                place = "Testing"
            );
        }
        
        assert_eq!(RESULT, "Hello Bob, welcome to Testing!");
    }
    
    #[cfg(feature = "handlebars")]
    #[test]
    fn test_handlebars_engine() {
        tomplate! {
            const QUERY = tomplate!(
                "handlebars_user_query",
                fields = "id, username",
                condition = "role = 'admin'"
            );
        }
        
        assert!(QUERY.contains("SELECT id, username"));
        assert!(QUERY.contains("WHERE role = 'admin'"));
    }
}