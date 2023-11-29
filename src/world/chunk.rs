use std::ops::{Index, IndexMut};

use bytemuck::Zeroable;

use super::{BlockId, ChunkGeometry};

const X_MASK: u16 = 0b11111;
const Y_MASK: u16 = 0b11111 << 5;
const Z_MASK: u16 = 0b11111 << 10;

/// A local block position within a chunk.
///
/// # Representation
///
/// Internally, this type is represented by a single index that is guaranteed to be less than
/// [`Chunk::SIZE`].
///
/// The formula to convert between a local position and its index is:
///
/// ```rust
/// index = x + y * Chunk::SIDE + z * Chunk::SIDE * Chunk::SIDE
/// ```
// OPTIMIZE: depending of how we end up using this type, we could switch up the layout of chunks
// to improve cache locality. If most of our iterations are done on the Y axis first, we could
// store the Y coordinate first in the index, then the X coordinate, and finally the Z coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalPos(u16);

impl LocalPos {
    /// Creates a new [`LocalPos`] from the given index.
    ///
    /// # Safety
    ///
    /// The index must be less than [`Chunk::SIZE`].
    #[inline]
    pub unsafe fn new_unchecked(index: usize) -> Self {
        Self(index as u16)
    }

    /// Creates a new [`LocalPos`] from the given coordinates without checking if they are
    /// in bounds.
    ///
    /// # Safety
    ///
    /// This function assumes that the coordinates are less than [`Chunk::SIDE`].
    pub unsafe fn from_xyz_unchecked(x: i32, y: i32, z: i32) -> Self {
        let index = x + y * Chunk::SIDE + z * Chunk::SIDE * Chunk::SIDE;
        Self::new_unchecked(index as usize)
    }

    /// Creates a new [`LocalPos`] from the given coordinates.
    ///
    /// # Panics
    ///
    /// This function panics if any of the provided coordinates are out of bounds.
    #[track_caller]
    pub fn from_xyz(x: i32, y: i32, z: i32) -> Self {
        assert!((0..Chunk::SIDE).contains(&x));
        assert!((0..Chunk::SIDE).contains(&y));
        assert!((0..Chunk::SIDE).contains(&z));
        unsafe { Self::from_xyz_unchecked(x, y, z) }
    }

    /// Clears the X coordinate of the position.
    #[inline]
    pub fn clear_x(&mut self) {
        self.0 &= !X_MASK;
    }

    /// Clears the Y coordinate of the position.
    #[inline]
    pub fn clear_y(&mut self) {
        self.0 &= !Y_MASK;
    }

    /// Clears the Z coordinate of the position.
    #[inline]
    pub fn clear_z(&mut self) {
        self.0 &= !Z_MASK;
    }

    /// Returns the X coordinate of the position.
    #[inline]
    pub fn x(&self) -> i32 {
        (self.0 & X_MASK) as _
    }

    /// Returns the Y coordinate of the position.
    #[inline]
    pub fn y(&self) -> i32 {
        ((self.0 & Y_MASK) >> 5) as _
    }

    /// Returns the Z coordinate of the position.
    #[inline]
    pub fn z(&self) -> i32 {
        ((self.0 & Z_MASK) >> 10) as _
    }

    /// Returns an iterator over all the [`LocalPos`] instances that have a Y coordinate equal to
    /// the provided one.
    ///
    /// # Panics
    ///
    /// This function panics if `y` is out of bounds.
    #[track_caller]
    pub fn iter_surface(y: i32) -> impl Iterator<Item = Self> {
        assert!((0..Chunk::SIDE).contains(&y));
        (0..Chunk::SIDE).flat_map(move |x| {
            (0..Chunk::SIDE).map(move |z| unsafe { Self::from_xyz_unchecked(x, y, z) })
        })
    }

    /// Returns an iterator over all the [`LocalPos`] instances in the column with the current
    /// Y coordinate.
    #[inline]
    pub fn iter_column(&self) -> impl Iterator<Item = Self> {
        const S: u16 = Chunk::SIDE as u16;

        let mut this = *self;
        this.clear_y();
        (0u16..S).map(move |y| Self(this.0 + y * S))
    }

    /// Returns an iterator over all the [`LocalPos`] instances in the chunk.
    #[inline]
    pub fn iter_all() -> impl Iterator<Item = Self> {
        Self::iter_surface(0).flat_map(|x| x.iter_column())
    }

    /// Returns the index of the block within the chunk.
    ///
    /// The returned index is guaranteed to be less than [`Chunk::SIZE`].
    #[inline]
    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

/// The data of a [`Chunk`], not including its built geometry and other metadata.
#[derive(Zeroable)]
pub struct ChunkData {
    /// The blocks in the chunk.
    pub blocks: [BlockId; Chunk::SIZE],
}

impl ChunkData {
    /// Creates a new, empty [`Chunk`].
    #[inline]
    pub fn empty() -> Box<Self> {
        bytemuck::zeroed_box()
    }
}

impl Index<LocalPos> for ChunkData {
    type Output = BlockId;

    #[inline]
    fn index(&self, index: LocalPos) -> &Self::Output {
        // SAFETY:
        //  By invariant, the index of a `LocalPos` is always in bounds of this
        //  array.
        unsafe { self.blocks.get_unchecked(index.index()) }
    }
}

impl IndexMut<LocalPos> for ChunkData {
    #[inline]
    fn index_mut(&mut self, index: LocalPos) -> &mut Self::Output {
        // SAFETY:
        //  By invariant, the index of a `LocalPos` is always in bounds of this
        //  array.
        unsafe { self.blocks.get_unchecked_mut(index.index()) }
    }
}

/// Stores the state of a chunk loaded in memory.
pub struct Chunk {
    // OPTIMIZE: use an option to avoid creating a [`ChunkData`] instance alltogether when
    // the chunk is empty to save memory. Maybe move the box within [`ChunkData`] itself then,
    // to make the API a bit more ergonomic.
    /// The data of the chunk.
    pub data: Box<ChunkData>,

    /// The geometry of the chunk.
    pub geometry: ChunkGeometry,

    /// Whether the geometry of the chunk is dirty and needs to be rebuilt.
    pub dirty: bool,
}

impl Chunk {
    /// The size of a chunk in a single dimension.
    pub const SIDE: i32 = 32;
    /// The total number of blocks in a chunk.
    pub const SIZE: usize = (Self::SIDE * Self::SIDE * Self::SIDE) as usize;

    /// Creates a new [`Chunk`] with the given data.
    pub fn new(data: Box<ChunkData>) -> Self {
        Self {
            data,
            geometry: ChunkGeometry::new(),
            dirty: true,
        }
    }
}
