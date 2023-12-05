use std::fmt::Debug;
use std::mem::MaybeUninit;
use std::ops::{Index, IndexMut};

use bytemuck::Zeroable;
use glam::IVec3;

use crate::{AppearanceMetadata, BlockId, InstanciatedBlock};

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
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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

    /// Creates a new [`LocalPos`] from the given world position.
    #[inline]
    pub fn from_world_pos(pos: IVec3) -> Self {
        let x = pos.x.div_euclid(pos.x);
        let y = pos.y.div_euclid(pos.y);
        let z = pos.z.div_euclid(pos.z);
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
    pub fn x(self) -> i32 {
        (self.0 & X_MASK) as _
    }

    /// Returns the Y coordinate of the position.
    #[inline]
    pub fn y(self) -> i32 {
        ((self.0 & Y_MASK) >> 5) as _
    }

    /// Returns the Z coordinate of the position.
    #[inline]
    pub fn z(self) -> i32 {
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
    pub fn iter_column(mut self) -> impl Iterator<Item = Self> {
        const S: u16 = Chunk::SIDE as u16;

        self.clear_y();
        (0u16..S).map(move |y| Self(self.0 + y * S))
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
    pub fn index(self) -> usize {
        self.0 as usize
    }

    /// Adds the provided value to the X coordinate of the position.
    ///
    /// # Safety
    ///
    /// The final X coordinate must be less than [`Chunk::SIDE`].
    #[inline]
    pub unsafe fn add_x_unchecked(self, x: i32) -> Self {
        Self(self.0.wrapping_add(x as u16))
    }

    /// Adds the provided value to the Y coordinate of the position.
    ///
    /// # Safety
    ///
    /// The final Y coordinate must be less than [`Chunk::SIDE`].
    #[inline]
    pub unsafe fn add_y_unchecked(self, y: i32) -> Self {
        Self(self.0.wrapping_add((y as u16) << 5))
    }

    /// Adds the provided value to the Z coordinate of the position.
    ///
    /// # Safety
    ///
    /// The final Z coordinate must be less than [`Chunk::SIDE`].
    #[inline]
    pub unsafe fn add_z_unchecked(self, z: i32) -> Self {
        Self(self.0.wrapping_add((z as u16) << 10))
    }

    /// Adds the provided offset to the position.
    ///
    /// # Safety
    ///
    /// The final coordinates must all be less than [`Chunk::SIDE`].
    #[inline]
    pub unsafe fn add_unchecked(self, offset: IVec3) -> Self {
        let offset = offset.x + (offset.y << 5) + (offset.z << 10);
        Self(self.0.wrapping_add(offset as u16))
    }
}

impl Debug for LocalPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalPos")
            .field("x", &self.x())
            .field("y", &self.y())
            .field("z", &self.z())
            .finish()
    }
}

/// A simple wrapper around a static array that can be indexed with a [`LocalPos`] with
/// no bound checking.
#[derive(Clone, Copy, Hash, Zeroable)]
#[repr(transparent)]
struct ChunkStore<T>([T; Chunk::SIZE]);

impl<T> Index<LocalPos> for ChunkStore<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: LocalPos) -> &Self::Output {
        unsafe { self.0.get_unchecked(index.index()) }
    }
}

