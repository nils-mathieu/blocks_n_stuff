use std::io;

/// Loads an image from the provided reader.
///
/// The image is expected to be in the PNG format.
pub fn load(reader: impl io::Read) -> Result<crate::Image, crate::Error> {
    let mut decoder = png::Decoder::new(reader);
    decoder.set_transformations(png::Transformations::STRIP_16 | png::Transformations::EXPAND);
    let mut reader = decoder.read_info().map_err(map_error)?;

    if reader.info().is_animated() {
        return Err(crate::Error::Format);
    }

    let (format, bit_depth) = reader.output_color_type();
    if bit_depth != png::BitDepth::Eight {
        return Err(crate::Error::UnsupportedFormat);
    }
    let format = map_format(format)?;

    let color_space = if reader.info().srgb.is_some() {
        crate::ColorSpace::Srgb
    } else {
        crate::ColorSpace::Linear
    };

    let mut pixels = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut pixels).map_err(map_error)?;

    Ok(crate::Image {
        width: reader.info().width,
        height: reader.info().height,
        pixels,
        format,
        color_space,
    })
}

/// Maps the provided error to a [`crate::Error`].
fn map_error(err: png::DecodingError) -> crate::Error {
    match err {
        png::DecodingError::Parameter(err) => panic!("invalid PNG parameters: {err}"),
        png::DecodingError::IoError(io) => crate::Error::Io(io),
        png::DecodingError::LimitsExceeded => panic!("PNG limits exceeded"),
        png::DecodingError::Format(..) => crate::Error::Format,
    }
}

/// Maps the provided format to a [`crate::Format`].
fn map_format(format: png::ColorType) -> Result<crate::Format, crate::Error> {
    match format {
        png::ColorType::Rgba => Ok(crate::Format::Rgba),
        png::ColorType::Rgb => Ok(crate::Format::Rgb),
        png::ColorType::Grayscale => Ok(crate::Format::Grayscale),
        png::ColorType::GrayscaleAlpha => Ok(crate::Format::GrayscaleAlpha),
        _ => Err(crate::Error::UnsupportedFormat),
    }
}
