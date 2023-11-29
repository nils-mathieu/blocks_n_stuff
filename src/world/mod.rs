use std::hash::BuildHasherDefault;
use std::sync::Arc;

use bns_core::Chunk;
use glam::IVec3;

use hashbrown::HashMap;

mod chunk_geometry;
pub use chunk_geometry::*;

use crate::gfx::Gpu;

/// The position of a chunk.
pub type ChunkPos = IVec3;

/// Describes how to generate a [`World`](World).
pub trait WorldGenerator {
    /// Generates a chunk for the provided position.
    ///
    /// # Purity
    ///
    /// This function is expected to be pure. Calling it multiple times with the same `pos` value
    /// should produce the same exact chunk.
    fn generate(&mut self, pos: ChunkPos) -> Chunk;
}

/// Stores the state of a chunk loaded in memory.
pub struct ChunkEntry {
    /// The inner chunk data.
    pub inner: Chunk,
    /// The geometry of the chunk.
    pub geometry: ChunkGeometry,
    /// Whether the geometry of the chunk is dirty and needs to be rebuilt.
    pub dirty: bool,
}

impl ChunkEntry {
    /// Creates a new [`Chunk`] with the given data.
    pub fn new(inner: Chunk) -> Self {
        Self {
            inner,
            geometry: ChunkGeometry::new(),
            dirty: true,
        }
    }
}

/// A collection of chunks.
#[derive(Default)]
struct Chunks {
    chunks: HashMap<ChunkPos, ChunkEntry, BuildHasherDefault<rustc_hash::FxHasher>>,
}

impl Chunks {
    /// Attempts to get a chunk at the provided position.
    ///
    /// If the chunk is available, it is returned. Otherwise, it is generated using the
    /// [`WorldGenerator`] that was provided to this [`World`] when it was created.
    ///
    /// # Remarks
    ///
    /// This function does not attempt to rebuild the geometry of the chunk.
    pub fn get_or_generate<F>(&mut self, pos: ChunkPos, generate: F) -> &mut ChunkEntry
    where
        F: FnOnce(ChunkPos) -> Chunk,
    {
        use hashbrown::hash_map::Entry;
        match self.chunks.entry(pos) {
            Entry::Vacant(e) => e.insert(ChunkEntry::new(generate(pos))),
            Entry::Occupied(e) => e.into_mut(),
        }
    }

    /// Gets the neighborhood of a chunk.
    fn get_chunk_neighborhood<F>(&mut self, pos: ChunkPos, mut generate: F) -> [&mut ChunkEntry; 7]
    where
        F: FnMut(ChunkPos) -> Chunk,
    {
        // Make sure that the chunk is loaded.
        self.get_or_generate(pos, &mut generate);
        self.get_or_generate(pos + IVec3::X, &mut generate);
        self.get_or_generate(pos + IVec3::NEG_X, &mut generate);
        self.get_or_generate(pos + IVec3::Y, &mut generate);
        self.get_or_generate(pos + IVec3::NEG_Y, &mut generate);
        self.get_or_generate(pos + IVec3::Z, &mut generate);
        self.get_or_generate(pos + IVec3::NEG_Z, &mut generate);

        // SAFETY:
        //  There's two things here:
        //  1. We just called `get_chunk` on every of those chunks, ensuring that they are found
        //     in the hashmap. This makes sure that the `.unwrap_unchecked` is valid.
        //  2. Even when wrapping, it is not possible that any of the keys we're requesting collide.
        unsafe {
            self.chunks
                .get_many_unchecked_mut([
                    &pos,
                    &(pos + IVec3::X),
                    &(pos + IVec3::NEG_X),
                    &(pos + IVec3::Y),
                    &(pos + IVec3::NEG_Y),
                    &(pos + IVec3::Z),
                    &(pos + IVec3::NEG_Z),
                ])
                .unwrap_unchecked()
        }
    }
}

/// Contains a dynamic collection chunks.
pub struct World {
    /// The list of chunks that are currently loaded in memory.
    chunks: Chunks,
    /// Generates chunks for the world.
    generator: Box<dyn WorldGenerator>,
    /// The context required to build chunks.
    chunk_build_context: ChunkBuildContext,
}

impl World {
    /// Creates a new [`World`] that uses the provided [`WorldGenerator`] to generate chunks.
    pub fn new(gpu: Arc<Gpu>, generator: Box<dyn WorldGenerator>) -> Self {
        Self {
            chunks: Chunks::default(),
            generator,
            chunk_build_context: ChunkBuildContext::new(gpu),
        }
    }

    /// Returns an existing chunk at the provided position.
    ///
    /// The chunk is not built if it was not already built.
    pub fn get_existing_chunk(&self, pos: ChunkPos) -> Option<&ChunkEntry> {
        self.chunks.chunks.get(&pos)
    }

    /// Requests a chunk.
    ///
    /// If the chunk is not currently available, [`None`] is returned and the chunk is queued
    /// for loading.
    ///
    /// # Request Priority
    ///
    /// `priority` is the priority of the request. This is a number representing how fast compared
    /// to the other requests the chunk should be made available if it's not already loaded.
    ///
    /// If the requested chunk is not avaialble, the chunk with the highest priority value will
    /// be loaded first.
    ///
    /// # Remarks
    ///
    /// If the chunk was already previously requested, the priority of the request is overwritten
    /// regardless of whether the new priority is higher or lower.
    pub fn request_chunk(&mut self, pos: ChunkPos, _priority: usize) -> Option<&mut ChunkEntry> {
        let mut generate = |pos| self.generator.generate(pos);

        let chunk = self.chunks.get_or_generate(pos, &mut generate);

        if !chunk.dirty {
            // SAFETY: https://github.com/rust-lang/rfcs/blob/master/text/2094-nll.md#problem-case-3-conditional-control-flow-across-functions
            let chunk = unsafe { std::mem::transmute::<&mut ChunkEntry, &mut ChunkEntry>(chunk) };
            return Some(chunk);
        }

        let [chunk, x, nx, y, ny, z, nz] = self.chunks.get_chunk_neighborhood(pos, generate);

        chunk.geometry.build(
            ChunkNeighborhood {
                this: &chunk.inner,
                x: &x.inner,
                nx: &nx.inner,
                y: &y.inner,
                ny: &ny.inner,
                z: &z.inner,
                nz: &nz.inner,
            },
            &mut self.chunk_build_context,
        );

        chunk.dirty = false;

        Some(chunk)
    }
}
