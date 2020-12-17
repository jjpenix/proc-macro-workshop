extern crate proc_macro;

use proc_macro::TokenStream;

use syn::{parse_macro_input, Error};
use quote::{ToTokens};
use proc_macro2::Span;

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;

    match sorted_impl(parse_macro_input!(input as syn::Item)) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn sorted_impl(item: syn::Item) -> syn::Result<proc_macro2::TokenStream> {
    if let syn::Item::Enum(_) = item {
        Ok(item.into_token_stream())
    } else {
        Err(Error::new(Span::call_site(), "expected enum or match expression"))
    }

    
}