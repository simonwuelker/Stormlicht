use crate::jpeg::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntropyCoding {
    Arithmetic,
    Huffman,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodingScheme {
    SequentialDiscreteCosineTransform,
    ProgressiveDiscreteCosineTransform,
    Lossless,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IsDifferential {
    Yes,
    No,
}

#[derive(Clone, Copy, Debug)]
pub struct FrameHeader {
    pub entropy_coding: EntropyCoding,
    pub coding_scheme: CodingScheme,
    pub is_differential: IsDifferential,

    /// Sample precision in bits
    #[allow(dead_code)] // FIXME: use this
    pub sample_precision: u8,

    /// The height of the image
    ///
    /// If there are multiple components then this is the height of the largest one.
    pub number_of_lines: u16,

    /// The width of the image
    ///
    /// If there are multiple components then this is the width of the largest one.
    pub samples_per_line: u16,
    pub num_image_components: u8,
}

impl FrameHeader {
    pub fn new(subscript: u8, bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < 6 {
            return Err(Error::BadChunk);
        }

        let (coding_scheme, entropy_coding, is_differential) = decode_subscript(subscript)?;

        let sample_precision = bytes[0];
        let number_of_lines = u16::from_be_bytes(
            bytes[1..3]
                .try_into()
                .expect("Slice is exactly two elements long"),
        );
        let samples_per_line = u16::from_be_bytes(
            bytes[3..5]
                .try_into()
                .expect("Slice is exactly two elements long"),
        );
        let num_image_components = bytes[5];

        let header = Self {
            coding_scheme,
            entropy_coding,
            is_differential,
            sample_precision,
            number_of_lines,
            samples_per_line,
            num_image_components,
        };

        Ok(header)
    }
}

fn decode_subscript(subscript: u8) -> Result<(CodingScheme, EntropyCoding, IsDifferential), Error> {
    let schemes = match subscript {
        0 | 1 => (
            CodingScheme::SequentialDiscreteCosineTransform,
            EntropyCoding::Huffman,
            IsDifferential::No,
        ),
        2 => (
            CodingScheme::ProgressiveDiscreteCosineTransform,
            EntropyCoding::Huffman,
            IsDifferential::No,
        ),
        3 => (
            CodingScheme::Lossless,
            EntropyCoding::Huffman,
            IsDifferential::No,
        ),
        5 => (
            CodingScheme::SequentialDiscreteCosineTransform,
            EntropyCoding::Huffman,
            IsDifferential::Yes,
        ),
        6 => (
            CodingScheme::ProgressiveDiscreteCosineTransform,
            EntropyCoding::Huffman,
            IsDifferential::Yes,
        ),
        7 => (
            CodingScheme::Lossless,
            EntropyCoding::Huffman,
            IsDifferential::Yes,
        ),
        8 => {
            log::error!("Use of reserved SOF marker: 8");
            return Err(Error::Unsupported);
        },
        9 => (
            CodingScheme::SequentialDiscreteCosineTransform,
            EntropyCoding::Arithmetic,
            IsDifferential::No,
        ),
        10 => (
            CodingScheme::ProgressiveDiscreteCosineTransform,
            EntropyCoding::Arithmetic,
            IsDifferential::No,
        ),
        11 => (
            CodingScheme::Lossless,
            EntropyCoding::Arithmetic,
            IsDifferential::No,
        ),
        13 => (
            CodingScheme::SequentialDiscreteCosineTransform,
            EntropyCoding::Arithmetic,
            IsDifferential::Yes,
        ),
        14 => (
            CodingScheme::ProgressiveDiscreteCosineTransform,
            EntropyCoding::Arithmetic,
            IsDifferential::Yes,
        ),
        15 => (
            CodingScheme::Lossless,
            EntropyCoding::Arithmetic,
            IsDifferential::Yes,
        ),
        other => {
            log::error!("Undefined SOF marker: {other}");
            return Err(Error::Unsupported);
        },
    };

    Ok(schemes)
}
