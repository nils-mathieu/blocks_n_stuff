//! Utility functions for the core library.

use glam::{IVec3, Vec3};

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
