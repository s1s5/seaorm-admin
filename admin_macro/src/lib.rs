extern crate proc_macro;
mod model_admin;
mod parse;
use proc_macro::TokenStream;
use syn::{self, parse_macro_input, Attribute, DeriveInput, Ident};

#[proc_macro_derive(ModelAdmin, attributes(model_admin))]
pub fn model_admin(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, attrs, .. } = parse_macro_input!(input as DeriveInput);
    match expand_model_admin(ident, attrs) {
        Ok(t) => t,
        Err(e) => e.into_compile_error().into(),
    }
}

fn expand_model_admin(ident: Ident, attrs: Vec<Attribute>) -> Result<TokenStream, syn::Error> {
    let expander = model_admin::ModelAdminExpander::new(ident, attrs)?;
    expander.expand().map(|x| x.into())
}
