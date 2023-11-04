const BASE: u32 = 65521;

pub fn adler32(bytes: &[u8]) -> u32 {
    let mut hasher = Adler32Hasher::default();
    hasher.write(bytes);
    hasher.finish()
}

#[derive(Clone, Copy, Debug)]
pub struct Adler32Hasher(u32, u32);

impl Default for Adler32Hasher {
    fn default() -> Self {
        Self(1, 0)
    }
}

impl Adler32Hasher {
    pub fn write(&mut self, bytes: &[u8]) {
        let mut chunks = bytes.chunks_exact(5552);
        for chunk in chunks.by_ref() {
            for inner_chunk in chunk.chunks_exact(16) {
                for b in inner_chunk {
                    self.0 += *b as u32;
                    self.1 += self.0;
                }
            }

            self.0 %= BASE;
            self.1 %= BASE;
        }

        let mut trailing_chunks = chunks.remainder().chunks_exact(16);
        for inner_chunk in trailing_chunks.by_ref() {
            for b in inner_chunk {
                self.0 += *b as u32;
                self.1 += self.0;
            }
        }

        let trailing_bytes = trailing_chunks.remainder();
        for b in trailing_bytes {
            self.0 += *b as u32;
            self.1 += self.0;
        }

        self.0 %= BASE;
        self.1 %= BASE;
    }

    pub fn finish(&self) -> u32 {
        self.1 << 16 | self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adler32() {
        let text = b"Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumx eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet.";
        let mut hasher = Adler32Hasher::default();
        hasher.write(&text[..]);
        assert_eq!(hasher.finish(), 0x6d7fd7c8);
    }
}
