extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

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
        let ty = &f.ty;
        quote! {
            fn #name(&mut self, #name: #ty) -> &mut Self {
                self.#name = Some(#name);
                self
            }
        }
    });

    let input_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name: self.#name.clone().ok_or_else(|| "Not implemented")?
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
