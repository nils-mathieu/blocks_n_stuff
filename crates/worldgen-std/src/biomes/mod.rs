//! Implementations of the [`Biome`](super::biome::Biome) trait.

mod desert;
mod oak_forest;
mod ocean;
mod pine_forest;
mod plains;

pub use desert::Desert;
pub use oak_forest::OakForest;
pub use ocean::Ocean;
pub use pine_forest::PineForest;
pub use plains::Plains;

mod structures;
mod utility;
