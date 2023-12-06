#[doc(hidden)]
pub mod __private_macro {
    pub use bns_core;
}

#[cfg(feature = "macros")]
pub use bns_worldgen_structure_macros::*;

pub use bns_worldgen_structure_types::*;
