//! The `.bmp` file format
//!
//! Information about the format can be found at
//! * <http://www.ece.ualberta.ca/~elliott/ee552/studentAppNotes/2003_w/misc/bmp_file_format/bmp_file_format.htm>
//! * <http://www.martinreddy.net/gfx/2d/BMP.txt>

use crate::{texture::Rgbaf32, Texture};
use sl_std::bytestream::ByteStream;

pub(crate) const BMP_MAGIC: [u8; 2] = [0x42, 0x4d];

const MAX_ACCEPTABLE_SIZE: u32 = 8096;

const BI_RGB: u32 = 0;
const BI_RLE8: u32 = 1;
const BI_RLE4: u32 = 2;
const BI_BITFIELDS: u32 = 3;
const BI_JPEG: u32 = 4;
const BI_PNG: u32 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    NotABmp,
    UnexpectedEndOfFile,
    UnknownColorFormat,
    UnknownCompression,
    NonPalletizedCompression,
    PaletteTooSmall,
    PaletteTooLarge,
    NegativeWidth,
    InvalidCompressionForFormat,
    MultiplePlanes,

    /// This image contains extreme values and cannot be parsed
    ///
    /// For example, the image might be too large to fit in memory.
    RefuseToParse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ImageType {
    Jpeg,
    Png,
    Monochrome,
    Palette4Bit(RunLengthEncoded),
    Palette8Bit(RunLengthEncoded),
    Rgb16,
    Rgb24,
    Rgb32,
    BitFields16,
    BitFields32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RunLengthEncoded {
    Yes,
    No,
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
    image_type: ImageType,
    compressed_image_size: u32,
    direction: Direction,
    colors_used: u32,
}

impl InfoHeader {
    #[must_use]
    fn palette_size(&self) -> usize {
        if self.colors_used == 0 {
            match self.image_type {
                ImageType::Monochrome => 2,
                ImageType::Palette4Bit(_) => 16,
                ImageType::Palette8Bit(_) => 256,
                _ => 0,
            }
        } else {
            self.colors_used as usize
        }
    }

    #[must_use]
    fn scanline_width(&self) -> usize {
        let bytes_per_scanline = match self.image_type {
            ImageType::Monochrome => {
                // 1 bit per pixel
                align_up::<8>(self.width as usize) / 8
            },
            ImageType::Palette8Bit(_) => self.width as usize,
            ImageType::Rgb16 => {
                // 16 bits per pixel
                2 * self.width as usize
            },
            ImageType::Rgb24 => {
                // 24 bits per pixel
                3 * self.width as usize
            },
            ImageType::Rgb32 => {
                // 32 bits per pixel
                4 * self.width as usize
            },
            _ => todo!(),
        };

        align_up::<4>(bytes_per_scanline)
    }

    fn for_each_scanline<F>(&self, scanline_data: &[u8], mut f: F) -> Result<(), Error>
    where
        F: FnMut(&[u8]) -> Result<(), Error>,
    {
        if self.width == 0 || self.height == 0 {
            return Ok(());
        }

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

        if width > MAX_ACCEPTABLE_SIZE || height > MAX_ACCEPTABLE_SIZE {
            log::error!("Refusing to allocate image of size {width}x{height}");
            return Err(Error::RefuseToParse);
        }

        let planes = byte_stream
            .next_le_u16()
            .ok_or(Error::UnexpectedEndOfFile)?;

        if planes != 1 {
            log::error!("Unexpected number of planes, expected 1, got {planes:?}");
            return Err(Error::MultiplePlanes);
        }

        let bits_per_pixel = byte_stream
            .next_le_u16()
            .ok_or(Error::UnexpectedEndOfFile)?;

        let compression = byte_stream
            .next_le_u32()
            .ok_or(Error::UnexpectedEndOfFile)?;

        let image_type = ImageType::from_bpp_and_compression(bits_per_pixel, compression)?;

        let compressed_image_size = byte_stream
            .next_le_u32()
            .ok_or(Error::UnexpectedEndOfFile)?;

        let _x_pixels_per_meter = byte_stream
            .next_le_u32()
            .ok_or(Error::UnexpectedEndOfFile)?;

        let _y_pixels_per_meter = byte_stream
            .next_le_u32()
            .ok_or(Error::UnexpectedEndOfFile)?;

        let colors_used = byte_stream
            .next_le_u32()
            .ok_or(Error::UnexpectedEndOfFile)?;

        if colors_used as usize > 1_usize.wrapping_shl(bits_per_pixel as u32) {
            log::error!(
                "image attempts to use {:?} colors but cannot adress more than {:?}",
                colors_used,
                1 << bits_per_pixel as usize
            );
            return Err(Error::PaletteTooLarge);
        }

        let _important_colors = byte_stream
            .next_le_u32()
            .ok_or(Error::UnexpectedEndOfFile)?;

        let info_header = Self {
            width,
            height,
            image_type,
            compressed_image_size,
            direction,
            colors_used,
        };

        Ok(info_header)
    }
}

pub fn decode(bytes: &[u8]) -> Result<Texture, Error> {
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

        palette.push(Rgbaf32::rgb(
            red as f32 / 255.,
            green as f32 / 255.,
            blue as f32 / 255.,
        ));
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

    let mut texture_data =
        Vec::with_capacity(info_header.width as usize * info_header.height as usize);
    match info_header.image_type {
        ImageType::Monochrome => {
            info_header.for_each_scanline(image_data, |scanline| {
                for i in 0..info_header.width as usize {
                    let byte_index = i / 8;
                    let bit_index = i % 8;
                    let palette_index = (scanline[byte_index] >> bit_index) & 1;

                    let pixel = palette
                        .get(palette_index as usize)
                        .ok_or(Error::PaletteTooSmall)?;

                    texture_data.push(*pixel);
                }

                Ok(())
            })?;
        },
        ImageType::Palette8Bit(run_length_encoded) => {
            if run_length_encoded == RunLengthEncoded::Yes {
                todo!("implement run length encoding");
            }

            let mut texture_data =
                Vec::with_capacity(info_header.width as usize * info_header.height as usize * 3);

            info_header.for_each_scanline(image_data, |scanline| {
                for palette_index in scanline.iter().take(info_header.width as usize) {
                    let pixel = palette
                        .get(*palette_index as usize)
                        .ok_or(Error::PaletteTooSmall)?;
                    texture_data.push(*pixel);
                }

                Ok(())
            })?;
        },
        ImageType::Rgb16 => {
            todo!("implement .bmp rgb16 format")
        },
        ImageType::Rgb24 | ImageType::Rgb32 => {
            let pixel_width = if info_header.image_type == ImageType::Rgb24 {
                3
            } else {
                4
            };

            let mut texture_data =
                Vec::with_capacity(info_header.width as usize * info_header.height as usize);

            info_header.for_each_scanline(image_data, |scanline| {
                for pixel in scanline
                    .chunks_exact(pixel_width)
                    .take(info_header.width as usize)
                {
                    let blue = pixel[0];
                    let green = pixel[1];
                    let red = pixel[2];

                    let color =
                        Rgbaf32::rgb(red as f32 / 255., green as f32 / 255., blue as f32 / 255.);
                    texture_data.push(color);
                }

                Ok(())
            })?;
        },
        _ => todo!("implement palletized bmp images"),
    }

    let texture = Texture::from_data(
        texture_data,
        info_header.width as usize,
        info_header.height as usize,
    );

    Ok(texture)
}

impl ImageType {
    fn from_bpp_and_compression(bpp: u16, compression: u32) -> Result<Self, Error> {
        let image_type = match bpp {
            0 => {
                // Bits per pixel are specified by the jpeg/png image
                match compression {
                    BI_JPEG => Self::Jpeg,
                    BI_PNG => Self::Png,
                    _ => return Err(Error::InvalidCompressionForFormat),
                }
            },
            1 => {
                // Monochrome images cannot be compressed
                if compression != BI_RGB {
                    return Err(Error::InvalidCompressionForFormat);
                }

                Self::Monochrome
            },
            4 => {
                // Palletized 4 bit per pixel
                let run_length_encoded = match compression {
                    BI_RGB => RunLengthEncoded::No,
                    BI_RLE4 => RunLengthEncoded::Yes,
                    _ => return Err(Error::InvalidCompressionForFormat),
                };

                Self::Palette4Bit(run_length_encoded)
            },
            8 => {
                // Palletized 8 bit per pixel
                let run_length_encoded = match compression {
                    BI_RGB => RunLengthEncoded::No,
                    BI_RLE8 => RunLengthEncoded::Yes,
                    _ => return Err(Error::InvalidCompressionForFormat),
                };

                Self::Palette8Bit(run_length_encoded)
            },
            16 => match compression {
                BI_RGB => Self::Rgb16,
                BI_BITFIELDS => Self::BitFields16,
                _ => return Err(Error::InvalidCompressionForFormat),
            },
            24 => {
                if compression != BI_RGB {
                    return Err(Error::InvalidCompressionForFormat);
                }

                Self::Rgb24
            },
            32 => match compression {
                BI_RGB => Self::Rgb32,
                BI_BITFIELDS => Self::BitFields32,
                _ => return Err(Error::InvalidCompressionForFormat),
            },
            other => {
                log::error!("No format known for the given number of bits per pixel: {other:?}");
                return Err(Error::UnknownColorFormat);
            },
        };

        Ok(image_type)
    }
}

#[must_use]
fn align_up<const N: usize>(x: usize) -> usize {
    (x + N - 1) & !(N - 1)
}
