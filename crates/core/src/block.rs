use bitflags::bitflags;
use bytemuck::{Contiguous, Zeroable};
use glam::IVec3;

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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    OakLog,
    OakLeaves,
    PineLog,
    PineLeaves,
    StructureBlock,
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
                flags: BlockFlags::empty(),
            },
            // Stone
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::Stone),
                visibility: BlockVisibility::Opaque,
                flags: BlockFlags::SOLID.union(BlockFlags::TANGIBLE),
            },
            // Grass
            BlockInfo {
                appearance: BlockAppearance::Regular {
                    top: TextureId::GrassTop,
                    bottom: TextureId::Dirt,
                    side: TextureId::GrassSide,
                },
                visibility: BlockVisibility::Opaque,
                flags: BlockFlags::SOLID.union(BlockFlags::TANGIBLE),
            },
            // Dirt
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::Dirt),
                visibility: BlockVisibility::Opaque,
                flags: BlockFlags::SOLID.union(BlockFlags::TANGIBLE),
            },
            // Andesite
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::Andesite),
                visibility: BlockVisibility::Opaque,
                flags: BlockFlags::SOLID.union(BlockFlags::TANGIBLE),
            },
            // Clay
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::Clay),
                visibility: BlockVisibility::Opaque,
                flags: BlockFlags::SOLID.union(BlockFlags::TANGIBLE),
            },
            // Diorite
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::Diorite),
                visibility: BlockVisibility::Opaque,
                flags: BlockFlags::SOLID.union(BlockFlags::TANGIBLE),
            },
            // Granite
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::Granite),
                visibility: BlockVisibility::Opaque,
                flags: BlockFlags::SOLID.union(BlockFlags::TANGIBLE),
            },
            // Gravel
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::Gravel),
                visibility: BlockVisibility::Opaque,
                flags: BlockFlags::SOLID.union(BlockFlags::TANGIBLE),
            },
            // Podzol
            BlockInfo {
                appearance: BlockAppearance::Regular {
                    top: TextureId::PodzolTop,
                    bottom: TextureId::Dirt,
                    side: TextureId::PodzolSide,
                },
                visibility: BlockVisibility::Opaque,
                flags: BlockFlags::SOLID.union(BlockFlags::TANGIBLE),
            },
            // RedSand
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::RedSand),
                visibility: BlockVisibility::Opaque,
                flags: BlockFlags::SOLID.union(BlockFlags::TANGIBLE),
            },
            // Sand
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::Sand),
                visibility: BlockVisibility::Opaque,
                flags: BlockFlags::SOLID.union(BlockFlags::TANGIBLE),
            },
            // Sandstone
            BlockInfo {
                appearance: BlockAppearance::Regular {
                    top: TextureId::SandstoneTop,
                    bottom: TextureId::SandstoneBottom,
                    side: TextureId::SandstoneSide,
                },
                visibility: BlockVisibility::Opaque,
                flags: BlockFlags::SOLID.union(BlockFlags::TANGIBLE),
            },
            // RedSandstone
            BlockInfo {
                appearance: BlockAppearance::Regular {
                    top: TextureId::RedSandstoneTop,
                    bottom: TextureId::RedSandstoneBottom,
                    side: TextureId::RedSandstoneSide,
                },
                visibility: BlockVisibility::Opaque,
                flags: BlockFlags::SOLID.union(BlockFlags::TANGIBLE),
            },
            // Water
            BlockInfo {
                appearance: BlockAppearance::Liquid(TextureId::Water),
                visibility: BlockVisibility::Transparent,
                flags: BlockFlags::empty(),
            },
            // Bedrock
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::Bedrock),
                visibility: BlockVisibility::Opaque,
                flags: BlockFlags::SOLID.union(BlockFlags::TANGIBLE),
            },
            // Daffodil
            BlockInfo {
                appearance: BlockAppearance::Flat(TextureId::Daffodil),
                visibility: BlockVisibility::SemiOpaque,
                flags: BlockFlags::SOLID.union(BlockFlags::TANGIBLE),
            },
            // Pebbles
            BlockInfo {
                appearance: BlockAppearance::Flat(TextureId::Pebbles),
                visibility: BlockVisibility::SemiOpaque,
                flags: BlockFlags::SOLID.union(BlockFlags::TANGIBLE),
            },
            // Cobblestone
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::Cobblestone),
                visibility: BlockVisibility::Opaque,
                flags: BlockFlags::SOLID.union(BlockFlags::TANGIBLE),
            },
            // MossyCobblestone
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::MossyCobblestone),
                visibility: BlockVisibility::Opaque,
                flags: BlockFlags::SOLID.union(BlockFlags::TANGIBLE),
            },
            // DiamondOre
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::DiamondOre),
                visibility: BlockVisibility::Opaque,
                flags: BlockFlags::SOLID.union(BlockFlags::TANGIBLE),
            },
            // OakLog
            BlockInfo {
                appearance: BlockAppearance::Regular {
                    top: TextureId::OakLogTop,
                    bottom: TextureId::OakLogTop,
                    side: TextureId::OakLogSide,
                },
                visibility: BlockVisibility::Opaque,
                flags: BlockFlags::SOLID.union(BlockFlags::TANGIBLE),
            },
            // OakLeaves
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::OakLeaves),
                visibility: BlockVisibility::SemiOpaque,
                flags: BlockFlags::SOLID.union(BlockFlags::TANGIBLE),
            },
            // PineLog
            BlockInfo {
                appearance: BlockAppearance::Regular {
                    top: TextureId::PineLogTop,
                    bottom: TextureId::PineLogTop,
                    side: TextureId::PineLogSide,
                },
                visibility: BlockVisibility::Opaque,
                flags: BlockFlags::SOLID.union(BlockFlags::TANGIBLE),
            },
            // PineLeaves
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::PineLeaves),
                visibility: BlockVisibility::SemiOpaque,
                flags: BlockFlags::SOLID.union(BlockFlags::TANGIBLE),
            },
            // StructureBlock
            BlockInfo {
                appearance: BlockAppearance::uniform(TextureId::StructureBlock),
                visibility: BlockVisibility::Opaque,
                flags: BlockFlags::TANGIBLE,
            },
        ];

        unsafe { INFOS.get_unchecked(self as usize) }
    }
}

