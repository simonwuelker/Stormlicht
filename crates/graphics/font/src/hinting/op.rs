pub type Opcode = u8;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#ELSE>
pub const ELSE: Opcode = 0x1B;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#CALL>
pub const CALL: Opcode = 0x2B;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#FDEF>
pub const FDEF: Opcode = 0x2C;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#ENDF>
pub const ENDF: Opcode = 0x2D;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#RS>
pub const RS: Opcode = 0x43;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#MPPEM>
pub const MPPEM: Opcode = 0x4B;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#LT>
pub const LT: Opcode = 0x50;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#LTEQ>
pub const LTEQ: Opcode = 0x51;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#IF>
pub const IF: Opcode = 0x58;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#EIF>
pub const EIF: Opcode = 0x59;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#SDS>
pub const SDS: Opcode = 0x5F;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#AA>
pub const AA: Opcode = 0x7F;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#PUSHB>
pub const PUSHB_START: Opcode = 0xB0;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#PUSHB>
pub const PUSHB_END: Opcode = 0xB7;
