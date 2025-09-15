use proc_macro2::{TokenStream, TokenTree, Group, Ident};
use quote::quote;
use syn::parse::Parser;

/// Process a TokenStream, eagerly evaluating tomplate! and concat! macros
pub fn process_eager(input: TokenStream) -> syn::Result<TokenStream> {
    let mut output = TokenStream::new();
    let mut tokens = input.into_iter().peekable();
    
    while let Some(token) = tokens.next() {
        match token {
            // Check for macro invocations
            TokenTree::Ident(ident) if is_evaluatable_macro(&ident) => {
                // Peek at the next token to see if it's a macro invocation
                if let Some(TokenTree::Punct(punct)) = tokens.peek() {
                    if punct.as_char() == '!' {
                        // Consume the '!'
                        tokens.next();
                        
                        // Next should be the macro arguments in a Group
                        if let Some(TokenTree::Group(group)) = tokens.next() {
                            // Process the macro invocation
                            let result = evaluate_macro(&ident, group)?;
                            output.extend(result);
                        } else {
                            // Not a macro invocation, restore tokens
                            output.extend(quote! { #ident ! });
                        }
                    } else {
                        // Not a macro invocation
                        output.extend(quote! { #ident });
                    }
                } else {
                    // Not a macro invocation
                    output.extend(quote! { #ident });
                }
            }
            // Recursively process groups
            TokenTree::Group(group) => {
                let processed = process_eager(group.stream())?;
                let new_group = Group::new(group.delimiter(), processed);
                output.extend(std::iter::once(TokenTree::Group(new_group)));
            }
            // Pass through other tokens unchanged
            other => {
                output.extend(std::iter::once(other));
            }
        }
    }
    
    Ok(output)
}

/// Check if an identifier is a macro we want to evaluate
fn is_evaluatable_macro(ident: &Ident) -> bool {
    let name = ident.to_string();
    name == "tomplate" || name == "concat"
}

/// Evaluate a macro invocation and return the result
fn evaluate_macro(name: &Ident, args: Group) -> syn::Result<TokenStream> {
    let macro_name = name.to_string();
    
    match macro_name.as_str() {
        "tomplate" => evaluate_tomplate(args.stream()),
        "concat" => evaluate_concat(args.stream()),
        _ => {
            // Should not happen due to is_evaluatable_macro check
            Ok(quote! { #name ! #args })
        }
    }
}

/// Evaluate a tomplate! macro call
fn evaluate_tomplate(input: TokenStream) -> syn::Result<TokenStream> {
    // Parse the tomplate input
    let tomplate_input = syn::parse2::<crate::TomplateInput>(input)?;
    
    // Process the template using the existing logic
    let result = crate::process_template(tomplate_input)?;
    
    // The result is already a string literal token
    Ok(result)
}

/// Evaluate a concat! macro call
fn evaluate_concat(input: TokenStream) -> syn::Result<TokenStream> {
    // First, recursively process the input to expand any nested tomplate! calls
    let processed_input = process_eager(input)?;
    
    let parser = |input: syn::parse::ParseStream| -> syn::Result<Vec<String>> {
        let mut parts = Vec::new();
        
        while !input.is_empty() {
            // Try to parse a string literal
            if let Ok(lit) = input.parse::<syn::LitStr>() {
                parts.push(lit.value());
            } 
            // Try to parse other literals and convert to string
            else if let Ok(lit) = input.parse::<syn::LitInt>() {
                parts.push(lit.to_string());
            }
            else if let Ok(lit) = input.parse::<syn::LitFloat>() {
                parts.push(lit.to_string());
            }
            else if let Ok(lit) = input.parse::<syn::LitBool>() {
                parts.push(lit.value.to_string());
            }
            else {
                // If we can't parse as a literal, skip the token
                // This handles cases where macros have been expanded
                input.step(|cursor| {
                    if let Some((_, rest)) = cursor.token_tree() {
                        Ok(((), rest))
                    } else {
                        Err(cursor.error("Unexpected end of input"))
                    }
                })?;
            }
            
            // Skip optional comma
            if input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>()?;
            }
        }
        
        Ok(parts)
    };
    
    let parts = parser.parse2(processed_input)?;
    let concatenated = parts.join("");
    
    Ok(quote! { #concatenated })
}