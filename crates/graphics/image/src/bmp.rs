//! The `.bmp` file format
//!
//! Information about the format can be found at
//! * <http://www.ece.ualberta.ca/~elliott/ee552/studentAppNotes/2003_w/misc/bmp_file_format/bmp_file_format.htm>
//! * <http://www.martinreddy.net/gfx/2d/BMP.txt>

use crate::{ColorFormat, DynamicTexture, Rgb, Texture};
use sl_std::bytestream::ByteStream;

pub(crate) const BMP_MAGIC: [u8; 2] = [0x42, 0x4d];

const MAX_ACCEPTABLE_SIZE: u32 = 8096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    NotABmp,
    UnexpectedEndOfFile,
    UnknownColorFormat,
    UnknownCompression,
    NonPalletizedCompression,
    PaletteTooSmall,

    /// This image contains extreme values and cannot be parsed
    ///
    /// For example, the image might be too large to fit in memory.
    RefuseToParse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BmpColorFormat {
    Monochrome,
    Palletized4Bit,
    Palletized8Bit,
    Rgb16,
    Rgb24,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Compression {
    None,
    Rle8Bit,
    Rle4Bit,
}

pub fn decode(bytes: &[u8]) -> Result<DynamicTexture, Error> {
    if bytes.len() < 0x35 {
        // Every file must at least contain the header and info header structures
        return Err(Error::UnexpectedEndOfFile);
    }

    let mut byte_stream = ByteStream::new(bytes);

    // Start of header
    if byte_stream.next_chunk() != Some(BMP_MAGIC) {
        return Err(Error::NotABmp);
    }

    let file_size = byte_stream
        .next_le_u32()
        .ok_or(Error::UnexpectedEndOfFile)?;

    if file_size as usize != bytes.len() {
        log::warn!(
            "bmp header states that the file size is 0x{file_size:x} bytes, but its 0x{:x}",
            bytes.len()
        );
    }

    let reserved = byte_stream
        .next_le_u32()
        .ok_or(Error::UnexpectedEndOfFile)?;

    if reserved != 0 {
        log::warn!("Reserved field in bmp header is not zero (it is 0x{reserved:x})")
    }

    let image_data_offset = byte_stream
        .next_le_u32()
        .ok_or(Error::UnexpectedEndOfFile)?;

    // Start of the Info Header
    let info_header_size = byte_stream
        .next_le_u32()
        .ok_or(Error::UnexpectedEndOfFile)?;

    if info_header_size != 40 {
        log::warn!("Incorrect bmp info header size: Should be 40, is {info_header_size:?}");
    }

    let width = byte_stream
        .next_le_u32()
        .ok_or(Error::UnexpectedEndOfFile)?;

    let height = byte_stream
        .next_le_u32()
        .ok_or(Error::UnexpectedEndOfFile)?;

    let planes = byte_stream
        .next_le_u16()
        .ok_or(Error::UnexpectedEndOfFile)?;

    if planes != 1 {
        log::warn!("Unexpected number of planes, expected 1, got {planes:?}");
    }

    let bits_per_pixel = byte_stream
        .next_le_u16()
        .ok_or(Error::UnexpectedEndOfFile)?;

    let (colorformat, palette_size) = match bits_per_pixel {
        1 => (BmpColorFormat::Monochrome, 1),
        4 => (BmpColorFormat::Palletized4Bit, 16),
        8 => (BmpColorFormat::Palletized8Bit, 256),
        16 => (BmpColorFormat::Rgb16, 0),
        24 => (BmpColorFormat::Rgb24, 0),
        other => {
            log::error!("No format known for the given number of bits per pixel: {other:?}");
            return Err(Error::UnknownColorFormat);
        },
    };

    let compression = byte_stream
        .next_le_u32()
        .ok_or(Error::UnexpectedEndOfFile)?;

    let compression = match compression {
        0 => Compression::None,
        1 => Compression::Rle8Bit,
        2 => Compression::Rle4Bit,
        other => {
            log::error!("Unknown compression format: {other:?}");
            return Err(Error::UnknownCompression);
        },
    };

    if compression != Compression::None
        && matches!(colorformat, BmpColorFormat::Rgb16 | BmpColorFormat::Rgb24)
    {
        log::error!("Non-palletized images cannot be compressed (got compression {compression:?} and color format {colorformat:?}");
        return Err(Error::NonPalletizedCompression);
    }

    if width > MAX_ACCEPTABLE_SIZE || height > MAX_ACCEPTABLE_SIZE {
        log::error!("Refusing to allocate image of size {width}x{height}");
        return Err(Error::RefuseToParse);
    }

    let compressed_image_size = byte_stream
        .next_le_u32()
        .ok_or(Error::UnexpectedEndOfFile)?;

    let x_pixels_per_meter = byte_stream
        .next_le_u32()
        .ok_or(Error::UnexpectedEndOfFile)?;

    let y_pixels_per_meter = byte_stream
        .next_le_u32()
        .ok_or(Error::UnexpectedEndOfFile)?;

    let colors_used = byte_stream
        .next_le_u32()
        .ok_or(Error::UnexpectedEndOfFile)?;

    let important_colors = byte_stream
        .next_le_u32()
        .ok_or(Error::UnexpectedEndOfFile)?;

    // Log all the info header information nicely
    #[derive(Debug)]
    #[allow(dead_code)]
    struct InfoHeader {
        size: u32,
        width: u32,
        height: u32,
        planes: u16,
        bits_per_pixel: BmpColorFormat,
        compression: Compression,
        compressed_image_size: u32,
        x_pixels_per_meter: u32,
        y_pixels_per_meter: u32,
        colors_used: u32,
        important_colors: u32,
    }
    let info_header = InfoHeader {
        size: info_header_size,
        width,
        height,
        planes,
        bits_per_pixel: colorformat,
        compression,
        compressed_image_size,
        x_pixels_per_meter,
        y_pixels_per_meter,
        colors_used,
        important_colors,
    };

    // Read the palette, if any
    let used_palette_size = if colorformat == BmpColorFormat::Monochrome {
        2
    } else {
        palette_size
    };

    let mut palette = Vec::with_capacity(used_palette_size);
    for _ in 0..used_palette_size {
        let [red, green, blue, reserved] =
            byte_stream.next_chunk().ok_or(Error::UnexpectedEndOfFile)?;

        if reserved != 0 {
            log::warn!("Reserved field in palette is not zero (is {reserved:?}");
        }

        palette.push(Rgb::from_channels(&[red, green, blue]));
    }

    if byte_stream.cursor() != image_data_offset as usize {
        log::warn!(
            "Expected image data to be at 0x{:x}, but its at 0x{image_data_offset:x} instead",
            byte_stream.cursor()
        );
        byte_stream.set_cursor(image_data_offset as usize);
    }

    let image_data = byte_stream.remaining();

    if image_data.len() != compressed_image_size as usize {
        log::warn!(
            "Expected 0x{compressed_image_size:x} bytes of image data, found 0x{:x}",
            image_data.len()
        );
    }

    let texture = match colorformat {
        BmpColorFormat::Rgb16 => {
            todo!("implement .bmp rgb16 format")
        },
        BmpColorFormat::Rgb24 => {
            let scanline_width = align_up::<4>(3 * width as usize);

            let scanlines = image_data.chunks_exact(scanline_width);
            if !scanlines.remainder().is_empty() {
                log::warn!("Trailing bytes after last scanline");
            }

            let mut texture_data = Vec::with_capacity(width as usize * height as usize * 3);
            for scanline in scanlines.rev() {
                for pixel in scanline.chunks_exact(3).take(width as usize) {
                    let blue = pixel[0];
                    let green = pixel[1];
                    let red = pixel[2];

                    texture_data.push(red);
                    texture_data.push(green);
                    texture_data.push(blue);
                }
            }

            Texture::<Rgb<u8>, Vec<u8>>::from_data(texture_data, width as usize, height as usize)
                .into()
        },
        BmpColorFormat::Monochrome => {
            let bytes_per_scanline = align_up::<8>(width as usize) / 8;
            let scanline_width = align_up::<4>(bytes_per_scanline);

            let scanlines = image_data.chunks_exact(scanline_width);
            if !scanlines.remainder().is_empty() {
                log::warn!("Trailing bytes after last scanline");
            }

            let mut texture_data = Vec::with_capacity(width as usize * height as usize * 3);
            for scanline in scanlines.rev() {
                for i in 0..width as usize {
                    let byte_index = i / 8;
                    let bit_index = i % 8;
                    let palette_index = (scanline[byte_index] >> bit_index) & 1;

                    let pixel = palette
                        .get(palette_index as usize)
                        .ok_or(Error::PaletteTooSmall)?;

                    texture_data.extend(pixel.channels());
                }
            }

            Texture::<Rgb<u8>, Vec<u8>>::from_data(texture_data, width as usize, height as usize)
                .into()
        },
        _ => todo!("implement palletized bmp images"),
    };

    Ok(texture)
}

#[must_use]
fn align_up<const N: usize>(x: usize) -> usize {
    (x + N - 1) & !(N - 1)
}
