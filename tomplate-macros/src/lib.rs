//! # Tomplate Procedural Macros
//!
//! This crate provides the procedural macros that power Tomplate's compile-time
//! template processing. These macros expand at compile time to produce static strings,
//! ensuring zero runtime overhead for template processing.
//!
//! ## Macros Provided
//!
//! ### `tomplate!` - Main Template Macro
//!
//! The `tomplate!` macro is the primary interface for template processing. It supports
//! two distinct modes of operation:
//!
//! #### Mode 1: Direct Template Invocation
//!
//! Process a single template with parameters:
//!
//! ```rust,ignore
//! // File-based template from registry
//! const QUERY: &str = tomplate!("user_query", 
//!     fields = "id, name",
//!     condition = "active = true"
//! );
//!
//! // Inline template (when not found in registry)
//! const GREETING: &str = tomplate!("Hello {name}!", 
//!     name = "World"
//! );
//! ```
//!
//! #### Mode 2: Composition Block
//!
//! Define multiple templates with local variables:
//!
//! ```rust,ignore
//! tomplate! {
//!     // Local variables (not exported)
//!     let common_fields = tomplate!("id, name, email");
//!     let active_filter = tomplate!("status = 'active'");
//!     
//!     // Exported constants (available outside block)
//!     const USER_QUERY = tomplate!(
//!         "SELECT {fields} FROM users WHERE {filter}",
//!         fields = common_fields,
//!         filter = active_filter
//!     );
//!     
//!     const COUNT_QUERY = tomplate!(
//!         "SELECT COUNT(*) FROM users WHERE {filter}",
//!         filter = active_filter
//!     );
//! }
//! ```
//!
//! ### `tomplate_eager!` - Eager Macro Expansion
//!
//! Eagerly expands nested `tomplate!` and `concat!` macros before passing to outer macros:
//!
//! ```rust,ignore
//! // Problem: sqlx expects a string literal
//! // This won't work:
//! // sqlx::query!(tomplate!("select_user", id = "5"))
//!
//! // Solution: Use tomplate_eager!
//! tomplate_eager! {
//!     sqlx::query!(tomplate!("select_user", id = "5"))
//!         .fetch_one(&pool)
//!         .await?
//! }
//! ```
//!
//! ## How Template Resolution Works
//!
//! The `tomplate!` macro uses a two-step resolution process:
//!
//! 1. **Registry Lookup**: First checks if the string matches a template name in the 
//!    amalgamated template registry (created by `tomplate-build` at build time)
//!    
//! 2. **Inline Fallback**: If not found in registry, treats the string itself as an
//!    inline template with the simple engine
//!
//! This allows seamless mixing of pre-defined and ad-hoc templates:
//!
//! ```rust,ignore
//! // If "header" exists in registry, uses that template
//! const HEADER: &str = tomplate!("header", title = "My App");
//!
//! // If "Welcome {user}!" doesn't exist in registry, uses it as inline template
//! const WELCOME: &str = tomplate!("Welcome {user}!", user = "Alice");
//! ```
//!
//! ## Parameter Types
//!
//! Templates accept various parameter types:
//!
//! - **String literals**: `"value"`
//! - **Numbers**: `42`, `3.14`
//! - **Booleans**: `true`, `false`
//! - **Nested templates**: `tomplate!("other_template", ...)`
//!
//! ```rust,ignore
//! const EXAMPLE: &str = tomplate!("template_name",
//!     text = "Hello",
//!     count = 42,
//!     pi = 3.14,
//!     enabled = true,
//!     nested = tomplate!("inner", value = "data")
//! );
//! ```
//!
//! ## Template Engines
//!
//! Templates can use different engines based on the `engine` field in TOML:
//!
//! - **simple** (default): Basic `{variable}` substitution
//! - **handlebars**: Full Handlebars with conditionals, loops, helpers
//! - **tera**: Jinja2-like with filters and control structures
//! - **minijinja**: Lightweight Jinja2 implementation
//!
//! The engine is determined at build time from the template definition.
//!
//! ## Compile-Time Processing
//!
//! All template processing happens at compile time:
//!
//! 1. Build script discovers and amalgamates templates
//! 2. Macro reads amalgamated templates at compile time
//! 3. Templates are processed and expanded to string literals
//! 4. Final binary contains only static strings
//!
//! This ensures zero runtime overhead and compile-time validation of templates.

mod block;
mod eager;
mod engines;
mod parser;
mod scope;
mod templates;

use proc_macro::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, Expr, Lit, Token, ExprMacro};

