use std::ops::{Deref, DerefMut};

use bytemuck::{Pod, Zeroable};

#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Eq, Zeroable, Pod)]
#[repr(C)]
#[cfg(target_endian = "big")]
#[non_exhaustive]
pub struct ColorDeref {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Eq, Zeroable, Pod)]
#[repr(C)]
#[cfg(target_endian = "little")]
#[non_exhaustive]
pub struct ColorDeref {
    pub a: u8,
    pub b: u8,
    pub g: u8,
    pub r: u8,
}

/// A color, represented as four 8-bit unsigned bytes.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Zeroable, Pod)]
#[repr(transparent)]
pub struct Color(u32);

impl Color {
    /// (0, 0, 0, 255)
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    /// (255, 255, 255, 255)
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    /// (255, 0, 0, 255)
    pub const RED: Self = Self::rgb(255, 0, 0);
    /// (0, 255, 0, 255)
    pub const GREEN: Self = Self::rgb(0, 255, 0);
    /// (0, 0, 255, 255)
    pub const BLUE: Self = Self::rgb(0, 0, 255);
    /// (0, 0, 0, 0)
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);
    /// (255, 255, 0, 255)
    pub const YELLOW: Self = Self::rgb(255, 255, 0);
    /// (0, 255, 255, 255)
    pub const CYAN: Self = Self::rgb(0, 255, 255);
    /// (255, 0, 255, 255)
    pub const MAGENTA: Self = Self::rgb(255, 0, 255);

    /// Creates a new [`Color`] from its RGB components.
    #[inline]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 255)
    }

    /// Creates a new [`Color`] from its RGBA components.
    #[inline]
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(u32::from_be_bytes([r, g, b, a]))
    }

    /// Changes the alpha component of the [`Color`], returning a new one.
    #[inline]
    pub fn with_alpha(self, alpha: u8) -> Self {
        Self::rgba(self.r, self.g, self.b, alpha)
    }

    /// Changes the red component of the [`Color`], returning a new one.
    #[inline]
    pub fn with_red(self, red: u8) -> Self {
        Self::rgba(red, self.g, self.b, self.a)
    }

    /// Changes the green component of the [`Color`], returning a new one.
    #[inline]
    pub fn with_green(self, green: u8) -> Self {
        Self::rgba(self.r, green, self.b, self.a)
    }

    /// Changes the blue component of the [`Color`], returning a new one.
    #[inline]
    pub fn with_blue(self, blue: u8) -> Self {
        Self::rgba(self.r, self.g, blue, self.a)
    }
}

impl Deref for Color {
    type Target = ColorDeref;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self as *const Self as *const ColorDeref) }
    }
}

impl DerefMut for Color {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self as *mut Self as *mut ColorDeref) }
    }
}
