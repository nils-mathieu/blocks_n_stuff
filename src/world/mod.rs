use std::hash::BuildHasherDefault;
use std::sync::Arc;

use glam::{IVec3, Vec3};
use hashbrown::HashMap;

use bns_core::{BlockFlags, BlockInstance, Chunk, ChunkPos, Face, LocalPos};
use bns_render::Gpu;
use bns_worldgen_core::WorldGenerator;

mod chunk_geometry;
pub use chunk_geometry::*;

use self::task_pool::TaskPool;

mod task_pool;

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

/// An entry into the [`Chunks`] map.
enum ChunkEntry {
    /// The chunk is already properly loaded.
    Loaded(LoadedChunk),
    /// The chunk is currently generating.
    Generating,
}

impl ChunkEntry {
    /// Returns the loaded chunk, if any.
    #[inline]
    pub fn loaded(&self) -> Option<&LoadedChunk> {
        match self {
            Self::Loaded(chunk) => Some(chunk),
            Self::Generating => None,
        }
    }
}

/// A collection of chunks.
type Chunks = HashMap<ChunkPos, ChunkEntry, BuildHasherDefault<rustc_hash::FxHasher>>;

/// A task that's submitted to the task pool.
struct Task {
    /// The generator that must be used to generate the chunk.
    generator: Arc<dyn WorldGenerator>,
    /// The position of the chunk that must be generated.
    pos: ChunkPos,
}

impl task_pool::Task for Task {
    type Output = (ChunkPos, Chunk);

    fn execute(self) -> Self::Output {
        let chunk = self.generator.generate(self.pos);
        (self.pos, chunk)
    }
}

/// Contains a dynamic collection chunks.
pub struct World {
    /// The list of chunks that are currently loaded in memory.
    chunks: Chunks,

    /// The task pool used to generate new chunks in the background (probably, that depends
    /// on the current compilation target).
    task_pool: TaskPool<Task>,

    /// The context used to build chunks.
    ///
    /// This is just a bunch of buffers that are re-used when a new chunk needs its geometry
    /// to be rebuilt.
    chunk_build_context: ChunkBuildContext,

    /// The current world generator. Used to generate new chunks when some are missing.
    generator: Arc<dyn WorldGenerator>,

    /// A list of chunks that must be submitted to the task pool for generation.
    ///
    /// This is used to avoid re-allocating a new vector every time we need to perform
    /// a submission.
    tasks_to_submit: Vec<Task>,
}

