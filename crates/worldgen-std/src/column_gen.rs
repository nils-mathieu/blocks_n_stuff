use std::hash::BuildHasherDefault;
use std::ops::{Index, IndexMut};
use std::sync::{Arc, OnceLock};

use bns_core::{Chunk, LocalPos};
use bns_rng::Noise;
use bytemuck::Zeroable;
use glam::IVec2;
use hashbrown::HashMap;
use parking_lot::RwLock;
use rustc_hash::FxHasher;

use crate::biome::BiomeId;
use crate::GenCtx;

/// The size of a chunk column side.
const COLUMN_SIDE: i32 = Chunk::SIDE;
/// The total number of columns in a chunk column.
const COLUMN_SIZE: usize = (COLUMN_SIDE * COLUMN_SIDE) as usize;

/// In each column, the number of samples taken from earby biomes to determine the height map
/// value.
///
/// Samples will be taken in randomly around the value, in a square of size
/// `HEIGHT_MAP_SAMPLE_DISPERSE`.
const HEIGHT_MAP_SAMPLE_COUNT: i32 = 16;

/// The maximum displacement of a heightmap sample from the center of the sampled square.
///
/// # Note
///
/// The total displacement will be in the range `[-HEIGHT_MAP_SAMPLE_DISPERSE / 2, HEIGHT_MAP_SAMPLE_DISPERSE / 2]`.
const HEIGHT_MAP_SAMPLE_DISPERSE: i32 = 48;

/// The interpolation granularity of the height map.
const HEIGHT_MAP_GRANULARITY: i32 = 8;

/// A simple wrapper around an array of size [`COLUMN_SIZE`] that allows unchecked access to it
/// using the [`ColumnPos`] type.
#[derive(Debug, Clone, Copy, Zeroable)]
pub struct ColumnStore<T>([T; COLUMN_SIZE]);

impl<T> ColumnStore<T> {
    /// Creates a new [`ColumnStore`] with the provided value.
    #[inline]
    pub const fn new(val: T) -> Self
    where
        T: Copy,
    {
        Self([val; COLUMN_SIZE])
    }
}

impl<T> Index<ColumnPos> for ColumnStore<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: ColumnPos) -> &Self::Output {
        unsafe { self.0.get_unchecked(index.index()) }
    }
}

impl<T> IndexMut<ColumnPos> for ColumnStore<T> {
    #[inline]
    fn index_mut(&mut self, index: ColumnPos) -> &mut Self::Output {
        unsafe { self.0.get_unchecked_mut(index.index()) }
    }
}

/// A position within a column.
///
/// This is guaranteed to contain an index that is less than [`COLUMN_SIZE`].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct ColumnPos(u16);

impl ColumnPos {
    /// Computes the local [`ColumnPos`] of the provided world position.
    pub fn from_world_pos(pos: IVec2) -> Self {
        let x = pos.x.rem_euclid(COLUMN_SIDE);
        let z = pos.y.rem_euclid(COLUMN_SIDE);
        let index = x + z * COLUMN_SIDE;
        Self(index as u16)
    }

    /// Creates a new [`ColumnPos`] from the provided local position.
    #[inline]
    pub fn from_local_pos(pos: LocalPos) -> Self {
        let index = pos.x() + pos.z() * Chunk::SIDE;
        Self(index as u16)
    }

    /// Returns the index value for that position.
    #[inline]
    pub fn index(self) -> usize {
        self.0 as usize
    }

    /// Returns an iterator over all possible [`ColumnPos`] values.
    #[inline]
    pub fn iter_all() -> impl Clone + ExactSizeIterator<Item = Self> {
        (0..COLUMN_SIZE as u16).map(Self)
    }

    /// Returns the X coordinate of the position.
    #[inline]
    pub fn x(self) -> i32 {
        self.index() as i32 % COLUMN_SIDE
    }

    /// Returns the Z coordinate of the position.
    #[inline]
    pub fn z(self) -> i32 {
        self.index() as i32 / COLUMN_SIDE
    }

    /// Adds the provided value to the X coordinate of the position.
    ///
    /// # Safety
    ///
    /// This function does not check whether the final X coordinate is less than [`COLUMN_SIDE`].
    #[inline]
    pub unsafe fn add_x_unchecked(self, x: i32) -> Self {
        debug_assert!(self.x() + x < COLUMN_SIDE);
        Self(self.0.wrapping_add(x as u16))
    }

    /// Adds the provided value to the Z coordinate of the position.
    ///
    /// # Safety
    ///
    /// This function does not check whether the final Z coordinate is less than [`COLUMN_SIDE`].
    #[inline]
    pub unsafe fn add_z_unchecked(self, z: i32) -> Self {
        debug_assert!(self.z() + z < COLUMN_SIDE);
        Self(self.0.wrapping_add((z * COLUMN_SIDE) as u16))
    }

    /// Returns a [`IVec2`] containing the X and Z coordinates of the position.
    #[inline]
    pub fn to_vec2(self) -> IVec2 {
        IVec2::new(self.x(), self.z())
    }
}

/// Caches (potentially partial) information about a particular column of chunks.
pub struct ColumnGen {
    /// The position of the column.
    pos: IVec2,
    /// The ID of the biomes generated in the chunk.
    biome_map: OnceLock<ColumnStore<BiomeId>>,
    /// The heightmap of the chunk at that particular position.
    height_map: OnceLock<ColumnStore<f32>>,
}

impl ColumnGen {
    /// Returns a new empty instance of [`ColumnGen`], initialized with unspecified values.
    #[inline]
    pub fn new(pos: IVec2) -> Self {
        Self {
            pos,
            biome_map: OnceLock::new(),
            height_map: OnceLock::new(),
        }
    }

