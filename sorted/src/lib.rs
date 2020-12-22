extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro2::Span;
use quote::{quote, ToTokens};
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

struct MatchArmSort {
    errors: Vec<syn::Error>,
}

// test test test
impl VisitMut for MatchArmSort {
    fn visit_expr_match_mut(&mut self, match_expr: &mut syn::ExprMatch) {
        if match_expr
            .attrs
            .iter()
            .find(|a| a.path.get_ident().unwrap() == "sorted")
            .is_none()
        {
            return;
        }

        match_expr
            .attrs
            .retain(|a| a.path.get_ident().unwrap() != "sorted");

        let mut seen_arms = Vec::new();
        let mut wild_seen = None;
        for arm in &match_expr.arms {
            if let Some(path) = get_pat_path(&arm.pat) {
                let path_str = get_path_as_string(&path);
                if seen_arms.is_empty() {
                    seen_arms.push(path_str);
                    continue;
                }

                if let Some(pat) = wild_seen {
                    self.errors.push(Error::new_spanned(
                        pat,
                        "wild pattern should come last",
                    ))
                }

                if path_str < *seen_arms.last().unwrap() {
                    let insert_pos = seen_arms.binary_search(&path_str).unwrap_err();
                    self.errors.push(Error::new_spanned(
                        path,
                        format!("{} should sort before {}", path_str, seen_arms[insert_pos]),
                    ));
                    return;
                } else {
                    seen_arms.push(path_str);
                }
            } else if let syn::Pat::Wild(_) = &arm.pat {
                wild_seen = Some(&arm.pat);
                continue;
            } else {
                self.errors
                    .push(Error::new_spanned(&arm.pat, "unsupported by #[sorted]"));
                return;
            }
        }
    }
}

#[proc_macro_attribute]
pub fn check(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let mut input = parse_macro_input!(input as syn::ItemFn);
    let mut sorter = MatchArmSort { errors: Vec::new() };
    sorter.visit_item_fn_mut(&mut input);

    let mut mutated_input_tokens = input.into_token_stream();
    for err in &sorter.errors {
        mutated_input_tokens.extend(err.to_compile_error());
    }

    mutated_input_tokens.into()
}

fn get_pat_path(pat: &syn::Pat) -> Option<syn::Path> {
    match pat {
        syn::Pat::TupleStruct(s) => Some(s.path.clone()),
        syn::Pat::Ident(i) => Some(i.ident.clone().into()),
        _ => None,
    }
}

fn get_path_as_string(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|s| format!("{}", quote! {#s}))
        .collect::<Vec<_>>()
        .join("::")
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

    let mut seen_variants = Vec::new();
    for var in en.variants.iter() {
        if seen_variants.is_empty() {
            seen_variants.push(&var.ident);
            continue;
        }

        if &var.ident < *seen_variants.last().unwrap() {
            let insert_pos = seen_variants.binary_search(&&var.ident).unwrap_err();
            return Some(Error::new(
                var.span(),
                format!(
                    "{} should sort before {}",
                    &var.ident, seen_variants[insert_pos]
                ),
            ));
        } else {
            seen_variants.push(&var.ident);
        }
    }

    None
}
