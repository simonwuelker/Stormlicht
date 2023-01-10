//! Utilities for the brotli static dictionary.
//!
//! You can find the raw dictionary file at <https://github.com/google/brotli/blob/master/c/common/dictionary.bin>

use std::{fs, io::Read};

use super::BrotliError;

const DICTIONARY_PATH: &'static str = "downloads/brotli/dictionary";
const DICT_SIZE: usize = 122784;

pub fn get_dictionary() -> std::io::Result<[u8; DICT_SIZE]> {
    let mut buffer = Vec::with_capacity(DICT_SIZE);
    let mut dictionary_file = fs::File::open(DICTIONARY_PATH)?;
    dictionary_file.read_to_end(&mut buffer)?;
    Ok(buffer.try_into().unwrap())
}

/// "Ferment" a byte string, as defined in <https://www.rfc-editor.org/rfc/rfc7932#section-8>
///
/// Note that the transformation is performed in-place
fn ferment(word: &mut [u8], pos: usize) -> usize {
    if word[pos] < 192 {
        if word[pos] > 97 && word[pos] <= 122 {
            word[pos] = word[pos] ^ 32;
        }

        1
    } else if word[pos] < 224 {
        if pos + 1 < word.len() {
            word[pos + 1] = word[pos + 1] ^ 32;
        }

        2
    } else {
        if pos + 2 < word.len() {
            word[pos + 2] = word[pos + 2] ^ 5;
        }

        3
    }
}

/// [ferment] the first letter in a byte string
pub fn ferment_first(word: &mut [u8]) {
    if word.len() != 0 {
        ferment(word, 0);
    }
}

/// [ferment] all letters in a byte string, in logical order.
pub fn ferment_all(word: &mut [u8]) {
    for i in 0..word.len() {
        ferment(word, i);
    }
}

pub fn omit_first_n(word: &[u8], n: usize) -> &[u8] {
    if n < word.len() {
        &word[n..]
    } else {
        &word[word.len()..]
    }
}

pub fn omit_last_n(word: &[u8], n: usize) -> &[u8] {
    if n < word.len() {
        &word[..n]
    } else {
        &word[0..0]
    }
}

