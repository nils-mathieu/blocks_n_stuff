use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
use glam::Vec2;

use crate::color::Color;

bitflags! {
    /// A bunch of flags passed to the text rendering shader.
    #[derive(Debug, Clone, Copy)]
    #[repr(transparent)]
    pub struct CharacterFlags: u32 {
        /// The part of the flags that are used to store the texture index.
        const TEXTURE_MASK = 0x0000_007F;
    }
}

impl CharacterFlags {
    /// Returns the [`CharacterFlags`] instance that represents the provided character, if it
    /// exists.
    #[inline]
    pub fn from_character(c: char) -> Option<Self> {
        let index = c as u32;

        if index < b' ' as u32 || index >= 0x80 {
            None
        } else {
            Some(Self::from_bits_retain(index))
        }
    }
}

unsafe impl Zeroable for CharacterFlags {}
unsafe impl Pod for CharacterFlags {}

/// A character instance.
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct CharacterInstance {
    /// Some flags that are passed to the shader.
    pub flags: CharacterFlags,
    /// The color of the character.
    pub color: Color,
    /// The position of the character on the screen.
    pub position: Vec2,
    /// The size of the character on the screen.
    pub size: Vec2,
}
