use bytemuck::{Contiguous, Zeroable};

use crate::TextureId;

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
    Grass,
    Dirt,
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
                visibility: BlockVisibility::Invisible,
            },
            // Stone
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::Stone),
                visibility: BlockVisibility::Opaque,
            },
            // Grass
            BlockInfo {
                appearance: BlockAppearance::Regular {
                    top: TextureId::GrassTop,
                    bottom: TextureId::Dirt,
                    side: TextureId::GrassSide,
                },
                visibility: BlockVisibility::Opaque,
            },
            // Dirt
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::Dirt),
                visibility: BlockVisibility::Opaque,
            },
        ];

        unsafe { INFOS.get_unchecked(self as usize) }
    }
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
    ///
    /// [`uniform`]: BlockAppearance::uniform
    Regular {
        /// The texture that should be applied to the top face of the block (facing the positive
        /// Y axis).
        top: TextureId,
        /// The texture that should be applied to the bottom face of the block (facing the negative
        /// Y axis).
        bottom: TextureId,
        /// The texture that should be applied to the side faces of the block (X and Y axis).
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

/// Describes the visibility of a block.
///
/// The visibility of a block defines a couple of things:
///
/// 1. Whether the block should be included in the geometry of the chunk that contains it.
///
/// 2. How the block should influence face culling of neighboring blocks.
///
/// 3. In what order the faces of the block should be rendered compared to other voxels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockVisibility {
    /// The block is completely opaque.
    Opaque,
    /// The block is semi-opaque.
    ///
    /// It contains either completely transparent pixels, or pixels that are completely opaque.
    SemiOpaque,
    /// The block contains semi-transparent pixels.
    Transparent,
    /// The block is invisible and is not included in the geometry of the chunk that contains it.
    Invisible,
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
    /// The visibility of the block.
    pub visibility: BlockVisibility,
}
