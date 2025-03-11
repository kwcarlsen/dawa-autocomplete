use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(SizeOf)]
pub fn derive_size_of(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = match input.data {
        syn::Data::Struct(ref data_struct) => {
            let fields = match data_struct.fields {
                syn::Fields::Named(ref fields_named) => &fields_named.named,
                syn::Fields::Unnamed(ref fields_unnamed) => &fields_unnamed.unnamed,
                syn::Fields::Unit => panic!("Unit structs are not supported"),
            };

            let field_sizes = fields.iter().map(|field| {
                let field_name = &field.ident;
                quote! {
                    + self.#field_name.size_of()
                }
            });

            quote! {
                impl crate::size_of::SizeOf for #name {
                    fn size_of(&self) -> usize {
                        size_of::<#name>()
                        #(#field_sizes)*
                    }
                }
            }
        }
        _ => panic!("SizeOf can only be derived for structs"),
    };

    TokenStream::from(expanded)
}
