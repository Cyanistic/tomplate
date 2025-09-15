use crate::parser::{CompositionBlock, Statement, TemplateCall, TemplateSource, ParamValue};
use crate::scope::Scope;
use crate::templates;
use proc_macro2::TokenStream;
use std::collections::HashSet;
use syn::Result;

/// Process a composition block and generate the resulting const declarations
pub fn process_block(block: CompositionBlock) -> Result<TokenStream> {
    // Initialize scope for tracking bindings
    let mut scope = Scope::new();
    
    // Validate the block (no duplicate names, let before const references, etc.)
    validate_block(&block)?;
    
    // Process all statements
    for statement in block.statements {
        match statement {
            Statement::Let { name, value } => {
                // Process the template call and store in local scope
                let resolved = process_template_call(&value, &scope)?;
                scope.add_local(name.to_string(), resolved);
            }
            Statement::Const { attrs, name, value } => {
                // Process the template call and add to exports
                let resolved = process_template_call(&value, &scope)?;
                scope.add_export(attrs, name.to_string(), resolved);
            }
        }
    }
    
    // Generate the output TokenStream with all const declarations
    Ok(scope.generate_output())
}

/// Validate that the block follows the rules
fn validate_block(block: &CompositionBlock) -> Result<()> {
    let mut defined_names = HashSet::new();
    let mut let_names = HashSet::new();
    
    for statement in &block.statements {
        match statement {
            Statement::Let { name, value } => {
                // Check for duplicate names
                if !defined_names.insert(name.to_string()) {
                    return Err(syn::Error::new_spanned(
                        name,
                        format!("Duplicate definition of '{}'", name),
                    ));
                }
                let_names.insert(name.to_string());
                
                // Validate that let only references earlier let bindings
                validate_references(value, &let_names)?;
            }
            Statement::Const { name, value, .. } => {
                // Check for duplicate names
                if !defined_names.insert(name.to_string()) {
                    return Err(syn::Error::new_spanned(
                        name,
                        format!("Duplicate definition of '{}'", name),
                    ));
                }
                
                // Const can reference any let binding (they're all defined by now)
                validate_references(value, &let_names)?;
            }
        }
    }
    
    Ok(())
}

/// Validate that a template call only references defined variables
fn validate_references(call: &TemplateCall, defined: &HashSet<String>) -> Result<()> {
    for (_, value) in &call.params {
        match value {
            ParamValue::Variable(name) => {
                if !defined.contains(name) {
                    return Err(syn::Error::new(
                        proc_macro2::Span::call_site(),
                        format!("Undefined variable: '{}'", name),
                    ));
                }
            }
            ParamValue::Nested(nested) => {
                validate_references(nested, defined)?;
            }
            ParamValue::Literal(_) => {}
        }
    }
    Ok(())
}

/// Process a template call, resolving all variables and nested calls
fn process_template_call(call: &TemplateCall, scope: &Scope) -> Result<String> {
    // First, determine if this is an inline template or a registry lookup
    let (template_string, engine_name) = match &call.source {
        TemplateSource::Name(name) => {
            // Try to find it in the registry
            let templates = templates::load_templates();
            if let Some(template) = templates.get(name) {
                // Found in registry, use its template and engine
                let template_str = template.template.clone();
                let engine = template.engine.as_deref().unwrap_or("simple").to_string();
                (template_str, engine)
            } else {
                // Not in registry, treat as inline template with simple engine
                (name.clone(), "simple".to_string())
            }
        }
    };
    
    // Process parameters, resolving variables and nested calls
    let mut resolved_params = std::collections::HashMap::new();
    for (key, value) in &call.params {
        let resolved_value = match value {
            ParamValue::Literal(s) => s.clone(),
            ParamValue::Variable(name) => {
                scope.get_local(name)
                    .ok_or_else(|| syn::Error::new(
                        proc_macro2::Span::call_site(),
                        format!("Undefined variable: '{}'", name),
                    ))?
                    .clone()
            }
            ParamValue::Nested(nested) => {
                // Recursively process nested template call
                process_template_call(nested, scope)?
            }
        };
        resolved_params.insert(key.clone(), resolved_value);
    }
    
    // Process the template with the resolved parameters
    crate::engines::process(&engine_name, &template_string, &resolved_params)
        .map_err(|e| syn::Error::new(proc_macro2::Span::call_site(), e))
}