/// The specific face of a bloc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

impl Face {
    /// Returns the normal vector of the face.
    pub fn normal(self) -> IVec3 {
        match self {
            Self::X => IVec3::X,
            Self::NegX => IVec3::NEG_X,
            Self::Y => IVec3::Y,
            Self::NegY => IVec3::NEG_Y,
            Self::Z => IVec3::Z,
            Self::NegZ => IVec3::NEG_Z,
        }
    }
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
#[derive(Debug, Clone, Copy)]
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

bitflags! {
    /// A bunch of flags associated with a block.
    #[derive(Default, Debug, Clone, Copy)]
    pub struct BlockFlags: u8 {
        /// The block can be interacted with, for example by placing on it, or breaking it.
        const TANGIBLE = 1 << 0;
        /// It's not possible to walk through the block.
        const SOLID = 1 << 1;
    }
}

/// Stores static information about a block.
///
/// An instance of this type can be obtained by calling the [`info`] method of a
/// [`BlockId`].
///
/// [`info`]: BlockId::info
#[derive(Debug, Clone)]
pub struct BlockInfo {
    /// Describes the appearance of a block.
    pub appearance: BlockAppearance,
    /// The visibility of the block.
    pub visibility: BlockVisibility,
    /// The flags associated with the block.
    pub flags: BlockFlags,
}

/// A block that is instanciated in the world.
///
/// # Remarks
///
/// This type isn't actually directly stored in the world for memory efficiency reasons. This type
/// is most useful to serialize/deserialize easily a bunch of blocks (for example to store
/// structures).
#[derive(Clone)]
pub struct InstanciatedBlock {
    id: BlockId,
    appearance: AppearanceMetadata,
}

impl InstanciatedBlock {
    /// Creates a new [`InstanciatedBlock`] instance.
    ///
    /// # Safety
    ///
    /// The provided appearance metadata must be valid for the associated block ID.
    #[inline]
    pub unsafe fn new_unchecked(id: BlockId, appearance: AppearanceMetadata) -> Self {
        Self { id, appearance }
    }

    /// Returns the ID of the block.
    #[inline]
    pub fn id(&self) -> BlockId {
        self.id
    }

    /// Returns the appearance metadata of the block.
    ///
    /// The metadata is guaranteed to be valid for the associated block ID.
    #[inline]
    pub fn appearance(&self) -> AppearanceMetadata {
        self.appearance
    }
}

impl std::fmt::Debug for InstanciatedBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_struct("InstanciatedBlock");
        f.field("id", &self.id);

        #[allow(clippy::single_match)]
        unsafe {
            match self.id.info().appearance {
                BlockAppearance::Flat(..) => {
                    f.field("appearance", &self.appearance.flat);
                }
                _ => (),
            }
        }

        f.finish()
    }
}

impl PartialEq<BlockId> for InstanciatedBlock {
    #[inline]
    fn eq(&self, other: &BlockId) -> bool {
        self.id == *other
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for InstanciatedBlock {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde_InstanciateBlock::*;

        let appearance = unsafe {
            match self.id.info().appearance {
                BlockAppearance::Flat(..) => AppearanceMetadataHelper::Flat(self.appearance.flat),
                _ => AppearanceMetadataHelper::NoMetadata,
            }
        };

        let helper = InstanciatedBlockHelper {
            id: self.id,
            appearance,
        };

        helper.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for InstanciatedBlock {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde_InstanciateBlock::*;

        let helper = InstanciatedBlockHelper::deserialize(deserializer)?;

        let appearance = match (helper.appearance, helper.id.info().appearance) {
            (AppearanceMetadataHelper::Flat(face), BlockAppearance::Flat(..)) => {
                AppearanceMetadata { flat: face }
            }
            (AppearanceMetadataHelper::NoMetadata, _) => AppearanceMetadata { no_metadata: () },
            _ => return Err(serde::de::Error::custom("invalid appearance metadata")),
        };

        Ok(Self {
            id: helper.id,
            appearance,
        })
    }
}

#[cfg(feature = "serde")]
#[allow(non_snake_case)]
mod serde_InstanciateBlock {
    use serde::{Deserialize, Serialize};

    use crate::{BlockId, Face};

    #[derive(Serialize, Deserialize)]
    pub enum AppearanceMetadataHelper {
        NoMetadata,
        Flat(Face),
    }

    impl AppearanceMetadataHelper {
        #[inline]
        pub fn has_no_metadata(&self) -> bool {
            matches!(self, Self::NoMetadata)
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct InstanciatedBlockHelper {
        pub id: BlockId,
        #[serde(skip_serializing_if = "AppearanceMetadataHelper::has_no_metadata")]
        pub appearance: AppearanceMetadataHelper,
    }
}
