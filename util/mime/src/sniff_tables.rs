//! Tables containing byte sequences used to identify a file's [MIMEType](crate::MIMEType).

use std::str::FromStr;

use crate::MIMEType;

/// <https://mimesniff.spec.whatwg.org/#whitespace-byte>
const WHITESPACE: &[u8] = &[0x09, 0x0A, 0x0C, 0x0D, 0x20];

pub const SCRIPTABLE_MIME_TYPES_TABLE: MIMESniffTable<36> = MIMESniffTable([
    // The case-insensitive string "<!DOCTYPE HTML" followed by a tag-terminating byte.
    MIMESniffPattern {
        pattern: &[
            0x3C, 0x21, 0x44, 0x4F, 0x43, 0x54, 0x59, 0x50, 0x45, 0x20, 0x48, 0x54, 0x4D, 0x4C,
            0x20,
        ],
        mask: &[
            0xFF, 0xFF, 0xDF, 0xDF, 0xDF, 0xDF, 0xDF, 0xDF, 0xDF, 0xFF, 0xDF, 0xDF, 0xDF, 0xDF,
            0xFF,
        ],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    MIMESniffPattern {
        pattern: &[
            0x3C, 0x21, 0x44, 0x4F, 0x43, 0x54, 0x59, 0x50, 0x45, 0x20, 0x48, 0x54, 0x4D, 0x4C,
            0x3E,
        ],
        mask: &[
            0xFF, 0xFF, 0xDF, 0xDF, 0xDF, 0xDF, 0xDF, 0xDF, 0xDF, 0xFF, 0xDF, 0xDF, 0xDF, 0xDF,
            0xFF,
        ],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    // The case-insensitive string "<HTML" followed by a tag-terminating byte.
    MIMESniffPattern {
        pattern: &[0x3C, 0x48, 0x54, 0x4D, 0x4C, 0x20],
        mask: &[0xFF, 0xDF, 0xDF, 0xDF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    MIMESniffPattern {
        pattern: &[0x3C, 0x48, 0x54, 0x4D, 0x4C, 0x3E],
        mask: &[0xFF, 0xDF, 0xDF, 0xDF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    // The case-insensitive string "<HEAD" followed by a tag-terminating byte.
    MIMESniffPattern {
        pattern: &[0x3C, 0x48, 0x45, 0x41, 0x44, 0x20],
        mask: &[0xFF, 0xDF, 0xDF, 0xDF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    MIMESniffPattern {
        pattern: &[0x3C, 0x48, 0x45, 0x41, 0x44, 0x3E],
        mask: &[0xFF, 0xDF, 0xDF, 0xDF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    // The case-insensitive string "<SCRIPT" followed by a tag-terminating byte.
    MIMESniffPattern {
        pattern: &[0x3C, 0x53, 0x43, 0x52, 0x49, 0x50, 0x54, 0x20],
        mask: &[0xFF, 0xDF, 0xDF, 0xDF, 0xDF, 0xDF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    MIMESniffPattern {
        pattern: &[0x3C, 0x53, 0x43, 0x52, 0x49, 0x50, 0x54, 0x3E],
        mask: &[0xFF, 0xDF, 0xDF, 0xDF, 0xDF, 0xDF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    // The case-insensitive string "<IFRAME" followed by a tag-terminating byte.
    MIMESniffPattern {
        pattern: &[0x3C, 0x49, 0x46, 0x52, 0x41, 0x4D, 0x45, 0x20],
        mask: &[0xFF, 0xDF, 0xDF, 0xDF, 0xDF, 0xDF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    MIMESniffPattern {
        pattern: &[0x3C, 0x49, 0x46, 0x52, 0x41, 0x4D, 0x45, 0x3E],
        mask: &[0xFF, 0xDF, 0xDF, 0xDF, 0xDF, 0xDF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    // The case-insensitive string "<H1" followed by a tag-terminating byte.
    MIMESniffPattern {
        pattern: &[0x3C, 0x48, 0x31, 0x20],
        mask: &[0xFF, 0xDF, 0xFF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    MIMESniffPattern {
        pattern: &[0x3C, 0x48, 0x31, 0x3E],
        mask: &[0xFF, 0xDF, 0xFF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    // The case-insensitive string "<DIV" followed by a tag-terminating byte.
    MIMESniffPattern {
        pattern: &[0x3C, 0x44, 0x49, 0x56, 0x20],
        mask: &[0xFF, 0xDF, 0xDF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    MIMESniffPattern {
        pattern: &[0x3C, 0x44, 0x49, 0x56, 0x3E],
        mask: &[0xFF, 0xDF, 0xDF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    // The case-insensitive string "<FONT" followed by a tag-terminating byte.
    MIMESniffPattern {
        pattern: &[0x3C, 0x46, 0x4F, 0x4E, 0x54, 0x20],
        mask: &[0xFF, 0xDF, 0xDF, 0xDF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    MIMESniffPattern {
        pattern: &[0x3C, 0x46, 0x4F, 0x4E, 0x54, 0x3E],
        mask: &[0xFF, 0xDF, 0xDF, 0xDF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    // The case-insensitive string "<TABLE" followed by a tag-terminating byte.
    MIMESniffPattern {
        pattern: &[0x3C, 0x54, 0x41, 0x42, 0x4C, 0x45, 0x20],
        mask: &[0xFF, 0xDF, 0xDF, 0xDF, 0xDF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    MIMESniffPattern {
        pattern: &[0x3C, 0x54, 0x41, 0x42, 0x4C, 0x45, 0x3E],
        mask: &[0xFF, 0xDF, 0xDF, 0xDF, 0xDF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    // The case-insensitive string "<A" followed by a tag-terminating byte.
    MIMESniffPattern {
        pattern: &[0x3C, 0x41, 0x20],
        mask: &[0xFF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    MIMESniffPattern {
        pattern: &[0x3C, 0x41, 0x3E],
        mask: &[0xFF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    // The case-insensitive string "<STYLE" followed by a tag-terminating byte.
    MIMESniffPattern {
        pattern: &[0x3C, 0x53, 0x54, 0x59, 0x4C, 0x45, 0x20],
        mask: &[0xFF, 0xDF, 0xDF, 0xDF, 0xDF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    MIMESniffPattern {
        pattern: &[0x3C, 0x53, 0x54, 0x59, 0x4C, 0x45, 0x3E],
        mask: &[0xFF, 0xDF, 0xDF, 0xDF, 0xDF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    // The case-insensitive string "<TITLE" followed by a tag-terminating byte.
    MIMESniffPattern {
        pattern: &[0x3C, 0x54, 0x49, 0x54, 0x4C, 0x45, 0x20],
        mask: &[0xFF, 0xDF, 0xDF, 0xDF, 0xDF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    MIMESniffPattern {
        pattern: &[0x3C, 0x54, 0x49, 0x54, 0x4C, 0x45, 0x3E],
        mask: &[0xFF, 0xDF, 0xDF, 0xDF, 0xDF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    // The case-insensitive string "<B" followed by a tag-terminating byte.
    MIMESniffPattern {
        pattern: &[0x3C, 0x42, 0x20],
        mask: &[0xFF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    MIMESniffPattern {
        pattern: &[0x3C, 0x42, 0x3E],
        mask: &[0xFF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    // The case-insensitive string "<BODY" followed by a tag-terminating byte.
    MIMESniffPattern {
        pattern: &[0x3C, 0x42, 0x4F, 0x44, 0x59, 0x20],
        mask: &[0xFF, 0xDF, 0xDF, 0xDF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    MIMESniffPattern {
        pattern: &[0x3C, 0x42, 0x4F, 0x44, 0x59, 0x3E],
        mask: &[0xFF, 0xDF, 0xDF, 0xDF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    // The case-insensitive string "<BR" followed by a tag-terminating byte.
    MIMESniffPattern {
        pattern: &[0x3C, 0x42, 0x52, 0x20],
        mask: &[0xFF, 0xDF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    MIMESniffPattern {
        pattern: &[0x3C, 0x42, 0x52, 0x3E],
        mask: &[0xFF, 0xDF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    // The case-insensitive string "<P" followed by a tag-terminating byte.
    MIMESniffPattern {
        pattern: &[0x3C, 0x50, 0x20],
        mask: &[0xFF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    MIMESniffPattern {
        pattern: &[0x3C, 0x50, 0x3E],
        mask: &[0xFF, 0xDF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    // The string "<!--" followed by a tag-terminating byte.
    MIMESniffPattern {
        pattern: &[0x3C, 0x21, 0x2D, 0x2D, 0x20],
        mask: &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    MIMESniffPattern {
        pattern: &[0x3C, 0x21, 0x2D, 0x2D, 0x3E],
        mask: &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/html",
    },
    // The string "<?xml".
    MIMESniffPattern {
        pattern: &[0x3C, 0x3F, 0x78, 0x6D, 0x6C],
        mask: &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
        ignore: WHITESPACE,
        computed_mime: "text/xml",
    },
    // The string "%PDF-", the PDF signature.
    MIMESniffPattern {
        pattern: &[0x25, 0x50, 0x44, 0x46, 0x2D],
        mask: &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
        ignore: &[],
        computed_mime: "application/pdf",
    },
]);

pub const UTF_POSTSCRIPT_MIME_TYPES_TABLE: MIMESniffTable<4> = MIMESniffTable([
    // The string "%!PS-Adobe-", the PostScript signature.
    MIMESniffPattern {
        pattern: &[
            0x25, 0x21, 0x50, 0x53, 0x2D, 0x41, 0x64, 0x6F, 0x62, 0x65, 0x2D,
        ],
        mask: &[
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ],
        ignore: &[],
        computed_mime: "application/postscript",
    },
    // UTF-16BE BOM
    MIMESniffPattern {
        pattern: &[0xFE, 0xFF, 0x00, 0x00],
        mask: &[0xFF, 0xFF, 0x00, 0x00],
        ignore: &[],
        computed_mime: "text/plain",
    },
    // UTF-16LE BOM
    MIMESniffPattern {
        pattern: &[0xFF, 0xFE, 0x00, 0x00],
        mask: &[0xFF, 0xFF, 0x00, 0x00],
        ignore: &[],
        computed_mime: "text/plain",
    },
    // UTF-8 BOM
    MIMESniffPattern {
        pattern: &[0xEF, 0xBB, 0xBF, 0x00],
        mask: &[0xFF, 0xFF, 0xFF, 0x00],
        ignore: &[],
        computed_mime: "text/plain",
    },
]);

pub const IMAGE_MIME_TYPES_TABLE: MIMESniffTable<8> = MIMESniffTable([
    // A Windows Icon signature.
    MIMESniffPattern {
        pattern: &[0x00, 0x00, 0x01, 0x00],
        mask: &[0xFF, 0xFF, 0xFF, 0xFF],
        ignore: &[],
        computed_mime: "image/x-icon",
    },
    // A Windows Cursor signature.
    MIMESniffPattern {
        pattern: &[0x00, 0x00, 0x02, 0x00],
        mask: &[0xFF, 0xFF, 0xFF, 0xFF],
        ignore: &[],
        computed_mime: "image/x-icon",
    },
    // The string "BM", a BMP signature.
    MIMESniffPattern {
        pattern: &[0x42, 0x4D],
        mask: &[0xFF, 0xFF],
        ignore: &[],
        computed_mime: "image/bmp",
    },
    // The string "GIF87a", a GIF signature.
    MIMESniffPattern {
        pattern: &[0x47, 0x49, 0x46, 0x38, 0x37, 0x61],
        mask: &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
        ignore: &[],
        computed_mime: "image/gif",
    },
    // The string "GIF89a", a GIF signature.
    MIMESniffPattern {
        pattern: &[0x47, 0x49, 0x46, 0x38, 0x39, 0x61],
        mask: &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
        ignore: &[],
        computed_mime: "image/gif",
    },
    // The string "RIFF" followed by four bytes followed by the string "WEBPVP".
    MIMESniffPattern {
        pattern: &[
            0x52, 0x49, 0x46, 0x46, 0x00, 0x00, 0x00, 0x00, 0x57, 0x45, 0x42, 0x50, 0x56, 0x50,
        ],
        mask: &[
            0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ],
        ignore: &[],
        computed_mime: "image/webp",
    },
    // An error-checking byte followed by the string "PNG" followed by CR LF SUB LF, the PNG signature.
    MIMESniffPattern {
        pattern: &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
        mask: &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
        ignore: &[],
        computed_mime: "image/png",
    },
    // The JPEG Start of Image marker followed by the indicator byte of another marker.
    MIMESniffPattern {
        pattern: &[0xFF, 0xD8, 0xFF],
        mask: &[0xFF, 0xFF, 0xFF],
        ignore: &[],
        computed_mime: "image/jpeg",
    },
]);

pub const AUDIO_OR_VIDEO_MIME_TYPES_TABLE: MIMESniffTable<6> = MIMESniffTable([
    // The string "FORM" followed by four bytes followed by the string "AIFF", the AIFF signature.
    MIMESniffPattern {
        pattern: &[
            0x46, 0x4F, 0x52, 0x4D, 0x00, 0x00, 0x00, 0x00, 0x41, 0x49, 0x46, 0x46,
        ],
        mask: &[
            0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF,
        ],
        ignore: &[],
        computed_mime: "audio/aiff",
    },
    // The string "ID3", the ID3v2-tagged MP3 signature.
    MIMESniffPattern {
        pattern: &[0x49, 0x44, 0x33],
        mask: &[0xFF, 0xFF, 0xFF],
        ignore: &[],
        computed_mime: "audio/mpeg",
    },
    // The string "OggS" followed by NUL, the Ogg container signature.
    MIMESniffPattern {
        pattern: &[0x4F, 0x67, 0x67, 0x53, 0x00],
        mask: &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
        ignore: &[],
        computed_mime: "application/ogg",
    },
    // The string "MThd" followed by four bytes representing the number 6 in 32 bits
    // (big-endian), the MIDI signature.
    MIMESniffPattern {
        pattern: &[0x4D, 0x54, 0x68, 0x64, 0x00, 0x00, 0x00, 0x06],
        mask: &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
        ignore: &[],
        computed_mime: "audio/midi",
    },
    // The string "RIFF" followed by four bytes followed by the string "AVI ", the AVI signature.
    MIMESniffPattern {
        pattern: &[
            0x52, 0x49, 0x46, 0x46, 0x00, 0x00, 0x00, 0x00, 0x41, 0x56, 0x49, 0x20,
        ],
        mask: &[
            0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF,
        ],
        ignore: &[],
        computed_mime: "video/avi",
    },
    // The string "RIFF" followed by four bytes followed by the string "WAVE", the WAVE signature.
    MIMESniffPattern {
        pattern: &[
            0x52, 0x49, 0x46, 0x46, 0x00, 0x00, 0x00, 0x00, 0x57, 0x41, 0x56, 0x45,
        ],
        mask: &[
            0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF,
        ],
        ignore: &[],
        computed_mime: "audio/wave",
    },
]);

pub const ARCHIVE_MIME_TYPES_TABLE: MIMESniffTable<3> = MIMESniffTable([
    //  The GZIP archive signature.
    MIMESniffPattern {
        pattern: &[0x1F, 0x8B, 0x08],
        mask: &[0xFF, 0xFF, 0xFF],
        ignore: &[],
        computed_mime: "application/x-gzip",
    },
    // The string "PK" followed by ETX EOT, the ZIP archive signature.
    MIMESniffPattern {
        pattern: &[0x50, 0x4B, 0x03, 0x04],
        mask: &[0xFF, 0xFF, 0xFF, 0xFF],
        ignore: &[],
        computed_mime: "application/zip",
    },
    // The string "Rar " followed by SUB BEL NUL, the RAR archive signature.
    MIMESniffPattern {
        pattern: &[0x52, 0x61, 0x72, 0x20, 0x1A, 0x07, 0x00],
        mask: &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
        ignore: &[],
        computed_mime: "application/zip",
    },
]);

#[derive(Clone, Copy, Debug)]
struct MIMESniffPattern {
    pattern: &'static [u8],
    mask: &'static [u8],
    ignore: &'static [u8],
    computed_mime: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct MIMESniffTable<const N: usize>([MIMESniffPattern; N]);

impl MIMESniffPattern {
    /// <https://mimesniff.spec.whatwg.org/#pattern-matching-algorithm>
    #[inline]
    fn matches(&self, input: &[u8]) -> bool {
        // 1. Assert: pattern’s length is equal to mask’s length.
        assert_eq!(self.pattern.len(), self.mask.len());

        // 2. If input’s length is less than pattern’s length, return false.
        if input.len() < self.pattern.len() {
            return false;
        }

        // 3. Let s be 0.
        let mut s = 0;

        // 4. While s < input’s length:
        while s < input.len() {
            // 1. If ignored does not contain input[s], break.
            if !self.ignore.contains(&input[s]) {
                break;
            }

            // 2. Set s to s + 1.
            s += 1;
        }

        // 5. Let p be 0.
        let mut p = 0;

        // 6. While p < pattern’s length:
        while p < self.pattern.len() {
            // 1. Let maskedData be the result of applying the bitwise AND operator to input[s] and mask[p].
            let masked_data = input[s] & self.mask[p];

            // 2. If maskedData is not equal to pattern[p], return false.
            if masked_data != self.pattern[p] {
                return false;
            }

            // 3. Set s to s + 1.
            s += 1;

            // 4. Set p to p + 1.
            p += 1;
        }

        // 7. Return true.
        true
    }
}

impl<const N: usize> MIMESniffTable<N> {
    pub fn lookup(&self, resource_header: &[u8]) -> Option<MIMEType> {
        for pattern in self.0 {
            if pattern.matches(resource_header) {
                return Some(
                    MIMEType::from_str(pattern.computed_mime)
                        .expect("Parsing a statically defined MIME type definition cannot fail"),
                );
            }
        }
        None
    }
}
