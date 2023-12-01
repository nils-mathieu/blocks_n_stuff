//! A simple image loading library.

use std::{fmt, io};

mod png;

/// An error that might occur when loading an image.
#[derive(Debug)]
pub enum Error {
    /// An I/O error occured.
    Io(io::Error),
    /// The format of the image is invalid.
    Format,

    /// The image is an animated image, which is not supported.
    UnsupportedAnimation,
    /// The format of the image is not supported.
    UnsupportedFormat,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Io(ref err) => write!(f, "I/O error: {}", err),
            Self::Format => write!(f, "invalid image format"),
            Self::UnsupportedAnimation => write!(f, "animated images are not supported"),
            Self::UnsupportedFormat => write!(f, "unsupported image format"),
        }
    }
}

/// The format of a loaded image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    /// The image is encoded in RGBA format, one byte per channel.
    Rgba,
    /// The image is encoded in BGRA format, one byte per channel.
    Bgra,
    /// The image is encoded in RGB format, one byte per channel.
    Rgb,
    /// The image is encoded in BGR format, one byte per channel.
    Bgr,
    /// The image is encoded in grayscale format, one byte per pixel.
    Grayscale,
    /// The image is encoded in grayscale with alpha format, two byte per pixel.
    GrayscaleAlpha,
}

/// The color space of a loaded image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorSpace {
    /// The image is encoded in the sRGB color space.
    Srgb,
    /// The image is encoded in the linear color space.
    Linear,
}

/// The format of an image.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImageMetadata {
    /// The width of the loaded image.
    pub width: u32,
    /// The height of the loaded image.
    pub height: u32,
    /// The format of the image.
    pub format: Format,
    /// The color space of the image.
    pub color_space: ColorSpace,
}

/// A loaded image.
pub struct Image {
    /// The pixels of the loaded image, encoded in RGBA format, one byte per channel.
    pub pixels: Vec<u8>,
    /// The metadata of the image.
    pub metadata: ImageMetadata,
}

impl Image {
    /// Load an image that's known to be in the PNG format.
    #[inline]
    pub fn load_png(reader: impl io::Read) -> Result<Self, Error> {
        png::load(reader)
    }

    /// Ensures that the image is encoded in [`Rgba`] format, eventually converting it if needed.
    ///
    /// [`Rgba`]: Format::Rgba
    #[allow(clippy::identity_op)]
    pub fn ensure_rgba(&mut self) {
        match self.metadata.format {
            Format::Rgba => (),
            Format::Rgb => {
                let cnt = self.pixels.len() / 3;
                self.pixels.resize(cnt * 4, 255);

                for i in (0..cnt).rev() {
                    unsafe {
                        *self.pixels.get_unchecked_mut(i * 4 + 0) =
                            *self.pixels.get_unchecked(i * 3 + 0);
                        *self.pixels.get_unchecked_mut(i * 4 + 1) =
                            *self.pixels.get_unchecked(i * 3 + 1);
                        *self.pixels.get_unchecked_mut(i * 4 + 2) =
                            *self.pixels.get_unchecked(i * 3 + 2);
                        *self.pixels.get_unchecked_mut(i * 4 + 3) = 255;
                    }
                }
            }
            Format::Bgra => {
                let cnt = self.pixels.len() / 4;

                for i in 0..cnt {
                    unsafe {
                        std::ptr::swap(
                            self.pixels.as_mut_ptr().add(i * 4 + 0),
                            self.pixels.as_mut_ptr().add(i * 4 + 2),
                        );
                    }
                }
            }
            Format::Bgr => {
                let cnt = self.pixels.len() / 3;
                self.pixels.resize(cnt * 4, 255);

                for i in (0..cnt).rev() {
                    unsafe {
                        *self.pixels.get_unchecked_mut(i * 4 + 0) =
                            *self.pixels.get_unchecked(i * 3 + 2);
                        *self.pixels.get_unchecked_mut(i * 4 + 1) =
                            *self.pixels.get_unchecked(i * 3 + 1);
                        *self.pixels.get_unchecked_mut(i * 4 + 2) =
                            *self.pixels.get_unchecked(i * 3 + 0);
                        *self.pixels.get_unchecked_mut(i * 4 + 3) = 255;
                    }
                }
            }
            Format::Grayscale => {
                let cnt = self.pixels.len();
                self.pixels.resize(cnt * 4, 255);

                for i in (0..cnt).rev() {
                    unsafe {
                        *self.pixels.get_unchecked_mut(i * 4 + 0) = *self.pixels.get_unchecked(i);
                        *self.pixels.get_unchecked_mut(i * 4 + 1) = *self.pixels.get_unchecked(i);
                        *self.pixels.get_unchecked_mut(i * 4 + 2) = *self.pixels.get_unchecked(i);
                        *self.pixels.get_unchecked_mut(i * 4 + 3) = 255;
                    }
                }
            }
            Format::GrayscaleAlpha => {
                let cnt = self.pixels.len() / 2;
                self.pixels.resize(cnt * 4, 255);

                for i in (0..cnt).rev() {
                    unsafe {
                        *self.pixels.get_unchecked_mut(i * 4 + 0) =
                            *self.pixels.get_unchecked(i * 2 + 0);
                        *self.pixels.get_unchecked_mut(i * 4 + 1) =
                            *self.pixels.get_unchecked(i * 2 + 0);
                        *self.pixels.get_unchecked_mut(i * 4 + 2) =
                            *self.pixels.get_unchecked(i * 2 + 0);
                        *self.pixels.get_unchecked_mut(i * 4 + 3) =
                            *self.pixels.get_unchecked(i * 2 + 1);
                    }
                }
            }
        }

        self.metadata.format = Format::Rgba;
    }

    /// Ensures that the image is encoded in the [`Srgb`](ColorSpace::Srgb) color space,
    /// eventually converting it if needed.
    pub fn ensure_srgb(&mut self) {
        match self.metadata.color_space {
            ColorSpace::Srgb => (),
            ColorSpace::Linear => {
                for channel in &mut self.pixels {
                    *channel = linear_to_srgb(*channel);
                }
            }
        }

        self.metadata.color_space = ColorSpace::Srgb;
    }
}

/// Converts the provided linear color to the sRGB color space.
fn linear_to_srgb(x: u8) -> u8 {
    if x <= 10 {
        0
    } else if x >= 245 {
        255
    } else {
        let x = x as f32 / 255.0;
        let x = x.powf(1.0 / 2.2);
        (x * 255.0) as u8
    }
}
