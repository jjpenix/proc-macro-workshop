extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // dbg!(input.clone());

    // fetch name, make name + Builder version
    let input_ident = &input.ident;
    let builder_name = format!("{}Builder", input_ident);
    let builder_ident = syn::Ident::new(&builder_name, input_ident.span());

    // get list of fields in the input struct
    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = input.data
    {
        named
    } else {
        panic!("Non-struct found!");
    };

    let builder_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        if get_inner_ty(ty, "Option").is_some() || has_each_attr(f).is_some() {
            quote! {
                #name: #ty
            }
        } else {
            quote! {
                #name: std::option::Option<#ty>
            }
        }
    });

    let builder_empty_fields = fields.iter().map(|f| {
        let name = &f.ident;
        if has_each_attr(f).is_some() {
            quote! {
                #name: std::vec::Vec::new()
            }
        } else {
            quote! {
                #name: std::option::Option::None
            }
        }
    });

    let builder_methods = fields.iter().map(|f| {
        let name = &f.ident.clone().unwrap();
        let ty = match get_inner_ty(&f.ty, "Option") {
            Some(s) => s,
            None => f.ty.clone(),
        };

        let each_attr = has_each_attr(&f);
        match each_attr {
            Some(s) => {
                let s = s.trim_end_matches("\"");
                // wrong span, but w/e
                let s = syn::Ident::new(s, name.span());
                let vec_ty = get_inner_ty(&f.ty, "Vec");
                if s.eq(&name.to_string()) {
                    quote! {
                        fn #s(&mut self, #s: #vec_ty) -> &mut Self {
                            self.#name.push(#s);
                            self
                        }
                    }
                } else {
                    quote! {
                        fn #s(&mut self, #s: #vec_ty) -> &mut Self {
                            self.#name.push(#s);
                            self
                        }
                        fn #name(&mut self, #name: #ty) -> &mut Self {
                            self.#name = #name;
                            self
                        }
                    }
                }
            }
            _ => {
                quote! {
                    fn #name(&mut self, #name: #ty) -> &mut Self {
                        self.#name = Some(#name);
                        self
                    }
                }
            }
        }
    });

    let input_fields = fields.iter().map(|f| {
        let name = &f.ident;

        if get_inner_ty(&f.ty, "Option").is_none() && has_each_attr(f).is_none() {
            quote! {
                #name: self.#name.clone().ok_or_else(|| "Not implemented")?
            }
        } else {
            quote! {
                #name: self.#name.clone()
            }
        }
    });

    let expanded = quote! {
        pub struct #builder_ident {
            #(#builder_fields,)*
        }

        impl #builder_ident {
            #(#builder_methods)*

            pub fn build(&mut self) -> Result<#input_ident, Box<dyn std::error::Error>> {
                Ok(Command {
                    #(#input_fields,)*
                })
            }
        }

        impl #input_ident {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    #(#builder_empty_fields,)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

// Check if type is outer_ty<T>, if it is, return T
fn get_inner_ty(ty: &syn::Type, outer_ty: &str) -> Option<syn::Type> {
    let segments = if let syn::Type::Path(syn::TypePath {
        path: syn::Path { ref segments, .. },
        ..
    }) = ty
    {
        segments
    } else {
        panic!("could not fetch segments!");
    };

    if segments.first().unwrap().ident.ne(outer_ty) {
        return None;
    }

    match segments.first().unwrap().clone().arguments {
        syn::PathArguments::AngleBracketed(s) => match s.args.first().unwrap() {
            syn::GenericArgument::Type(t) => return Some(t.clone()),
            _ => return None,
        },
        _ => return None,
    }
}

fn has_each_attr(field: &syn::Field) -> Option<std::string::String> {
    if field.attrs.is_empty() {
        return None;
    }

    for attr in &field.attrs {
        let meta = match attr.parse_meta() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let meta_list = match meta {
            syn::Meta::List(ml) => ml,
            _ => continue,
        };

        if meta_list.path.segments.first().unwrap().ident.ne("builder") {
            continue;
        }

        let x = meta_list.nested.first().unwrap();

        match x {
            syn::NestedMeta::Meta(syn::Meta::NameValue(m)) => {
                if m.path.segments.first().unwrap().ident.ne("each") {
                    continue;
                }

                if let syn::Lit::Str(ref s) = m.lit {
                    return Some(s.value());
                } else {
                    continue;
                }
            }
            _ => continue,
        }
    }

    None
}
