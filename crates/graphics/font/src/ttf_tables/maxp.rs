//! [MaxP](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6maxp.html) table implementation.

use crate::ttf::read_u16_at;

#[derive(Clone, Copy, Debug)]
pub struct MaxPTable {
    /// Number of glyphs defined in the font
    pub num_glyphs: u16,

    /// Maximum number of storage units used by the interpreter
    pub max_storage: u16,

    /// Maximum number of function definitions
    pub max_function_defs: u16,
}

impl MaxPTable {
    #[inline]
    #[must_use]
    pub fn new(data: &[u8]) -> Self {
        Self {
            num_glyphs: read_u16_at(data, 4),
            max_storage: read_u16_at(data, 18),
            max_function_defs: read_u16_at(data, 20),
        }
    }
}
