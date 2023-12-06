//! Utility functions for the core library.

use glam::{IVec3, Vec3};

use crate::{ChunkPos, LocalPos};

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

/// Computes the [`ChunkPos`] and [`LocalPos`] of the provided world-space position.
pub fn chunk_and_local_pos(pos: IVec3) -> (ChunkPos, LocalPos) {
    let chunk_pos = ChunkPos::from_world_pos_i(pos);
    let origin = chunk_pos.origin();

    // SAFETY:
    //  By removing the origin from the world position, we are guaranteed that the resulting
    //  position is within the chunk (each coordinate will be less than 32).
    let local_pos = unsafe {
        LocalPos::from_xyz_unchecked(pos.x - origin.x, pos.y - origin.y, pos.z - origin.z)
    };

    (chunk_pos, local_pos)
}
