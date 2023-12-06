use std::hash::BuildHasherDefault;
use std::num::NonZeroUsize;
use std::sync::Arc;

use glam::{IVec3, Vec3};
use hashbrown::HashMap;

use bns_core::{BlockFlags, BlockInstance, Chunk, ChunkPos, Face, LocalPos};
use bns_render::Gpu;
use bns_workers::{Priority, TaskPool, Worker};
use bns_worldgen_core::WorldGenerator;

mod chunk_geometry;
pub use chunk_geometry::*;

/// Stores the state of a chunk loaded in memory.
pub struct LoadedChunk {
    /// The inner chunk data.
    pub data: Chunk,
    /// The geometry of the chunk.
    pub geometry: ChunkGeometry,
    /// Whether the chunk's geometry is dirty and must be rebuilt.
    pub is_dirty: bool,
}

impl LoadedChunk {
    /// Creates a new [`Chunk`] with the given data.
    pub fn new(inner: Chunk) -> Self {
        Self {
            data: inner,
            geometry: ChunkGeometry::new(),
            is_dirty: true,
        }
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
}

impl WorldWorker {
    /// Creates a new [`WorldWorker`] that uses the provided [`WorldGenerator`] to generate chunks.
    pub fn new(generator: Arc<dyn WorldGenerator>) -> Self {
        Self { generator }
    }
}

impl Worker for WorldWorker {
    type Input = ChunkPos;
    type Output = (ChunkPos, Chunk);

    fn run(&mut self, input: Self::Input) -> Self::Output {
        (input, self.generator.generate(input))
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
            .map_or(4 + 5, NonZeroUsize::get)
            .saturating_sub(5)
            .max(1);

        let task_pool = TaskPool::default();

        for _ in 0..worker_count {
            task_pool.spawn(WorldWorker::new(Arc::clone(&generator)));
        }

        Self {
            task_pool,
            chunks: Chunks::default(),
            chunk_build_context: ChunkBuildContext::new(gpu),
            generator,
        }
    }

    /// Returns the generator that the world uses to generate chunks.
    #[inline]
    pub fn generator(&self) -> &dyn WorldGenerator {
        &*self.generator
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
    #[profiling::function]
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

    /// Gets the block at the provided position, or [`None`] if the chunk is not loaded yet.
    pub fn get_block(&self, pos: IVec3) -> Option<BlockInstance> {
        let (chunk_pos, local_pos) = bns_core::utility::chunk_and_local_pos(pos);

        let chunk = match self.chunks.get(&chunk_pos) {
            Some(ChunkEntry::Loaded(chunk)) => chunk,
            _ => return None,
        };

        Some(chunk.data.get_block_instance(local_pos))
    }

    /// Makes sure that the chunks that have been generated in the background are loaded and
    /// available to the current thread.
    #[profiling::function]
    pub fn fetch_available_chunks(&mut self) {
        for (pos, chunk) in self.task_pool.fetch_results() {
            self.chunks
                .insert(pos, ChunkEntry::Loaded(LoadedChunk::new(chunk)));

            // Mark nearby chunks as dirty.
            for x in -1..=1 {
                for y in -1..=1 {
                    for z in -1..=1 {
                        if let Some(ChunkEntry::Loaded(chunk)) =
                            self.chunks.get_mut(&(pos + IVec3::new(x, y, z)))
                        {
                            chunk.is_dirty = true;
                        }
                    }
                }
            }
        }
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
    #[profiling::function]
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
            //  Actually it's more complicated than this, but this issue is solved
            //  by polonius.
            return unsafe { std::mem::transmute::<&mut ChunkEntry, &mut ChunkEntry>(entry) };
        };

        // If the chunk is already built, we can return it immediately.
        if !loaded.is_dirty {
            // SAFETY: same as above.
            return unsafe { std::mem::transmute::<&mut ChunkEntry, &mut ChunkEntry>(entry) };
        }

        // Build the chunk.
        self.chunk_build_context
            .build(pos, |pos| match self.chunks.get(&pos) {
                Some(ChunkEntry::Loaded(chunk)) => Some(&chunk.data),
                _ => None,
            });

        // Applied to built data to the chunk.
        let Some(entry) = self.chunks.get_mut(&pos) else {
            unsafe { std::hint::unreachable_unchecked() };
        };

        let ChunkEntry::Loaded(chunk) = entry else {
            unsafe { std::hint::unreachable_unchecked() };
        };

        self.chunk_build_context.apply(&mut chunk.geometry);
        chunk.is_dirty = false;

        entry
    }

