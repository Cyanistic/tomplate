# Tomplate: TOML-Based Compile-Time Template Composition for Rust

## Table of Contents

1. [README - User Documentation](#readme---user-documentation)
   - [Overview](#overview)
   - [Motivation](#motivation)
   - [Key Features](#key-features)
   - [Installation](#installation)
   - [Quick Start](#quick-start)
   - [Core Capabilities](#core-capabilities)
   - [Use Cases](#use-cases)
   - [Comparison with Alternatives](#comparison-with-alternatives)
   - [Performance](#performance)
   - [Limitations](#limitations)
2. [Implementation Plan](#implementation-plan)
   - [Architecture Overview](#architecture-overview)
   - [Phase 1: Core Infrastructure](#phase-1-core-infrastructure)
   - [Phase 2: Template Discovery](#phase-2-template-discovery)
   - [Phase 3: Composition Block](#phase-3-composition-block)
   - [Phase 4: Template Engines](#phase-4-template-engines)
   - [Phase 5: Advanced Features](#phase-5-advanced-features)
   - [Technical Challenges](#technical-challenges)

---

## README - User Documentation

### Overview

Tomplate is a revolutionary compile-time template composition library for Rust that processes templates at compile time using familiar templating engines like Handlebars and Tera. Unlike traditional templating solutions, Tomplate evaluates all templates during compilation, resulting in zero runtime overhead and compile-time validation of your templates.

The name "Tomplate" is a clever play on TOML + template, reflecting the library's unique approach of using TOML files for template configuration.

### Motivation

The Rust ecosystem has excellent runtime templating solutions, but compile-time templating has been limited to basic string manipulation with `macro_rules!` or `const_format`. This creates several pain points:

1. **Runtime Overhead**: Traditional template engines parse and process templates at runtime, adding latency and memory usage
2. **Complex Macro Syntax**: Building complex strings with `macro_rules!` quickly becomes unreadable
3. **Limited Compile-Time Tools**: `concat!` and `format_args!` only support basic substitution
4. **No Composition**: Existing solutions don't allow building templates from reusable parts
5. **No Template Reuse**: Can't share template fragments across different compositions

Tomplate was born from a real need: building complex SQL queries at compile time without the syntactic nightmare of nested macros or the runtime cost of template processing.

### Key Features

#### 🚀 **Zero Runtime Overhead**
All template processing happens at compile time. Your final binary contains only static strings.

#### 🧩 **Template Composition**
Build complex templates from simple, reusable parts using an innovative scoped block syntax.

#### 🎯 **Multiple Template Engines**
Use Handlebars, Tera, MiniJinja, or a simple substitution engine - whatever fits your needs.

#### 📁 **File-Based Templates**
Organize templates in `.tomplate.toml` files that are automatically discovered and processed.

#### 🔍 **Compile-Time Validation**
Catch template errors during compilation, not at runtime.

#### 🏗️ **Scoped Composition Blocks**
Define local template variables and export constants using familiar Rust syntax.

### Installation

```toml
[dependencies]
tomplate = "0.1"

[build-dependencies]
tomplate-build = "0.1"
```

### Quick Start

#### Simple Inline Template

```rust
use tomplate::tomplate;

// Process a template at compile time
const QUERY: &str = tomplate!("SELECT {fields} FROM {table}", 
    fields = "id, name",
    table = "users"
);

assert_eq!(QUERY, "SELECT id, name FROM users");
```

#### Template Composition

```rust
use tomplate::tomplate;

tomplate! {
    // Define local template helpers
    let user_fields = template!("user_fields");
    let where_active = template!("where_active");
    
    // Export as real Rust constants
    const USER_QUERY = template!("select_query",
        fields = user_fields,
        table = "users",
        where = where_active
    );
    
    const POST_QUERY = template!("select_query",
        fields = "id, title, content",
        table = "posts",
        where = where_active
    );
}

// Use the generated constants
let users = sqlx::query(USER_QUERY).fetch_all(&pool).await?;
```

#### File-Based Templates

Create a `queries.stencil.toml`:

```toml
[user_list]
template = "SELECT {{fields}} FROM users WHERE {{condition | default: '1=1'}}"
engine = "handlebars"

[user_insert]
template = """
INSERT INTO users ({{columns}}) 
VALUES ({{values}})
"""
```

Use in your code:

```rust
const LIST_USERS: &str = mosaic!("user_list",
    fields = "id, name, email",
    condition = "active = true"
);
```

### Core Capabilities

#### 1. **Template Composition Through Nesting**

Since Mosaic operates at compile time, templates can be composed by nesting template calls:

```rust
const COMPLEX_QUERY: &str = mosaic!("union_query",
    first = mosaic!("select_query", 
        table = "users",
        fields = "id, name"
    ),
    second = mosaic!("select_query",
        table = "archived_users", 
        fields = "id, name"
    )
);
```

#### 2. **Scoped Composition Blocks**

The killer feature: define template variables in a scope and compose them:

```rust
tomplate! {
    // Local bindings - only visible in this block
    let base_fields = template!("id, name, created_at");
    let pagination = template!("LIMIT {limit} OFFSET {offset}",
        limit = 10,
        offset = 0
    );
    
    // Export constants - available outside the block
    const USERS_PAGE_1 = template!("SELECT {fields} FROM users {pagination}",
        fields = base_fields,
        pagination = pagination
    );
    
    const POSTS_PAGE_1 = template!("SELECT {fields} FROM posts {pagination}",
        fields = base_fields,
        pagination = pagination
    );
}
```

#### 3. **Multiple Template Engines**

Configure different engines per template:

```toml
[simple_template]
template = "Hello {name}"
engine = "simple"  # Basic substitution

[complex_template]
template = """
{{#if users}}
  {{#each users}}
    <li>{{this.name}}</li>
  {{/each}}
{{/if}}
"""
engine = "handlebars"  # Full Handlebars features

[jinja_template]
template = """
{% for item in items %}
  {{ item.name | upper }}
{% endfor %}
"""
engine = "tera"  # Jinja2-like syntax
```

#### 4. **Build Script Configuration**

```rust
// build.rs
fn main() {
    mosaic::Builder::new()
        .default_engine("handlebars")
        .discover_pattern("**/*.stencil.toml")
        .discover_pattern("**/*.stencil")
        .discover_directory("templates")
        .validate_templates(true)
        .build();
}
```

### Use Cases

#### SQL Query Building

The original motivation - build complex SQL queries without string concatenation hell:

```rust
tomplate! {
    let user_columns = template!("id, username, email, created_at");
    let active_check = template!("status = 'active' AND verified = true");
    
    const GET_ACTIVE_USERS = template!(
        "SELECT {columns} FROM users WHERE {condition}",
        columns = user_columns,
        condition = active_check
    );
    
    const COUNT_ACTIVE_USERS = template!(
        "SELECT COUNT(*) FROM users WHERE {condition}",
        condition = active_check
    );
}
```

#### Configuration Generation

Generate environment-specific configurations at compile time:

```rust
tomplate! {
    let base_config = template!("base_nginx_config");
    
    #[cfg(feature = "production")]
    const NGINX_CONFIG = template!("nginx_with_ssl",
        base = base_config,
        domain = "example.com",
        ssl_cert = "/etc/ssl/prod.crt"
    );
    
    #[cfg(not(feature = "production"))]
    const NGINX_CONFIG = template!("nginx_simple",
        base = base_config,
        port = 8080
    );
}
```

#### GraphQL Schema Composition

Build complex GraphQL queries from fragments:

```rust
tomplate! {
    let user_fragment = template!("fragment_user");
    let post_fragment = template!("fragment_post",
        author = user_fragment
    );
    
    const FEED_QUERY = template!("query_feed",
        posts = post_fragment,
        user = user_fragment
    );
}
```

#### Static Site Generation

Generate HTML at compile time:

```rust
tomplate! {
    let header = template!("site_header", title = "My Site");
    let footer = template!("site_footer", year = "2024");
    
    const HOME_PAGE = template!("page_layout",
        header = header,
        content = template!("home_content"),
        footer = footer
    );
    
    const ABOUT_PAGE = template!("page_layout",
        header = header,
        content = template!("about_content"),
        footer = footer
    );
}
```

### Comparison with Alternatives

| Feature | Mosaic | Runtime Templates (Tera/Handlebars) | macro_rules! | const_format |
|---------|---------|---------------------------------------|--------------|--------------|
| Runtime Overhead | None ✅ | Parse + Process ❌ | None ✅ | None ✅ |
| Template Engines | Multiple ✅ | Single ⚠️ | None ❌ | None ❌ |
| Composition | Advanced ✅ | Limited ⚠️ | Manual ⚠️ | None ❌ |
| File-based | Yes ✅ | Yes ✅ | No ❌ | No ❌ |
| Learning Curve | Moderate | Low | High | Low |
| IDE Support | Good ✅ | Excellent ✅ | Poor ❌ | Good ✅ |
| Complex Logic | Yes ✅ | Yes ✅ | Limited ⚠️ | No ❌ |

### Performance

#### Compile Time Impact
- Initial compilation: ~10-30% increase depending on template complexity
- Incremental compilation: Minimal impact (templates only reprocessed when changed)
- Template caching: Build script caches processed templates

#### Runtime Performance
- **Zero overhead**: Templates are const strings in the final binary
- **No allocation**: All strings are &'static str
- **No parsing**: Templates are pre-processed
- **Binary size**: Slightly larger due to expanded templates

### Limitations

1. **No Runtime Data**: Templates must be fully resolvable at compile time
2. **Const Context**: Can't use runtime variables or function results
3. **Compilation Time**: Heavy template use increases build times
4. **Debugging**: Template errors appear as macro expansion errors
5. **Binary Size**: All template variations are included in the binary

---

## Implementation Plan

### Architecture Overview

Mosaic consists of three main components:

1. **Build Script Library** (`mosaic::Builder`)
   - Discovers template files
   - Processes TOML/JSON template definitions
   - Generates a unified registry
   - Handles template engine initialization

2. **Proc Macro** (`mosaic!` macro)
   - Parses the composition block syntax
   - Manages scoped variable bindings
   - Processes template references
   - Generates final const declarations

3. **Template Engine Adapters**
   - Unified interface for different engines
   - Compile-time template processing
   - Error handling and validation

### Phase 1: Core Infrastructure

#### 1.1 Project Structure
```
tomplate/
├── Cargo.toml           # Workspace root
├── tomplate/            # Main library
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs       # Public API
│   │   ├── builder.rs   # Build script API
│   │   └── engines/     # Template engine adapters
├── tomplate-macros/     # Proc macro crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs       # Macro entry point
│       ├── parser.rs    # Block syntax parser
│       └── composer.rs  # Template composition logic
└── examples/
    └── sql_queries/     # Example project
```

#### 1.2 Basic Dependencies
```toml
# mosaic/Cargo.toml
[dependencies]
handlebars = { version = "5.0", optional = true }
tera = { version = "1.19", optional = true }
minijinja = { version = "1.0", optional = true }
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"

[features]
default = ["handlebars", "simple"]
all-engines = ["handlebars", "tera", "minijinja"]

# mosaic-macros/Cargo.toml
[dependencies]
proc-macro2 = "1.0"
quote = "1.0"
syn = { version = "2.0", features = ["full"] }
```

### Phase 2: Template Discovery

#### 2.1 File Discovery System
```rust
// builder.rs
impl Builder {
    pub fn discover_pattern(mut self, pattern: &str) -> Self {
        self.patterns.push(pattern.to_string());
        self
    }
    
    pub fn discover_directory(mut self, dir: &str) -> Self {
        self.directories.push(dir.to_string());
        self
    }
    
    fn discover_templates(&self) -> HashMap<String, Template> {
        // Scan filesystem for .stencil.toml, .stencil files
        // Parse and validate templates
        // Return unified registry
    }
}
```

#### 2.2 TOML Template Format
```toml
# Template definition schema
[template_name]
template = "..." # Template string or file path
engine = "..."   # Optional: handlebars|tera|minijinja|simple
metadata = {}    # Optional: additional metadata
schema = {}      # Optional: parameter validation
```

#### 2.3 Registry Generation
```rust
// Generate unified registry file
fn generate_registry(templates: HashMap<String, Template>) {
    let out_dir = env::var("OUT_DIR").unwrap();
    let registry_path = Path::new(&out_dir).join("mosaic_registry.rs");
    
    // Write registry as Rust code
    let mut file = File::create(registry_path).unwrap();
    writeln!(file, "pub const MOSAIC_REGISTRY: &str = r#\"{}\"#;",
        toml::to_string(&templates).unwrap()
    );
}
```

### Phase 3: Composition Block

#### 3.1 Syntax Parser
```rust
// Parser for the tomplate! { ... } block syntax
enum Statement {
    Let {
        name: Ident,
        value: TemplateCall,
    },
    Const {
        name: Ident,
        value: TemplateCall,
    },
    Template(TemplateCall),
}

struct TemplateCall {
    name: String,
    params: HashMap<String, Value>,
}
```

#### 3.2 Dependency Resolution
```rust
// Track dependencies between templates
struct DependencyGraph {
    nodes: HashMap<String, TemplateNode>,
    edges: Vec<(String, String)>,
}

impl DependencyGraph {
    fn topological_sort(&self) -> Result<Vec<String>, CycleError> {
        // Ensure templates are processed in correct order
    }
}
```

#### 3.3 Variable Binding Management
```rust
// Manage local and exported bindings
struct BindingScope {
    locals: HashMap<String, String>,  // let bindings
    exports: Vec<(String, String)>,   // const bindings
}

impl BindingScope {
    fn resolve(&self, name: &str) -> Option<String> {
        self.locals.get(name).cloned()
    }
    
    fn add_local(&mut self, name: String, value: String) {
        self.locals.insert(name, value);
    }
    
    fn add_export(&mut self, name: String, value: String) {
        self.exports.push((name, value));
    }
}
```

#### 3.4 Code Generation
```rust
// Generate final Rust code
fn generate_output(scope: BindingScope) -> TokenStream {
    let mut output = TokenStream::new();
    
    // Generate const declarations for exports
    for (name, value) in scope.exports {
        output.extend(quote! {
            const #name: &str = #value;
        });
    }
    
    output
}
```

### Phase 4: Template Engines

#### 4.1 Engine Trait
```rust
pub trait TemplateEngine {
    fn process(&self, template: &str, params: &HashMap<String, Value>) -> Result<String, Error>;
    fn validate(&self, template: &str) -> Result<(), Error>;
}
```

#### 4.2 Engine Implementations
```rust
// Simple substitution engine
struct SimpleEngine;
impl TemplateEngine for SimpleEngine {
    fn process(&self, template: &str, params: &HashMap<String, Value>) -> Result<String, Error> {
        // Basic {key} replacement
    }
}

// Handlebars adapter
struct HandlebarsEngine {
    registry: Handlebars<'static>,
}

// Tera adapter
struct TeraEngine {
    tera: Tera,
}
```

#### 4.3 Engine Registry
```rust
struct EngineRegistry {
    engines: HashMap<String, Box<dyn TemplateEngine>>,
    default: String,
}

impl EngineRegistry {
    fn get(&self, name: Option<&str>) -> &dyn TemplateEngine {
        let name = name.unwrap_or(&self.default);
        self.engines.get(name).expect("Unknown engine")
    }
}
```

### Phase 5: Advanced Features

#### 5.1 Template Validation
```toml
[strict_insert]
template = "INSERT INTO users ({{id}}, {{name}}) VALUES ({{id_value}}, {{name_value}})"
schema = {
    id = { type = "integer", required = true },
    name = { type = "string", max_length = 100 }
}
```

#### 5.2 Conditional Compilation
```rust
tomplate! {
    #[cfg(feature = "postgres")]
    const QUERY = template!("postgres_syntax");
    
    #[cfg(feature = "sqlite")]
    const QUERY = template!("sqlite_syntax");
}
```

#### 5.3 Template Inheritance
```toml
[base_query]
template = "SELECT {{fields}} FROM {{table}}"

[extended_query]
extends = "base_query"
template = "{{super}} WHERE {{condition}}"
```

#### 5.4 Error Reporting
```rust
// Enhanced error messages with template location
#[derive(Debug)]
struct TemplateError {
    template_name: String,
    line: usize,
    column: usize,
    message: String,
    suggestion: Option<String>,
}
```

### Technical Challenges

#### Challenge 1: Compile-Time String Processing
**Problem**: Rust macros receive tokens, not evaluated strings. Constants aren't resolved during macro expansion.

**Solution**: The scoped block approach where the macro maintains its own binding registry and resolves references during expansion.

#### Challenge 2: Template Engine Integration
**Problem**: Template engines are designed for runtime use, not compile-time processing.

**Solution**: Use build script to pre-process templates and generate static strings. The macro then works with these pre-processed results.

#### Challenge 3: Cross-Crate Template Sharing
**Problem**: Templates defined in one crate need to be accessible in dependent crates.

**Solution**: Export processed templates as public constants. Consider a registry trait for discovering templates from dependencies.

#### Challenge 4: Incremental Compilation
**Problem**: Changes to templates shouldn't trigger full rebuilds.

**Solution**: 
- Track template dependencies in build script
- Use cargo:rerun-if-changed directives
- Cache processed templates with content hashing

#### Challenge 5: IDE Support
**Problem**: IDE needs to understand template syntax and provide completions.

**Solution**:
- Generate TypeScript-style definition files for templates
- Provide Language Server Protocol (LSP) implementation
- Create IDE plugins for popular editors

#### Challenge 6: Debugging Template Errors
**Problem**: Template errors appear as opaque macro expansion errors.

**Solution**:
- Custom error types with span information
- Generate intermediate files for debugging
- Provide verbose mode showing expansion steps

### Implementation Timeline

**Week 1-2**: Core infrastructure and basic macro
- Set up project structure
- Implement basic mosaic! macro
- Simple template substitution

**Week 3-4**: Build script and discovery
- File discovery system
- TOML parsing
- Registry generation

**Week 5-6**: Composition blocks
- Let/const binding parser
- Dependency resolution
- Variable substitution

**Week 7-8**: Template engines
- Engine trait and registry
- Handlebars integration
- Tera integration

**Week 9-10**: Testing and documentation
- Comprehensive test suite
- Documentation
- Examples

**Week 11-12**: Polish and release
- Performance optimization
- Error message improvement
- Crates.io release preparation

### Success Metrics

1. **Functionality**: All planned features working
2. **Performance**: < 30% compile time increase for typical usage
3. **Usability**: Clear documentation and helpful error messages
4. **Adoption**: 100+ downloads in first month
5. **Community**: At least 3 external contributors

### Future Enhancements

1. **Template Hot Reloading**: Development mode with file watching
2. **WASM Support**: Compile templates to WASM for browser use
3. **Template Marketplace**: Community template sharing
4. **Visual Editor**: GUI for template composition
5. **Migration Tools**: Convert from runtime templates to Mosaic
6. **Benchmarking Suite**: Performance comparison tools
7. **Integration Plugins**: SQLx, Diesel, GraphQL client integrations