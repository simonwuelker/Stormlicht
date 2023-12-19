//! TLS Record Layer Protocol.

use std::io;

use crate::{
    connection::{TLSError, TLS_VERSION},
    encoding::{Cursor, Decoding},
    enum_encoding,
    handshake::CompressionMethod,
    Encoding,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConnectionEnd {
    Server,
    Client,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum PRFAlgorithm {
    TLS_PRF_SHA256,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BulkCipherAlgorithm {
    Null,
    RC4,
    TDES,
    AES,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CipherType {
    Stream,
    Block,
    AEAD,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum MACAlgorithm {
    Null,
    HMAC_MD5,
    HMAC_SHA1,
    HMAC_SHA256,
    HMAC_SHA384,
    HMAC_SHA512,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SecurityParameters {
    entity: ConnectionEnd,
    prf_algorithm: PRFAlgorithm,
    bulk_cipher_algorithm: BulkCipherAlgorithm,
    cipher_type: CipherType,
    enc_key_length: u8,
    block_length: u8,
    fixed_iv_length: u8,
    record_iv_length: u8,
    mac_algorith: MACAlgorithm,
    mac_length: u8,
    mac_key_length: u8,
    compression_algorithm: CompressionMethod,
    master_secret: [u8; 48],
    client_random: [u8; 32],
    server_random: [u8; 32],
}

enum_encoding!(
    pub enum ContentType(u8) {
        ChangeCipherSpec = 20,
        Alert = 21,
        Handshake = 22,
        ApplicationData = 23,
    }
);

/// The maximum length allowed for an individual TLS record
const TLS_RECORD_MAX_LENGTH: usize = (1 << 14) + 1024;

const BUFFER_SIZE: usize = 1024;

#[derive(Clone, Debug)]
pub struct Record {
    pub content_type: ContentType,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct TLSRecordWriter<W> {
    /// An internal buffer so records can be constructed incrementally
    ///
    /// The buffer contains data *before* it is encrypted
    cursor: io::Cursor<[u8; BUFFER_SIZE]>,

    /// The writer that encrypted TLS records should be written to
    ///
    /// For performance, this writer should be buffered.
    out: W,

    /// The content type of the last message that was sent.
    ///
    /// The Writer remembers this, since multiple consecutive messages
    /// with the same content type can be coalesced into a single record.
    content_type: ContentType,
}

#[derive(Debug)]
pub struct TLSRecordReader<R> {
    reader: R,
}

impl<W: io::Write> TLSRecordWriter<W> {
    #[must_use]
    pub fn new(out: W) -> Self {
        Self {
            cursor: io::Cursor::new([0; BUFFER_SIZE]),
            out,
            content_type: ContentType::Handshake,
        }
    }

    /// The message might not be sent immediately if it gets buffered
    pub fn writer_for(&mut self, content_type: ContentType) -> io::Result<MessageWriter<'_, W>> {
        self.set_content_type(content_type)?;

        Ok(MessageWriter { writer: self })
    }

    #[inline]
    fn set_content_type(&mut self, content_type: ContentType) -> io::Result<()> {
        // Quoting the spec (https://www.rfc-editor.org/rfc/rfc5246#section-6.2.1):
        // > multiple client messages of the same ContentType MAY be coalesced
        // > into a single TLSPlaintext record
        if content_type != self.content_type {
            self.flush_current_record()?;
        }

        self.content_type = content_type;
        Ok(())
    }

    /// Writes the current record to the output stream, *without* flushing *that* stream.
    ///
    /// After calling this function, the internal buffer is guaranteed to be empty
    fn flush_current_record(&mut self) -> io::Result<()> {
        if self.cursor.position() == 0 {
            return Ok(());
        }

        let data: &[u8] = &self.cursor.get_ref()[..self.cursor.position() as usize];

        // FIXME: actually encrypt the data here
        let encrypted = data;
        assert!(encrypted.len() < TLS_RECORD_MAX_LENGTH);

        // Assemble the bytes as they are sent on the wire
        let mut bytes = vec![];
        self.content_type.encode(&mut bytes);
        TLS_VERSION.encode(&mut bytes);
        (encrypted.len() as u16).encode(&mut bytes);
        bytes.extend_from_slice(encrypted);

        self.cursor.set_position(0);

        self.out.write_all(&bytes)
    }

    /// The number of bytes that can be written before the buffer needs to be flushed
    fn remaining_size(&self) -> usize {
        self.cursor.remaining_slice().len()
    }
}

impl<R: io::Read> TLSRecordReader<R> {
    #[must_use]
    pub const fn new(reader: R) -> Self {
        Self { reader }
    }

    pub fn next_record(&mut self) -> Result<Record, TLSError> {
        let mut record_header = [0; 5];
        self.reader.read_exact(&mut record_header)?;
        let mut record_header = Cursor::new(&record_header);

        let content_type = record_header.decode()?;
        let tls_version = record_header.decode()?;
        let data_length = u16::decode(&mut record_header)? as usize;

        if TLS_VERSION < tls_version {
            log::error!("Unsupported TLS version: We implement {TLS_VERSION:?} but the server sent {tls_version:?}");
            return Err(TLSError::UnsupportedTLSVersion);
        }

        let mut data = vec![0; data_length];
        self.reader.read_exact(&mut data)?;

        Ok(Record { content_type, data })
    }
}

/// A short-lived writer for a single TLS message
pub struct MessageWriter<'a, W> {
    writer: &'a mut TLSRecordWriter<W>,
}

impl<'a, W: io::Write> io::Write for MessageWriter<'a, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.len() <= self.writer.remaining_size() {
            // The entire buffer fits into the record, no need
            // for fragmentation
            self.writer.cursor.write_all(buf)?;
        } else {
            // Fill the remaining space in the buffer, then flush it
            self.writer
                .cursor
                .write_all(&buf[..self.writer.remaining_size()])?;
            self.writer.flush_current_record()?;

            // Chunk the remainder into records and flush each one individually
            let buf = &buf[self.writer.remaining_size()..];
            let mut chunks = buf.array_chunks::<TLS_RECORD_MAX_LENGTH>();

            for chunk in &mut chunks {
                self.writer.cursor.write_all(chunk)?;
                self.writer.flush_current_record()?;
            }

            self.writer.cursor.write_all(chunks.remainder())?;
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush_current_record()?;
        self.writer.out.flush()
    }
}
