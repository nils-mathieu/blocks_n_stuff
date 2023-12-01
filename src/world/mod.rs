use std::collections::BinaryHeap;
use std::hash::BuildHasherDefault;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;

use bitflags::bitflags;
use bns_core::Chunk;
use bns_render::Gpu;
use glam::IVec3;

use hashbrown::HashMap;

mod chunk_geometry;
pub use chunk_geometry::*;
use parking_lot::{Condvar, Mutex, MutexGuard};
use smallvec::SmallVec;

/// The position of a chunk.
pub type ChunkPos = IVec3;

/// Describes how to generate a [`World`].
pub trait WorldGenerator: Send + Clone {
    /// Generates a chunk for the provided position.
    ///
    /// # Purity
    ///
    /// This function is expected to be pure. Calling it multiple times with the same `pos` value
    /// should produce the same exact chunk.
    fn generate(&mut self, pos: ChunkPos) -> Chunk;
}

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

/// The type used to represent the priority of a task.
pub type Priority = usize;

/// A task to be executed by a worker thread.
struct Task {
    /// The priority of the task.
    priority: Priority,
    /// The position of the chunk that must be generated.
    pos: ChunkPos,
}

impl PartialEq for Task {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for Task {}

impl PartialOrd for Task {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.priority.cmp(&other.priority))
    }
}

impl Ord for Task {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}

/// A shared pool of tasks to be executed by worker threads.
struct TaskPool {
    /// The task that must be executed.
    tasks: Mutex<BinaryHeap<Task>>,
    /// Whether the worker threads should be stopping.
    should_stop: AtomicBool,
    /// A condition variable used to wait on the `tasks` field.
    condvar: Condvar,
    /// The list of chunks that were generated by the worker threads.
    results: Mutex<Vec<(ChunkPos, LoadedChunk)>>,
}

impl TaskPool {
    /// Creates a new [`TaskPool`].
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(BinaryHeap::new()),
            results: Mutex::new(Vec::new()),
            should_stop: AtomicBool::new(false),
            condvar: Condvar::new(),
        }
    }

    /// Returns the number of tasks that are currently queued.
    #[inline]
    pub fn task_count(&self) -> usize {
        self.tasks.lock().len()
    }

    /// Returns whether the worker threads should stop.
    #[inline]
    pub fn should_stop(&self) -> bool {
        self.should_stop.load(Relaxed)
    }

    /// Fetches a new task to be executed.
    ///
    /// This function returns [`None`] if the worker thread should stop.
    pub fn fetch_task(&self) -> Option<Task> {
        if self.should_stop() {
            // fast path: don't even try to acquire the lock if we know that we're stoppign
            // anyway.
            return None;
        }

        let mut lock = self.tasks.lock();
        loop {
            match lock.pop() {
                Some(task) => return Some(task),
                None => self.condvar.wait(&mut lock),
            }

            if self.should_stop() {
                return None;
            }
        }
    }

    /// Pushes a new task to be executed.
    pub fn push_task(&self, pos: ChunkPos, priority: Priority) {
        let mut lock = self.tasks.lock();
        lock.push(Task { pos, priority });
        self.condvar.notify_one();
    }

    /// Pushes a collection of tasks to be executed.
    pub fn push_tasks(&self, tasks: impl IntoIterator<Item = (ChunkPos, Priority)>) {
        let mut count = 0;

        self.tasks.lock().extend(
            tasks
                .into_iter()
                .inspect(|_| count += 1)
                .map(|(pos, priority)| Task { pos, priority }),
        );

        for _ in 0..count {
            self.condvar.notify_one();
        }
    }

    /// Requests the worker threads to stop.
    pub fn stop(&self) {
        self.should_stop.store(true, Relaxed);
        self.condvar.notify_all();
    }

    /// Adds a result to the list of results.
    pub fn push_result(&self, pos: ChunkPos, entry: LoadedChunk) {
        self.results.lock().push((pos, entry));
    }

    /// Returns an iterator over the results that were received by the [`TaskPool`].
    #[inline]
    pub fn fetch_results(&self) -> Results<'_> {
        Results(self.results.lock())
    }
}

impl Drop for TaskPool {
    #[inline]
    fn drop(&mut self) {
        self.stop();
    }
}

/// An iterator over the results that were received by a [`TaskPool`].
struct Results<'a>(MutexGuard<'a, Vec<(ChunkPos, LoadedChunk)>>);

impl<'a> Iterator for Results<'a> {
    type Item = (ChunkPos, LoadedChunk);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.len(), Some(self.0.len()))
    }
}

impl<'a> ExactSizeIterator for Results<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

/// Contains a dynamic collection chunks.
pub struct World {
    /// The list of chunks that are currently loaded in memory.
    chunks: Chunks,
    /// The task pool used to generate new chunks.
    task_pool: Arc<TaskPool>,
    /// The context used to build chunks.
    chunk_build_context: ChunkBuildContext,
}

impl World {
    /// Creates a new [`World`] that uses the provided [`WorldGenerator`] to generate chunks.
    pub fn new<W: 'static + WorldGenerator>(gpu: Arc<Gpu>, generator: W) -> Self {
        let task_pool = Arc::new(TaskPool::new());

        for _ in 0..num_cpus::get().saturating_sub(3).max(1) {
            let task_pool = task_pool.clone();
            let mut generator = generator.clone();
            let gpu = gpu.clone();
            std::thread::spawn(move || {
                let mut build_context = ChunkBuildContext::new(gpu);
                while let Some(task) = task_pool.fetch_task() {
                    let chunk = generator.generate(task.pos);
                    let mut entry = LoadedChunk::new(chunk);

                    // Build the inner geometry of the chunk.
                    build_context.reset();
                    build_context.build_inner(&entry.data);
                    build_context.append_to(&mut entry.geometry);
                    entry.dirty_flags.remove(DirtyFlags::INNER_DIRTY);

                    task_pool.push_result(task.pos, entry);
                }
            });
        }

        Self {
            task_pool,
            chunks: Chunks::default(),
            chunk_build_context: ChunkBuildContext::new(gpu),
        }
    }

    /// Returns the number of chunks that are currently being generated.
    #[inline]
    pub fn chunks_in_flight(&self) -> usize {
        self.task_pool.task_count()
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

        // Get the chunks that have been received so far.
        self.chunks.extend(
            self.task_pool
                .fetch_results()
                .map(|(pos, c)| (pos, ChunkEntry::Loaded(c))),
        );

        let entry = match self.chunks.entry(pos) {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => {
                self.task_pool.push_task(pos, priority);
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
                .push_tasks(to_request.iter().map(|&pos| (pos, priority)));
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
