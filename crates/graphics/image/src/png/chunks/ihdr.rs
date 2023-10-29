//! [IHDR](https://www.w3.org/TR/png/#11IHDR) chunk

#[derive(Clone, Copy, Debug)]
pub enum ImageHeaderError {
    // NOTE: A value is considered to be "unknown" if the specification reserves it for future use.
    // Otherwise, it is "invalid".
    InvalidImageType,
    UnknownCompressionMethod,
    UnknownFilterMethod,
    UnknownInterlaceMethod,
    // NOTE: not all image-type/bit depth combinations are allowed
    DisallowedBitDepth,
    IncorrectNumberOfBytes,
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
    pub fn new(data: &[u8]) -> Result<Self, ImageHeaderError> {
        if data.len() != 13 {
            log::warn!("IHDR length must be exactly 13 bytes, found {}", data.len());
            return Err(ImageHeaderError::IncorrectNumberOfBytes);
        }

        let width = u32::from_be_bytes(data[0..4].try_into().unwrap());
        let height = u32::from_be_bytes(data[4..8].try_into().unwrap());
        let bit_depth = data[8];
        let image_type: ImageType = data[9].try_into()?;
        let compression_method = data[10];
        let filter_method = data[11];
        let interlace_method = data[12].try_into()?;

        if !image_type.is_allowed_bit_depth(bit_depth) {
            log::warn!("Bit depth {bit_depth} is not allowed for image type {image_type:?}");
            return Err(ImageHeaderError::DisallowedBitDepth);
        }

        if compression_method != 0 {
            log::warn!("Unknown compression method: {compression_method}");
            return Err(ImageHeaderError::UnknownCompressionMethod);
        }

        if filter_method != 0 {
            log::warn!("Unknown filter method: {filter_method}");
            return Err(ImageHeaderError::UnknownFilterMethod);
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
            _ => {
                log::warn!("Unknown image type: {value}");
                Err(ImageHeaderError::InvalidImageType)
            },
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
            _ => {
                log::warn!("Unknown interlace method: {value}");
                Err(ImageHeaderError::UnknownInterlaceMethod)
            },
        }
    }
}
