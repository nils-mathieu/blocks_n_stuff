use std::sync::OnceLock;

use bns_core::{Chunk, ChunkPos};
use bns_worldgen_structure::Structure;

use bitflags::bitflags;
use glam::IVec3;

use crate::GenCtx;

bitflags! {
    /// A bunch of transformations that can be applied to a structure while writing it to the
    /// world.
    #[derive(Debug, Clone, Copy)]
    pub struct StructureTransformations: u32 {
        /// No transformation.
        const IDENTITY = 0;
        /// Rotate the structure by 90 degrees around the Y axis.
        const ROTATE_90 = 1 << 0;
        /// Rotate the structure by 180 degrees around the Y axis.
        const ROTATE_180 = 1 << 1;
        /// Rotate the structure by 270 degrees around the Y axis.
        const ROTATE_270 = Self::ROTATE_90.bits() | Self::ROTATE_180.bits();
    }
}

/// A structure that hasn't been inserted into the world completely yet.
#[derive(Debug)]
pub struct PendingStructure {
    /// The world-space position of the insertion.
    pub position: IVec3,
    /// The structure itself.
    pub contents: Structure<'static>,
    /// Some transformations to apply to the structure before inserting it into the world.
    pub transformations: StructureTransformations,
}

impl PendingStructure {
    /// The bounds of the inserted structure in world-space.
    pub fn bounds(&self) -> (IVec3, IVec3) {
        let min = self.position;
        let max = self.position + self.contents.bounds;
        (min, max)
    }

    /// Returns whether this structure is at least partially part of the chunk at the provided
    /// position.
    pub fn is_part_of_chunk(&self, pos: ChunkPos) -> bool {
        let (min, max) = self.bounds();

        if min == max {
            return false;
        }

        let min = ChunkPos::from_world_pos_i(min);
        let max = ChunkPos::from_world_pos_i(max - IVec3::ONE);

        min.x <= pos.x
            && pos.x <= max.x
            && min.z <= pos.z
            && pos.z <= max.z
            && min.y <= pos.y
            && pos.y <= max.y
    }

    /// Writes the part of the structure that's in the provided chunk.
    pub fn write_to(&self, pos: ChunkPos, chunk: &mut Chunk) {
        if !self.is_part_of_chunk(pos) {
            return;
        }

        for edit in self.contents.edits.iter() {
            if let Some(pos) = pos.checked_local_pos(self.position + edit.position) {
                chunk.set_block(pos, edit.block.clone());
            }
        }
    }
}

/// Contains information about a chunk that's in the process of being generated.
pub struct ChunkGen {
    /// The position of the chun being generated.
    pos: ChunkPos,
    /// When set, indicates that the chunk has requested the structures that it needs to spawn
    /// from nearby biomes.
    structures: OnceLock<Vec<PendingStructure>>,
}

impl ChunkGen {
    /// Creates a new [`ChunkGen`] with the provided position.
    #[inline]
    pub fn new(pos: ChunkPos) -> Self {
        Self {
            pos,
            structures: OnceLock::new(),
        }
    }

    /// Ensures that the chunk has has requested the structures that it needs to spawn.
    pub fn structures(&self, ctx: &GenCtx) -> &[PendingStructure] {
        self.structures.get_or_init(|| {
            profiling::scope!("ChunkGen::structures");

            let mut result = Vec::new();

            let col = ctx.cache.get_column(self.pos.xz());
            let biomes_in_chunk = &col.biome_stage(ctx).unique_biomes;
            for &biome in biomes_in_chunk {
                ctx.biome_registry[biome]
                    .implementation
                    .register_structures(self.pos, &col, ctx, &mut result);
            }

            result
        })
    }
}
