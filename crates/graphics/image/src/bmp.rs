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
    NegativeWidth,

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum Direction {
    #[default]
    BottomUp,
    TopDown,
}

#[derive(Debug)]
struct InfoHeader {
    width: u32,
    height: u32,
    bits_per_pixel: BmpColorFormat,
    compressed_image_size: u32,
    direction: Direction,
}

impl InfoHeader {
    #[must_use]
    fn palette_size(&self) -> usize {
        match self.bits_per_pixel {
            BmpColorFormat::Monochrome => 2,
            BmpColorFormat::Palletized4Bit => 16,
            BmpColorFormat::Palletized8Bit => 256,
            _ => 0,
        }
    }

    #[must_use]
    fn scanline_width(&self) -> usize {
        let bytes_per_scanline = match self.bits_per_pixel {
            BmpColorFormat::Monochrome => {
                // 1 bit per pixel
                align_up::<8>(self.width as usize) / 8
            },
            BmpColorFormat::Rgb16 => {
                // 16 bits per pixel
                2 * self.width as usize
            },
            BmpColorFormat::Rgb24 => {
                // 24 bits per pixel
                3 * self.width as usize
            },
            _ => todo!(),
        };

        align_up::<4>(bytes_per_scanline)
    }

    fn for_each_scanline<F>(&self, scanline_data: &[u8], mut f: F) -> Result<(), Error>
    where
        F: FnMut(&[u8]) -> Result<(), Error>,
    {
        let scanlines = scanline_data.chunks_exact(self.scanline_width());
        if !scanlines.remainder().is_empty() {
            log::warn!("Trailing bytes after last scanline");
        }

        if self.direction == Direction::TopDown {
            for scanline in scanlines {
                f(scanline)?;
            }
        } else {
            for scanline in scanlines.rev() {
                f(scanline)?;
            }
        }

        Ok(())
    }

    fn read(byte_stream: &mut ByteStream<'_>) -> Result<Self, Error> {
        let info_header_size = byte_stream
            .next_le_u32()
            .ok_or(Error::UnexpectedEndOfFile)?;

        if info_header_size != 40 {
            log::warn!("Incorrect bmp info header size: Should be 40, is {info_header_size:?}");
        }

        let width = byte_stream
            .next_le_i32()
            .ok_or(Error::UnexpectedEndOfFile)?;

        let width = u32::try_from(width).map_err(|_| Error::NegativeWidth)?;

        let height = byte_stream
            .next_le_i32()
            .ok_or(Error::UnexpectedEndOfFile)?;

        // A negative height indicates a top-down image
        let (direction, height) = match u32::try_from(height) {
            Ok(height) => (Direction::BottomUp, height),
            Err(_) => (Direction::TopDown, height.abs() as u32),
        };

        let planes = byte_stream
            .next_le_u16()
            .ok_or(Error::UnexpectedEndOfFile)?;

        if planes != 1 {
            log::warn!("Unexpected number of planes, expected 1, got {planes:?}");
        }

        let bits_per_pixel = byte_stream
            .next_le_u16()
            .ok_or(Error::UnexpectedEndOfFile)?;

        let colorformat = match bits_per_pixel {
            1 => BmpColorFormat::Monochrome,
            4 => BmpColorFormat::Palletized4Bit,
            8 => BmpColorFormat::Palletized8Bit,
            16 => BmpColorFormat::Rgb16,
            24 => BmpColorFormat::Rgb24,
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

        let _x_pixels_per_meter = byte_stream
            .next_le_u32()
            .ok_or(Error::UnexpectedEndOfFile)?;

        let _y_pixels_per_meter = byte_stream
            .next_le_u32()
            .ok_or(Error::UnexpectedEndOfFile)?;

        let _colors_used = byte_stream
            .next_le_u32()
            .ok_or(Error::UnexpectedEndOfFile)?;

        let _important_colors = byte_stream
            .next_le_u32()
            .ok_or(Error::UnexpectedEndOfFile)?;

        let info_header = Self {
            width,
            height,
            bits_per_pixel: colorformat,
            compressed_image_size,
            direction,
        };

        Ok(info_header)
    }
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
    let info_header = InfoHeader::read(&mut byte_stream)?;

    // Read the palette, if any
    let palette_size = info_header.palette_size();

    let mut palette = Vec::with_capacity(palette_size);
    for _ in 0..palette_size {
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

    if image_data.len() != info_header.compressed_image_size as usize {
        log::warn!(
            "Expected 0x{:x} bytes of image data, found 0x{:x}",
            info_header.compressed_image_size,
            image_data.len()
        );
    }

    let texture = match info_header.bits_per_pixel {
        BmpColorFormat::Rgb16 => {
            todo!("implement .bmp rgb16 format")
        },
        BmpColorFormat::Rgb24 => {
            let mut texture_data =
                Vec::with_capacity(info_header.width as usize * info_header.height as usize * 3);

            info_header.for_each_scanline(image_data, |scanline| {
                for pixel in scanline.chunks_exact(3).take(info_header.width as usize) {
                    let blue = pixel[0];
                    let green = pixel[1];
                    let red = pixel[2];

                    texture_data.push(red);
                    texture_data.push(green);
                    texture_data.push(blue);
                }

                Ok(())
            })?;

            Texture::<Rgb<u8>, Vec<u8>>::from_data(
                texture_data,
                info_header.width as usize,
                info_header.height as usize,
            )
            .into()
        },
        BmpColorFormat::Monochrome => {
            let mut texture_data =
                Vec::with_capacity(info_header.width as usize * info_header.height as usize * 3);

            info_header.for_each_scanline(image_data, |scanline| {
                for i in 0..info_header.width as usize {
                    let byte_index = i / 8;
                    let bit_index = i % 8;
                    let palette_index = (scanline[byte_index] >> bit_index) & 1;

                    let pixel = palette
                        .get(palette_index as usize)
                        .ok_or(Error::PaletteTooSmall)?;

                    texture_data.extend(pixel.channels());
                }

                Ok(())
            })?;

            Texture::<Rgb<u8>, Vec<u8>>::from_data(
                texture_data,
                info_header.width as usize,
                info_header.height as usize,
            )
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
