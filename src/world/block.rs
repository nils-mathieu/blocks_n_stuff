use std::ops::Index;

use bytemuck::{Contiguous, Zeroable};

/// A unique ID for a block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Contiguous)]
#[repr(u8)]
pub enum BlockId {
    /// The most basic block type possible.
    ///
    /// Blocks with this ID are considered empty.
    #[allow(dead_code)] // TODO: remove this
    Air,

    Stone,
}

impl BlockId {
    /// The total number of [`BlockId`] instances.
    pub const COUNT: usize = <Self as Contiguous>::MAX_VALUE as usize + 1;
}

// SAFETY:
//  The block with ID 0 is `BlockId::Air`.
unsafe impl Zeroable for BlockId {}

/// The ID of a texture.
pub enum TextureId {
    Stone,
}

/// The appearance of a block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockTransparency {
    /// The block is not actually visible.
    Invisible,
    /// The block is completely opaque.
    ///
    /// Blocks with this [`BlockTransparency`] can be rendered in any order.
    Opaque,
    /// The block contains both completely opaque and completely transparent parts.
    ///
    /// Blocks with this [`BlockTransparency`] can be rendered in any order without any issues, but
    /// they cannot benefit from Mip Maps.
    SemiOpaque,
    /// Blocks with this [`BlockTransparency`] must be rendered in order, from back to front.
    Transparent,
}

/// The appearance of a block.
pub enum BlockAppearance {
    /// The block is not actually visible.
    ///
    /// No appearance metadata are associated with this block.
    Invisible,
    /// The block has a regular appearance with separate textures for each face.
    ///
    /// No appearance metadata are associated with this block.
    Regular {
        /// The texture to apply to the top face of the block (toward the positive Y axis).
        top: TextureId,
        /// The texture to apply to the bottom face of the block (toward the negative Y axis).
        bottom: TextureId,
        /// The texture to apply to the side faces of the block (along the X and Z axis).
        side: TextureId,
    },
}

/// Stores information about a block identified by a [`BlockId`].
pub struct BlockInfo {
    /// The transparency value of the block.
    pub transparency: BlockTransparency,
    /// The appearance of the block.
    pub appearance: BlockAppearance,
}

/// Contains data about all existing blocks identified by their [`BlockId`].
pub struct BlockRegistry {
    /// The information about each block.
    infos: [BlockInfo; BlockId::COUNT],
}

impl BlockRegistry {
    /// Constructs a new [`BlockRegistry`] instance.
    const fn load() -> Self {
        Self {
            infos: [
                // BlockId::Air
                BlockInfo {
                    transparency: BlockTransparency::Invisible,
                    appearance: BlockAppearance::Invisible,
                },
                // BlockId::Stone
                BlockInfo {
                    transparency: BlockTransparency::Opaque,
                    appearance: BlockAppearance::Regular {
                        top: TextureId::Stone,
                        bottom: TextureId::Stone,
                        side: TextureId::Stone,
                    },
                },
            ],
        }
    }
}

impl Index<BlockId> for BlockRegistry {
    type Output = BlockInfo;

    #[inline]
    fn index(&self, index: BlockId) -> &Self::Output {
        unsafe { self.infos.get_unchecked(index as usize) }
    }
}

/// The global block registry.
pub const BLOCK_REGISTRY: BlockRegistry = BlockRegistry::load();
