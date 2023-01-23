//! [IHDR](https://www.w3.org/TR/png/#11IHDR) chunk

use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImageHeaderError {
    // NOTE: A value is considered to be "unknown" if the specification reserves it for future use.
    // Otherwise, it is "invalid".
    #[error("Invalid image type value: {} (allowed values are: 0, 2, 3, 4, 6)", .0)]
    InvalidImageType(u8),
    #[error("Unknown compression method: {}", .0)]
    UnknownCompressionMethod(u8),
    #[error("Unknown filter method: {}", .0)]
    UnknownFilterMethod(u8),
    #[error("Unknown interlace method: {}", .0)]
    UnknownInterlaceMethod(u8),
    #[error("Bit depth {bit_depth} is not allowed for image type {image_type:?}")]
    DisallowedBitDepth {
        image_type: ImageType,
        bit_depth: u8,
    },
}

#[derive(Clone, Copy, Debug)]
pub struct ImageHeader {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub image_type: ImageType,
    pub compression_method: u8,
    pub filter_method: u8,
    pub interlace_method: InterlaceMethod,
}

impl ImageHeader {
    pub fn new(
        width: u32,
        height: u32,
        bit_depth: u8,
        image_type: ImageType,
        compression_method: u8,
        filter_method: u8,
        interlace_method: InterlaceMethod,
    ) -> Result<Self> {
        if !image_type.is_allowed_bit_depth(bit_depth) {
            return Err(ImageHeaderError::DisallowedBitDepth {
                image_type: image_type,
                bit_depth: bit_depth,
            }
            .into());
        }

        if compression_method != 0 {
            return Err(ImageHeaderError::UnknownCompressionMethod(compression_method).into());
        }

        if filter_method != 0 {
            return Err(ImageHeaderError::UnknownFilterMethod(filter_method).into());
        }

        Ok(Self {
            width,
            height,
            bit_depth,
            image_type,
            compression_method,
            filter_method,
            interlace_method,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ImageType {
    /// Each pixel is a greyscale sample
    GrayScale,
    /// Each pixel is an R,G,B triple
    TrueColor,
    /// Each pixel is a palette index; a PLTE chunk shall appear.
    IndexedColor,
    /// Each pixel is a greyscale sample followed by an alpha sample.
    GrayScaleWithAlpha,
    /// Each pixel is an R,G,B triple followed by an alpha sample.
    TrueColorWithAlpha,
}

impl From<ImageType> for canvas::PixelFormat {
    fn from(value: ImageType) -> Self {
        match &value {
            ImageType::GrayScale => Self::GrayScale,
            ImageType::TrueColor => Self::RGB8,
            ImageType::TrueColorWithAlpha => Self::RGBA8,
            _ => todo!(),
        }
    }
}

impl ImageType {
    pub fn is_allowed_bit_depth(&self, bit_depth: u8) -> bool {
        matches!(
            (self, bit_depth),
            (Self::GrayScale, 1 | 2 | 4 | 8 | 16)
                | (Self::TrueColor, 8 | 16)
                | (Self::IndexedColor, 1 | 2 | 4 | 8)
                | (Self::GrayScaleWithAlpha, 8 | 16)
                | (Self::TrueColorWithAlpha, 8 | 16)
        )
    }

    /// Return the number of bytes per pixel in the given format
    pub fn pixel_width(&self) -> usize {
        match self {
            Self::GrayScale => 1,
            Self::GrayScaleWithAlpha => 2,
            Self::TrueColor => 3,
            Self::TrueColorWithAlpha => 4,
            Self::IndexedColor => 1,
        }
    }
}

impl TryFrom<u8> for ImageType {
    type Error = ImageHeaderError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::GrayScale),
            2 => Ok(Self::TrueColor),
            3 => Ok(Self::IndexedColor),
            4 => Ok(Self::GrayScaleWithAlpha),
            6 => Ok(Self::TrueColorWithAlpha),
            _ => Err(ImageHeaderError::InvalidImageType(value)),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum InterlaceMethod {
    None,
    Adam7,
}

impl TryFrom<u8> for InterlaceMethod {
    type Error = ImageHeaderError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Adam7),
            _ => Err(ImageHeaderError::UnknownInterlaceMethod(value)),
        }
    }
}
