use bns_core::{BlockId, BlockInstance, Chunk, ChunkPos, LocalPos};
use bns_rng::noises::{Mixer, SuperSimplex2};
use bns_rng::{FromRng, Noise, Rng};
use bns_worldgen_structure::Structure;
use glam::{IVec2, IVec3};

use crate::biome::BiomeId;
use crate::chunk_gen::{PendingStructure, StructureTransformations};
use crate::column_gen::{ColumnGen, ColumnPos};
use crate::GenCtx;

/// A prop that can be spawned in a biome.
///
/// Props are spawned on top of the surface, but must fit on a single block.
struct Props {
    block: BlockInstance,
    probability: u64,
    noise: Mixer<2>,
}

/// A set of stuctures that can spawn in a biome.
struct StructureSet {
    set: &'static [&'static Structure<'static>],
    probability: u64,
    noise: Mixer<2>,
    value_noise: Mixer<2>,
    transform_noise: Mixer<2>,
}

/// A noise
struct NoiseEntry {
    scale: f32,
    frequency: f32,
    noise: SuperSimplex2,
}

/// Defines methods to create a [`StandardBiome`] instance.
pub struct StandardBiomeBuilder<'a, R> {
    inner: StandardBiome,
    rng: &'a mut R,
}

impl<'a, R: Rng> StandardBiomeBuilder<'a, R> {
    /// Creates a new [`StandardBiomeBuilder`] instance.
    pub fn new(rng: &'a mut R, filter: BiomeId) -> Self {
        Self {
            inner: StandardBiome {
                filter,
                surface: BlockId::Grass.into(),
                dirt: BlockId::Dirt.into(),
                underground: BlockId::Stone.into(),
                min_dirt_depth: 2,
                max_dirt_depth: 5,
                base_height: 0.0,
                height_noises: Vec::new(),
                dirt_noise: SuperSimplex2::from_rng(rng),
                props: Vec::new(),
                structures: Vec::new(),
            },
            rng,
        }
    }

    /// Sets the surface block of the biome.
    pub fn set_surface_block(&mut self, block: BlockInstance) {
        self.inner.surface = block;
    }

    /// Sets dirt block of the biome.
    ///
    /// Note that the min and max depth can be negative, in which case the dirt will replace the
    /// surface block, but never above the world height.
    pub fn set_dirt(&mut self, block: BlockInstance, min_depth: i32, max_depth: i32) {
        self.inner.dirt = block;
        self.inner.min_dirt_depth = min_depth;
        self.inner.max_dirt_depth = max_depth;
    }

    /// Sets the underground block.
    pub fn set_underground(&mut self, block: BlockInstance) {
        self.inner.underground = block;
    }

    /// Sets the base height of the biome.
    pub fn set_base_height(&mut self, base_height: f32) {
        self.inner.base_height = base_height;
    }

    /// Add a height noise.
    pub fn add_height_noise(&mut self, scale: f32, freq: f32) {
        self.inner.height_noises.push(NoiseEntry {
            scale,
            frequency: freq,
            noise: SuperSimplex2::from_rng(self.rng),
        });
    }

    /// Add a prop to the biome.
    pub fn add_prop(&mut self, block: BlockInstance, probability: u64) {
        self.inner.props.push(Props {
            block,
            probability,
            noise: Mixer::from_rng(self.rng),
        });
    }

    /// Add a structure to the biome.
    pub fn add_structure(&mut self, set: &'static [&'static Structure<'static>], probability: u64) {
        self.inner.structures.push(StructureSet {
            set,
            probability,
            noise: Mixer::from_rng(self.rng),
            value_noise: Mixer::from_rng(self.rng),
            transform_noise: Mixer::from_rng(self.rng),
        });
    }

    /// Turns this builder into a [`StandardBiome`].
    #[inline]
    pub fn build(self) -> StandardBiome {
        self.inner
    }
}

/// Contains the state of a standard biome with standard generation.
pub struct StandardBiome {
    filter: BiomeId,
    surface: BlockInstance,
    dirt: BlockInstance,
    underground: BlockInstance,
    min_dirt_depth: i32,
    max_dirt_depth: i32,
    base_height: f32,
    height_noises: Vec<NoiseEntry>,
    dirt_noise: SuperSimplex2,
    props: Vec<Props>,
    structures: Vec<StructureSet>,
}

impl StandardBiome {
    /// Computes the biome's requested height for the given column in world-space.
    pub fn height(&self, pos: IVec2) -> f32 {
        let mut ret = self.base_height;

        for noise in &self.height_noises {
            ret += noise.noise.sample([
                pos.x as f32 * noise.frequency,
                pos.y as f32 * noise.frequency,
            ]) * noise.scale;
        }

        ret
    }

