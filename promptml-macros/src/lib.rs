use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, Ident, LitStr, Token};

fn extract_vars(template: &str) -> Result<Vec<String>, String> {
    let mut vars = Vec::new();
    let mut rest = template;

    while !rest.is_empty() {
        if rest.starts_with("{{") {
            let close = rest[2..]
                .find("}}")
                .ok_or_else(|| "unclosed block directive `{{`".to_string())?;
            rest = &rest[2 + close + 2..];
        } else if rest.starts_with('{') {
            let close = rest[1..]
                .find('}')
                .ok_or_else(|| "unclosed variable brace `{`".to_string())?;
            let name = rest[1..1 + close].trim();
            if name.is_empty() {
                return Err("empty variable name `{}`".to_string());
            }
            let owned = name.to_string();
            if !vars.contains(&owned) {
                vars.push(owned);
            }
            rest = &rest[1 + close + 1..];
        } else {
            let next = rest.find('{').unwrap_or(rest.len());
            rest = &rest[next..];
        }
    }

    Ok(vars)
}

/// Validate a prompt template at compile time and produce a [`PromptTemplate`].
///
/// Accepts a single string literal.  Every `{variable}` name found in the
/// string is registered as a required variable.  Syntax errors (unclosed
/// braces, empty variable names) are reported as compile-time errors.
///
/// # Example
/// ```rust,ignore
/// let tmpl = prompt!("Hello, {name}!");
/// let out  = tmpl.render().set("name", "World").build().unwrap();
/// ```
#[proc_macro]
pub fn prompt(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as LitStr);
    let template_str = lit.value();

    match extract_vars(&template_str) {
        Ok(vars) => {
            let var_lits: Vec<_> = vars.iter().map(String::as_str).collect();
            quote! {
                ::promptml::PromptTemplate::new_validated(#template_str, &[#(#var_lits),*])
            }
            .into()
        }
        Err(msg) => syn::Error::new(lit.span(), msg).to_compile_error().into(),
    }
}

struct ChatPromptInput {
    system: Option<LitStr>,
    user: LitStr,
}

impl Parse for ChatPromptInput {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let mut system: Option<LitStr> = None;
        let mut user: Option<LitStr> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            let _colon: Token![:] = input.parse()?;
            let value: LitStr = input.parse()?;

            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            }

            match key.to_string().as_str() {
                "system" => system = Some(value),
                "user" => user = Some(value),
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("unknown key `{other}`: expected `system` or `user`"),
                    ))
                }
            }
        }

        let user =
            user.ok_or_else(|| syn::Error::new(Span::call_site(), "missing required key `user`"))?;

        Ok(Self { system, user })
    }
}

/// Validate a two-part (system + user) chat template at compile time.
///
/// Both strings are parsed for `{variable}` syntax errors at compile time.
/// Expands to a [`PromptTemplate`] built with `new_with_system`.
///
/// # Example
/// ```rust,ignore
/// let tmpl = chat_prompt! {
///     system: "You are {persona}.",
///     user:   "Answer this: {question}",
/// };
/// ```
#[proc_macro]
pub fn chat_prompt(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as ChatPromptInput);

    let user_str = parsed.user.value();
    if let Err(msg) = extract_vars(&user_str) {
        return syn::Error::new(parsed.user.span(), msg)
            .to_compile_error()
            .into();
    }

    let user_lit = &parsed.user;

    if let Some(sys_lit) = &parsed.system {
        let sys_str = sys_lit.value();
        if let Err(msg) = extract_vars(&sys_str) {
            return syn::Error::new(sys_lit.span(), msg)
                .to_compile_error()
                .into();
        }
        quote! {
            ::promptml::PromptTemplate::new_with_system(#user_lit, Some(#sys_lit))
                .expect("chat_prompt! validated template at compile time")
        }
        .into()
    } else {
        quote! {
            ::promptml::PromptTemplate::new_with_system(#user_lit, None)
                .expect("chat_prompt! validated template at compile time")
        }
        .into()
    }
}
