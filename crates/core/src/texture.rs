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
    Daffodil,
    Pebbles,
    Cobblestone,
    MossyCobblestone,
    DiamondOre,
    OakLogTop,
    OakLogSide,
    OakLeaves,
    PineLogTop,
    PineLogSide,
    PineLeaves,
    StructureBlock,
    StructureOriginBlock,
    OakPlanks,
    PinePlanks,
    Glass,
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
        match self {
            Self::Stone => "stone",
            Self::Dirt => "dirt",
            Self::GrassSide => "grass_side",
            Self::GrassTop => "grass_top",
            Self::Andesite => "andesite",
            Self::Clay => "clay",
            Self::Diorite => "diorite",
            Self::Granite => "granite",
            Self::Gravel => "gravel",
            Self::PodzolTop => "podzol_top",
            Self::PodzolSide => "podzol_side",
            Self::RedSand => "red_sand",
            Self::Sand => "sand",
            Self::SandstoneTop => "sandstone_top",
            Self::SandstoneBottom => "sandstone_bottom",
            Self::SandstoneSide => "sandstone_side",
            Self::RedSandstoneTop => "red_sandstone_top",
            Self::RedSandstoneBottom => "red_sandstone_bottom",
            Self::RedSandstoneSide => "red_sandstone_side",
            Self::Water => "water",
            Self::Bedrock => "bedrock",
            Self::Daffodil => "daffodil",
            Self::Pebbles => "pebbles",
            Self::Cobblestone => "cobblestone",
            Self::MossyCobblestone => "mossy_cobblestone",
            Self::DiamondOre => "diamond_ore",
            Self::OakLogTop => "oak_log_top",
            Self::OakLogSide => "oak_log_side",
            Self::OakLeaves => "oak_leaves",
            Self::PineLogTop => "pine_log_top",
            Self::PineLogSide => "pine_log_side",
            Self::PineLeaves => "pine_leaves",
            Self::StructureBlock => "structure_block",
            Self::StructureOriginBlock => "structure_origin_block",
            Self::OakPlanks => "oak_planks",
            Self::PinePlanks => "pine_planks",
            Self::Glass => "glass",
        }
    }

    /// Returns the PNG image that's embedded in the binary for this texture.
    #[cfg(feature = "embedded-textures")]
    pub const fn embeded_texture(self) -> &'static [u8] {
        macro_rules! include_asset {
            ($name:literal) => {
                include_bytes!(concat!("../../../assets/", $name, ".png"))
            };
        }

        match self {
            Self::Stone => include_asset!("stone"),
            Self::Dirt => include_asset!("dirt"),
            Self::GrassSide => include_asset!("grass_side"),
            Self::GrassTop => include_asset!("grass_top"),
            Self::Andesite => include_asset!("andesite"),
            Self::Clay => include_asset!("clay"),
            Self::Diorite => include_asset!("diorite"),
            Self::Granite => include_asset!("granite"),
            Self::Gravel => include_asset!("gravel"),
            Self::PodzolTop => include_asset!("podzol_top"),
            Self::PodzolSide => include_asset!("podzol_side"),
            Self::RedSand => include_asset!("red_sand"),
            Self::Sand => include_asset!("sand"),
            Self::SandstoneTop => include_asset!("sandstone_top"),
            Self::SandstoneBottom => include_asset!("sandstone_bottom"),
            Self::SandstoneSide => include_asset!("sandstone_side"),
            Self::RedSandstoneTop => include_asset!("red_sandstone_top"),
            Self::RedSandstoneBottom => include_asset!("red_sandstone_bottom"),
            Self::RedSandstoneSide => include_asset!("red_sandstone_side"),
            Self::Water => include_asset!("water"),
            Self::Bedrock => include_asset!("bedrock"),
            Self::Daffodil => include_asset!("daffodil"),
            Self::Pebbles => include_asset!("pebbles"),
            Self::Cobblestone => include_asset!("cobblestone"),
            Self::MossyCobblestone => include_asset!("mossy_cobblestone"),
            Self::DiamondOre => include_asset!("diamond_ore"),
            Self::OakLogTop => include_asset!("oak_log_top"),
            Self::OakLogSide => include_asset!("oak_log_side"),
            Self::OakLeaves => include_asset!("oak_leaves"),
            Self::PineLogTop => include_asset!("pine_log_top"),
            Self::PineLogSide => include_asset!("pine_log_side"),
            Self::PineLeaves => include_asset!("pine_leaves"),
            Self::StructureBlock => include_asset!("structure_block"),
            Self::StructureOriginBlock => include_asset!("structure_origin_block"),
            Self::OakPlanks => include_asset!("oak_planks"),
            Self::PinePlanks => include_asset!("pine_planks"),
            Self::Glass => include_asset!("glass"),
        }
    }

    /// Returns an iterator over all the [`TextureId`] instances.
    #[inline]
    pub fn all() -> impl Clone + ExactSizeIterator<Item = Self> {
        unsafe { (0..Self::COUNT).map(|x| Self::from_index_unchecked(x)) }
    }
}