    /// Builds the biome in the given chunk.
    pub fn build(&self, pos: ChunkPos, column: &ColumnGen, ctx: &GenCtx, chunk: &mut Chunk) {
        let biome_ids = &column.biome_stage(ctx).ids;

        for local_pos in LocalPos::iter_all() {
            if biome_ids[local_pos.into()] != self.filter {
                continue;
            }

            let world_pos = pos.origin() + local_pos.to_ivec3();
            let height = column.height_stage(ctx)[local_pos.into()];

            let dirt_depth = bns_rng::utility::floor_i32(
                self.dirt_noise
                    .sample([world_pos.x as f32 * 0.125, world_pos.z as f32 * 0.125])
                    * (self.max_dirt_depth - self.min_dirt_depth) as f32
                    + self.min_dirt_depth as f32,
            );

            if world_pos.y <= height {
                if world_pos.y < height - dirt_depth {
                    chunk.set_block(local_pos, self.underground.clone());
                } else if world_pos.y <= 2 {
                    chunk.set_block(local_pos, BlockId::Sand.into());
                } else if world_pos.y < height {
                    chunk.set_block(local_pos, self.dirt.clone());
                } else {
                    chunk.set_block(local_pos, self.surface.clone());
                }
            } else if world_pos.y < 0 {
                chunk.set_block(local_pos, BlockId::Water.into());
            } else if world_pos.y == height + 1 && world_pos.y >= 4 {
                for prop in &self.props {
                    if prop.noise.sample([world_pos.x as u64, world_pos.z as u64])
                        % prop.probability
                        == 0
                    {
                        chunk.set_block(local_pos, prop.block.clone());
                    }
                }
            }
        }
    }

    /// Registers the structures that can spawn in the biome.
    pub fn register_structures(
        &self,
        pos: ChunkPos,
        column: &ColumnGen,
        ctx: &GenCtx,
        out: &mut Vec<PendingStructure>,
    ) {
        let origin = pos.origin();

        for local_pos in ColumnPos::iter_all() {
            if column.biome_stage(ctx).ids[local_pos] != self.filter {
                continue;
            }

            let height = column.height_stage(ctx)[local_pos];

            if height < origin.y || height >= origin.y + Chunk::SIDE {
                continue;
            }

            let world_pos = origin + IVec3::new(local_pos.x(), height, local_pos.z());

            for set in &self.structures {
                if set.noise.sample([world_pos.x as u64, world_pos.z as u64]) % set.probability != 0
                {
                    continue;
                }

                let value = set
                    .value_noise
                    .sample([world_pos.x as u64, world_pos.z as u64]);

                let transform_noise = set
                    .transform_noise
                    .sample([world_pos.x as u64, world_pos.y as u64]);
                let mut transformations = match transform_noise % 4 % 4 {
                    0 => StructureTransformations::IDENTITY,
                    1 => StructureTransformations::ROTATE_90,
                    2 => StructureTransformations::ROTATE_180,
                    3 => StructureTransformations::ROTATE_270,
                    _ => unreachable!(),
                };
                if (transform_noise >> 5) & 1 != 0 {
                    transformations.insert(StructureTransformations::FLIP_HORIZONTAL);
                }

                out.push(PendingStructure {
                    position: world_pos,
                    contents: set.set[value as usize % set.set.len()].clone(),
                    transformations,
                });
            }
        }
    }
}

/// Creates a struct that implements the [`Biome`](crate::biome::Biome) trait using
/// a [`StandardBiome`].
#[macro_export]
macro_rules! make_standard_biome {
    (
        $( #[doc = $doc:literal] )*
        pub struct $name:ident ($create_fn:expr);
    ) => {
        $( #[doc = $doc] )*
        pub struct $name($crate::biomes::utility::StandardBiome);

        impl bns_rng::FromRng for $name {
            fn from_rng(rng: &mut impl bns_rng::Rng) -> Self
            where
                Self: Sized,
            {
                fn inner<'a, F, R>(f: F, rng: &'a mut R) -> $name
                where
                    R: bns_rng::Rng,
                    F: FnOnce(&mut $crate::biomes::utility::StandardBiomeBuilder<'a, R>),
                {
                    let mut builder = $crate::biomes::utility::StandardBiomeBuilder::new(rng, $crate::biome::BiomeId::$name);
                    f(&mut builder);
                    $name(builder.build())
                }

                inner($create_fn, rng)
            }
        }

        impl $crate::biome::Biome for $name {
            fn height(&self, pos: glam::IVec2) -> f32 {
                self.0.height(pos)
            }

            fn build(&self, pos: bns_core::ChunkPos, column: &$crate::column_gen::ColumnGen, ctx: &$crate::GenCtx, chunk: &mut bns_core::Chunk) {
                self.0.build(pos, column, ctx, chunk);
            }

            fn register_structures(
                &self,
                pos: bns_core::ChunkPos,
                column: &$crate::column_gen::ColumnGen,
                ctx: &$crate::GenCtx,
                out: &mut ::std::vec::Vec<$crate::chunk_gen::PendingStructure>,
            ) {
                self.0.register_structures(pos, column, ctx, out);
            }

            fn debug_info(&self, w: &mut dyn std::fmt::Write, pos: glam::IVec3) -> std::fmt::Result {
                let _ = (w, pos);
                Ok(())
            }
        }
    };
}
