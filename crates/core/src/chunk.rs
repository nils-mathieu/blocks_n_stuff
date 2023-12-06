use std::fmt::Debug;
use std::hash::Hash;
use std::mem::MaybeUninit;
use std::ops::{Index, IndexMut};

use bytemuck::Zeroable;
use glam::{IVec2, IVec3, Vec3};

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
    #[inline]
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
    pub fn set_block(&mut self, pos: LocalPos, block: InstanciatedBlock) {
        unsafe {
            if block.id() != BlockId::Air {
                *self.get_block_mut(pos) = block.id();
            }
            if block.id().info().appearance.has_metadata() {
                *self.get_appearance_mut(pos) = block.appearance();
            }
        }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPos {
    /// The X coordinate of the chunk.
    pub x: i32,
    /// The Y coordinate of the chunk.
    pub y: i32,
    /// The Z coordinate of the chunk.
    pub z: i32,
}

impl ChunkPos {
    /// Creates a new [`ChunkPos`] from the provided coordinates.
    #[inline]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Converts the provided world-space position into a chunk position.
    #[inline]
    pub const fn from_world_pos_i(pos: IVec3) -> Self {
        Self {
            x: pos.x.div_euclid(Chunk::SIDE),
            y: pos.y.div_euclid(Chunk::SIDE),
            z: pos.z.div_euclid(Chunk::SIDE),
        }
    }

    /// Converts the provided world-space position into a chunk position.
    #[inline]
    pub fn from_world_pos(pos: Vec3) -> Self {
        fn coord_to_chunk(coord: f32) -> i32 {
            if coord >= 0.0 {
                coord as i32 / Chunk::SIDE
            } else {
                coord as i32 / Chunk::SIDE - 1
            }
        }

        ChunkPos {
            x: coord_to_chunk(pos.x),
            y: coord_to_chunk(pos.y),
            z: coord_to_chunk(pos.z),
        }
    }

    /// Returns the world-space origin of the chunk.
    #[inline]
    pub const fn origin(self) -> IVec3 {
        IVec3::new(
            self.x * Chunk::SIDE,
            self.y * Chunk::SIDE,
            self.z * Chunk::SIDE,
        )
    }

    /// Returns a 2D vector that contains the X and Z coordinates of the chunk.
    #[inline]
    pub const fn xz(self) -> IVec2 {
        IVec2::new(self.x, self.z)
    }

    /// Returns the chunk position as an [`IVec3`].
    #[inline]
    pub fn as_ivec3(self) -> IVec3 {
        IVec3::new(self.x, self.y, self.z)
    }

    /// Returns the chunk position as a [`Vec3`].
    #[inline]
    pub fn as_vec3(self) -> Vec3 {
        self.as_ivec3().as_vec3()
    }

    /// Computes the squared distance between two chunk positions.
    #[inline]
    pub fn distance_squared(self, other: Self) -> i32 {
        self.as_ivec3().distance_squared(other.as_ivec3())
    }

    /// If the provided `world_pos` is part of the chunk with this position, returns its
    /// local position within that chunk.
    pub fn checked_local_pos(self, world_pos: IVec3) -> Option<LocalPos> {
        let origin = self.origin();

        let x = world_pos.x.checked_sub(origin.x)?;
        let y = world_pos.y.checked_sub(origin.y)?;
        let z = world_pos.z.checked_sub(origin.z)?;

        if x >= Chunk::SIDE || y >= Chunk::SIDE || z >= Chunk::SIDE {
            return None;
        }

        // SAFETY:
        //  We just made sure that the coordinates were in bounds.
        Some(unsafe { LocalPos::from_xyz_unchecked(x, y, z) })
    }
}

impl Hash for ChunkPos {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        #[cfg(target_pointer_width = "64")]
        {
            state.write_usize((self.x as usize) << 32 | self.y as usize);
            state.write_i32(self.z);
        }

        #[cfg(target_pointer_width = "32")]
        {
            self.x.hash(state);
            self.y.hash(state);
            self.z.hash(state);
        }
    }
}

impl std::ops::Add<IVec3> for ChunkPos {
    type Output = Self;

    #[inline]
    fn add(self, rhs: IVec3) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl std::ops::AddAssign<IVec3> for ChunkPos {
    #[inline]
    fn add_assign(&mut self, rhs: IVec3) {
        *self = *self + rhs;
    }
}

impl std::ops::Sub<IVec3> for ChunkPos {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: IVec3) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl std::ops::SubAssign<IVec3> for ChunkPos {
    #[inline]
    fn sub_assign(&mut self, rhs: IVec3) {
        *self = *self - rhs;
    }
}

/// Creates a new uninitialized [`ChunkStore`] of `T`s.
fn new_uninit_store<T>() -> Box<ChunkStore<MaybeUninit<T>>> {
    let layout = std::alloc::Layout::new::<ChunkStore<MaybeUninit<T>>>();
    let ptr = unsafe { std::alloc::alloc_zeroed(layout) };
    if ptr.is_null() {
        std::alloc::handle_alloc_error(layout);
    }
    unsafe { Box::from_raw(ptr as *mut ChunkStore<MaybeUninit<T>>) }
}
