//! This crate simply defines the base world generator trait for use by other crates.

use bns_core::{Chunk, ChunkPos};

use glam::IVec3;

/// Describes how to generate new chunks for a world.
pub trait WorldGenerator: Send + Sync {
    /// Generates a chunk for the provided position.
    ///
    /// # Purity
    ///
    /// This function is expected to be pure. Calling it multiple times with the same `pos` value
    /// should produce the same exact chunk.
    fn generate(&self, pos: ChunkPos) -> Chunk;

    /// Requests the world generator to cleanup any unused memory.
    ///
    /// The provided cylinder describes the area that should be kept in memory. Any chunks that
    /// are outside of this cylinder can be unloaded.
    fn request_cleanup(&self, center: ChunkPos, h_radius: u32, v_radius: u32);

    /// Prints debug information about a particular position in the world using the provided
    /// buffer.
    fn debug_info(&self, buf: &mut String, pos: IVec3);
}
