extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    dbg!(input.clone());

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
        let ty = match get_inner_ty(&f.ty) {
            Some(s) => s,
            None => f.ty.clone(),
        };

        quote! {
            #name: std::option::Option<#ty>
        }
    });

    let builder_empty_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name: None
        }
    });

    let builder_methods = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = match get_inner_ty(&f.ty) {
            Some(s) => s,
            None => f.ty.clone(),
        };

        let each_attr = has_each_attr(&ty);
        if each_attr.is_some() {
            // TODO: Finish, need to extract inner type from vec as well as option
        }

        quote! {
            fn #name(&mut self, #name: #ty) -> &mut Self {
                self.#name = Some(#name);
                self
            }
        }
    });

    let input_fields = fields.iter().map(|f| {
        let name = &f.ident;

        if get_inner_ty(&f.ty).is_none() {
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

// Check if type is Option<T>, if it is, return T
fn get_inner_ty(ty: &syn::Type) -> Option<syn::Type> {
    let segments = if let syn::Type::Path(syn::TypePath {
        path: syn::Path { ref segments, .. },
        ..
    }) = ty
    {
        segments
    } else {
        panic!("could not fetch segments!");
    };

    if segments.first().unwrap().ident.ne("Option") {
        return None;
    }

    match segments.first().unwrap().clone().arguments {
        syn::PathArguments::AngleBracketed(s) => {
            match s.args.first().unwrap() {
                syn::GenericArgument::Type(t) => return Some(t.clone()),
                _ => return None,
            }
        }
        _ => return None,
    }
}
