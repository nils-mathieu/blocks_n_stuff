use std::path::PathBuf;

use bns_core::{BlockAppearance, BlockId, BlockInstance, Face};
use bns_worldgen_structure_types::{Structure, StructureEdit};
use glam::IVec3;

use proc_macro2::{Span, TokenStream, TokenTree};
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
        quote_spanned!( span => compile_error!(#message) )
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

fn quote_vec3(val: IVec3) -> proc_macro2::TokenStream {
    let [x, y, z] = val.into();
    quote! { ::glam::IVec3::new(#x, #y, #z) }
}

fn quote_block_id(id: BlockId) -> TokenStream {
    let variant = create_ident(&format!("{:?}", id));

    quote! {
        ::bns_worldgen_structure::__private_macro::bns_core::BlockId::#variant
    }
}

fn quote_face(face: Face) -> TokenStream {
    let variant = create_ident(&format!("{:?}", face));

    quote! {
        ::bns_worldgen_structure::__private_macro::bns_core::Face::#variant
    }
}

fn quote_block_appearance(b: &BlockInstance) -> TokenStream {
    let core = quote! { ::bns_worldgen_structure::__private_macro::bns_core };

    match b.id().info().appearance {
        BlockAppearance::Flat(_) => {
            let face = quote_face(unsafe { b.appearance().flat });
            quote! { #core ::AppearanceMetadata { flat: #face } }
        }
        _ => quote! { #core ::AppearanceMetadata { no_metadata: () } },
    }
}

fn quote_block_instance(b: &BlockInstance) -> TokenStream {
    let core = quote! { ::bns_worldgen_structure::__private_macro::bns_core };

    let block_id = quote_block_id(b.id());
    let appearance = quote_block_appearance(b);

    quote! {
        unsafe {
            #core ::BlockInstance::new_unchecked(
                #block_id,
                #appearance,
            )
        }
    }
}

fn quote_structure_edit(e: &StructureEdit) -> TokenStream {
    let position = quote_vec3(e.position);
    let block = quote_block_instance(&e.block);

    quote! {
        ::bns_worldgen_structure::StructureEdit {
            position: #position,
            block: #block,
        }
    }
}

fn quote_structure(s: &Structure) -> TokenStream {
    let edits = s.edits.iter().map(quote_structure_edit);
    let min = quote_vec3(s.min);
    let max = quote_vec3(s.max);

    quote! {
        ::bns_worldgen_structure::Structure {
            edits: ::std::borrow::Cow::Borrowed(&[ #(#edits,)* ]),
            min: #min,
            max: #max,
        }
    }
}

fn include_structure_impl(input: TokenStream) -> Result<TokenStream, Error> {
    let path = read_input(input)?;
    let path = get_base_dir().join(path);

    let file = std::fs::read_to_string(&path).map_err(|err| Error {
        message: format!("failed to open '{}': {}", path.display(), err),
        span: Span::call_site(),
    })?;

    let structure: Structure = ron::de::from_str(&file).map_err(|err| Error {
        message: format!("failed to parse '{}': {}", path.display(), err),
        span: Span::call_site(),
    })?;

    let file_dependency = path.to_str().unwrap();
    let structure = quote_structure(&structure);
    Ok(quote! {
        {
            let _ = ::core::include_bytes!(#file_dependency);
            #structure
        }
    })
}

/// Includes a structure by parsing it from a `.ron` file.
#[proc_macro]
pub fn include_structure(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match include_structure_impl(input.into()) {
        Ok(stream) => stream.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
