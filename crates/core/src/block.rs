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
    Andesite,
    Clay,
    Diorite,
    Granite,
    Gravel,
    Podzol,
    RedSand,
    Sand,
    Sandstone,
    RedSandstone,
    Water,
    Bedrock,
    Daffodil,
    Pebbles,
    Cobblestone,
    MossyCobblestone,
    DiamondOre,
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
            // Andesite
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::Andesite),
                visibility: BlockVisibility::Opaque,
            },
            // Clay
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::Clay),
                visibility: BlockVisibility::Opaque,
            },
            // Diorite
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::Diorite),
                visibility: BlockVisibility::Opaque,
            },
            // Granite
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::Granite),
                visibility: BlockVisibility::Opaque,
            },
            // Gravel
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::Gravel),
                visibility: BlockVisibility::Opaque,
            },
            // Podzol
            BlockInfo {
                appearance: BlockAppearance::Regular {
                    top: TextureId::PodzolTop,
                    bottom: TextureId::Dirt,
                    side: TextureId::PodzolSide,
                },
                visibility: BlockVisibility::Opaque,
            },
            // RedSand
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::RedSand),
                visibility: BlockVisibility::Opaque,
            },
            // Sand
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::Sand),
                visibility: BlockVisibility::Opaque,
            },
            // Sandstone
            BlockInfo {
                appearance: BlockAppearance::Regular {
                    top: TextureId::SandstoneTop,
                    bottom: TextureId::SandstoneBottom,
                    side: TextureId::SandstoneSide,
                },
                visibility: BlockVisibility::Opaque,
            },
            // RedSandstone
            BlockInfo {
                appearance: BlockAppearance::Regular {
                    top: TextureId::RedSandstoneTop,
                    bottom: TextureId::RedSandstoneBottom,
                    side: TextureId::RedSandstoneSide,
                },
                visibility: BlockVisibility::Opaque,
            },
            // Water
            BlockInfo {
                appearance: BlockAppearance::Liquid(TextureId::Water),
                visibility: BlockVisibility::Transparent,
            },
            // Bedrock
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::Bedrock),
                visibility: BlockVisibility::Opaque,
            },
            // Daffodil
            BlockInfo {
                appearance: BlockAppearance::Flat(TextureId::Daffodil),
                visibility: BlockVisibility::SemiOpaque,
            },
            // Pebbles
            BlockInfo {
                appearance: BlockAppearance::Flat(TextureId::Pebbles),
                visibility: BlockVisibility::SemiOpaque,
            },
            // Cobblestone
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::Cobblestone),
                visibility: BlockVisibility::Opaque,
            },
            // MossyCobblestone
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::MossyCobblestone),
                visibility: BlockVisibility::Opaque,
            },
            // DiamondOre
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::DiamondOre),
                visibility: BlockVisibility::Opaque,
            },
        ];

        unsafe { INFOS.get_unchecked(self as usize) }
    }
}

/// The specific face of a bloc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Face {
    /// The face is facing the positive X axis.
    X,
    /// The face is facing the negative X axis.
    NegX,
    /// The face is facing the positive Y axis.
    Y,
    /// The face is facing the negative Y axis.
    NegY,
    /// The face is facing the positive Z axis.
    Z,
    /// The face is facing the negative Z axis.
    NegZ,
}

/// Some metadata about the appearance of a block.
#[derive(Clone, Copy)]
pub union AppearanceMetadata {
    /// The block has no associated metadata.
    pub no_metadata: (),
    /// The block has a flat appearance.
    ///
    /// This metadata indicates which direction of the block is facing.
    pub flat: Face,
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
    /// The block has the appearance of a liquid.
    Liquid(TextureId),
    /// The block has a flat appearance.
    ///
    /// When this appearance is used, an appearance metadata is stored in the chunk that contains
    /// the block.
    Flat(TextureId),
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

    /// Returns whether this [`BlockAppearance`] instance requires some metadata.
    #[inline]
    pub const fn has_metadata(&self) -> bool {
        match self {
            Self::Invisible => false,
            Self::Regular { .. } => false,
            Self::Liquid(..) => false,
            Self::Flat(..) => true,
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
