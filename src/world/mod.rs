use std::hash::BuildHasherDefault;
use std::num::NonZeroUsize;
use std::sync::Arc;

use bitflags::bitflags;
use glam::{IVec3, Vec3Swizzles};
use hashbrown::HashMap;
use smallvec::SmallVec;

use bns_core::{Chunk, ChunkPos};
use bns_render::Gpu;
use bns_workers::{Priority, TaskPool, Worker};
use bns_worldgen_core::WorldGenerator;

mod chunk_geometry;
pub use chunk_geometry::*;

bitflags! {
    /// Some flags associated with a [`LoadedChunk`].
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct DirtyFlags: u8 {
        /// Whether the inner geometry of the chunk is dirty and must be rebuilt.
        const INNER_DIRTY = 1 << 0;

        /// Whether the positive X boundary of the chunk is dirty and must be rebuilt.
        const BOUNDARY_DIRTY_X = 1 << 1;
        /// Whether the negative X boundary of the chunk is dirty and must be rebuilt.
        const BOUNDARY_DIRTY_NEG_X = 1 << 2;
        /// Whether the positive Y boundary of the chunk is dirty and must be rebuilt.
        const BOUNDARY_DIRTY_Y = 1 << 3;
        /// Whether the negative Y boundary of the chunk is dirty and must be rebuilt.
        const BOUNDARY_DIRTY_NEG_Y = 1 << 4;
        /// Whether the positive Z boundary of the chunk is dirty and must be rebuilt.
        const BOUNDARY_DIRTY_Z = 1 << 5;
        /// Whether the negative Z boundary of the chunk is dirty and must be rebuilt.
        const BOUNDARY_DIRTY_NEG_Z = 1 << 6;

        /// Union of all the dirty flags.
        const ANY_DIRTY = Self::INNER_DIRTY.bits()
            | Self::BOUNDARY_DIRTY_X.bits()
            | Self::BOUNDARY_DIRTY_NEG_X.bits()
            | Self::BOUNDARY_DIRTY_Y.bits()
            | Self::BOUNDARY_DIRTY_NEG_Y.bits()
            | Self::BOUNDARY_DIRTY_Z.bits()
            | Self::BOUNDARY_DIRTY_NEG_Z.bits();
    }
}

/// Stores the state of a chunk loaded in memory.
pub struct LoadedChunk {
    /// The inner chunk data.
    pub data: Chunk,
    /// The geometry of the chunk.
    pub geometry: ChunkGeometry,
    /// Whether the geometry of the chunk is dirty and needs to be rebuilt.
    pub dirty_flags: DirtyFlags,
}

impl LoadedChunk {
    /// Creates a new [`Chunk`] with the given data.
    pub fn new(inner: Chunk) -> Self {
        Self {
            data: inner,
            geometry: ChunkGeometry::new(),
            dirty_flags: DirtyFlags::ANY_DIRTY,
        }
    }

    /// Returns whether the chunk is missing some geometry.
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty_flags.intersects(DirtyFlags::ANY_DIRTY)
    }
}

/// A chunk entry into the [`World`].
pub enum ChunkEntry {
    /// The chunk is currently being generated.
    Generating,
    /// The chunk is loaded in memory.
    Loaded(LoadedChunk),
}

/// A collection of chunks.
type Chunks = HashMap<ChunkPos, ChunkEntry, BuildHasherDefault<rustc_hash::FxHasher>>;

struct WorldWorker {
    generator: Arc<dyn WorldGenerator>,
    build_context: ChunkBuildContext,
}

impl WorldWorker {
    /// Creates a new [`WorldWorker`] that uses the provided [`WorldGenerator`] to generate chunks.
    pub fn new(gpu: Arc<Gpu>, generator: Arc<dyn WorldGenerator>) -> Self {
        Self {
            generator,
            build_context: ChunkBuildContext::new(gpu),
        }
    }
}

impl Worker for WorldWorker {
    type Input = ChunkPos;
    type Output = (ChunkPos, LoadedChunk);

