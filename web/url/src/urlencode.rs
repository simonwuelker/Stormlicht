use crate::parser::is_c0_control;

/// <https://url.spec.whatwg.org/#c0-control-percent-encode-set>
pub(crate) fn is_c0_percent_encode_set(c: char) -> bool {
    is_c0_control(c) | matches!(c, '\u{007F}'..)
}

/// <https://url.spec.whatwg.org/#fragment-percent-encode-set>
pub(crate) fn is_fragment_percent_encode_set(c: char) -> bool {
    is_c0_percent_encode_set(c) | matches!(c, ' ' | '"' | '#' | '<' | '>')
}

/// <https://url.spec.whatwg.org/#query-percent-encode-set>
pub(crate) fn is_query_percent_encode_set(c: char) -> bool {
    is_c0_percent_encode_set(c) | matches!(c, ' ' | '"' | '#' | '<' | '>')
}

/// <https://url.spec.whatwg.org/#special-query-percent-encode-set>
pub(crate) fn is_special_query_percent_encode_set(c: char) -> bool {
    is_query_percent_encode_set(c) || c == '\''
}

/// <https://url.spec.whatwg.org/#path-percent-encode-set>
pub(crate) fn is_path_percent_encode_set(c: char) -> bool {
    is_query_percent_encode_set(c) | matches!(c, '?' | '`' | '{' | '}')
}

/// <https://url.spec.whatwg.org/#userinfo-percent-encode-set>
pub(crate) fn is_userinfo_percent_encode_set(c: char) -> bool {
    is_path_percent_encode_set(c) | matches!(c, '/' | ':' | ';' | '=' | '@' | '['..='^' | '|')
}
/// <https://url.spec.whatwg.org/#string-percent-encode-after-encoding>
pub fn percent_encode<F: Fn(char) -> bool>(input: &str, in_encode_set: F) -> String {
    let mut result = String::new();
    for c in input.chars() {
        result.push_str(percent_encode_char(c, &in_encode_set).as_str());
    }
    result
}

pub fn percent_encode_char<F: Fn(char) -> bool>(c: char, in_encode_set: F) -> String {
    let mut out = String::new();
    let mut buffer = [0; 4];
    let encoded = c.encode_utf8(&mut buffer);
    for b in encoded.chars() {
        if in_encode_set(b) {
            // percent-encode byte and append the result to output.
            out.push('%');
            out.push_str(b.to_string().as_str());
        } else {
            out.push(b);
        }
    }
    out
}