impl World {
    /// Creates a new [`World`] that uses the provided [`WorldGenerator`] to generate chunks.
    pub fn new(gpu: Arc<Gpu>, generator: Arc<dyn WorldGenerator>) -> Self {
        Self {
            chunks: Chunks::default(),
            chunk_build_context: ChunkBuildContext::new(gpu),
            task_pool: TaskPool::new(),
            generator,
            tasks_to_submit: Vec::new(),
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
        self.task_pool.pending_tasks()
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

        let retain_chunk = |pos: ChunkPos| {
            let hd = pos.xz().distance_squared(center.xz()) as u32;
            let vd = (pos.y - center.y).unsigned_abs();
            hd < h_radius * h_radius && vd < v_radius
        };

        self.chunks.retain(|&pos, _| retain_chunk(pos));
        self.chunks
            .shrink_to(h_radius as usize * h_radius as usize * v_radius as usize);

        self.task_pool.retain_tasks(|task| retain_chunk(task.pos));
    }

    /// Gets the block at the provided position, or [`None`] if the chunk is not loaded yet.
    pub fn get_block(&self, pos: IVec3) -> Option<BlockInstance> {
        let (chunk_pos, local_pos) = bns_core::utility::chunk_and_local_pos(pos);

        self.chunks
            .get(&chunk_pos)
            .and_then(ChunkEntry::loaded)
            .map(|chunk| chunk.data.get_block_instance(local_pos))
    }

    /// Returns the chunk at the provided position, or [`None`] if the chunk is not loaded yet.
    ///
    /// # Remarks
    ///
    /// This function does not:
    ///
    /// 1. Request the chunk for loading if it's not already loaded.
    ///
    /// 2. Rebuild the chunk's geometry if it's dirty.
    ///
    /// However, it's possible to check both of those things using the returned value.
    #[inline]
    pub fn get_chunk(&self, pos: ChunkPos) -> Option<&LoadedChunk> {
        self.chunks.get(&pos).and_then(ChunkEntry::loaded)
    }

    /// Requests a chunk.
    ///
    /// If the chunk is not currently available, [`None`] is returned and the chunk is queued
    /// for loading.
    ///
    /// # Returns
    ///
    /// The built chunk, if it was already available.
    #[profiling::function]
    pub fn request_chunk(&mut self, pos: ChunkPos) -> Option<&mut LoadedChunk> {
        use hashbrown::hash_map::Entry;

        match self.chunks.entry(pos) {
            Entry::Occupied(e) => {
                match e.into_mut() {
                    ChunkEntry::Loaded(chunk) => {
                        if !chunk.is_dirty {
                            // The chunk is already built and up-to-date. We can return it right now.
                            // Unfortunately, the borrow checker does not seem to be able to figure
                            // out what's going on here. This is a known problem that's supposed to be
                            // fixed by Polonius.
                            // This bit of unsafe code simply unties the return value of the function from
                            // the borrow of `self`, allowing us to return the chunk mutably while maintaining
                            // our right to use the world later.
                            return Some(unsafe {
                                std::mem::transmute::<&mut LoadedChunk, &mut LoadedChunk>(chunk)
                            });
                        }

                        self.chunk_build_context.build(pos, |pos| {
                            self.chunks
                                .get(&pos)
                                .and_then(ChunkEntry::loaded)
                                .map(|chunk| &chunk.data)
                        });

                        // Re-borrow the chunk mutably and return it.
                        // We can use unsafe to hint the compiler that the lookup cannot fail.
                        let Some(ChunkEntry::Loaded(chunk)) = self.chunks.get_mut(&pos) else {
                            unsafe { std::hint::unreachable_unchecked() }
                        };

                        chunk.is_dirty = false;
                        self.chunk_build_context.apply(&mut chunk.geometry);

                        Some(chunk)
                    }
                    ChunkEntry::Generating => {
                        // The chunk has already been requested.
                        // No need to do anything.
                        None
                    }
                }
            }
            Entry::Vacant(e) => {
                // The chunk is not loaded yet.
                // We need to request it from the task pool.
                e.insert(ChunkEntry::Generating);
                self.tasks_to_submit.push(Task {
                    generator: self.generator.clone(),
                    pos,
                });
                None
            }
        }
    }

    /// Sorts the chunks that are currently pending for generation.
    ///
    /// This function can be used to prioritize the chunks that are the closest to the player.
    ///
    /// The *last* chunks in the array will be the first to be submitted to the task pool.
    pub fn sort_pending_chunks<F, O>(&mut self, mut key: F)
    where
        F: FnMut(ChunkPos) -> O,
        O: Ord,
    {
        self.tasks_to_submit
            .sort_unstable_by_key(|task| key(task.pos))
    }

    /// Removes any currently pending chunks from the task pool and submits the last chunk that
    /// were requested instead.
    #[profiling::function]
    pub fn flush_pending_chunks(&mut self) {
        // Check if the task pool has sent us some results.
        for (pos, chunk) in self.task_pool.fetch_outputs() {
            use hashbrown::hash_map::Entry;

            match self.chunks.entry(pos) {
                Entry::Occupied(mut e) => {
                    match e.get() {
                        ChunkEntry::Loaded(_) => {
                            // We received a chunk that we already have.
                            // This can happen if one of two worker threads generated the same chunk
                            // because we requested it twice (when cleaning occurs while some chunks
                            // are still loading). This should be rare enough.
                            bns_log::warning!("received a chunk that we already have");
                        }
                        ChunkEntry::Generating => {
                            e.insert(ChunkEntry::Loaded(LoadedChunk::new(chunk)));

                            // Mark nearby chunks as dirty so that they get rebuilt.
                            const OFFSETS: [IVec3; 6] = [
                                IVec3::X,
                                IVec3::NEG_X,
                                IVec3::Y,
                                IVec3::NEG_Y,
                                IVec3::Z,
                                IVec3::NEG_Z,
                            ];

                            for offset in OFFSETS {
                                if let Some(ChunkEntry::Loaded(chunk)) =
                                    self.chunks.get_mut(&(pos + offset))
                                {
                                    chunk.is_dirty = true;
                                }
                            }
                        }
                    }
                }
                Entry::Vacant(_) => {
                    // We just received a chunk we did not ask for.
                    // Usually occurs when we clean up the world while some chunks are
                    // still loading.
                    // It's not a big deal, just discard the chunk.
                }
            }
        }

        self.task_pool.submit_batch(&mut self.tasks_to_submit);
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
        let mut chunk = self
            .chunks
            .get(&current_chunk)
            .and_then(ChunkEntry::loaded)
            .ok_or(QueryError::MissingChunk(current_chunk))?;
        let mut world_pos = bns_core::utility::world_pos_of(cur);

        while length > 0.0 {
            let new_current_chunk = ChunkPos::from_world_pos(cur);
            if new_current_chunk != current_chunk {
                current_chunk = new_current_chunk;
                chunk = self
                    .chunks
                    .get(&current_chunk)
                    .and_then(ChunkEntry::loaded)
                    .ok_or(QueryError::MissingChunk(current_chunk))?;
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
