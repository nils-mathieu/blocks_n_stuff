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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Contiguous)]
#[repr(u8)]
pub enum TextureId {
    Stone,
    Dirt,
    Grass,
}
