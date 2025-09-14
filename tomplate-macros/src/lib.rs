mod engines;
mod templates;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, Expr, Lit, Token};

#[proc_macro]
pub fn tomplate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TomplateInput);
    
    match process_template(input) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

struct TomplateInput {
    template_name: String,
    params: Vec<(String, String)>,
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
                        
                        // Extract parameter value
                        let param_value = match &*assign.right {
                            Expr::Lit(lit) => match &lit.lit {
                                Lit::Str(s) => s.value(),
                                Lit::Int(i) => i.to_string(),
                                Lit::Float(f) => f.to_string(),
                                Lit::Bool(b) => b.value.to_string(),
                                _ => {
                                    return Err(syn::Error::new_spanned(
                                        lit,
                                        "Unsupported literal type",
                                    ))
                                }
                            },
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    assign.right,
                                    "Expected literal value",
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
    // Load templates from the amalgamated file
    let templates = templates::load_templates()
        .map_err(|e| syn::Error::new(proc_macro2::Span::call_site(), e))?;
    
    // Find the requested template
    let template = templates
        .get(&input.template_name)
        .ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Template '{}' not found", input.template_name),
            )
        })?;
    
    // Convert params to HashMap for engine
    let mut params = std::collections::HashMap::new();
    for (key, value) in input.params {
        params.insert(key, value);
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