macro_rules! make_transform {
    ($prefix: expr, identity, $suffix: expr, $word: ident) => {{
        let total_capacity = $prefix.len() + $word.len() + $suffix.len();
        let mut result = Vec::with_capacity(total_capacity);

        result.extend_from_slice($prefix);
        result.extend_from_slice($word);
        result.extend_from_slice($suffix);

        result
    }};

    // ferment
    ($prefix: expr, ferment_first, $suffix: expr, $word: ident) => {{
        let total_capacity = $prefix.len() + $word.len() + $suffix.len();
        let mut result = Vec::with_capacity(total_capacity);

        result.extend_from_slice($prefix);
        result.extend_from_slice($word);
        result.extend_from_slice($suffix);

        ferment_first(&mut result[$prefix.len()..]);

        result
    }};
    ($prefix: expr, ferment_all, $suffix: expr, $word: ident) => {{
        let total_capacity = $prefix.len() + $word.len() + $suffix.len();
        let mut result = Vec::with_capacity(total_capacity);

        result.extend_from_slice($prefix);
        result.extend_from_slice($word);
        result.extend_from_slice($suffix);

        ferment_all(&mut result[$prefix.len()..][..$word.len()]);

        result
    }};

    // omit first
    ($prefix: expr, omit_first_1, $suffix: expr, $word: ident) => {{
        let total_capacity = $prefix.len() + $word.len() - 1 + $suffix.len();
        let mut result = Vec::with_capacity(total_capacity);

        result.extend_from_slice($prefix);
        result.extend_from_slice(omit_first_n($word, 1));
        result.extend_from_slice($suffix);

        result
    }};
    ($prefix: expr, omit_first_2, $suffix: expr, $word: ident) => {{
        let total_capacity = $prefix.len() + $word.len() - 2 + $suffix.len();
        let mut result = Vec::with_capacity(total_capacity);

        result.extend_from_slice($prefix);
        result.extend_from_slice(omit_first_n($word, 2));
        result.extend_from_slice($suffix);

        result
    }};
    ($prefix: expr, omit_first_3, $suffix: expr, $word: ident) => {{
        let total_capacity = $prefix.len() + $word.len() - 3 + $suffix.len();
        let mut result = Vec::with_capacity(total_capacity);

        result.extend_from_slice($prefix);
        result.extend_from_slice(omit_first_n($word, 3));
        result.extend_from_slice($suffix);

        result
    }};
    ($prefix: expr, omit_first_4, $suffix: expr, $word: ident) => {{
        let total_capacity = $prefix.len() + $word.len() - 4 + $suffix.len();
        let mut result = Vec::with_capacity(total_capacity);

        result.extend_from_slice($prefix);
        result.extend_from_slice(omit_first_n($word, 4));
        result.extend_from_slice($suffix);

        result
    }};
    ($prefix: expr, omit_first_5, $suffix: expr, $word: ident) => {{
        let total_capacity = $prefix.len() + $word.len() - 5 + $suffix.len();
        let mut result = Vec::with_capacity(total_capacity);

        result.extend_from_slice($prefix);
        result.extend_from_slice(omit_first_n($word, 5));
        result.extend_from_slice($suffix);

        result
    }};
    ($prefix: expr, omit_first_6, $suffix: expr, $word: ident) => {{
        let total_capacity = $prefix.len() + $word.len() - 6 + $suffix.len();
        let mut result = Vec::with_capacity(total_capacity);

        result.extend_from_slice($prefix);
        result.extend_from_slice(omit_first_n($word, 6));
        result.extend_from_slice($suffix);

        result
    }};
    ($prefix: expr, omit_first_7, $suffix: expr, $word: ident) => {{
        let total_capacity = $prefix.len() + $word.len() - 7 + $suffix.len();
        let mut result = Vec::with_capacity(total_capacity);

        result.extend_from_slice($prefix);
        result.extend_from_slice(omit_first_n($word, 7));
        result.extend_from_slice($suffix);

        result
    }};
    ($prefix: expr, omit_first_8, $suffix: expr, $word: ident) => {{
        let total_capacity = $prefix.len() + $word.len() - 8 + $suffix.len();
        let mut result = Vec::with_capacity(total_capacity);

        result.extend_from_slice($prefix);
        result.extend_from_slice(omit_first_n($word, 8));
        result.extend_from_slice($suffix);

        result
    }};
    ($prefix: expr, omit_first_9, $suffix: expr, $word: ident) => {{
        let total_capacity = $prefix.len() + $word.len() - 9 + $suffix.len();
        let mut result = Vec::with_capacity(total_capacity);

        result.extend_from_slice($prefix);
        result.extend_from_slice(omit_first_n($word, 9));
        result.extend_from_slice($suffix);

        result
    }};

    // omit last
    ($prefix: expr, omit_last_1, $suffix: expr, $word: ident) => {{
        let total_capacity = $prefix.len() + $word.len() - 1 + $suffix.len();
        let mut result = Vec::with_capacity(total_capacity);

        result.extend_from_slice($prefix);
        result.extend_from_slice(omit_last_n($word, 1));
        result.extend_from_slice($suffix);

        result
    }};
    ($prefix: expr, omit_last_2, $suffix: expr, $word: ident) => {{
        let total_capacity = $prefix.len() + $word.len() - 2 + $suffix.len();
        let mut result = Vec::with_capacity(total_capacity);

        result.extend_from_slice($prefix);
        result.extend_from_slice(omit_last_n($word, 2));
        result.extend_from_slice($suffix);

        result
    }};
    ($prefix: expr, omit_last_3, $suffix: expr, $word: ident) => {{
        let total_capacity = $prefix.len() + $word.len() - 3 + $suffix.len();
        let mut result = Vec::with_capacity(total_capacity);

        result.extend_from_slice($prefix);
        result.extend_from_slice(omit_last_n($word, 3));
        result.extend_from_slice($suffix);

        result
    }};
    ($prefix: expr, omit_last_4, $suffix: expr, $word: ident) => {{
        let total_capacity = $prefix.len() + $word.len() - 4 + $suffix.len();
        let mut result = Vec::with_capacity(total_capacity);

        result.extend_from_slice($prefix);
        result.extend_from_slice(omit_last_n($word, 4));
        result.extend_from_slice($suffix);

        result
    }};
    ($prefix: expr, omit_last_5, $suffix: expr, $word: ident) => {{
        let total_capacity = $prefix.len() + $word.len() - 5 + $suffix.len();
        let mut result = Vec::with_capacity(total_capacity);

        result.extend_from_slice($prefix);
        result.extend_from_slice(omit_last_n($word, 5));
        result.extend_from_slice($suffix);

        result
    }};
    ($prefix: expr, omit_last_6, $suffix: expr, $word: ident) => {{
        let total_capacity = $prefix.len() + $word.len() - 6 + $suffix.len();
        let mut result = Vec::with_capacity(total_capacity);

        result.extend_from_slice($prefix);
        result.extend_from_slice(omit_last_n($word, 6));
        result.extend_from_slice($suffix);

        result
    }};
    ($prefix: expr, omit_last_7, $suffix: expr, $word: ident) => {{
        let total_capacity = $prefix.len() + $word.len() - 7 + $suffix.len();
        let mut result = Vec::with_capacity(total_capacity);

        result.extend_from_slice($prefix);
        result.extend_from_slice(omit_last_n($word, 7));
        result.extend_from_slice($suffix);

        result
    }};
    ($prefix: expr, omit_last_8, $suffix: expr, $word: ident) => {{
        let total_capacity = $prefix.len() + $word.len() - 8 + $suffix.len();
        let mut result = Vec::with_capacity(total_capacity);

        result.extend_from_slice($prefix);
        result.extend_from_slice(omit_last_n($word, 8));
        result.extend_from_slice($suffix);

        result
    }};
    ($prefix: expr, omit_last_9, $suffix: expr, $word: ident) => {{
        let total_capacity = $prefix.len() + $word.len() - 9 + $suffix.len();
        let mut result = Vec::with_capacity(total_capacity);

        result.extend_from_slice($prefix);
        result.extend_from_slice(omit_last_n($word, 9));
        result.extend_from_slice($suffix);

        result
    }};
}

