use std::{collections::HashMap, fmt, str::FromStr};

/// <https://mimesniff.spec.whatwg.org/#http-token-code-point>
#[inline]
fn is_http_token_code_point(c: char) -> bool {
    matches!(c, '!' | '#' | '$' | '%' | '&' | '\'' | '*' | '+' | '-' | '.' | '^' | '_' | '`' | '|' | '~' | 'a'..='z' | 'A'..='Z' | '0'..='9')
}

/// <https://mimesniff.spec.whatwg.org/#mime-type>
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MIMEType {
    /// <https://mimesniff.spec.whatwg.org/#type>
    pub mime_type: String,

    /// <https://mimesniff.spec.whatwg.org/#subtype>
    pub mime_subtype: String,

    // TODO: ordering
    /// <https://mimesniff.spec.whatwg.org/#parameters>
    pub parameters: HashMap<String, String>,
}

impl MIMEType {
    pub fn new(mime_type: &str, mime_subtype: &str) -> Self {
        Self {
            mime_type: mime_type.into(),
            mime_subtype: mime_subtype.into(),
            parameters: HashMap::default(),
        }
    }

    pub fn essence(&self) -> String {
        format!("{}/{}", self.mime_type, self.mime_subtype)
    }

    /// <https://mimesniff.spec.whatwg.org/#image-mime-type>
    pub fn is_image(&self) -> bool {
        self.mime_type == "image"
    }

    /// <https://mimesniff.spec.whatwg.org/#audio-or-video-mime-type>
    pub fn is_audio_or_video(&self) -> bool {
        self.mime_type == "audio"
            || self.mime_type == "video"
            || self.essence() == "application/ogg"
    }

    /// <https://mimesniff.spec.whatwg.org/#font-mime-type>
    pub fn is_font(&self) -> bool {
        self.mime_type == "font"
            || matches!(
                self.essence().as_str(),
                "application/font-cff"
                    | "application/font-off"
                    | "application/font-sfnt"
                    | "application/font-ttf"
                    | "application/font-woff"
                    | "application/vnd.ms-fontobject"
                    | "application/vnd.ms-opentype"
            )
    }

    /// <https://mimesniff.spec.whatwg.org/#zip-based-mime-type>
    pub fn is_zip_based(&self) -> bool {
        self.mime_subtype.ends_with("+zip") || self.essence() == "application/zip"
    }

    /// <https://mimesniff.spec.whatwg.org/#archive-mime-type>
    pub fn is_archive(&self) -> bool {
        matches!(
            self.essence().as_str(),
            "application/x-rar-compressed" | "application/zip" | "application/x-gzip"
        )
    }

    /// <https://mimesniff.spec.whatwg.org/#xml-mime-type>
    pub fn is_xml(&self) -> bool {
        self.mime_subtype.ends_with("+xml")
            || matches!(self.essence().as_str(), "text/xml" | "application/xml")
    }

    /// <https://mimesniff.spec.whatwg.org/#html-mime-type>
    pub fn is_html(&self) -> bool {
        self.essence() == "text/html"
    }

    /// <https://mimesniff.spec.whatwg.org/#scriptable-mime-type>
    pub fn is_scriptable(&self) -> bool {
        self.is_xml() || self.is_html() || self.essence() == "application/pdf"
    }

    /// <https://mimesniff.spec.whatwg.org/#javascript-mime-type>
    pub fn is_javascript(&self) -> bool {
        matches!(
            self.essence().as_str(),
            "application/ecmascript"
                | "application/javascript"
                | "application/x-ecmascript"
                | "application/x-javascript"
                | "text/ecmascript"
                | "text/javascript"
                | "text/javascript1.0"
                | "text/javascript1.1"
                | "text/javascript1.2"
                | "text/javascript1.3"
                | "text/javascript1.4"
                | "text/javascript1.5"
                | "text/jscript"
                | "text/x-ecmascript"
                | "text/x-javascript"
        )
    }

    /// <https://mimesniff.spec.whatwg.org/#json-mime-type>
    pub fn is_json(&self) -> bool {
        self.mime_subtype.ends_with("+json")
            || matches!(self.essence().as_str(), "application/json" | "text/json")
    }
}

impl fmt::Display for MIMEType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.essence())
    }
}

/// Errors that can occur while parsing a [MIMEType]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MIMEParseError {
    NoSubType,
    TypeContainsNonHTTPCodePoint,
    SubTypeContainsNonHTTPCodePoint,
    EmptyType,
    EmptySubType,
}

impl FromStr for MIMEType {
    type Err = MIMEParseError;

    // <https://mimesniff.spec.whatwg.org/#parse-a-mime-type>
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // 1. Remove any leading and trailing HTTP whitespace from input.
        let input = input.trim();

        // 2. Let position be a position variable for input, initially pointing at the start of input.
        // 3. Let type be the result of collecting a sequence of code points that are not U+002F (/) from input, given position.
        let (mime_type, remaining_input) =
            input.split_once('/').ok_or(MIMEParseError::NoSubType)?;

        // 4. If type is the empty string or does not solely contain HTTP token code points, then return failure.
        if mime_type.is_empty() {
            return Err(MIMEParseError::EmptyType);
        }

        if !mime_type.chars().all(is_http_token_code_point) {
            return Err(MIMEParseError::TypeContainsNonHTTPCodePoint);
        }

        // 5. If position is past the end of input, then return failure.
        // NOTE: split_once did that for us