    /// Queries the world for the first block that intersects the line defined by `start`,
    /// `direction` and `end`.
    ///
    /// The block that's the closest to `start` is returned (or [`NotFound`] if no blocks intersect
    /// the line).
    ///
    /// If the line goes through a chunk that's not yet loaded, the query fails with
    /// [`MissingChunk`].
    ///
    /// [`NotFound`]: QueryError::NotFound
    /// [`MissingChunk`]: QueryError::MissingChunk
    ///
    /// # Arguments
    ///
    /// - `start`: The starting point of the line.
    ///
    /// - `direction`: The direction of the line. This is expected to be a normalized vector.
    ///
    /// - `length`: The length of the line.
    #[profiling::function]
    pub fn query_line(
        &self,
        start: Vec3,
        direction: Vec3,
        mut length: f32,
    ) -> Result<QueryResult, QueryError> {
        // FIXME: This is the naive implementation.
        // It's pretty easy to come up with a better one that increases the cursor by the right
        // amount every time. + that would allow us to properly compute which face has been hit.

        const STEP: f32 = 0.05;

        let mut cur = start;

        let mut current_chunk = ChunkPos::from_world_pos(cur);
        let mut chunk = match self.chunks.get(&current_chunk) {
            Some(ChunkEntry::Loaded(chunk)) => chunk,
            _ => return Err(QueryError::MissingChunk(current_chunk)),
        };
        let mut world_pos = bns_core::utility::world_pos_of(cur);

        while length > 0.0 {
            let new_current_chunk = ChunkPos::from_world_pos(cur);
            if new_current_chunk != current_chunk {
                current_chunk = new_current_chunk;
                chunk = match self.chunks.get(&current_chunk) {
                    Some(ChunkEntry::Loaded(chunk)) => chunk,
                    _ => return Err(QueryError::MissingChunk(current_chunk)),
                };
            }

            let new_world_pos = bns_core::utility::world_pos_of(cur);
            if new_world_pos == world_pos {
                cur += direction * STEP;
                length -= STEP;
                continue;
            }
            world_pos = new_world_pos;

            let local_pos = unsafe {
                LocalPos::from_xyz_unchecked(
                    world_pos.x - current_chunk.x * Chunk::SIDE,
                    world_pos.y - current_chunk.y * Chunk::SIDE,
                    world_pos.z - current_chunk.z * Chunk::SIDE,
                )
            };

            if chunk
                .data
                .get_block(local_pos)
                .info()
                .flags
                .contains(BlockFlags::TANGIBLE)
            {
                // Hit!

                let xf = if direction.x > 0.0 {
                    cur.x - world_pos.x as f32
                } else {
                    1.0 - (cur.x - world_pos.x as f32)
                };

                let yf = if direction.y > 0.0 {
                    cur.y - world_pos.y as f32
                } else {
                    1.0 - (cur.y - world_pos.y as f32)
                };

                let zf = if direction.z > 0.0 {
                    cur.z - world_pos.z as f32
                } else {
                    1.0 - (cur.z - world_pos.z as f32)
                };

                #[allow(clippy::collapsible_else_if)]
                let face = if xf < yf {
                    if xf < zf {
                        if direction.x > 0.0 {
                            Face::NegX
                        } else {
                            Face::X
                        }
                    } else {
                        if direction.z > 0.0 {
                            Face::NegZ
                        } else {
                            Face::Z
                        }
                    }
                } else {
                    if yf < zf {
                        if direction.y > 0.0 {
                            Face::NegY
                        } else {
                            Face::Y
                        }
                    } else {
                        if direction.z > 0.0 {
                            Face::NegZ
                        } else {
                            Face::Z
                        }
                    }
                };

                return Ok(QueryResult {
                    face,
                    local_pos,
                    world_pos,
                    chunk_pos: current_chunk,
                    chunk: &chunk.data,
                    hit: cur,
                });
            }
        }

        Err(QueryError::NotFound)
    }

    /// Replaces the provided block with another one.
    ///
    /// # Returns
    ///
    /// This function returns `true` if the block was successfully replaced, or `false` if the
    /// the provided position was part of an unloaded chunk.
    #[profiling::function]
    pub fn set_block(&mut self, world_pos: IVec3, block: BlockInstance) -> bool {
        let (chunk_pos, local_pos) = bns_core::utility::chunk_and_local_pos(world_pos);

        let Some(ChunkEntry::Loaded(chunk)) = self.chunks.get_mut(&chunk_pos) else {
            return false;
        };

        chunk.data.set_block(local_pos, block);
        chunk.is_dirty = true;

        let mut make_dirty = |pos: ChunkPos| {
            if let Some(ChunkEntry::Loaded(chunk)) = self.chunks.get_mut(&pos) {
                chunk.is_dirty = true;
            }
        };

        if local_pos.is_x_min() {
            make_dirty(chunk_pos - IVec3::X);
        } else if local_pos.is_x_max() {
            make_dirty(chunk_pos + IVec3::X);
        }

        if local_pos.is_y_min() {
            make_dirty(chunk_pos - IVec3::Y);
        } else if local_pos.is_y_max() {
            make_dirty(chunk_pos + IVec3::Y);
        }

        if local_pos.is_z_min() {
            make_dirty(chunk_pos - IVec3::Z);
        } else if local_pos.is_z_max() {
            make_dirty(chunk_pos + IVec3::Z);
        }

        true
    }
}

/// An error that might occur while querying a block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryError {
    /// The query went through a chunk that was not yet loaded.
    MissingChunk(ChunkPos),
    /// No block matched the query.
    NotFound,
}

/// The result of a [`World::query_line`] query.
#[derive(Clone, Copy)]
pub struct QueryResult<'a> {
    /// The block face that was hit.
    pub face: Face,
    /// The local position of the block.
    pub local_pos: LocalPos,
    /// The world position of the block.
    pub world_pos: IVec3,
    /// The chunk that the block is in.
    pub chunk_pos: ChunkPos,
    /// The location of the hit.
    pub hit: Vec3,
    /// The chunk that the block is in.
    pub chunk: &'a Chunk,
}
