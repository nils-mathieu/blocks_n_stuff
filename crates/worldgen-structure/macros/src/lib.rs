use std::path::PathBuf;

use bns_core::BlockAppearance;
use bns_worldgen_structure_types::Structure;

use proc_macro::{Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned};

fn get_base_dir() -> PathBuf {
    let base_dir = std::env::var("CARGO_MANIFEST_DIR").expect("can only be used in build time");
    PathBuf::from(base_dir)
}

struct Error {
    message: String,
    span: Span,
}

impl Error {
    fn to_compile_error(&self) -> TokenStream {
        let span = self.span;
        let message = self.message.clone();
        quote_spanned!( span.into() => compile_error!(#message) ).into()
    }
}

fn read_input(input: TokenStream) -> Result<String, Error> {
    let mut stream = input.into_iter();

    let first_token = stream.next().ok_or_else(|| Error {
        message: "expected a string literal, found nothing".to_owned(),
        span: Span::call_site(),
    })?;

    let literal = match first_token {
        TokenTree::Literal(lit) => lit,
        _ => {
            return Err(Error {
                message: format!("expected a string literal, found '{}'", first_token),
                span: first_token.span(),
            });
        }
    };

    if let Some(token) = stream.next() {
        return Err(Error {
            message: format!("expected a string literal, found '{}'", literal),
            span: token.span(),
        });
    }

    let literal_to_str = literal.to_string();

    if literal_to_str.starts_with('"') && literal_to_str.ends_with('"') {
        Ok(literal_to_str[1..literal_to_str.len() - 1].to_owned())
    } else {
        Err(Error {
            message: format!("expected a string literal, found '{}'", literal),
            span: literal.span(),
        })
    }
}

fn create_ident(name: &str) -> proc_macro2::Ident {
    proc_macro2::Ident::new(name, proc_macro2::Span::call_site())
}

fn include_structure_impl(input: TokenStream) -> Result<TokenStream, Error> {
    let path = read_input(input)?;
    let path = get_base_dir().join(path);

    let file = std::fs::File::open(&path).map_err(|err| Error {
        message: format!("failed to open '{}': {}", path.display(), err),
        span: Span::call_site(),
    })?;

    let structure: Structure = ron::de::from_reader(file).map_err(|err| Error {
        message: format!("failed to parse '{}': {}", path.display(), err),
        span: Span::call_site(),
    })?;

    let core = quote! { ::bns_worldgen_structure::__private_macro::core };

    let name = structure.name.map_or(
        quote! { ::core::option::Option::None },
        |name| quote! { ::core::option::Option::Some(#name) },
    );
    let x = structure.bounds.x;
    let y = structure.bounds.y;
    let z = structure.bounds.z;
    let bounds = quote! { ::glam::IVec3::new(#x, #y, #z) };
    let edits = structure.edits.iter().map(|e| {
        let x = e.position.x;
        let y = e.position.y;
        let z = e.position.z;
        let id = create_ident(&format!("{:?}", e.block.id()));
        let appearance = e.block.appearance();
        let appearance = match e.block.id().info().appearance {
            BlockAppearance::Flat(_) => {
                let face = unsafe { appearance.flat };
                let face = create_ident(&format!("{:?}", face));
                quote! { #core ::AppearanceMetadata { flat: #core ::Face:: #face } }
            }
            _ => quote! { #core ::AppearanceMetadata { no_metadata: () } },
        };

        let block = quote! {
            unsafe {
                #core ::InstanciatedBlock::new_unchecked(
                    #core ::BlockId:: #id,
                    #appearance,
                )
            }
        };

        quote! {
            ::bns_worldgen_structure::StructureEdit {
                position: ::glam::IVec3::new(#x, #y, #z),
                block: #block,
            }
        }
    });

    let path = proc_macro2::Literal::string(path.to_str().unwrap());
    Ok(quote! {
        {
            let _ = ::core::include_bytes!(#path);
            ::bns_worldgen_structure::Structure {
                name: #name,
                bounds: #bounds,
                edits: &[ #(#edits,)* ],
            }
        }
    }
    .into())
}

/// Includes a structure by parsing it from a `.ron` file.
#[proc_macro]
pub fn include_structure(input: TokenStream) -> TokenStream {
    match include_structure_impl(input) {
        Ok(stream) => stream,
        Err(err) => err.to_compile_error(),
    }
}