    fn run(&mut self, input: Self::Input) -> Self::Output {
        let chunk = self.generator.generate(input);
        let mut entry = LoadedChunk::new(chunk);

        // Build the inner geometry of the chunk.
        self.build_context.reset();
        self.build_context.build_inner(&entry.data);
        self.build_context.append_to(&mut entry.geometry);
        entry.dirty_flags.remove(DirtyFlags::INNER_DIRTY);

        (input, entry)
    }
}

/// Contains a dynamic collection chunks.
pub struct World {
    /// The list of chunks that are currently loaded in memory.
    chunks: Chunks,
    /// The task pool used to generate new chunks.
    task_pool: TaskPool<WorldWorker>,
    /// The context used to build chunks.
    chunk_build_context: ChunkBuildContext,

    /// The current world generator.
    generator: Arc<dyn WorldGenerator>,
}

impl World {
    /// Creates a new [`World`] that uses the provided [`WorldGenerator`] to generate chunks.
    pub fn new(gpu: Arc<Gpu>, generator: Arc<dyn WorldGenerator>) -> Self {
        let worker_count = std::thread::available_parallelism()
            .map_or(4 + 3, NonZeroUsize::get)
            .saturating_sub(3)
            .max(1);

        let task_pool = TaskPool::default();

        for _ in 0..worker_count {
            task_pool.spawn(WorldWorker::new(gpu.clone(), Arc::clone(&generator)));
        }

        Self {
            task_pool,
            chunks: Chunks::default(),
            chunk_build_context: ChunkBuildContext::new(gpu),
            generator,
        }
    }

    /// Returns the number of chunks that are currently being generated.
    #[inline]
    pub fn loading_chunk_count(&self) -> usize {
        self.task_pool.task_count()
    }

    /// Returns the number of chunks that are currently loaded in memory.
    #[inline]
    pub fn loaded_chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Hints the [`World`] that the player is currently at the provided position, requesting
    /// chunks to be loaded around the player.
    pub fn request_cleanup(&mut self, center: ChunkPos, h_radius: u32, v_radius: u32) {
        self.generator.request_cleanup(center, h_radius, v_radius);

        self.chunks.retain(|&pos, _| {
            let hd = pos.xz().distance_squared(center.xz()) as u32;
            let vd = (pos.y - center.y).unsigned_abs();
            hd < h_radius * h_radius && vd < v_radius
        });
        self.chunks.shrink_to_fit();
    }

    /// Returns an existing chunk at the provided position.
    ///
    /// The chunk is not built if it was not already built.
    ///
    /// # Remarks
    ///
    /// This function does not check whether the chunk is missing some geometry.
    #[inline]
    pub fn get_existing_chunk(&self, pos: ChunkPos) -> Option<&LoadedChunk> {
        match self.chunks.get(&pos) {
            Some(ChunkEntry::Loaded(chunk)) => Some(chunk),
            _ => None,
        }
    }

