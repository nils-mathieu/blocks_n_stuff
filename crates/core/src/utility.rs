//! Utility functions for the core library.

use glam::{IVec3, Vec3};

use crate::{Chunk, ChunkPos};

/// Returns the chunk that the provided position is in.
pub fn chunk_pos_of(pos: Vec3) -> ChunkPos {
    fn coord_to_chunk(coord: f32) -> i32 {
        if coord >= 0.0 {
            coord as i32 / Chunk::SIDE
        } else {
            coord as i32 / Chunk::SIDE - 1
        }
    }

    ChunkPos::new(
        coord_to_chunk(pos.x),
        coord_to_chunk(pos.y),
        coord_to_chunk(pos.z),
    )
}

/// Returns the position of the block that the provided position is in.
pub fn world_pos_of(pos: Vec3) -> IVec3 {
    fn coord_to_world(coord: f32) -> i32 {
        if coord >= 0.0 {
            coord as i32
        } else {
            coord as i32 - 1
        }
    }

    IVec3::new(
        coord_to_world(pos.x),
        coord_to_world(pos.y),
        coord_to_world(pos.z),
    )
}
