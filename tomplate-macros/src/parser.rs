use syn::{
    parse::{Parse, ParseStream, Parser},
    punctuated::Punctuated,
    Attribute, Expr, ExprLit, ExprMacro, Ident, Lit, Result, Token,
};

/// A composition block containing let bindings and const exports
pub struct CompositionBlock {
    pub statements: Vec<Statement>,
}

/// A statement within a composition block
pub enum Statement {
    Let {
        name: Ident,
        value: TemplateCall,
    },
    Const {
        attrs: Vec<Attribute>,
        name: Ident,
        value: TemplateCall,
    },
}

/// A call to template!() within a block
pub struct TemplateCall {
    pub source: TemplateSource,
    pub params: Vec<(String, ParamValue)>,
}

/// Source of a template - either a name reference or inline template
pub enum TemplateSource {
    /// Reference to a named template from the registry (or inline if not found)
    Name(String),
    // TODO: Add explicit inline template support later
    // /// Inline template string
    // Inline(String),
}

/// Value of a parameter - literal, variable reference, or nested call
pub enum ParamValue {
    /// String, number, or boolean literal
    Literal(String),
    /// Reference to a let binding
    Variable(String),
    /// Nested template!() call
    Nested(TemplateCall),
}

impl Parse for CompositionBlock {
    fn parse(input: ParseStream) -> Result<Self> {
        let content = input;
        
        let mut statements = Vec::new();
        
        while !content.is_empty() {
            // Parse attributes if any
            let attrs = content.call(Attribute::parse_outer)?;
            
            if content.peek(Token![let]) {
                if !attrs.is_empty() {
                    return Err(syn::Error::new_spanned(
                        &attrs[0],
                        "Attributes are not allowed on let bindings",
                    ));
                }
                statements.push(parse_let_statement(&content)?);
            } else if content.peek(Token![const]) {
                statements.push(parse_const_statement(&content, attrs)?);
            } else {
                return Err(content.error("Expected 'let' or 'const' statement"));
            }
            
            // Consume optional trailing comma
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }
        
        Ok(CompositionBlock { statements })
    }
}

fn parse_let_statement(input: ParseStream) -> Result<Statement> {
    input.parse::<Token![let]>()?;
    let name = input.parse::<Ident>()?;
    input.parse::<Token![=]>()?;
    let value = parse_template_call(input)?;
    input.parse::<Token![;]>()?;
    
    Ok(Statement::Let { name, value })
}

fn parse_const_statement(input: ParseStream, attrs: Vec<Attribute>) -> Result<Statement> {
    input.parse::<Token![const]>()?;
    let name = input.parse::<Ident>()?;
    input.parse::<Token![=]>()?;
    let value = parse_template_call(input)?;
    input.parse::<Token![;]>()?;
    
    Ok(Statement::Const { attrs, name, value })
}

fn parse_template_call(input: ParseStream) -> Result<TemplateCall> {
    // Expect tomplate!(...) 
    let mac: ExprMacro = input.parse()?;
    
    // Verify it's a tomplate! macro
    if !mac.mac.path.is_ident("tomplate") {
        return Err(syn::Error::new_spanned(
            mac,
            "Expected 'tomplate!' macro call",
        ));
    }
    
    // Parse the macro arguments
    parse_template_args(mac.mac.tokens)
}

fn parse_template_args(tokens: proc_macro2::TokenStream) -> Result<TemplateCall> {
    let parser = |input: ParseStream| -> Result<TemplateCall> {
        // First argument is either a template name or inline template
        let first_arg = input.parse::<Expr>()?;
        let source = match first_arg {
            Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) => {
                // This could be either a template name or inline template
                // We'll determine this later based on registry lookup
                TemplateSource::Name(s.value())
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    first_arg,
                    "Template source must be a string literal",
                ));
            }
        };
        
        let mut params = Vec::new();
        
        // Parse optional parameters
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            
            let args = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;
            
            for arg in args {
                match arg {
                    // key = value syntax
                    Expr::Assign(assign) => {
                        let param_name = match &*assign.left {
                            Expr::Path(path) if path.path.segments.len() == 1 => {
                                path.path.segments[0].ident.to_string()
                            }
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    assign.left,
                                    "Parameter name must be a simple identifier",
                                ));
                            }
                        };
                        
                        let param_value = parse_param_value(&*assign.right)?;
                        params.push((param_name, param_value));
                    }
                    _ => {
                        return Err(syn::Error::new_spanned(
                            arg,
                            "Expected 'key = value' syntax",
                        ));
                    }
                }
            }
        }
        
        Ok(TemplateCall { source, params })
    };
    
    parser.parse2(tokens)
}

fn parse_param_value(expr: &Expr) -> Result<ParamValue> {
    match expr {
        // Literal values
        Expr::Lit(lit) => {
            let value = match &lit.lit {
                Lit::Str(s) => s.value(),
                Lit::Int(i) => i.to_string(),
                Lit::Float(f) => f.to_string(),
                Lit::Bool(b) => b.value.to_string(),
                _ => {
                    return Err(syn::Error::new_spanned(
                        lit,
                        "Unsupported literal type",
                    ));
                }
            };
            Ok(ParamValue::Literal(value))
        }
        // Variable reference (simple identifier)
        Expr::Path(path) if path.path.segments.len() == 1 => {
            Ok(ParamValue::Variable(path.path.segments[0].ident.to_string()))
        }
        // Nested tomplate!() call
        Expr::Macro(mac) if mac.mac.path.is_ident("tomplate") => {
            let nested = parse_template_args(mac.mac.tokens.clone())?;
            Ok(ParamValue::Nested(nested))
        }
        _ => Err(syn::Error::new_spanned(
            expr,
            "Parameter value must be a literal, variable reference, or tomplate!() call",
        )),
    }
}
