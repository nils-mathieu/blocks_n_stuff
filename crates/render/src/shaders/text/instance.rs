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

/// A helper structure that helps creating buffers of [`CharacterInstance`]s.
#[derive(Debug, Clone)]
pub struct CharacterInstanceCursor {
    /// The top-left position of the buffer.
    top_left: Vec2,
    /// The current cursor position.
    cursor: Vec2,
    /// The current color.
    color: Color,
    /// The size of the characters.
    size: Vec2,
    /// The spacing between characters.
    spacing: Vec2,
}

impl CharacterInstanceCursor {
    /// Creates a new [`CharacterInstanceBuffer`] instance.
    pub const fn new(top_left: Vec2, size: Vec2, spacing: Vec2) -> Self {
        Self {
            top_left,
            cursor: top_left,
            color: Color::WHITE,
            size,
            spacing,
        }
    }

    /// Updates the color of all subsequent characters written
    /// to the buffer.
    #[inline]
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// Advances the buffer without actually writing to the inner container.
    pub fn advance(&mut self, c: char) -> CharacterInstance {
        if c == '\n' {
            self.cursor.x = self.top_left.x;
            self.cursor.y += self.size.y + self.spacing.y;
            return CharacterInstance {
                flags: CharacterFlags::from_character(' ').unwrap(),
                color: self.color,
                position: self.cursor,
                size: self.size,
            };
        }

        let flags = CharacterFlags::from_character(c)
            .or(CharacterFlags::from_character(' '))
            .unwrap();
        let instance = CharacterInstance {
            flags,
            color: self.color,
            position: self.cursor,
            size: self.size,
        };

        self.cursor.x += self.size.x + self.spacing.x;

        instance
    }
}