        // 6. Advance position by 1. (This skips past U+002F (/).)
        // 7. Let subtype be the result of collecting a sequence of code points that are not U+003B (;) from input, given position.
        let (mime_subtype, mut remaining_input) = remaining_input
            .split_once(';')
            .unwrap_or((remaining_input, ""));

        // 8. Remove any trailing HTTP whitespace from subtype.
        let mime_subtype = mime_subtype.trim_end();

        // 9. If subtype is the empty string or does not solely contain HTTP token code points, then return failure.
        if mime_subtype.is_empty() {
            return Err(MIMEParseError::EmptySubType);
        }

        if !mime_subtype.chars().all(is_http_token_code_point) {
            return Err(MIMEParseError::SubTypeContainsNonHTTPCodePoint);
        }

        // 10. Let mimeType be a new MIME type record whose type is type, in ASCII lowercase, and subtype is subtype, in ASCII lowercase.
        let mut mime = MIMEType::new(
            &mime_type.to_ascii_lowercase(),
            &mime_subtype.to_ascii_lowercase(),
        );

        // 11. While position is not past the end of input:
        while !remaining_input.is_empty() {
            // 1. Advance position by 1. (This skips past U+003B (;).)
            // NOTE: rust takes care of that for us

            // 2. Collect a sequence of code points that are HTTP whitespace from input given position.
            remaining_input = remaining_input.trim_start();

            // 3. Let parameterName be the result of collecting a sequence of code points that are not U+003B (;) or U+003D (=) from input, given position.
            let parameter_name_end = remaining_input.find(|c| matches!(c, '=' | ';'));

            // NOTE: reordered the steps 4 and 5 because they do not depend on each other
            // and 4 does work that might later become unnecessary

            // 5. If position is not past the end of input, then:
            let parameter_name = match parameter_name_end {
                Some(index) => {
                    //      1. If the code point at position within input is U+003B (;), then continue.
                    if remaining_input.chars().nth(index) == Some(';') {
                        continue;
                    }

                    // 4. Set parameterName to parameterName, in ASCII lowercase.
                    let parameter_name = remaining_input[..index].to_ascii_lowercase();

                    //      2. Advance position by 1. (This skips past U+003D (=).)
                    remaining_input = &remaining_input[1..];

                    parameter_name
                },
                None => {
                    // 6. If position is past the end of input, then break.
                    break;
                },
            };

            // 7. Let parameterValue be null.
            // 8. If the code point at position within input is U+0022 ("), then:
            let parameter_value = if remaining_input.starts_with('"') {
                // 1. Set parameterValue to the result of collecting an HTTP quoted string from input, given position and the extract-value flag.
                // NOTE: this is not entirely correct, wait for <https://fetch.spec.whatwg.org> to be implemented
                let parts = remaining_input[1..]
                    .split_once('"')
                    .unwrap_or((&remaining_input[1..], ""));
                let parameter_value = parts.0;
                remaining_input = parts.1;

                // 2. Collect a sequence of code points that are not U+003B (;) from input, given position.
                remaining_input = remaining_input.split_once(';').map(|a| a.1).unwrap_or("");

                parameter_value
            }
            // 9. Otherwise:
            else {
                // 1. Set parameterValue to the result of collecting a sequence of code points that are not U+003B (;) from input, given position.
                let parts = remaining_input
                    .split_once(';')
                    .unwrap_or((remaining_input, ""));
                let parameter_value = parts.0;
                remaining_input = parts.1;

                // 2. Remove any trailing HTTP whitespace from parameterValue.
                let parameter_value = parameter_value.trim_end();

                // 3. If parameterValue is the empty string, then continue.
                if parameter_value.is_empty() {
                    continue;
                }

                parameter_value
            };

            // 10. If all of the following are true
            // * parameterName is not the empty string
            // * parameterName solely contains HTTP token code points
            // * parameterValue solely contains HTTP quoted-string token code points
            // * mimeType’s parameters[parameterName] does not exist
            if !parameter_name.is_empty()
                && parameter_name.chars().all(is_http_token_code_point)
                && parameter_value.chars().all(is_http_token_code_point)
            {
                // then set mimeType’s parameters[parameterName] to parameterValue.
                mime.parameters
                    .entry(parameter_name)
                    .or_insert(parameter_value.to_string());
            }
        }

        // 12. Return mimeType.
        Ok(mime)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::{MIMEParseError, MIMEType};

    #[test]
    fn invalid_mime_type() {
        // MIME type without a subtype
        assert_eq!(MIMEType::from_str("foo"), Err(MIMEParseError::NoSubType));

        // Empty type
        assert_eq!(MIMEType::from_str("/foo"), Err(MIMEParseError::EmptyType));

        // Empty subtype
        assert_eq!(
            MIMEType::from_str("foo/"),
            Err(MIMEParseError::EmptySubType)
        );

        // Type containing non-http codepoints
        assert_eq!(
            MIMEType::from_str("foo@bar/foo"),
            Err(MIMEParseError::TypeContainsNonHTTPCodePoint)
        );

        // Subtype containing non-http codepoints
        assert_eq!(
            MIMEType::from_str("foo/foo@bar"),
            Err(MIMEParseError::SubTypeContainsNonHTTPCodePoint)
        );
    }

    #[test]
    fn valid_mime_type() {
        // Simple MIME type
        assert_eq!(
            MIMEType::from_str("foo/bar"),
            Ok(MIMEType::new("foo", "bar"))
        );

        // Leading/trailing whitespace
        assert_eq!(
            MIMEType::from_str("  foo/bar  "),
            Ok(MIMEType::new("foo", "bar"))
        );
    }
}