impl<T> IndexMut<LocalPos> for ChunkStore<T> {
    #[inline]
    fn index_mut(&mut self, index: LocalPos) -> &mut Self::Output {
        unsafe { self.0.get_unchecked_mut(index.index()) }
    }
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
    /// The inner blocks of the chunk.
    blocks: Option<Box<ChunkStore<BlockId>>>,
    /// Metadata about the chunk's appearance.
    ///
    /// # Note
    ///
    /// I'm not sure if the `MaybeUninit` really is necessary for soundness as `AppearanceMetadata`
    /// already is an union with a zero-sized field. But because it's a lang item, rustc might do
    /// something special with it.
    appearances: Option<Box<ChunkStore<MaybeUninit<AppearanceMetadata>>>>,
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
        Self {
            blocks: None,
            appearances: None,
        }
    }

    /// Returns the block at the provided position.
    #[inline]
    pub fn get_block(&self, pos: LocalPos) -> BlockId {
        match &self.blocks {
            Some(data) => data[pos],
            None => BlockId::Air,
        }
    }

    /// Returns the [`AppearanceMetadata`] of the block at the provided position.
    pub fn get_appearance(&self, pos: LocalPos) -> AppearanceMetadata {
        match &self.appearances {
            // SAFETY:
            //  An `AppearanceMetadata` instance is always initialized, even if it's because of a
            //  zero-sized field in the union.
            Some(data) => unsafe { data[pos].assume_init() },
            None => AppearanceMetadata { no_metadata: () },
        }
    }

    /// Returns an [`InstanciatedBlock`] for the block at the provided position.
    #[inline]
    pub fn get_instanciated_block(&self, pos: LocalPos) -> InstanciatedBlock {
        // SAFETY:
        //  We know that a chunk always contain valid blocks and associated metadata.
        unsafe { InstanciatedBlock::new_unchecked(self.get_block(pos), self.get_appearance(pos)) }
    }

    /// Returns a mutable reference to the block at the provided position.
    ///
    /// # Remarks
    ///
    /// This function forces the chunk to allocate its data if it was empty. If you know
    /// that the value you're trying to insert is [`BlockId::Air`], you should skip calling
    /// the function.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it allows the caller to change the block at the provided
    /// location without updating related metadata.
    ///
    /// If the inserted block requires additional metadata, it must be added manually.
    #[inline]
    pub unsafe fn get_block_mut(&mut self, pos: LocalPos) -> &mut BlockId {
        &mut self.blocks.get_or_insert_with(bytemuck::zeroed_box)[pos]
    }

    /// Sets the block at the provided position.
    ///
    /// # Safety
    ///
    /// This function assumes that the inserted block requires no additional metadata to be
    /// valid.
    ///
    /// If the inserted block requires additional metadata, the [`Chunk::get_block_mut`] function
    /// msut be used directly.
    ///
    /// # Panics
    ///
    /// In debug builds, this function panics if `block` requires some metadata.
    #[inline]
    pub unsafe fn set_block_unchecked(&mut self, pos: LocalPos, block: BlockId) {
        if block == BlockId::Air {
            return;
        }

        debug_assert!(!block.info().appearance.has_metadata());

        *self.get_block_mut(pos) = block;
    }

    /// Sets a block at the provided position.
    ///
    /// # Panics
    ///
    /// This function panics if `block` requires some metadata to be complete.
    #[inline]
    pub fn set_block(&mut self, pos: LocalPos, block: BlockId) {
        if block == BlockId::Air && self.blocks.is_none() {
            return;
        }

        assert!(!block.info().appearance.has_metadata());

        unsafe { *self.get_block_mut(pos) = block };
    }

    /// Returns a mutable reference to the [`AppearanceMetadata`] of the block at the provided
    /// position.
    ///
    /// # Remarks
    ///
    /// This function forces the chunk to allocate its data if it was empty. If you know
    /// that the value you're trying to insert represents no metadata, you should skip calling
    /// the function.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it allows the caller to change the metadata of an existing
    /// block without updating the block itself.
    #[inline]
    pub unsafe fn get_appearance_mut(&mut self, pos: LocalPos) -> &mut AppearanceMetadata {
        self.appearances.get_or_insert_with(new_uninit_store)[pos].assume_init_mut()
    }

    /// Returns whether the chunk is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        match self.blocks.as_ref() {
            Some(data) => data.0.iter().all(|&id| id == BlockId::Air),
            None => true,
        }
    }
}

/// The 3D position of a chunk in the world.
pub type ChunkPos = IVec3;

/// Creates a new uninitialized [`ChunkStore`] of `T`s.
fn new_uninit_store<T>() -> Box<ChunkStore<MaybeUninit<T>>> {
    let layout = std::alloc::Layout::new::<ChunkStore<MaybeUninit<T>>>();
    let ptr = unsafe { std::alloc::alloc_zeroed(layout) };
    if ptr.is_null() {
        std::alloc::handle_alloc_error(layout);
    }
    unsafe { Box::from_raw(ptr as *mut ChunkStore<MaybeUninit<T>>) }
}
