use std::ops::Index;

use bns_core::{Chunk, ChunkPos};
use bns_rng::{FromRng, Rng};

use bytemuck::{Contiguous, Zeroable};
use glam::{IVec2, IVec3};

use crate::biomemap::Climate;
use crate::column_gen::ColumnGen;
use crate::GenCtx;

/// A unique identifier for the biomes generated by the standard world generator.
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug, Contiguous)]
#[repr(u8)]
#[allow(dead_code)] // biomes are generally constructed through transmutation
pub enum BiomeId {
    #[default]
    Void = 0,
    Plains,
    OakForest,
    Desert,
    PineForest,
    Ocean,
}

// SAFETY:
//  0 is the value of BiomeId::Void.
unsafe impl Zeroable for BiomeId {}

impl BiomeId {
    /// The total number of [`BiomeId`] instances.
    pub const COUNT: usize = <Self as Contiguous>::MAX_VALUE as usize + 1;

    /// Returns an iterator over all possible [`BiomeId`]s, excluding [`BiomeId::Void`].
    pub fn iter_all() -> impl Clone + ExactSizeIterator<Item = Self> {
        (1..Self::COUNT as u8).map(|x| unsafe { std::mem::transmute(x) })
    }
}

/// Stores information about a particular biome.
pub struct BiomeInfo {
    /// The allowed continentality range for the biome.
    pub continentality_range: (f32, f32),
    /// The allowed temperature range for the biome.
    pub temperature_range: (f32, f32),
    /// The allowed humidity range for the biome.
    pub humidity_range: (f32, f32),
    /// A weight value used to determine how likely the biome is to spawn compared to the other
    /// biomes.
    pub weight: u32,
    /// The [`Biome`] implementation associated with the biome.
    pub implementation: Box<dyn Send + Sync + Biome>,
}

impl BiomeInfo {
    /// Returns whether the provided [`Climate`] is allowed to spawn in a biome with this
    /// [`BiomeInfo`].
    pub fn is_climate_allowed(&self, climate: &Climate) -> bool {
        self.continentality_range.0 <= climate.continentality
            && climate.continentality <= self.continentality_range.1
            && self.temperature_range.0 <= climate.temperature
            && climate.temperature <= self.temperature_range.1
            && self.humidity_range.0 <= climate.humidity
            && climate.humidity <= self.humidity_range.1
    }
}

/// The interface that's provided to biomes to generate new chunks.
pub trait Biome {
    /// Returns the height value of the biome at the provided position.
    fn height(&self, pos: IVec2) -> f32;

    /// Place the base blocks of the biome in the provided chunk.
    ///
    /// This function is expected to only place blocks that are part of the biome itself
    /// (potentially by checking the biome values in `column`). Additionally, this function
    /// is expected to losely follow the heightmap generated for the column.
    fn geological_stage(&self, pos: ChunkPos, column: &ColumnGen, ctx: &GenCtx, chunk: &mut Chunk);

    /// Prints debug information about itself in the provided buffer.
    fn debug_info(&self, buf: &mut String, pos: IVec3);
}

/// The registry of all available biomes.
pub struct BiomeRegistry {
    biomes: [BiomeInfo; BiomeId::COUNT],
}

impl FromRng for BiomeRegistry {
    fn from_rng(rng: &mut impl Rng) -> Self {
        Self {
            biomes: [
                // Void
                BiomeInfo {
                    continentality_range: (0.0, 0.0),
                    temperature_range: (0.0, 0.0),
                    humidity_range: (0.0, 0.0),
                    weight: 0,
                    implementation: Box::new(crate::biomes::Void),
                },
                // Plains
                BiomeInfo {
                    continentality_range: (0.0, 1.0),
                    temperature_range: (-1.0, 1.0),
                    humidity_range: (-1.0, 1.0),
                    weight: 100,
                    implementation: Box::new(crate::biomes::Plains::from_rng(rng)),
                },
                // OakForest
                BiomeInfo {
                    continentality_range: (0.0, 1.0),
                    temperature_range: (-1.0, 1.0),
                    humidity_range: (-1.0, 1.0),
                    weight: 100,
                    implementation: Box::new(crate::biomes::OakForest::from_rng(rng)),
                },
                // Desert
                BiomeInfo {
                    continentality_range: (0.0, 1.0),
                    temperature_range: (-1.0, 1.0),
                    humidity_range: (-1.0, 1.0),
                    weight: 100,
                    implementation: Box::new(crate::biomes::Desert::from_rng(rng)),
                },
                // PineForest
                BiomeInfo {
                    continentality_range: (0.0, 1.0),
                    temperature_range: (-1.0, 1.0),
                    humidity_range: (-1.0, 1.0),
                    weight: 50,
                    implementation: Box::new(crate::biomes::PineForest::from_rng(rng)),
                },
                // Ocean
                BiomeInfo {
                    continentality_range: (-1.0, 0.0),
                    temperature_range: (-1.0, 1.0),
                    humidity_range: (-1.0, 1.0),
                    weight: 100,
                    implementation: Box::new(crate::biomes::Ocean::from_rng(rng)),
                },
            ],
        }
    }
}

impl Index<BiomeId> for BiomeRegistry {
    type Output = BiomeInfo;

    #[inline(always)]
    fn index(&self, index: BiomeId) -> &Self::Output {
        unsafe { self.biomes.get_unchecked(index as usize) }
    }
}
