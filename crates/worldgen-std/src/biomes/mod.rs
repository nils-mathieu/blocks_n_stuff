//! Implementations of the [`Biome`](super::biome::Biome) trait.

mod standard;
pub use standard::*;

mod ocean;
pub use ocean::Ocean;

mod structures;
mod utility;
