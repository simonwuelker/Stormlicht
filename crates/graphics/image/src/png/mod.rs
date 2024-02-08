//! Implements a [PNG](https://www.w3.org/TR/png) decoder

// The chunk types don't necessarily start with uppercase characters and renaming them would be silly
// #![allow(non_upper_case_globals)]

pub mod chunks;

use std::{
    convert::FloatToInt,
    fs,
    io::{self, Cursor, Read},
    path::Path,
};

use compression::zlib;

use hash::Crc32Hasher;

use crate::{
    format::{GrayScale, GrayScaleAlpha},
    ColorFormat, DynamicTexture, Rgb, Rgba, Texture,
};

use self::chunks::ihdr::ImageType;

pub(crate) const PNG_HEADER: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

#[derive(Debug)]
pub enum Error {
    NotAPng,
    ExpectedIHDR,
    UnknownChunk,
    MismatchedChecksum,
    InvalidIHDRChunk(chunks::ihdr::ImageHeaderError),
    InvalidcHRMChunk,
    InvalidPLTEChunk(chunks::plte::PaletteError),
    NonConsecutiveIDATChunk,
    /// Expected the length of the decompressed zlib stream to be a multiple of the scanline width plus the filter byte
    MismatchedDecompressedZlibSize,
    UnknownFilterType,
    IndexedImageWithoutPLTE,
    NotImplemented,
    IncorrectLengthOfImageData,
    ZLib(zlib::Error),
    IO(io::Error),
}

pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<DynamicTexture, Error> {
    let mut file_contents = vec![];
    fs::File::open(&path)?.read_to_end(&mut file_contents)?;
    decode(&file_contents)
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Chunk {
    /// Image Header
    IHDR(chunks::ImageHeader),
    /// Color Palette
    PLTE(chunks::Palette),
    /// Image Data
    IDAT(chunks::ImageData),
    /// Image End
    IEND,
    cHRM(chunks::Chromacities),
    /// Digital Signatures
    dSIG,
    /// Exif Metadata
    eXIf,
    gAMA,
    /// Color Histogram
    hIST,
    /// ICC color profile
    iCCP,
    iTXt,
    pHYs,
    sBIT,
    sPLT,
    sRGB,
    sTER,
    tEXt,
    tIME,
    tRNS,
    zTXt,
    /// Background
    bKGD,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ParserStage {
    BeforeIDAT,
    DuringIDAT,
    AfterIDAT,
}

pub(crate) fn decode(bytes: &[u8]) -> Result<DynamicTexture, Error> {
    let mut reader = Cursor::new(bytes);

    let mut signature = [0; 8];
    reader.read_exact(&mut signature)?;

    if signature != PNG_HEADER {
        return Err(Error::NotAPng);
    }

    let ihdr_chunk = read_chunk(&mut reader)?;
    let image_header = if let Chunk::IHDR(image_header) = ihdr_chunk {
        image_header
    } else {
        log::warn!("Expected IHDR chunk, found {ihdr_chunk:?}");
        return Err(Error::ExpectedIHDR);
    };
    let image_width = image_header.width as usize;
    let image_height = image_header.height as usize;

    let mut parser_stage = ParserStage::BeforeIDAT;
    let mut idat = vec![];
    let mut palette = None;

    // Read all the PNG chunks in the fule
    loop {
        let chunk = read_chunk(&mut reader)?;

        if parser_stage == ParserStage::DuringIDAT && !matches!(chunk, Chunk::IDAT(_)) {
            parser_stage = ParserStage::AfterIDAT;
        }

        match chunk {
            Chunk::IEND => break,
            Chunk::IDAT(data) => {
                match parser_stage {
                    ParserStage::BeforeIDAT => parser_stage = ParserStage::DuringIDAT,
                    ParserStage::AfterIDAT => return Err(Error::NonConsecutiveIDATChunk),
                    _ => {},
                }
                idat.extend(data.bytes());
            },
            Chunk::PLTE(plte) => palette = Some(plte),
            _ => {},
        }
    }

    if image_header.image_type == chunks::ihdr::ImageType::IndexedColor && palette.is_none() {
        log::error!("Cannot decode indexed color image without palette table");
        return Err(Error::IndexedImageWithoutPLTE);
    }

    let decompressed_body = zlib::decompress(&idat)?;

    let scanline_width = image_width * image_header.image_type.pixel_width();

    // NOTE: need to add 1 here because each scanline also contains a byte specifying a filter type
    if decompressed_body.len() % (scanline_width + 1) != 0 {
        log::error!(
            "Decompressed data size {} is not a multiple of scanline size {}",
            decompressed_body.len(),
            scanline_width + 1
        );
        return Err(Error::MismatchedDecompressedZlibSize);
    }

    let mut image_data = vec![0; image_height * scanline_width];
    apply_filters(
        &decompressed_body,
        &mut image_data,
        scanline_width,
        image_header.image_type.pixel_width(),
    )?;

    let dynamic_texture: DynamicTexture = match image_header.image_type {
        ImageType::GrayScale => {
            texture_from_bytes::<GrayScale<u8>>(image_data, image_width, image_height)?.into()
        },
        ImageType::GrayScaleWithAlpha => {
            texture_from_bytes::<GrayScaleAlpha<u8>>(image_data, image_width, image_height)?.into()
        },
        ImageType::TrueColor => {
            texture_from_bytes::<Rgb<u8>>(image_data, image_width, image_height)?.into()
        },
        ImageType::TrueColorWithAlpha => {
            texture_from_bytes::<Rgba<u8>>(image_data, image_width, image_height)?.into()
        },
        ImageType::IndexedColor => {
            log::error!("FIXME: png implement indexed color");
            return Err(Error::NotImplemented);
        },
    };

    Ok(dynamic_texture)
}

fn read_chunk<R: Read>(reader: &mut R) -> Result<Chunk, Error> {
    let mut length_bytes = [0; 4];
    reader.read_exact(&mut length_bytes)?;
    let length = u32::from_be_bytes(length_bytes) as usize;

    let mut chunk_name_bytes = [0; 4];
    reader.read_exact(&mut chunk_name_bytes)?;

    let mut data = vec![0; length];
    reader.read_exact(data.as_mut_slice())?;

    let mut crc_bytes = [0; 4];
    reader.read_exact(&mut crc_bytes)?;
    let expected_crc = u32::from_be_bytes(crc_bytes);

    let mut hasher = Crc32Hasher::default();
    hasher.write(&chunk_name_bytes);
    hasher.write(&data);
    let computed_crc = hasher.finish();

    if expected_crc != computed_crc {
        log::error!(
            "Incorrect chunk checksum: expected {expected_crc:0>8x}, found {computed_crc:0>8x}"
        );
        return Err(Error::MismatchedChecksum);
    }

    let chunk = match &chunk_name_bytes {
        b"IHDR" => Chunk::IHDR(chunks::ImageHeader::new(&data).map_err(Error::InvalidIHDRChunk)?),
        b"PLTE" => Chunk::PLTE(chunks::Palette::new(&data).map_err(Error::InvalidPLTEChunk)?),
        b"IDAT" => Chunk::IDAT(chunks::ImageData::new(data)),
        b"IEND" => {
            if length != 0 {
                log::warn!("IEND is not empty, found {length} bytes")
            }

            Chunk::IEND
        },
        b"cHRM" => {
            if length != 32 {
                log::error!("cHRM length must be exactly 32 bytes, found {length}");
                return Err(Error::InvalidcHRMChunk);
            }

            let white_point = (
                u32::from_be_bytes(data[0..4].try_into().unwrap()),
                u32::from_be_bytes(data[4..8].try_into().unwrap()),
            );
            let red_point = (
                u32::from_be_bytes(data[8..12].try_into().unwrap()),
                u32::from_be_bytes(data[12..16].try_into().unwrap()),
            );
            let green_point = (
                u32::from_be_bytes(data[16..20].try_into().unwrap()),
                u32::from_be_bytes(data[20..24].try_into().unwrap()),
            );
            let blue_point = (
                u32::from_be_bytes(data[24..28].try_into().unwrap()),
                u32::from_be_bytes(data[28..32].try_into().unwrap()),
            );
            Chunk::cHRM(chunks::Chromacities::new(
                white_point,
                red_point,
                green_point,
                blue_point,
            ))
        },
        b"dSIG" => Chunk::dSIG,
        b"eXIf" => Chunk::eXIf,
        b"gAMA" => Chunk::gAMA,
        b"hIST" => Chunk::hIST,
        b"iCCP" => Chunk::iCCP,
        b"iTXt" => Chunk::iTXt,
        b"pHYs" => Chunk::pHYs,
        b"sBIT" => Chunk::sBIT,
        b"sPLT" => Chunk::sPLT,
        b"sRGB" => Chunk::sRGB,
        b"sTER" => Chunk::sTER,
        b"tEXt" => Chunk::tEXt,
        b"tIME" => Chunk::tIME,
        b"tRNS" => Chunk::tRNS,
        b"zTXt" => Chunk::zTXt,
        b"bKGD" => Chunk::bKGD,
        unknown_chunk_type => {
            // Any chunk that we don't know about is not critical (since only IHDR, IDAT, PLTE and IEND are critical)
            log::info!(
                "Ignoring unknown chunk type: {}",
                String::from_utf8_lossy(unknown_chunk_type)
            );

            // Just read another chunk
            read_chunk(reader)?
        },
    };

    Ok(chunk)
}

/// Apply one of the filter specified in <https://www.w3.org/TR/png/#9-table91> to a scanline
fn apply_filters(
    from: &[u8],
    to: &mut [u8],
    scanline_width: usize,
    pixel_width: usize,
) -> Result<(), Error> {
    // For each scanline, apply a filter (which is the first byte) to the scanline data (which is the rest)
    let mut previous_scanline = vec![0; scanline_width];

    for (index, scanline_data_and_filter_method) in
        from.chunks_exact(scanline_width + 1).enumerate()
    {
        let (filter_type, filtered_data) = (
            scanline_data_and_filter_method[0],
            &scanline_data_and_filter_method[1..],
        );

        let scanline_base_index = index * scanline_width;
        let filter = Filter::try_from(filter_type)?;

        let current_scanline = &mut to[scanline_base_index..scanline_base_index + scanline_width];

        // FIXME: this code assumes 3 bytes per pixel
        match filter {
            Filter::None => current_scanline.copy_from_slice(filtered_data),
            Filter::Sub => {
                // Unfiltered = Filtered + a
                let mut a = vec![0; pixel_width];
                for (unfiltered_pixel, filtered_pixel) in current_scanline
                    .chunks_exact_mut(pixel_width)
                    .zip(filtered_data.chunks_exact(pixel_width))
                {
                    for i in 0..pixel_width {
                        unfiltered_pixel[i] = filtered_pixel[i].wrapping_add(a[i]);
                    }
                    a.copy_from_slice(unfiltered_pixel);
                }
            },
            Filter::Up => {
                // Unfiltered = Filtered + b
                for ((unfiltered_pixel, filtered_pixel), b) in current_scanline
                    .chunks_exact_mut(pixel_width)
                    .zip(filtered_data.chunks_exact(pixel_width))
                    .zip(previous_scanline.chunks_exact(pixel_width))
                {
                    for i in 0..pixel_width {
                        unfiltered_pixel[i] = filtered_pixel[i].wrapping_add(b[i]);
                    }
                }
            },
            Filter::Average => {
                // Unfiltered = Filtered + (a + b) // 2
                let mut a = vec![0; pixel_width];
                for ((unfiltered_pixel, filtered_pixel), b) in current_scanline
                    .chunks_exact_mut(pixel_width)
                    .zip(filtered_data.chunks_exact(pixel_width))
                    .zip(previous_scanline.chunks_exact(pixel_width))
                {
                    for i in 0..pixel_width {
                        let avg = ((a[i] as u16 + b[i] as u16) / 2) as u8;
                        unfiltered_pixel[i] = filtered_pixel[i].wrapping_add(avg);
                    }
                    a.copy_from_slice(unfiltered_pixel);
                }
            },
            Filter::Paeth => {
                // Unfiltered = Filtered + Unpaeth(a, b, c)
                let mut a = vec![0; pixel_width];
                let mut c = vec![0; pixel_width];

                for ((unfiltered_pixel, filtered_pixel), b) in current_scanline
                    .chunks_exact_mut(pixel_width)
                    .zip(filtered_data.chunks_exact(pixel_width))
                    .zip(previous_scanline.chunks_exact(pixel_width))
                {
                    for i in 0..pixel_width {
                        let paeth_value = paeth(a[i], b[i], c[i]);
                        unfiltered_pixel[i] = filtered_pixel[i].wrapping_add(paeth_value);
                    }

                    a.copy_from_slice(unfiltered_pixel);
                    c.copy_from_slice(b);
                }
            },
        }
        previous_scanline.copy_from_slice(current_scanline);
    }
    Ok(())
}

/// <https://www.w3.org/TR/png/#9Filter-type-4-Paeth>
/// Note that this function only implements a single selection
/// step in the paeth algorithm
#[inline]
#[must_use]
fn paeth(a: u8, b: u8, c: u8) -> u8 {
    // Note that we need to use i16's because all the calculations
    // must be performed without overflows
    let a_i16 = a as i16;
    let b_i16 = b as i16;
    let c_i16 = c as i16;

    let p = a_i16 + b_i16 - c_i16;
    let pa = (p - a_i16).abs();
    let pb = (p - b_i16).abs();
    let pc = (p - c_i16).abs();

    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

#[derive(Clone, Copy, Debug)]
enum Filter {
    None = 0,
    Sub = 1,
    Up = 2,
    Average = 3,
    Paeth = 4,
}

impl TryFrom<u8> for Filter {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Sub),
            2 => Ok(Self::Up),
            3 => Ok(Self::Average),
            4 => Ok(Self::Paeth),
            _ => Err(Error::UnknownFilterType),
        }
    }
}

pub fn texture_from_bytes<C: ColorFormat>(
    data: Vec<u8>,
    width: usize,
    height: usize,
) -> Result<Texture<C, Vec<u8>>, Error>
where
    f32: FloatToInt<C::Channel>,
{
    if data.len() != width * height * C::N_CHANNELS {
        return Err(Error::IncorrectLengthOfImageData);
    }

    let texture = Texture::from_data(data, width, height);

    Ok(texture)
}

impl From<zlib::Error> for Error {
    fn from(value: zlib::Error) -> Self {
        Self::ZLib(value)
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}
