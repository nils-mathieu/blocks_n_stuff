use std::hash::BuildHasherDefault;
use std::sync::Arc;

use bns_core::ChunkPos;
use glam::IVec2;
use hashbrown::HashMap;
use parking_lot::RwLock;
use rustc_hash::FxHasher;

use crate::chunk_gen::ChunkGen;
use crate::column_gen::ColumnGen;

/// A collection of [`ColumnGen`] instances, which can be retrieved when needed.
#[derive(Default)]
pub struct Cache {
    /// The columns that have been generated so far.
    columns: RwLock<HashMap<IVec2, Arc<ColumnGen>, BuildHasherDefault<FxHasher>>>,
    /// The chunks that have been generated so far.
    chunks: RwLock<HashMap<ChunkPos, Arc<ChunkGen>, BuildHasherDefault<FxHasher>>>,
}

impl Cache {
    /// Attempt to get a [`ColumnGen`] instance from the cache, or create a new one if it's not
    /// present.
    pub fn get_column(&self, pos: IVec2) -> Arc<ColumnGen> {
        // Try to get the column from the cache.
        let lock = self.columns.read();

        // Try to get the column from the cache.
        if let Some(col) = lock.get(&pos) {
            return col.clone();
        }

        // We do not have the column in cache.
        // We have to write to the map.
        drop(lock);

        // `entry` is necessary here, because we might have raced with another thread to
        // initialize the column.
        self.columns
            .write()
            .entry(pos)
            .or_insert_with(|| Arc::new(ColumnGen::new(pos)))
            .clone()
    }

    pub fn get_chunk(&self, pos: ChunkPos) -> Arc<ChunkGen> {
        // Try to get the chunk from the cache.
        let lock = self.chunks.read();

        // Try to get the chunk from the cache.
        if let Some(chunk) = lock.get(&pos) {
            return chunk.clone();
        }

        // We do not have the chunk in cache.
        // We have to write to the map.
        drop(lock);

        // `entry` is necessary here, because we might have raced with another thread to
        // initialize the chunk.
        self.chunks
            .write()
            .entry(pos)
            .or_insert_with(|| Arc::new(ChunkGen::new(pos)))
            .clone()
    }

    /// Hints the collection that some columns are unlikely to be used anymore, and can therefor
    /// be unloaded.
    #[profiling::function]
    pub fn request_cleanup(&self, center: ChunkPos, h_radius: u32, v_radius: u32) {
        {
            let mut guard = self.columns.write();
            guard.retain(|pos, _| pos.distance_squared(center.xz()) as u32 <= h_radius * h_radius);
            guard.shrink_to(h_radius as usize * v_radius as usize);
        }

        {
            let mut guard = self.chunks.write();
            guard.retain(|pos, _| {
                pos.xz().distance_squared(center.xz()) as u32 <= h_radius * h_radius
                    || (pos.y - center.y).unsigned_abs() <= v_radius
            });
            guard.shrink_to(h_radius as usize * h_radius as usize * v_radius as usize);
        }
    }
}
