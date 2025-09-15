# Tomplate: Zero-Runtime Template Engine for Rust

[![Crates.io](https://img.shields.io/crates/v/tomplate.svg)](https://crates.io/crates/tomplate)
[![Documentation](https://docs.rs/tomplate/badge.svg)](https://docs.rs/tomplate)
[![License](https://img.shields.io/crates/l/tomplate.svg)](LICENSE-MIT)

Tomplate is a powerful compile-time template engine for Rust that processes templates at compile time, resulting in zero runtime overhead. Templates are defined in TOML files and can use various template engines including Handlebars, Tera, and MiniJinja.

## ✨ Features

- **🚀 Zero Runtime Overhead** - All template processing happens at compile time
- **🔧 `#[no_std]` Compatible** - Works in embedded and bare-metal environments
- **🧩 Template Composition** - Build complex templates from reusable parts
- **🎯 Multiple Template Engines** - Choose from Simple, Handlebars, Tera, or MiniJinja
- **📁 File Organization** - Store templates in `.tomplate.toml` files
- **🔍 Compile-Time Validation** - Catch template errors during compilation
- **⚡ Eager Evaluation** - Solve macro expansion order issues with `tomplate_eager!`

## 📦 Installation

```toml
[dependencies]
tomplate = "0.1"

[build-dependencies]
tomplate-build = "0.1"

# Optional: Enable additional template engines
# [dependencies.tomplate]
# version = "0.1"
# features = ["handlebars", "tera", "minijinja"]
```

## 🚀 Quick Start

### Step 1: Create a Build Script

Create `build.rs` in your project root:

```rust
fn main() {
    tomplate_build::Builder::new()
        .add_patterns([
            "**/*.tomplate.toml",
            "templates/*.toml"
        ])
        .build()
        .expect("Failed to build templates");
}
```

### Step 2: Define Templates

Create `templates/queries.tomplate.toml`:

```toml
[user_query]
template = "SELECT {fields} FROM users WHERE {condition}"

[user_insert]
template = """
INSERT INTO users ({columns}) 
VALUES ({values})
RETURNING id
"""

[paginated_query]
template = "SELECT * FROM {table} LIMIT {limit} OFFSET {offset}"
```

### Step 3: Use Templates in Your Code

```rust
use tomplate::tomplate;

// Simple template usage
const GET_USER: &str = tomplate!("user_query",
    fields = "id, name, email",
    condition = "id = $1"
);

// Inline templates (when not found in registry)
const GREETING: &str = tomplate!(
    "Hello {name}, welcome to {place}!",
    name = "Alice",
    place = "Wonderland"
);

// Use the generated SQL in your application
async fn get_user(pool: &PgPool, id: i32) -> Result<User> {
    sqlx::query_as!(User, GET_USER, id)
        .fetch_one(pool)
        .await
}
```

## 🎨 Major Features

### Template Composition Blocks

Build complex templates from reusable parts:

```rust
tomplate! {
    // Local variables - reusable within the block
    let base_fields = tomplate!("id, name, created_at");
    let active_condition = tomplate!("status = 'active'");
    let pagination = tomplate!("LIMIT {limit} OFFSET {offset}",
        limit = "10",
        offset = "0"
    );
    
    // Export constants - available outside the block
    const GET_ACTIVE_USERS = tomplate!(
        "SELECT {fields} FROM users WHERE {condition} {page}",
        fields = base_fields,
        condition = active_condition,
        page = pagination
    );
    
    const COUNT_ACTIVE = tomplate!(
        "SELECT COUNT(*) FROM users WHERE {condition}",
        condition = active_condition
    );
}

// Use the exported constants
let users = sqlx::query!(GET_ACTIVE_USERS).fetch_all(&pool).await?;
```

### Multiple Template Engines

Choose the right engine for each template:

```toml
# Simple substitution (default)
[greeting]
template = "Hello {name}!"
engine = "simple"

# Handlebars for logic
[user_list]
template = """
{{#if users}}
  {{#each users}}
    <li>{{name}} ({{email}})</li>
  {{/each}}
{{else}}
  <li>No users found</li>
{{/if}}
"""
engine = "handlebars"

# Tera for filters
[formatted_output]
template = """
{% for item in items %}
  {{ item.name | upper | truncate(20) }}
{% endfor %}
"""
engine = "tera"
```

### Eager Macro Evaluation

Make Tomplate work with macros that expect string literals:

```rust
use tomplate::{tomplate, tomplate_eager};

// Problem: sqlx::query! expects a string literal
// ❌ This fails:
// sqlx::query!(tomplate!("select_user", id = "5"))

// Solution: Use tomplate_eager! to expand inner macros first
// ✅ This works:
tomplate_eager! {
    sqlx::query!(tomplate!("select_user", id = "5"))
        .fetch_one(&pool)
        .await?
}

// Also works with concat! for combining templates
tomplate_eager! {
    const UNION_QUERY: &str = concat!(
        tomplate!("get_users", status = "active"),
        " UNION ALL ",
        tomplate!("get_users", status = "pending")
    );
}
```

### Nested Template Composition

Templates can use other templates as parameters:

```rust
const COMPLEX_QUERY: &str = tomplate!("union_query",
    first = tomplate!("select_with_filter", 
        table = "users",
        filter = "age > 18"
    ),
    second = tomplate!("select_with_filter",
        table = "accounts",
        filter = "active = true"
    )
);
```

## 🎯 Use Cases

### SQL Query Building

Build complex, reusable SQL queries without runtime string manipulation:

```rust
tomplate! {
    let user_fields = tomplate!("u.id, u.name, u.email");
    let post_fields = tomplate!("p.id, p.title, p.created_at");
    let join_clause = tomplate!("JOIN posts p ON p.user_id = u.id");
    
    const USER_WITH_POSTS = tomplate!(
        "SELECT {user_fields}, {post_fields} FROM users u {join} WHERE u.id = $1",
        user_fields = user_fields,
        post_fields = post_fields,
        join = join_clause
    );
}
```

### Configuration Generation

Generate environment-specific configurations at compile time:

```rust
tomplate! {
    #[cfg(debug_assertions)]
    const API_ENDPOINT = tomplate!("http://localhost:3000/api/{version}",
        version = "v1"
    );
    
    #[cfg(not(debug_assertions))]
    const API_ENDPOINT = tomplate!("https://api.example.com/{version}",
        version = "v1"
    );
}
```

### GraphQL Query Composition

Build GraphQL queries from reusable fragments:

```rust
tomplate! {
    let user_fragment = tomplate!("id name email avatar");
    let post_fragment = tomplate!("id title content createdAt");
    
    const GET_USER_FEED = tomplate!(
        "query GetFeed($userId: ID!) {
            user(id: $userId) { {user_fields} }
            posts(userId: $userId) { {post_fields} }
        }",
        user_fields = user_fragment,
        post_fields = post_fragment
    );
}
```

## 📊 Performance

### Compile Time
- Initial compilation: ~10-30% increase (templates are processed once)
- Incremental builds: Minimal impact (only changed templates reprocessed)
- Templates are cached during build

### Runtime Performance
- **Zero overhead**: Templates become `const &str` in your binary
- **No allocations**: All strings are `&'static str`
- **No parsing**: Templates are fully processed at compile time
- **Optimized binary**: Compiler can optimize const strings

## 🔄 Comparison with Alternatives

| Feature | Tomplate | Runtime Templates | macro_rules! | const_format |
|---------|----------|-------------------|--------------|--------------|
| Runtime Overhead | ✅ None | ❌ High | ✅ None | ✅ None |
| Template Engines | ✅ Multiple | ⚠️ Single | ❌ None | ❌ None |
| Composition | ✅ Advanced | ⚠️ Limited | ⚠️ Manual | ❌ None |
| File-based | ✅ Yes | ✅ Yes | ❌ No | ❌ No |
| Macro Compatible | ✅ Yes | ❌ No | ⚠️ Limited | ⚠️ Limited |
| Complex Logic | ✅ Yes | ✅ Yes | ⚠️ Limited | ❌ No |

## ⚠️ Limitations

1. **Compile-time only**: All template data must be known at compile time
2. **Const context**: Can't use runtime variables or function results
3. **Build time**: Heavy template use increases compilation time
4. **Binary size**: All generated strings are included in the binary

## 💡 Why I Built Tomplate

I created Tomplate out of a specific frustration: I wanted to reuse SQL query fragments across multiple `sqlx::query!` macro calls, but Rust's macro system made this surprisingly difficult. The `sqlx::query!` macro requires a string literal - you can't pass it a const variable or the result of another macro. This meant I was either duplicating SQL fragments everywhere or doing error-prone string concatenation at runtime.

Initially, I just wanted a way to compose SQL queries at compile time. But as I built the solution, I realized this pattern could solve a much broader problem. Many Rust macros expect string literals, and there was no good way to build those strings from reusable parts at compile time. 

Tomplate grew from this simple need into a complete compile-time template system. Now you can:
- Define reusable template fragments in TOML files
- Compose complex templates from simple parts
- Use powerful template engines like Handlebars at compile time
- Generate any kind of string (SQL, HTML, GraphQL, configs) with zero runtime cost

What started as a workaround for `sqlx::query!` became a general-purpose tool for anyone who wants the power of template engines without the runtime overhead. If you've ever been frustrated by Rust's macro limitations or wanted to eliminate runtime template processing, Tomplate might be exactly what you need.

## 📚 Documentation

For detailed documentation and examples, visit [docs.rs/tomplate](https://docs.rs/tomplate).

## 📄 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
