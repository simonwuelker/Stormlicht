// https://url.spec.whatwg.org/#start-with-a-windows-drive-letter
/// A string starts with a Windows drive letter if all of the following are true:
/// * its length is greater than or equal to 2
/// * its first two code points are a Windows drive letter
/// * its length is 2 or its third code point is U+002F (/), U+005C (\), U+003F (?), or U+0023 (#).
pub(crate) fn starts_with_windows_drive_letter(input: &str) -> bool {
    // the length check is done inside is_windows_drive_letter

    if !is_windows_drive_letter(&input) {
        return false;
    }

    if input.len() != 2 {
        let third_character = input.chars().nth(3).unwrap();

        if third_character != '/'
            && third_character != '\\'
            && third_character != '?'
            && third_character != '#'
        {
            return false;
        }
    }

    true
}

// https://url.spec.whatwg.org/#windows-drive-letter
pub(crate) fn is_windows_drive_letter(letter: &str) -> bool {
    if letter.len() < 2 {
        return false;
    }

    let first_character = letter.chars().nth(0).unwrap();
    let second_character = letter.chars().nth(1).unwrap();
    first_character.is_ascii_alphabetic() && (second_character == ':' || second_character == '|')
}

// https://url.spec.whatwg.org/#normalized-windows-drive-letter
pub(crate) fn is_normalized_windows_drive_letter(letter: &str) -> bool {
    if letter.len() < 2 {
        return false;
    }

    let first_character = letter.chars().nth(0).unwrap();
    let second_character = letter.chars().nth(1).unwrap();
    first_character.is_ascii_alphabetic() && second_character == ':'
}

// https://infra.spec.whatwg.org/#c0-control
pub(crate) fn is_c0_or_space(c: char) -> bool {
    match c {
        '\u{0000}'..='\u{001F}' | '\u{0020}' => true,
        _ => false,
    }
}

pub(crate) fn is_ascii_tab_or_newline(c: char) -> bool {
    match c {
        '\u{0009}' | '\u{000A}' | '\u{000D}' => true,
        _ => false,
    }
}
// https://url.spec.whatwg.org/#single-dot-path-segment
pub(crate) fn is_single_dot_path_segment(input: &str) -> bool {
    input == "." || input.eq_ignore_ascii_case("%2e")
}

// https://url.spec.whatwg.org/#double-dot-path-segment
pub(crate) fn is_double_dot_path_segment(input: &str) -> bool {
    input == ".."
        || input.eq_ignore_ascii_case(".%2e")
        || input.eq_ignore_ascii_case("%2e.")
        || input.eq_ignore_ascii_case("%2e%2e")
}

// https://url.spec.whatwg.org/#url-code-points
pub(crate) fn is_url_codepoint(c: char) -> bool {
    c.is_alphanumeric()
        | match c {
            '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | '-' | '.' | '/' | ':' | ';'
            | '=' | '?' | '@' | '_' | '~' => true,
            // range excludes surrogates and noncharacters
            '\u{00A0}'..='\u{D7FF}' | '\u{E000}'..='\u{10FFFD}' => {
                // check for noncharacters
                // return true if c is not a noncharacter
                match c {
                    '\u{FDD0}'..='\u{FDEF}'
                    | '\u{FFFE}'
                    | '\u{FFFF}'
                    | '\u{1FFFE}'
                    | '\u{1FFFF}'
                    | '\u{2FFFE}'
                    | '\u{2FFFF}'
                    | '\u{3FFFE}'
                    | '\u{3FFFF}'
                    | '\u{4FFFE}'
                    | '\u{4FFFF}'
                    | '\u{5FFFE}'
                    | '\u{5FFFF}'
                    | '\u{6FFFE}'
                    | '\u{6FFFF}'
                    | '\u{7FFFE}'
                    | '\u{7FFFF}'
                    | '\u{8FFFE}'
                    | '\u{8FFFF}'
                    | '\u{9FFFE}'
                    | '\u{9FFFF}'
                    | '\u{AFFFE}'
                    | '\u{AFFFF}'
                    | '\u{BFFFE}'
                    | '\u{BFFFF}'
                    | '\u{CFFFE}'
                    | '\u{CFFFF}'
                    | '\u{DFFFE}'
                    | '\u{DFFFF}'
                    | '\u{EFFFE}'
                    | '\u{EFFFF}'
                    | '\u{FFFFE}'
                    | '\u{FFFFF}'
                    | '\u{10FFFE}'
                    | '\u{10FFFF}' => false,
                    _ => true,
                }
            },
            _ => false,
        }
}