/// Process templates at compile time with zero runtime overhead.
///
/// This macro can be used in two ways:
///
/// ## Direct Template Invocation
///
/// Process a single template with parameters:
///
/// ```rust,ignore
/// // From template registry (defined in .tomplate.toml files)
/// const QUERY: &str = tomplate!("user_query", 
///     fields = "id, name, email",
///     table = "users"
/// );
///
/// // Inline template (when not found in registry)
/// const MSG: &str = tomplate!("Hello {name}!", name = "World");
/// ```
///
/// ## Composition Block
///
/// Define multiple templates with shared local variables:
///
/// ```rust,ignore
/// tomplate! {
///     // Local variables - reusable within block
///     let base_fields = tomplate!("id, created_at, updated_at");
///     
///     // Export constants - available outside block
///     const USER_FIELDS = tomplate!("{base}, name, email",
///         base = base_fields
///     );
///     
///     const POST_FIELDS = tomplate!("{base}, title, content", 
///         base = base_fields
///     );
/// }
///
/// // Use the exported constants
/// println!("{}", USER_FIELDS);
/// ```
///
/// ## Parameters
///
/// - First argument: Template name (from registry) or inline template string
/// - Named parameters: `key = value` pairs for template variables
/// - Values can be literals or nested `tomplate!` calls
///
/// ## Template Resolution
///
/// 1. Checks if first argument matches a template name in registry
/// 2. If found, uses that template with its configured engine
/// 3. If not found, treats the string as an inline template using simple engine
///
/// ## Examples
///
/// ```rust,ignore
/// // Using different parameter types
/// const EXAMPLE: &str = tomplate!("my_template",
///     string = "text",
///     number = 42,
///     float = 3.14,
///     boolean = true,
///     nested = tomplate!("other", value = "data")
/// );
///
/// // Inline templates for quick use
/// const QUICK: &str = tomplate!(
///     "User: {name}, Status: {status}",
///     name = "Alice",
///     status = "active"
/// );
/// ```
#[proc_macro]
pub fn tomplate(input: TokenStream) -> TokenStream {
    // Try to parse as a composition block first
    let input_clone = input.clone();
    match syn::parse::<parser::CompositionBlock>(input_clone) {
        Ok(block) => {
            // Successfully parsed as a block
            match block::process_block(block) {
                Ok(output) => output.into(),
                Err(err) => err.to_compile_error().into(),
            }
        }
        Err(_block_err) => {
            // Not a block, try as direct template call
            match syn::parse::<TomplateInput>(input) {
                Ok(direct) => {
                    match process_template(direct) {
                        Ok(output) => output.into(),
                        Err(err) => err.to_compile_error().into(),
                    }
                }
                Err(direct_err) => {
                    // Failed both parsers, return the direct error as it's more common
                    direct_err.to_compile_error().into()
                }
            }
        }
    }
}

struct TomplateInput {
    template_name: String,
    params: Vec<(String, ParamValue)>,
}

enum ParamValue {
    Literal(String),
    Macro(ExprMacro),
}

impl syn::parse::Parse for TomplateInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Parse template name
        let template_name = match input.parse::<Expr>()? {
            Expr::Lit(lit) => match lit.lit {
                Lit::Str(s) => s.value(),
                _ => return Err(syn::Error::new_spanned(lit, "Expected string literal")),
            },
            _ => return Err(input.error("Expected template name as string literal")),
        };
        
        let mut params = Vec::new();
        
        // Parse optional parameters
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            
            let args = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;
            
            for arg in args {
                match arg {
                    Expr::Assign(assign) => {
                        // Extract parameter name
                        let param_name = match &*assign.left {
                            Expr::Path(path) if path.path.segments.len() == 1 => {
                                path.path.segments[0].ident.to_string()
                            }
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    assign.left,
                                    "Expected simple identifier",
                                ))
                            }
                        };
                        
                        // Extract parameter value (literal or macro)
                        let param_value = match &*assign.right {
                            Expr::Lit(lit) => match &lit.lit {
                                Lit::Str(s) => ParamValue::Literal(s.value()),
                                Lit::Int(i) => ParamValue::Literal(i.to_string()),
                                Lit::Float(f) => ParamValue::Literal(f.to_string()),
                                Lit::Bool(b) => ParamValue::Literal(b.value.to_string()),
                                _ => {
                                    return Err(syn::Error::new_spanned(
                                        lit,
                                        "Unsupported literal type",
                                    ))
                                }
                            },
                            Expr::Macro(macro_expr) => {
                                // Check if it's a tomplate! macro call
                                if let Some(ident) = macro_expr.mac.path.get_ident() {
                                    if ident == "tomplate" {
                                        ParamValue::Macro(macro_expr.clone())
                                    } else {
                                        return Err(syn::Error::new_spanned(
                                            macro_expr,
                                            "Only tomplate! macro calls are supported in parameters",
                                        ))
                                    }
                                } else {
                                    return Err(syn::Error::new_spanned(
                                        macro_expr,
                                        "Expected tomplate! macro call",
                                    ))
                                }
                            },
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    assign.right,
                                    "Expected literal value or tomplate! macro call",
                                ))
                            }
                        };
                        
                        params.push((param_name, param_value));
                    }
                    _ => {
                        return Err(syn::Error::new_spanned(
                            arg,
                            "Expected key = value syntax",
                        ))
                    }
                }
            }
        }
        
        Ok(TomplateInput {
            template_name,
            params,
        })
    }
}

