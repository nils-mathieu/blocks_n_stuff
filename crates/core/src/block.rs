use bytemuck::{Contiguous, Zeroable};

/// A block identifier.
///
/// This enumeration defines what blocks are authorized to exist in a game world.
///
/// # Remarks
///
/// If, in the future, we need to support modding and custom blocks, we will need to remove this
/// type in favor of a more flexible system.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Contiguous)]
#[repr(u8)]
pub enum BlockId {
    #[default]
    Air,
    Stone,
}

// SAFETY:
//  The block with ID 0 is `BlockId::Air`, which is valid.
unsafe impl Zeroable for BlockId {}

impl BlockId {
    /// The total number of [`BlockId`] instances.
    pub const COUNT: usize = <Self as Contiguous>::MAX_VALUE as usize + 1;

    /// Returns the [`BlockInfo`] instance associated with this [`BlockId`].
    ///
    /// # Remarks
    ///
    /// Currently, this function takes the [`BlockInfo`] instances from a static array. If in the
    /// future we need to load custom textures (with potentially different appearances), we will
    /// need to load this dynamically and store it somewhere.
    #[inline]
    pub fn info(self) -> &'static BlockInfo {
        const INFOS: [BlockInfo; BlockId::COUNT] = [
            // Air
            BlockInfo {
                appearance: BlockAppearance::Invisible,
            },
            // Stone
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::Stone),
            },
        ];

        unsafe { INFOS.get_unchecked(self as usize) }
    }
}

/// The ID of a texture.
///
/// This enumeration defines a list of indices that are valid for the global texture
/// atlas.
///
/// # Remarks
///
/// Just like [`BlockId`], this type will need to be replaced if we want to support modding
/// and custom blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Contiguous)]
#[repr(u8)]
pub enum TextureId {
    Stone,
}

/// Describes the appearance of a block.
pub enum BlockAppearance {
    /// The block is invisible.
    ///
    /// It should not be included in the geometry of the chunk that contains
    /// it.
    Invisible,

    /// The block has a regular appearance, with separate textures for each
    /// face.
    ///
    /// The helper function [`uniform`] can be used to create a [`BlockAppearance`]
    /// instance with the same texture for all faces.
    Regular {
        top: TextureId,
        bottom: TextureId,
        side: TextureId,
    },
}

impl BlockAppearance {
    /// Creates a [`BlockAppearance`] instance with the same texture for all faces.
    #[inline]
    pub const fn uniform(texture: TextureId) -> Self {
        Self::Regular {
            top: texture,
            bottom: texture,
            side: texture,
        }
    }
}

/// Stores static information about a block.
///
/// An instance of this type can be obtained by calling the [`info`] method of a
/// [`BlockId`].
///
/// [`info`]: BlockId::info
pub struct BlockInfo {
    /// Describes the appearance of a block.
    pub appearance: BlockAppearance,
}