pub fn transform(word: &[u8], transform_id: usize) -> Result<Vec<u8>, BrotliError> {
    let transformed = match transform_id {
        0 => make_transform!(b"", identity, b"", word),
        1 => make_transform!(b"", identity, b" ", word),
        2 => make_transform!(b" ", identity, b" ", word),
        3 => make_transform!(b"", omit_first_1, b"", word),
        4 => make_transform!(b"", ferment_first, b" ", word),
        5 => make_transform!(b"", identity, b" the ", word),
        6 => make_transform!(b" ", identity, b"", word),
        7 => make_transform!(b"s ", identity, b" ", word),
        8 => make_transform!(b"", identity, b" of ", word),
        9 => make_transform!(b"", ferment_first, b"", word),
        10 => make_transform!(b"", identity, b" and ", word),
        11 => make_transform!(b"", omit_first_2, b"", word),
        12 => make_transform!(b"", omit_last_1, b"", word),
        13 => make_transform!(b", ", identity, b" ", word),
        14 => make_transform!(b"", identity, b", ", word),
        15 => make_transform!(b" ", ferment_first, b" ", word),
        16 => make_transform!(b"", identity, b" in ", word),
        17 => make_transform!(b"", identity, b" to ", word),
        18 => make_transform!(b"e ", identity, b" ", word),
        19 => make_transform!(b"", identity, b"\"", word),
        20 => make_transform!(b"", identity, b".", word),
        21 => make_transform!(b"", identity, b"\">", word),
        22 => make_transform!(b"", identity, b"\n", word),
        23 => make_transform!(b"", omit_last_3, b"", word),
        24 => make_transform!(b"", identity, b"]", word),
        25 => make_transform!(b"", identity, b" for ", word),
        26 => make_transform!(b"", omit_first_3, b"", word),
        27 => make_transform!(b"", omit_last_2, b"", word),
        28 => make_transform!(b"", identity, b" a ", word),
        29 => make_transform!(b"", identity, b" that ", word),
        30 => make_transform!(b" ", ferment_first, b"", word),
        31 => make_transform!(b"", identity, b". ", word),
        32 => make_transform!(b".", identity, b"", word),
        33 => make_transform!(b" ", identity, b", ", word),
        34 => make_transform!(b"", omit_first_4, b"", word),
        35 => make_transform!(b"", identity, b" with ", word),
        36 => make_transform!(b"", identity, b"'", word),
        37 => make_transform!(b"", identity, b" from ", word),
        38 => make_transform!(b"", identity, b" by ", word),
        39 => make_transform!(b"", omit_first_5, b"", word),
        40 => make_transform!(b"", omit_first_6, b"", word),
        41 => make_transform!(b" the ", identity, b"", word),
        42 => make_transform!(b"", omit_last_4, b"", word),
        43 => make_transform!(b"", identity, b". The ", word),
        44 => make_transform!(b"", ferment_all, b"", word),
        45 => make_transform!(b"", identity, b" on ", word),
        46 => make_transform!(b"", identity, b" as ", word),
        47 => make_transform!(b"", identity, b" is ", word),
        48 => make_transform!(b"", omit_last_7, b"", word),
        49 => make_transform!(b"", omit_last_1, b"ing ", word),
        50 => make_transform!(b"", identity, b"\n\t", word),
        51 => make_transform!(b"", identity, b":", word),
        52 => make_transform!(b" ", identity, b". ", word),
        53 => make_transform!(b"", identity, b"ed ", word),
        54 => make_transform!(b"", omit_first_9, b"", word),
        55 => make_transform!(b"", omit_first_7, b"", word),
        56 => make_transform!(b"", omit_last_6, b"", word),
        57 => make_transform!(b"", identity, b"(", word),
        58 => make_transform!(b"", ferment_first, b", ", word),
        59 => make_transform!(b"", omit_last_8, b"", word),
        60 => make_transform!(b"", identity, b" at ", word),
        61 => make_transform!(b"", identity, b"ly ", word),
        62 => make_transform!(b" the ", identity, b" of ", word),
        63 => make_transform!(b"", omit_last_5, b"", word),
        64 => make_transform!(b"", omit_last_9, b"", word),
        65 => make_transform!(b" ", ferment_first, b", ", word),
        66 => make_transform!(b"", ferment_first, b"\"", word),
        67 => make_transform!(b".", identity, b"(", word),
        68 => make_transform!(b"", ferment_all, b" ", word),
        69 => make_transform!(b"", ferment_first, b"\">", word), // nice
        70 => make_transform!(b"", identity, b"=\"", word),
        71 => make_transform!(b" ", identity, b".", word),
        72 => make_transform!(b".com/", identity, b"", word),
        73 => make_transform!(b" the ", identity, b" of the ", word),
        74 => make_transform!(b"", ferment_first, b"'", word),
        75 => make_transform!(b"", identity, b". This ", word),
        76 => make_transform!(b"", identity, b",", word),
        77 => make_transform!(b".", identity, b" ", word),
        78 => make_transform!(b"", ferment_first, b"(", word),
        79 => make_transform!(b"", ferment_first, b".", word),
        80 => make_transform!(b"", identity, b" not ", word),
        81 => make_transform!(b" ", identity, b"=\"", word),
        82 => make_transform!(b"", identity, b"er ", word),
        83 => make_transform!(b" ", ferment_all, b" ", word),
        84 => make_transform!(b"", identity, b"al ", word),
        85 => make_transform!(b" ", ferment_all, b"", word),
        86 => make_transform!(b"", identity, b"='", word),
        87 => make_transform!(b"", ferment_all, b"\"", word),
        88 => make_transform!(b"", ferment_first, b". ", word),
        89 => make_transform!(b" ", identity, b"(", word),
        90 => make_transform!(b"", identity, b"ful ", word),
        91 => make_transform!(b" ", ferment_first, b". ", word),
        92 => make_transform!(b"", identity, b"ive ", word),
        93 => make_transform!(b"", identity, b"less ", word),
        94 => make_transform!(b"", ferment_all, b"'", word),
        95 => make_transform!(b"", identity, b"est ", word),
        96 => make_transform!(b" ", ferment_first, b".", word),
        97 => make_transform!(b"", ferment_all, b"\">", word),
        98 => make_transform!(b" ", identity, b"='", word),
        99 => make_transform!(b"", ferment_first, b",", word),
        100 => make_transform!(b"", identity, b"ize ", word),
        101 => make_transform!(b"", ferment_all, b".", word),
        102 => make_transform!(b"\xc2\xa0", identity, b"", word),
        103 => make_transform!(b" ", identity, b",", word),
        104 => make_transform!(b"", ferment_first, b"=\"", word),
        105 => make_transform!(b"", ferment_all, b"=\"", word),
        106 => make_transform!(b"", identity, b"ous ", word),
        107 => make_transform!(b"", ferment_all, b", ", word),
        108 => make_transform!(b"", ferment_first, b"='", word),
        109 => make_transform!(b" ", ferment_first, b",", word),
        110 => make_transform!(b" ", ferment_all, b"=\"", word),
        111 => make_transform!(b" ", ferment_all, b", ", word),
        112 => make_transform!(b"", ferment_all, b",", word),
        113 => make_transform!(b"", ferment_all, b"(", word),
        114 => make_transform!(b"", ferment_all, b". ", word),
        115 => make_transform!(b" ", ferment_all, b".", word),
        116 => make_transform!(b" ", ferment_all, b"='", word),
        117 => make_transform!(b" ", ferment_all, b". ", word),
        118 => make_transform!(b" ", ferment_first, b"=\"", word),
        119 => make_transform!(b" ", ferment_all, b"='", word),
        120 => make_transform!(b" ", ferment_first, b"='", word),
        _ => return Err(BrotliError::InvalidTransformID),
    };
    Ok(transformed)
}
