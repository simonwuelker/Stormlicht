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

use chunk::{Chunk, Chunks};
use huffman_table::HuffmanTables;
use quantization_table::QuantizationTables;

use crate::Texture;

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

    /// A feature is not yet implemented
    Unsupported,
}

pub fn decode(bytes: &[u8]) -> Result<Texture, Error> {
    Decoder::decode(bytes)
}

#[derive(Clone)]
struct Decoder {
    huffman_tables: HuffmanTables,
    quantization_tables: QuantizationTables,

    /// Mapping from image components to their used
    /// quantization tables
    quantization_table_mapping: [u8; 3],

    /// The frame currently being decoded
    current_frame: Option<Frame>,
}

#[derive(Clone)]
struct Frame {
    header: FrameHeader,
    texture: Texture,
}

impl Default for Decoder {
    fn default() -> Self {
        Self {
            huffman_tables: HuffmanTables::default(),
            quantization_tables: QuantizationTables::default(),
            quantization_table_mapping: [0; 3],
            current_frame: None,
        }
    }
}

impl Decoder {
    fn decode(bytes: &[u8]) -> Result<Texture, Error> {
        let mut chunks = Chunks::new(bytes);
        let mut decoder = Self::default();

        let first_chunk = chunks.next();
        if !matches!(first_chunk, Some(Ok(Chunk::StartOfImage))) {
            log::error!("Expected SOI chunk, found {first_chunk:?}");
            return Err(Error::UnexpectedChunk);
        }

        for chunk in chunks {
            let chunk = chunk?;

            match chunk {
                Chunk::EndOfImage => break,
                Chunk::Comment(_) => {},
                Chunk::ApplicationSpecific { .. } => {},
                Chunk::StartOfFrame { subscript, data } => {
                    if decoder.current_frame.is_some() {
                        // Section 4.10
                        log::error!("FIXME: Implement hierarchical jpeg (with multiple frames)");
                        return Err(Error::Unsupported);
                    }

                    let frame_header = FrameHeader::new(subscript, data)?;
                    decoder.process_frame(frame_header, chunks.remaining())?;
                },
                Chunk::DefineHuffmanTable(huffman_table) => {
                    decoder.huffman_tables.add_table(huffman_table)?;
                },
                Chunk::DefineQuantizationTable(quantization_table) => {
                    decoder.quantization_tables.add_tables(quantization_table)?;
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

        todo!();
    }

    fn process_frame(
        &mut self,
        frame_header: FrameHeader,
        frame_bytes: &[u8],
    ) -> Result<(), Error> {
        let texture = Texture::new(
            frame_header.samples_per_line as usize,
            frame_header.number_of_lines as usize,
        );

        let frame = Frame {
            header: frame_header,
            texture,
        };
        self.current_frame = Some(frame);

        let mut bytes = frame_bytes.iter();

        // Read the image components
        if frame_header.num_image_components != 3 {
            log::error!(
                "Image has {} components but we only understand images with 3 components (YCbCr)",
                frame_header.num_image_components
            )
        }

        for component_id in 0..frame_header.num_image_components {
            let _id = *bytes.next().ok_or(Error::BadFrame)?;
            let _sampling_factor = *bytes.next().ok_or(Error::BadFrame)?;
            let used_quantization_table = *bytes.next().ok_or(Error::BadFrame)?;

            self.quantization_table_mapping[component_id as usize] = used_quantization_table;
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
struct FrameHeader {
    /// Specifies whether the encoding process is baseline sequential, extended sequential,
    /// progressive, or lossless, as well as which entropy encoding procedure is used.
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
