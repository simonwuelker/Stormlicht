use crate::ttf::{read_u16_at, read_u32_at};

#[derive(Clone, Debug)]
pub struct KernTable {}

#[derive(Clone, Copy, Debug)]
pub enum Error {
    /// Expected version `0x00010000`
    InvalidVersion,

    InvalidCoverage,

    UnexpectedEndOfTable,
}

impl KernTable {
    pub fn new(bytes: &[u8]) -> Result<Self, Error> {
        const MAX_SUBTABLES_TO_RESERVE: usize = 16; // Ad-hoc

        if bytes.len() < 8 {
            return Err(Error::UnexpectedEndOfTable);
        }

        // FIXME: Apparently older fonts sometimes represent these
        //        first two fonts as 16 bit sometimes. Is this something
        //        that we need to worry about in practice?
        if read_u32_at(bytes, 0) != 0x00010000 {
            return Err(Error::InvalidVersion);
        }

        let n_subtables = read_u32_at(bytes, 4) as usize;
        let mut subtable_list = Vec::with_capacity(n_subtables.min(MAX_SUBTABLES_TO_RESERVE));

        let mut cursor = 8;
        for _ in 0..n_subtables {
            let remaining = &bytes[cursor..];
            if remaining.len() < 8 {
                return Err(Error::UnexpectedEndOfTable);
            }

            let length = read_u32_at(remaining, 0) as usize;

            let kind = match remaining[4] & !0x1F {
                0x80 => Kind::Vertical,
                0x40 => Kind::CrossStream,
                0x20 => {
                    let tuple_index = read_u16_at(remaining, 6);
                    Kind::Variation { tuple_index }
                },
                _ => return Err(Error::InvalidCoverage),
            };

            let format = remaining[5];

            let content = &remaining[8..length];

            let subtable = Subtable {
                kind,
                format,
                bytes: content,
            };
            cursor += length;
            log::info!("subtable kind {:?}", subtable.format);
            subtable_list.push(subtable);
        }
        panic!()
    }
}

#[derive(Clone, Copy, Debug)]
enum Kind {
    Vertical,
    CrossStream,
    Variation { tuple_index: u16 },
}

#[derive(Clone, Copy, Debug)]
struct Subtable<'a> {
    kind: Kind,
    format: u8,
    bytes: &'a [u8],
}
