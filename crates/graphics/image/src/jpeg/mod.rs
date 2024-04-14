//! JPEG decoding module
//!
//! ## Resources
//! * <https://www.w3.org/Graphics/JPEG/itu-t81.pdf>
//! * <https://www.w3.org/Graphics/JPEG/>
//! * <https://imrannazar.com/series/lets-build-a-jpeg-decoder>
//! * <http://www.opennet.ru/docs/formats/jpeg.txt>

mod bit_reader;
mod chunk;
mod huffman_table;
mod quantization_table;

use crate::{jpeg::chunk::Chunk, Texture};

use self::{chunk::Chunks, huffman_table::HuffmanTables, quantization_table::QuantizationTables};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    BadChunk,
    UnknownChunk,
    UnexpectedChunk,
    IncompleteImage,

    /// Failed to parse frame data
    BadFrame,

    /// A `DHT` chunk failed to parse
    BadHuffmanTable,

    /// A `DQT` chunk failed to parse
    BadQuantizationTable,
}

pub fn decode(bytes: &[u8]) -> Result<Texture, Error> {
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
        texture: Texture,
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

    fn decode(mut self) -> Result<Texture, Error> {
        let first_chunk = self.chunks.next();
        if !matches!(first_chunk, Some(Ok(Chunk::StartOfImage))) {
            log::error!("Expected SOI chunk, found {first_chunk:?}");
            return Err(Error::UnexpectedChunk);
        }

        let mut huffman_tables = HuffmanTables::default();
        let mut quantization_tables = QuantizationTables::default();
        for chunk in self.chunks {
            let chunk = chunk?;

            match chunk {
                Chunk::EndOfImage => break,
                Chunk::Comment(_) => {},
                Chunk::ApplicationSpecific { .. } => {},
                Chunk::StartOfFrame { subscript, data } => {
                    let frame_header = FrameHeader::new(subscript, data)?;
                    process_frame(frame_header, self.chunks.remaining())?;
                },
                Chunk::DefineHuffmanTable(huffman_table) => {
                    huffman_tables.add_table(huffman_table)?;
                },
                Chunk::DefineQuantizationTable(quantization_table) => {
                    quantization_tables.add_tables(quantization_table)?;
                },
                Chunk::StartOfScan { header, scan } => {
                    // we don't care about the SOS header, for now
                    _ = header;
                    _ = scan;

                    todo!()
                },
                _ => {},
            }
        }

        let DecoderStage::InFrame {
            frame_header,
            texture,
        } = self.stage
        else {
            log::error!("Decoder did not terminate with a complete decoded image");
            return Err(Error::IncompleteImage);
        };

        Ok(texture)
    }

    fn process_frame_header(&mut self, frame_header: FrameHeader) -> Result<(), Error> {
        if !matches!(self.stage, DecoderStage::BeforeFrameHeader) {
            return Err(Error::UnexpectedChunk);
        }

        let texture = Texture::new(
            frame_header.samples_per_line as usize,
            frame_header.number_of_lines as usize,
        );

        self.stage = DecoderStage::InFrame {
            frame_header: frame_header,
            texture,
        };

        Ok(())
    }
}

fn process_frame(frame_header: FrameHeader, frame_bytes: &[u8]) -> Result<(), Error> {
    let texture = DynamicTexture::Rgb8(Texture::new(
        frame_header.samples_per_line as usize,
        frame_header.number_of_lines as usize,
    ));

    let mut bytes = frame_bytes.iter();

    // Read the image components
    let mut components = Vec::with_capacity(frame_header.num_image_components as usize);
    for _ in 0..frame_header.num_image_components {
        let id = *bytes.next().ok_or(Error::BadFrame)?;
        let sampling = *bytes.next().ok_or(Error::BadFrame)?;
        let q_table = *bytes.next().ok_or(Error::BadFrame)?;

        let component = Component {
            id,
            _sampling: sampling,
            _q_table: q_table,
        };
        components.push(component.id);
    }

    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct Component {
    id: u8,
    _sampling: u8,
    _q_table: u8,
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