    /// Gets the biome map of the column, or initializes it if it's not present.
    pub fn biome_map(&self, ctx: &GenCtx) -> &ColumnStore<BiomeId> {
        self.biome_map.get_or_init(|| {
            let mut ret = ColumnStore::new(BiomeId::Void);

            for pos in ColumnPos::iter_all() {
                let world_pos = self.pos * Chunk::SIDE + pos.to_vec2();
                ret[pos] = ctx.biomes.sample(world_pos, &ctx.biome_registry);
            }

            ret
        })
    }

    /// Gets the height map of the column, or initializes it if it's not present.
    pub fn height_map(&self, ctx: &GenCtx) -> &ColumnStore<f32> {
        self.height_map.get_or_init(|| {
            let mut ret = ColumnStore::new(0.0);

            let chunk_origin = self.pos * Chunk::SIDE;

            // Computes the heightmap contribution at the provided value.
            // The result of the function is then used to interpolate between four distinct values.
            let heightmap_contribution = |world_pos: IVec2| {
                let mut height = 0.0; // cumulative height so far
                let mut weight = 0.0; // cumulative weight so far, used to normalize the height value at the end
                let mut noise_index = 0; // index within the `ctx.heightmap_noises` array
                let mut next_sampled_pos = [world_pos.x as u64, world_pos.y as u64]; // next position to feed to the noise.
                for _ in 0..HEIGHT_MAP_SAMPLE_COUNT {
                    // Compute the displacement value for the next sample.
                    let displacement_x = ctx.heightmap_noises[noise_index].sample(next_sampled_pos);
                    noise_index = (noise_index + 1) % ctx.heightmap_noises.len();
                    let displacement_y = ctx.heightmap_noises[noise_index].sample(next_sampled_pos);
                    noise_index = (noise_index + 1) % ctx.heightmap_noises.len();

                    let displacement = IVec2::new(
                        (displacement_x % HEIGHT_MAP_SAMPLE_DISPERSE as u64) as i32
                            - HEIGHT_MAP_SAMPLE_DISPERSE / 2,
                        (displacement_y % HEIGHT_MAP_SAMPLE_DISPERSE as u64) as i32
                            - HEIGHT_MAP_SAMPLE_DISPERSE / 2,
                    );

                    let sampled_pos = world_pos + displacement;
                    next_sampled_pos = [sampled_pos.x as u64, sampled_pos.y as u64];

                    let sampled_chunk = IVec2::new(
                        sampled_pos.x.div_euclid(Chunk::SIDE),
                        sampled_pos.y.div_euclid(Chunk::SIDE),
                    );

                    // Take the height value for that position.
                    // If the position is outside of the current column, we need to query it.
                    // Otherwise, we save a lookup by simply using the value from the current
                    // column.
                    let biome = if sampled_chunk != self.pos {
                        // We need to query the heightmap of the column at that position.
                        let col = ctx.columns.get(sampled_chunk);
                        col.biome_map(ctx)[ColumnPos::from_world_pos(sampled_pos)]
                    } else {
                        self.biome_map(ctx)[ColumnPos::from_world_pos(sampled_pos)]
                    };

                    // Compute the weight from the distance between the sampled position and the
                    // current position.
                    // The farther the sampled point, the less weight it has.
                    let sq_dist = sampled_pos.distance_squared(world_pos);
                    let w = 1.0 / (sq_dist as f32 + 1.0);

                    height += w * ctx.biome_registry[biome].implementation.height(sampled_pos);
                    weight += w;
                }

                height / weight
            };

            for pos in ColumnPos::iter_all() {
                let world_pos = chunk_origin + pos.to_vec2();

                let c00 = world_pos.div_euclid(IVec2::splat(HEIGHT_MAP_GRANULARITY));
                let c10 = c00 + IVec2::new(1, 0);
                let c01 = c00 + IVec2::new(0, 1);
                let c11 = c00 + IVec2::new(1, 1);

                let h00 = heightmap_contribution(c00 * HEIGHT_MAP_GRANULARITY);
                let h10 = heightmap_contribution(c10 * HEIGHT_MAP_GRANULARITY);
                let h01 = heightmap_contribution(c01 * HEIGHT_MAP_GRANULARITY);
                let h11 = heightmap_contribution(c11 * HEIGHT_MAP_GRANULARITY);

                let x = world_pos.x.rem_euclid(HEIGHT_MAP_GRANULARITY) as f32
                    * (1.0 / HEIGHT_MAP_GRANULARITY as f32);
                let z = world_pos.y.rem_euclid(HEIGHT_MAP_GRANULARITY) as f32
                    * (1.0 / HEIGHT_MAP_GRANULARITY as f32);

                #[inline]
                fn interpolate(a: f32, b: f32, x: f32) -> f32 {
                    // a * (1.0 - x) + b * x

                    let x2 = x * x;
                    let x3 = x2 * x;
                    let f = 3.0 * x2 - 2.0 * x3;
                    a * (1.0 - f) + b * f
                }

                ret[pos] = interpolate(interpolate(h00, h10, x), interpolate(h01, h11, x), z);
            }

            ret
        })
    }
}

/// A collection of [`ColumnGen`] instances, which can be retrieved when needed.
#[derive(Default)]
pub struct Columns {
    /// The columns that have been generated so far.
    columns: RwLock<HashMap<IVec2, Arc<ColumnGen>, BuildHasherDefault<FxHasher>>>,
}

impl Columns {
    /// Attempt to get a [`ColumnGen`] instance from the cache, or create a new one if it's not
    /// present.
    pub fn get(&self, pos: IVec2) -> Arc<ColumnGen> {
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
}
