use crate::{jpeg::chunk::Chunk, DynamicTexture, Texture};

use self::chunk::Chunks;

mod chunk;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    BadChunk,
    UnknownChunk,
    UnexpectedChunk,
    IncompleteImage,
}

pub fn decode(bytes: &[u8]) -> Result<DynamicTexture, Error> {
    Decoder::new(bytes).decode()
}

#[derive(Clone)]
struct Decoder<'a> {
    chunks: Chunks<'a>,
    stage: DecoderStage,
}

#[derive(Clone, Debug, Default)]
enum DecoderStage {
    #[default]
    BeforeFrameHeader,
    InFrame {
        frame_header: FrameHeader,
        texture: DynamicTexture,
    },
}

impl<'a> Decoder<'a> {
    #[must_use]
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            chunks: Chunks::new(bytes),
            stage: DecoderStage::default(),
        }
    }

    fn decode(mut self) -> Result<DynamicTexture, Error> {
        let first_chunk = self.chunks.next();
        if !matches!(first_chunk, Some(Ok(Chunk::StartOfImage))) {
            log::error!("Expected SOI chunk, found {first_chunk:?}");
            return Err(Error::UnexpectedChunk);
        }

        for chunk in self.chunks {
            let chunk = chunk?;

            match chunk {
                Chunk::EndOfImage => break,
                Chunk::Comment(_) => {},
                Chunk::ApplicationSpecific { .. } => {},
                Chunk::StartOfFrame { subscript, data } => {
                    let frame_header = FrameHeader::new(subscript, data)?;
                    self.process_frame_header(frame_header)?;
                },
                _ => {},
            }
        }

        let DecoderStage::InFrame {
            frame_header,
            texture,
        } = self.stage
        else {
            log::error!("Decoder diard not terminate with a complete decoded image");
            return Err(Error::IncompleteImage);
        };

        Ok(texture)
    }

    fn process_frame_header(&mut self, frame_header: FrameHeader) -> Result<(), Error> {
        if !matches!(self.stage, DecoderStage::BeforeFrameHeader) {
            return Err(Error::UnexpectedChunk);
        }

        let texture = DynamicTexture::Rgb8(Texture::new(
            frame_header.samples_per_line as usize,
            frame_header.number_of_lines as usize,
        ));

        self.stage = DecoderStage::InFrame {
            frame_header: frame_header,
            texture,
        };

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
struct FrameHeader {
    subscript: u8,

    /// Sample precision in bits
    sample_precision: u8,

    number_of_lines: u16,
    samples_per_line: u16,
    num_image_components: u8,
}

impl FrameHeader {
    fn new(subscript: u8, bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < 6 {
            return Err(Error::BadChunk);
        }

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
            subscript,
            sample_precision,
            number_of_lines,
            samples_per_line,
            num_image_components,
        };

        Ok(header)
    }
}
