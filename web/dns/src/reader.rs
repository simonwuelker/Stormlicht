use std::io;

use crate::{DNSError, Domain};

/// A special type of reader that allows backward references
/// like they are used by the DNS protocol
pub struct Reader<'a> {
    cursor: io::Cursor<&'a [u8]>,
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            cursor: io::Cursor::new(bytes),
        }
    }

    pub fn domain_at(&mut self, offset: u64) -> Result<Domain, DNSError> {
        let old_position = self.cursor.position();
        self.cursor.set_position(offset);
        let domain = Domain::read_from(self)?;
        self.cursor.set_position(old_position);
        Ok(domain)
    }

    pub fn position(&self) -> u64 {
        self.cursor.position()
    }

    pub fn set_position(&mut self, position: u64) {
        self.cursor.set_position(position);
    }
}

impl<'a> io::Read for Reader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.cursor.read(buf)
    }
}
