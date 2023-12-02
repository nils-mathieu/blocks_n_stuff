use bytemuck::Contiguous;

/// The ID of a texture.
///
/// This enumeration defines a list of indices that are valid for the global texture
/// atlas.
///
/// # Remarks
///
/// Just like [`BlockId`], this type will need to be replaced if we want to support modding
/// and custom blocks.
///
/// [`BlockId`]: crate::BlockId
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Contiguous)]
#[repr(u8)]
pub enum TextureId {
    Stone,
    Dirt,
    GrassSide,
    GrassTop,
    Andesite,
    Clay,
    Diorite,
    Granite,
    Gravel,
    PodzolTop,
    PodzolSide,
    RedSand,
    Sand,
    SandstoneTop,
    SandstoneBottom,
    SandstoneSide,
    RedSandstoneTop,
    RedSandstoneBottom,
    RedSandstoneSide,
    Water,
    Bedrock,
}

impl TextureId {
    /// The total number of [`TextureId`] instances.
    pub const COUNT: usize = <Self as Contiguous>::MAX_VALUE as usize + 1;

    /// Creates a new [`TextureId`] from the given index.
    ///
    /// # Safety
    ///
    /// The index must be less than [`TextureId::COUNT`].
    #[inline]
    pub const unsafe fn from_index_unchecked(index: usize) -> Self {
        std::mem::transmute(index as u8)
    }

    /// Returns the file name that the image representing this texture should have.
    ///
    /// # Remarks
    ///
    /// The extension of the file name is not included.
    #[inline]
    pub fn file_name(self) -> &'static str {
        const NAMES: [&str; TextureId::COUNT] = [
            "stone",
            "dirt",
            "grass_side",
            "grass_top",
            "andesite",
            "clay",
            "diorite",
            "granite",
            "gravel",
            "podzol_top",
            "podzol_side",
            "red_sand",
            "sand",
            "sandstone_top",
            "sandstone_bottom",
            "sandstone_side",
            "red_sandstone_top",
            "red_sandstone_bottom",
            "red_sandstone_side",
            "water",
            "bedrock",
        ];
        unsafe { NAMES.get_unchecked(self as usize) }
    }

    /// Returns an iterator over all the [`TextureId`] instances.
    #[inline]
    pub fn all() -> impl Clone + ExactSizeIterator<Item = Self> {
        unsafe { (0..Self::COUNT).map(|x| Self::from_index_unchecked(x)) }
    }
}
