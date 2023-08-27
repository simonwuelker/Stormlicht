/// <https://url.spec.whatwg.org/#start-with-a-windows-drive-letter>
/// A string starts with a Windows drive letter if all of the following are true:
/// * its length is greater than or equal to 2
/// * its first two code points are a Windows drive letter
/// * its length is 2 or its third code point is U+002F (/), U+005C (\), U+003F (?), or U+0023 (#).
#[must_use]
pub fn starts_with_windows_drive_letter(input: &str) -> bool {
    if !is_windows_drive_letter(input) {
        return false;
    }

    if input
        .chars()
        .nth(2)
        .is_some_and(|c| !matches!(c, '/' | '\\' | '?' | '#'))
    {
        return false;
    }

    true
}

/// <https://url.spec.whatwg.org/#windows-drive-letter>
#[must_use]
pub fn is_windows_drive_letter(letter: &str) -> bool {
    let mut chars = letter.chars();

    if !chars.next().as_ref().is_some_and(char::is_ascii_alphabetic) {
        return false;
    }

    if !matches!(chars.next(), Some(':' | '|')) {
        return false;
    }

    true
}

/// <https://url.spec.whatwg.org/#normalized-windows-drive-letter>
pub fn is_normalized_windows_drive_letter(letter: &str) -> bool {
    let mut chars = letter.chars();

    if !chars.next().as_ref().is_some_and(char::is_ascii_alphabetic) {
        return false;
    }

    if !matches!(chars.next(), Some(':')) {
        return false;
    }

    true
}

/// <https://infra.spec.whatwg.org/#c0-control>
#[inline]
#[must_use]
pub fn is_c0_or_space(c: char) -> bool {
    matches!(c, '\u{0000}'..='\u{001F}' | '\u{0020}')
}

#[inline]
#[must_use]
pub fn is_ascii_tab_or_newline(c: char) -> bool {
    matches!(c, '\u{0009}' | '\u{000A}' | '\u{000D}')
}

/// <https://url.spec.whatwg.org/#single-dot-path-segment>
#[inline]
#[must_use]
pub fn is_single_dot_path_segment(input: &str) -> bool {
    input == "." || input.eq_ignore_ascii_case("%2e")
}

/// <https://url.spec.whatwg.org/#double-dot-path-segment>
#[inline]
#[must_use]
pub fn is_double_dot_path_segment(input: &str) -> bool {
    input == ".."
        || input.eq_ignore_ascii_case(".%2e")
        || input.eq_ignore_ascii_case("%2e.")
        || input.eq_ignore_ascii_case("%2e%2e")
}

/// <https://url.spec.whatwg.org/#url-code-points>
#[must_use]
pub fn is_url_codepoint(c: char) -> bool {
    c.is_alphanumeric()
        | match c {
            '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | '-' | '.' | '/' | ':' | ';'
            | '=' | '?' | '@' | '_' | '~' => true,
            // range excludes surrogates and noncharacters
            '\u{00A0}'..='\u{D7FF}' | '\u{E000}'..='\u{10FFFD}' => {
                // check for noncharacters
                // return true if c is not a noncharacter
                !matches!(
                    c,
                    '\u{FDD0}'
                        ..='\u{FDEF}'
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
                            | '\u{10FFFF}'
                )
            },
            _ => false,
        }
}
