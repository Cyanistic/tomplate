use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::Attribute;

/// Scope for tracking let bindings and const exports in a composition block
pub struct Scope {
    /// Local let bindings - only visible within the block
    locals: HashMap<String, String>,
    /// Exported const declarations - visible outside the block
    exports: Vec<Export>,
}

/// An exported const declaration
struct Export {
    /// Attributes like #[cfg(...)]
    attrs: Vec<Attribute>,
    /// Name of the const
    name: String,
    /// Resolved template value
    value: String,
}

impl Scope {
    /// Create a new empty scope
    pub fn new() -> Self {
        Scope {
            locals: HashMap::new(),
            exports: Vec::new(),
        }
    }
    
    /// Add a local let binding
    pub fn add_local(&mut self, name: String, value: String) {
        self.locals.insert(name, value);
    }
    
    /// Get a local binding by name
    pub fn get_local(&self, name: &str) -> Option<&String> {
        self.locals.get(name)
    }
    
    /// Add an exported const declaration
    pub fn add_export(&mut self, attrs: Vec<Attribute>, name: String, value: String) {
        self.exports.push(Export { attrs, name, value });
    }
    
    /// Generate the output TokenStream with all const declarations
    pub fn generate_output(&self) -> TokenStream {
        let mut output = TokenStream::new();
        
        for export in &self.exports {
            let name = syn::Ident::new(&export.name, proc_macro2::Span::call_site());
            let value = &export.value;
            let attrs = &export.attrs;
            
            // Generate: #[attrs] const NAME: &str = "value";
            output.extend(quote! {
                #(#attrs)*
                const #name: &str = #value;
            });
        }
        
        output
    }
}