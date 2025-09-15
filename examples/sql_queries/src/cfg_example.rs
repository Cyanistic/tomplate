// Example demonstrating cfg attributes in tomplate blocks
use tomplate::tomplate;

// This example shows how to use cfg attributes to generate
// different SQL queries for different database backends
pub fn setup_queries() {
    tomplate! {
        // Common fields used by all database backends
        let common_fields = tomplate!("id, name, email, created_at");
        let user_table = tomplate!("users");
        
        // Generate different queries based on target database
        // Note: These need unique names since cfg validation happens after parsing
        
        // PostgreSQL-specific queries (uses $1, $2 placeholders)
        #[cfg(feature = "postgres")]
        const GET_USER_POSTGRES = tomplate!(
            "SELECT {fields} FROM {table} WHERE id = $1",
            fields = common_fields,
            table = user_table
        );
        
        #[cfg(feature = "postgres")]
        const USER_WITH_POSTS_PG = tomplate!(
            "SELECT u.*, array_agg(p.*) as posts FROM {table} u 
             LEFT JOIN posts p ON p.user_id = u.id 
             WHERE u.id = $1 
             GROUP BY u.id",
            table = user_table
        );
        
        // SQLite-specific queries (uses ? placeholders)
        #[cfg(feature = "sqlite")]
        const GET_USER_SQLITE = tomplate!(
            "SELECT {fields} FROM {table} WHERE id = ?",
            fields = common_fields,
            table = user_table
        );
        
        #[cfg(feature = "sqlite")]
        const USER_WITH_POSTS_SQLITE = tomplate!(
            "SELECT u.*, GROUP_CONCAT(p.id) as post_ids FROM {table} u 
             LEFT JOIN posts p ON p.user_id = u.id 
             WHERE u.id = ? 
             GROUP BY u.id",
            table = user_table
        );
        
        // MySQL-specific queries (uses ? placeholders)
        #[cfg(feature = "mysql")]
        const GET_USER_MYSQL = tomplate!(
            "SELECT {fields} FROM {table} WHERE id = ?",
            fields = common_fields,
            table = user_table
        );
        
        #[cfg(feature = "mysql")]
        const USER_WITH_POSTS_MYSQL = tomplate!(
            "SELECT u.*, JSON_ARRAYAGG(p.id) as post_ids FROM {table} u 
             LEFT JOIN posts p ON p.user_id = u.id 
             WHERE u.id = ? 
             GROUP BY u.id",
            table = user_table
        );
        
        // Default fallback if no database feature is enabled
        #[cfg(not(any(feature = "postgres", feature = "sqlite", feature = "mysql")))]
        const GET_USER_DEFAULT = tomplate!(
            "SELECT {fields} FROM {table} WHERE id = :id",
            fields = common_fields,
            table = user_table
        );
        
        #[cfg(not(any(feature = "postgres", feature = "sqlite", feature = "mysql")))]
        const USER_WITH_POSTS_DEFAULT = tomplate!(
            "SELECT u.*, GROUP_CONCAT(p.id) as post_ids FROM {table} u 
             LEFT JOIN posts p ON p.user_id = u.id 
             WHERE u.id = :id 
             GROUP BY u.id",
            table = user_table
        );
    }
    
    // Use the appropriate query based on compile-time features
    #[cfg(feature = "postgres")]
    {
        println!("PostgreSQL query: {}", GET_USER_POSTGRES);
        println!("PostgreSQL with posts: {}", USER_WITH_POSTS_PG);
    }
    
    #[cfg(feature = "sqlite")]
    {
        println!("SQLite query: {}", GET_USER_SQLITE);
        println!("SQLite with posts: {}", USER_WITH_POSTS_SQLITE);
    }
    
    #[cfg(feature = "mysql")]
    {
        println!("MySQL query: {}", GET_USER_MYSQL);
        println!("MySQL with posts: {}", USER_WITH_POSTS_MYSQL);
    }
    
    #[cfg(not(any(feature = "postgres", feature = "sqlite", feature = "mysql")))]
    {
        println!("Default query: {}", GET_USER_DEFAULT);
        println!("Default with posts: {}", USER_WITH_POSTS_DEFAULT);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cfg_compilation() {
        // This test verifies that the code compiles with cfg attributes
        setup_queries();
    }
}