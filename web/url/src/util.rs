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
