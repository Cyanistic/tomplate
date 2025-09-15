mod block;
mod eager;
mod engines;
mod parser;
mod scope;
mod templates;

use proc_macro::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, Expr, Lit, Token, ExprMacro};

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
    
    // Find the requested template
    let template = templates
        .get(&input.template_name)
        .ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Template '{}' not found", input.template_name),
            )
        })?;
    
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
    let engine_name = template.engine.as_deref().unwrap_or("simple");
    let processed = engines::process(engine_name, &template.template, &params)
        .map_err(|e| syn::Error::new(proc_macro2::Span::call_site(), e))?;
    
    // Return the processed template as a string literal
    Ok(quote! {
        #processed
    })
}

#[proc_macro]
pub fn tomplate_eager(input: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    
    match eager::process_eager(input) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}