use bytemuck::Zeroable;
use glam::IVec3;

use crate::BlockId;

const X_MASK: u16 = 0b11111;
const Y_MASK: u16 = 0b11111 << 5;
const Z_MASK: u16 = 0b11111 << 10;

/// A local block position within a [`Chunk`].
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
    #[inline]
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

    /// Returns the position as a [`IVec3`].
    #[inline]
    pub fn to_ivec3(self) -> IVec3 {
        IVec3::new(self.x(), self.y(), self.z())
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

/// The inner chunk data.
///
/// This type is ***huge*** and should basically never be instanciated on the stack.
#[derive(Zeroable)]
struct ChunkData {
    blocks: [BlockId; Chunk::SIZE],
}

/// Represents the content of a chunk.
///
/// # Remarks
///
/// Because this crate is meant to be used in both the client and the server, this type *does not*
/// include built geometry information or other metadata that can be derived from the chunk data
/// itself.
///
/// Those should instead be stored in a separate structure defined in downstream crates.
pub struct Chunk {
    /// The inner data of the chunk.
    ///
    /// This `Option<T>` is set to [`None`] when the chunk is empty to avoid allocating a huge
    /// chunk of memory for nothing.
    data: Option<Box<ChunkData>>,
}

impl Chunk {
    /// The side-length of a chunk, in blocks.
    ///
    /// The total size of a chunk is the cube of this value.
    pub const SIDE: i32 = 32;

    /// The total size of a chunk, in blocks.
    ///
    /// This is equal to `SIDE * SIDE * SIDE`.
    pub const SIZE: usize = (Self::SIDE * Self::SIDE * Self::SIDE) as usize;

    /// Creates a new [`Chunk`] instance with the provided data.
    #[inline]
    pub fn empty() -> Self {
        Self { data: None }
    }

    /// Returns the block at the provided position.
    #[inline]
    pub fn get_block(&self, pos: LocalPos) -> BlockId {
        match &self.data {
            Some(data) => unsafe { *data.blocks.get_unchecked(pos.index()) },
            None => BlockId::Air,
        }
    }

    /// Returns a mutable reference to the block at the provided position.
    ///
    /// # Remarks
    ///
    /// This function forces the chunk to allocate its data if it was empty. If you know
    /// that the value you're trying to insert is [`BlockId::Air`], you should skip calling
    /// the function.
    #[inline]
    pub fn get_block_mut(&mut self, pos: LocalPos) -> &mut BlockId {
        let data = self.data.get_or_insert_with(bytemuck::zeroed_box);
        unsafe { data.blocks.get_unchecked_mut(pos.index()) }
    }
}
