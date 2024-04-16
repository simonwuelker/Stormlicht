//! JPEG decoding module
//!
//! ## Resources
//! * <https://www.w3.org/Graphics/JPEG/itu-t81.pdf>
//! * <https://www.w3.org/Graphics/JPEG/>
//! * <https://imrannazar.com/series/lets-build-a-jpeg-decoder>
//! * <http://www.opennet.ru/docs/formats/jpeg.txt>

mod bit_reader;
mod chunk;
mod colors;
mod cosine_transform;
mod frame_header;
mod huffman_table;
mod quantization_table;

use bit_reader::BitReader;
use chunk::{Chunk, Chunks};
use huffman_table::HuffmanTables;
use quantization_table::{QuantizationTable, QuantizationTables};

use crate::{jpeg::cosine_transform::dequantize_and_perform_idct, Texture};

use self::frame_header::{CodingScheme, EntropyCoding, FrameHeader, IsDifferential};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    BadChunk,
    UnknownChunk,
    UnexpectedChunk,
    IncompleteImage,
    UndefinedQuantizationTable,
    UndefinedHuffmanTable,

    ZeroInQuantizationTable,

    /// Failed to parse frame data
    BadFrame,

    /// A `DHT` chunk failed to parse
    BadHuffmanTable,

    /// A `DQT` chunk failed to parse
    BadQuantizationTable,

    /// Magnitude difference outside of valid range (0-11)
    ///
    /// See Figure F.1
    InvalidDcMagnitudeDifference,

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

                    decoder.process_frame(subscript, data)?;
                },
                Chunk::DefineHuffmanTable(huffman_table) => {
                    decoder.huffman_tables.add_table(huffman_table)?;
                },
                Chunk::DefineQuantizationTable(quantization_table) => {
                    decoder.quantization_tables.add_tables(quantization_table)?;
                },
                Chunk::StartOfScan { header, scan } => {
                    decoder.decode_scan(header, scan)?;
                },
                _ => {},
            }
        }

        let Some(frame) = decoder.current_frame else {
            log::error!("Decoder terminated without a frame");
            return Err(Error::IncompleteImage);
        };

        Ok(frame.texture)
    }

    fn process_frame(&mut self, subscript: u8, bytes: &[u8]) -> Result<(), Error> {
        let frame_header = FrameHeader::new(subscript, bytes)?;

        if frame_header.is_differential == IsDifferential::Yes {
            log::warn!("Differential encodings are not implemented")
        }

        if frame_header.entropy_coding == EntropyCoding::Arithmetic {
            log::warn!("Arithmetic entropy encodings are not implemented")
        }

        if frame_header.coding_scheme != CodingScheme::SequentialDiscreteCosineTransform {
            log::warn!(
                "Lossless/Progressive coding schemes are not implemented (image uses {:?})",
                frame_header.coding_scheme
            );
        }

        let texture = Texture::new(
            frame_header.samples_per_line as usize,
            frame_header.number_of_lines as usize,
        );

        let frame = Frame {
            header: frame_header,
            texture,
        };
        self.current_frame = Some(frame);

        // Read the image components
        if frame_header.num_image_components != 3 {
            log::error!(
                "Image has {} components but we only understand images with 3 components (YCbCr)",
                frame_header.num_image_components
            )
        }

        let mut bytes = bytes[6..].iter();
        for component_id in 0..frame_header.num_image_components {
            let _id = *bytes.next().ok_or(Error::BadFrame)?;
            let _sampling_factor = *bytes.next().ok_or(Error::BadFrame)?;
            let used_quantization_table = *bytes.next().ok_or(Error::BadFrame)?;

            self.quantization_table_mapping[component_id as usize] = used_quantization_table;
        }

        Ok(())
    }

    // Section B.2.3
    fn decode_scan(&mut self, header: &[u8], scan: Vec<u8>) -> Result<(), Error> {
        let Some(frame) = &mut self.current_frame else {
            log::error!("Start of scan without frame header");
            return Err(Error::IncompleteImage);
        };

        // Decode scan header
        // See Figure B.4
        let Some(&num_components) = header.get(0) else {
            return Err(Error::BadChunk);
        };

        struct ScanComponentData {
            #[allow(dead_code)] // FIXME: use this
            component_selector: u8,
            ac_table: u8,
            dc_table: u8,
        }

        let mut component_data = vec![];
        for i in 0..num_components as usize {
            let component_selector = *header.get(1 + i * 2).ok_or(Error::BadChunk)?;
            let td_ta = header.get(2 + i * 2).ok_or(Error::BadChunk)?;

            let dc_table = td_ta >> 4;
            let ac_table = td_ta & 0xF;

            let component = ScanComponentData {
                component_selector,
                ac_table,
                dc_table,
            };

            component_data.push(component);
        }

        // Start of spectral or predictor selection
        let ss = *header
            .get(1 + num_components as usize * 2)
            .ok_or(Error::BadChunk)?;
        _ = ss;

        // End of spectral or predictor selection
        let se = *header
            .get(2 + num_components as usize * 2)
            .ok_or(Error::BadChunk)?;
        _ = se;

        let ah_al = *header
            .get(3 + num_components as usize * 2)
            .ok_or(Error::BadChunk)?;
        _ = ah_al;

        // Decode the actual scan data
        let mut bit_reader = BitReader::new(&scan);

        // (Matrix, Coefficient) for each component
        let mut component_matrices = vec![([0; 64], 0); num_components as usize];
        for y in 0..frame.header.number_of_lines / 8 {
            for x in 0..frame.header.samples_per_line / 8 {
                // Update component matrices for this MCU
                for (i, (matrix, coefficient)) in component_matrices.iter_mut().enumerate() {
                    let used_quantization_table = self
                        .quantization_tables
                        .get(self.quantization_table_mapping[i])?;

                    let mut coefficients = [0; 64];
                    decode_coefficients(
                        &mut coefficients,
                        &mut bit_reader,
                        component_data[i].dc_table,
                        component_data[i].ac_table,
                        used_quantization_table,
                        &self.huffman_tables,
                        coefficient,
                    )?;

                    // De-quantize the coefficients
                    dequantize_and_perform_idct(&coefficients, used_quantization_table, matrix)
                }

                // Write the decoded block to the texture
                for block_offset_y in 0..8 {
                    for block_offset_x in 0..8 {
                        let index =
                            cosine_transform::MATRIX_INDEX_TO_ORDER[block_offset_x][block_offset_y];

                        let luminance = component_matrices[0].0[index];
                        let cr = component_matrices[1].0[index];
                        let cb = component_matrices[2].0[index];

                        let color = colors::ycbcr_to_rgb(luminance as f32, cb as f32, cr as f32);

                        frame.texture.set_pixel(
                            x as usize * 8 + block_offset_x,
                            y as usize * 8 + block_offset_y,
                            color,
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

fn decode_coefficients(
    coefficients: &mut [i16; 64],
    reader: &mut BitReader<'_>,
    dc_table: u8,
    ac_table: u8,
    quantization_table: &QuantizationTable,
    huffman_tables: &HuffmanTables,
    dc_coefficient: &mut i16,
) -> Result<(), Error> {
    // F.2.2.1
    let length_of_magnitude_diff = huffman_tables
        .get(dc_table)?
        .lookup_code_from_reader(reader);
    let magnitude_difference = match length_of_magnitude_diff {
        0 => 0,
        1..=11 => reader.get_bits_extended(length_of_magnitude_diff),
        _ => {
            // Section F.1.2.1.1 / Figure F.1
            return Err(Error::InvalidDcMagnitudeDifference);
        },
    };

    *dc_coefficient += magnitude_difference;

    // Assign DC
    coefficients[0] = *dc_coefficient * quantization_table[0] as i16;

    // Assign AC values
    let mut ac_index = 1;
    while ac_index < 64 {
        let code = huffman_tables
            .get(ac_table + 16)?
            .lookup_code_from_reader(reader);

        if code == 0 {
            break;
        }

        let n_ac_codes_to_skip = code >> 4;
        let length_of_v = code & 0xF;

        ac_index += n_ac_codes_to_skip;

        let ac_coefficient = reader.get_bits_extended(length_of_v);

        if ac_index < 64 {
            coefficients[ac_index as usize] =
                ac_coefficient * quantization_table[ac_index as usize] as i16;
            ac_index += 1;
        }
    }

    Ok(())
}
