use proc_macro::TokenStream;
use quote::quote;

/// Derives the [`FromRng`] trait.
#[proc_macro_derive(FromRng)]
pub fn derive_from_rng(input: TokenStream) -> TokenStream {
    let s: syn::Item = syn::parse(input).expect("failed to parse the token stream");

    let ret = match s {
        syn::Item::Struct(s) => {
            let name = s.ident;
            let field_init = s.fields.iter().map(|field| {
                let ident = field.ident.as_ref().unwrap();

                quote! {
                    #ident : bns_rng::FromRng::from_rng(rng),
                }
            });

            quote! {
                impl bns_rng::FromRng for #name {
                    fn from_rng(rng: &mut impl bns_rng::Rng) -> Self {
                        Self {
                            #(#field_init)*
                        }
                    }
                }
            }
        }
        _ => panic!("unsupported item type"),
    };

    ret.into()
}