    /// Makes sure that the chunks that have been generated in the background are loaded and
    /// available to the current thread.
    pub fn fetch_available_chunks(&mut self) {
        self.chunks.extend(
            self.task_pool
                .fetch_results()
                .map(|(pos, c)| (pos, ChunkEntry::Loaded(c))),
        );
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
    ///
    /// # Returns
    ///
    /// The built chunk, if it was already available.
    pub fn request_chunk(&mut self, pos: ChunkPos, priority: Priority) -> &mut ChunkEntry {
        use hashbrown::hash_map::Entry;

        let entry = match self.chunks.entry(pos) {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => {
                self.task_pool.submit(pos, priority);
                e.insert(ChunkEntry::Generating)
            }
        };

        let ChunkEntry::Loaded(loaded) = entry else {
            // SAFETY:
            //  The borrow checker is not smart enough to realize that returning the entry
            //  makes it valid to borrow the map mutably later.
            return unsafe { std::mem::transmute::<&mut ChunkEntry, &mut ChunkEntry>(entry) };
        };

        // If the chunk is already built, we can return it immediately.
        if !loaded.is_dirty() {
            // SAFETY: same as above.
            return unsafe { std::mem::transmute::<&mut ChunkEntry, &mut ChunkEntry>(entry) };
        }

        // Reborrow the chunk using shared references.
        let Some(ChunkEntry::Loaded(chunk)) = self.chunks.get(&pos) else {
            // SAFETY:
            //  We know that the chunk is present in the map because we just inserted it.
            unsafe { std::hint::unreachable_unchecked() }
        };

        let mut dirty_flags = chunk.dirty_flags;
        let mut to_request = SmallVec::<[ChunkPos; 6]>::new();
        let mut get_or_request = |pos: ChunkPos| match self.chunks.get(&pos) {
            Some(ChunkEntry::Loaded(chunk)) => Some(chunk),
            Some(ChunkEntry::Generating) => None,
            None => {
                to_request.push(pos);
                None
            }
        };

        // Some parts of the chunk is dirty, we need to rebuild those.
        self.chunk_build_context.reset();
        if dirty_flags.contains(DirtyFlags::BOUNDARY_DIRTY_X) {
            if let Some(other) = get_or_request(pos + IVec3::X) {
                self.chunk_build_context
                    .build_boundary_x(&chunk.data, &other.data);
                dirty_flags.remove(DirtyFlags::BOUNDARY_DIRTY_X);
            }
        }
        if dirty_flags.contains(DirtyFlags::BOUNDARY_DIRTY_NEG_X) {
            if let Some(other) = get_or_request(pos - IVec3::X) {
                self.chunk_build_context
                    .build_boundary_neg_x(&chunk.data, &other.data);
                dirty_flags.remove(DirtyFlags::BOUNDARY_DIRTY_NEG_X);
            }
        }
        if dirty_flags.contains(DirtyFlags::BOUNDARY_DIRTY_Y) {
            if let Some(other) = get_or_request(pos + IVec3::Y) {
                self.chunk_build_context
                    .build_boundary_y(&chunk.data, &other.data);
                dirty_flags.remove(DirtyFlags::BOUNDARY_DIRTY_Y);
            }
        }
        if dirty_flags.contains(DirtyFlags::BOUNDARY_DIRTY_NEG_Y) {
            if let Some(other) = get_or_request(pos - IVec3::Y) {
                self.chunk_build_context
                    .build_boundary_neg_y(&chunk.data, &other.data);
                dirty_flags.remove(DirtyFlags::BOUNDARY_DIRTY_NEG_Y);
            }
        }
        if dirty_flags.contains(DirtyFlags::BOUNDARY_DIRTY_Z) {
            if let Some(other) = get_or_request(pos + IVec3::Z) {
                self.chunk_build_context
                    .build_boundary_z(&chunk.data, &other.data);
                dirty_flags.remove(DirtyFlags::BOUNDARY_DIRTY_Z);
            }
        }
        if dirty_flags.contains(DirtyFlags::BOUNDARY_DIRTY_NEG_Z) {
            if let Some(other) = get_or_request(pos - IVec3::Z) {
                self.chunk_build_context
                    .build_boundary_neg_z(&chunk.data, &other.data);
                dirty_flags.remove(DirtyFlags::BOUNDARY_DIRTY_NEG_Z);
            }
        }

        if !to_request.is_empty() {
            self.chunks
                .extend(to_request.iter().map(|&pos| (pos, ChunkEntry::Generating)));
            self.task_pool
                .submit_batch(to_request.iter().map(|&pos| (pos, priority)));
        }

        // Reborrow again >:( the chunk for an exclusive reference.

        // SAFETY:
        //  We know that the chunk is present in the map because we just inserted it.
        let entry = unsafe { self.chunks.get_mut(&pos).unwrap_unchecked() };

        if let ChunkEntry::Loaded(chunk) = entry {
            if dirty_flags != chunk.dirty_flags {
                // We added some geometry to the chunk.
                // We need to apply the changes to the GPU.
                self.chunk_build_context.append_to(&mut chunk.geometry);
                chunk.dirty_flags = dirty_flags;
            }
        } else {
            // SAFETY:
            //  We know that the chunk is laoded.
            unsafe { std::hint::unreachable_unchecked() }
        }

        entry
    }
}