fn process_template(input: TomplateInput) -> syn::Result<proc_macro2::TokenStream> {
    // Get a clone of the cached templates
    let templates = templates::load_templates();
    
    // Try to find the template in registry, or use as inline template
    let (template_string, engine_name) = if let Some(template) = templates.get(&input.template_name) {
        // Found in registry
        (template.template.clone(), template.engine.as_deref().unwrap_or("simple"))
    } else {
        // Not in registry, treat as inline template
        (input.template_name.clone(), "simple")
    };
    
    // Process parameters, expanding any nested macros
    let mut params = std::collections::HashMap::new();
    for (key, value) in input.params {
        let expanded_value = match value {
            ParamValue::Literal(s) => s,
            ParamValue::Macro(macro_expr) => {
                // Recursively expand the nested tomplate! macro
                let tokens = macro_expr.mac.tokens.clone();
                let nested_input = syn::parse2::<TomplateInput>(tokens)?;
                let nested_result = process_template(nested_input)?;
                
                // Extract the string literal from the nested result
                // The nested result is a quote! { "string" }, so we need to extract the string
                let token_string = nested_result.to_string();
                // Remove the quotes from the token string
                token_string.trim_matches('"').to_string()
            }
        };
        params.insert(key, expanded_value);
    }
    
    // Process the template with the appropriate engine
    let processed = engines::process(engine_name, &template_string, &params)
        .map_err(|e| syn::Error::new(proc_macro2::Span::call_site(), e))?;
    
    // Return the processed template as a string literal
    Ok(quote! {
        #processed
    })
}

/// Eagerly expand `tomplate!` and `concat!` macros within a token stream.
///
/// This macro solves the problem where outer macros expect string literals but
/// receive unexpanded macro calls. It walks the token tree and expands inner
/// macros first, allowing seamless integration with other macro systems.
///
/// ## Problem It Solves
///
/// Many macros (like `sqlx::query!`) require string literals as arguments.
/// They cannot accept unexpanded macro calls:
///
/// ```rust,ignore
/// // ❌ This fails - sqlx::query! sees the macro call, not the string
/// sqlx::query!(tomplate!("select_user", id = user_id))
///     .fetch_one(&pool)
///     .await?;
/// ```
///
/// ## Solution
///
/// `tomplate_eager!` pre-expands the inner macros:
///
/// ```rust,ignore
/// // ✅ This works - tomplate! is expanded first, then passed to sqlx
/// tomplate_eager! {
///     sqlx::query!(tomplate!("select_user", id = user_id))
///         .fetch_one(&pool)
///         .await?
/// }
/// ```
///
/// ## Supported Inner Macros
///
/// - `tomplate!` - Expands template macros
/// - `concat!` - Expands string concatenation
///
/// ## Examples
///
/// ### With SQL Query Builders
///
/// ```rust,ignore
/// // Using with sqlx
/// tomplate_eager! {
///     let user = sqlx::query_as!(
///         User,
///         tomplate!("get_user_by_id", 
///             fields = "id, name, email",
///             table = "users"
///         ),
///         user_id
///     )
///     .fetch_one(&pool)
///     .await?;
/// }
/// ```
///
/// ### With String Concatenation
///
/// ```rust,ignore
/// tomplate_eager! {
///     const FULL_QUERY: &str = concat!(
///         tomplate!("select_part"),
///         " UNION ALL ",
///         tomplate!("select_other_part")
///     );
/// }
/// ```
///
/// ### Multiple Expansions
///
/// ```rust,ignore
/// tomplate_eager! {
///     // Multiple statements with macro expansions
///     let query1 = sqlx::query!(tomplate!("query1"))
///         .fetch_all(&pool)
///         .await?;
///     
///     let query2 = sqlx::query!(tomplate!("query2"))
///         .fetch_optional(&pool)
///         .await?;
/// }
/// ```
///
/// ## How It Works
///
/// 1. Recursively walks through the provided token stream
/// 2. Finds any `tomplate!` or `concat!` invocations
/// 3. Evaluates them at compile time
/// 4. Replaces them with their resulting string literals
/// 5. Returns the modified token stream
///
/// All processing happens at compile time with zero runtime overhead.
#[proc_macro]
pub fn tomplate_eager(input: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    
    match eager::process_eager(input) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}