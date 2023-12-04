//! Utility functions for the core library.

use glam::Vec3;

use crate::{Chunk, ChunkPos};

/// Returns the chunk that the provided position is in.
pub fn chunk_of(pos: Vec3) -> ChunkPos {
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
