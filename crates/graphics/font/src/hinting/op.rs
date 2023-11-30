pub type Opcode = u8;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#SRPO>
pub const SRP0: Opcode = 0x10;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#SRP1>
pub const SRP1: Opcode = 0x11;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#SRP2>
pub const SRP2: Opcode = 0x12;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#SZP0>
pub const SZP0: Opcode = 0x13;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#SZP1>
pub const SZP1: Opcode = 0x14;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#SZP2>
pub const SZP2: Opcode = 0x15;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#SZPS>
pub const SZPS: Opcode = 0x16;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#ELSE>
pub const ELSE: Opcode = 0x1B;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#SSW>
pub const SSW: Opcode = 0x1F;

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

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#DEBUG>
pub const DEBUG: Opcode = 0x4F;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#LT>
pub const LT: Opcode = 0x50;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#LTEQ>
pub const LTEQ: Opcode = 0x51;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#IF>
pub const IF: Opcode = 0x58;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#EIF>
pub const EIF: Opcode = 0x59;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#SDB>
pub const SDB: Opcode = 0x5E;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#SDS>
pub const SDS: Opcode = 0x5F;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#ABS>
pub const ABS: Opcode = 0x64;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#AA>
pub const AA: Opcode = 0x7F;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#PUSHB>
pub const PUSHB_START: Opcode = 0xB0;

/// <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM05/Chap5.html#PUSHB>
pub const PUSHB_END: Opcode = 0xB7;
