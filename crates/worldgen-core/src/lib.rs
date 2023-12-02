//! This crate simply defines the base world generator trait for use by other crates.

use bns_core::{Chunk, ChunkPos};

use glam::IVec3;

/// Describes how to generate new chunks for a world.
pub trait WorldGenerator: Send + Clone {
    /// Generates a chunk for the provided position.
    ///
    /// # Purity
    ///
    /// This function is expected to be pure. Calling it multiple times with the same `pos` value
    /// should produce the same exact chunk.
    fn generate(&mut self, pos: ChunkPos) -> Chunk;

    /// Prints debug information about the world generator using the provided buffer.
    ///
    /// This information will be displayed on the debug UI in-game.
    fn debug_info(&self, buf: &mut String) {
        let _ = buf;
    }

    /// Prints debug information about a particular position in the world using the provided
    /// buffer.
    fn debug_info_pos(&self, buf: &mut String, pos: IVec3) {
        let _ = buf;
        let _ = pos;
    }
}
