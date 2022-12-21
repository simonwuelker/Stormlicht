//! Implements https://en.wikipedia.org/wiki/Xorshift

pub struct RNG {
    state: u64,
}

impl RNG {
    pub fn next_u64(&mut self) -> u64 {
        self.state  ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() & u32::MAX as u64) as u32
    }

    pub fn next_u16(&mut self) -> u16 {
        (self.next_u64() & u16::MAX as u64) as u16
    }

    pub fn next_u8(&mut self) -> u8 {
        (self.next_u64() & u8::MAX as u64) as u8
    }
}

impl Default for RNG {
    fn default() -> Self {
        Self {
            state: 0xcafebabedeadbeef,
        }
    }
}