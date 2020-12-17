extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro2::Span;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::visit_mut::VisitMut;
use syn::{parse_macro_input, Error};

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;

    match sorted_impl(&parse_macro_input!(input as syn::Item)) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

struct MatchArmSort;

impl VisitMut for MatchArmSort {
    fn visit_expr_match_mut(&mut self, match_expr: &mut syn::ExprMatch) {
        if match_expr
            .attrs
            .iter()
            .find(|a| a.path.get_ident().unwrap() == "sorted")
            .is_some()
        {
            for arm in match_expr.arms {
                
            }
        }
    }
}

#[proc_macro_attribute]
pub fn check(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let mut input = parse_macro_input!(input as syn::ItemFn);
    MatchArmSort.visit_item_fn_mut(&mut input);
    input.into_token_stream().into()
}

fn sorted_impl(item: &syn::Item) -> syn::Result<proc_macro2::TokenStream> {
    if let syn::Item::Enum(en) = item {
        match enum_unsorted(en) {
            Some(err) => Err(err),
            None => Ok(item.into_token_stream()),
        }
    } else {
        Err(Error::new(
            Span::call_site(),
            "expected enum or match expression",
        ))
    }
}

fn enum_unsorted(en: &syn::ItemEnum) -> Option<syn::Error> {
    if en.variants.is_empty() {
        return None;
    }

    let mut prev = en.variants.first().unwrap();
    for var in en.variants.iter().skip(1) {
        if var.ident < prev.ident {
            // Have to run through a second time to find the insert position.
            // Would be nice to use something like lower_bound instead
            for insert_pos in en.variants.iter() {
                if var.ident < insert_pos.ident {
                    return Some(Error::new(
                        var.span(),
                        format!("{} should sort before {}", var.ident, insert_pos.ident),
                    ));
                }
            }
        }
        prev = var;
    }

    None
}
