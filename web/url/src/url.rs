//! Implements <https://url.spec.whatwg.org>

use crate::{
    host::Host,
    parser::{URLParser, URLParserState},
    util,
};

pub type Port = u16;

pub(crate) fn is_special_scheme(scheme: &str) -> bool {
    matches!(scheme, "ftp" | "file" | "http" | "https" | "ws" | "wss")
}

pub(crate) fn default_port_for_scheme(scheme: &str) -> Option<Port> {
    match scheme {
        "ftp" => Some(21),
        "http" | "ws" => Some(80),
        "https" | "wss" => Some(443),
        _ => None,
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Scheme<'a>(&'a str);

impl<'a> Scheme<'a> {
    /// <https://url.spec.whatwg.org/#special-scheme>
    #[inline]
    #[must_use]
    pub fn is_special(&self) -> bool {
        is_special_scheme(self.0)
    }

    /// <https://url.spec.whatwg.org/#default-port>
    #[inline]
    #[must_use]
    pub fn default_port(&self) -> Option<Port> {
        default_port_for_scheme(self.0)
    }
}

impl<'a> AsRef<str> for Scheme<'a> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl<'a> PartialEq<&str> for Scheme<'a> {
    fn eq(&self, other: &&str) -> bool {
        self.as_ref().eq(*other)
    }
}

/// A **U**niform **R**esource **L**ocator
///
/// [Specification](https://url.spec.whatwg.org/#concept-url)
#[derive(Default, Clone, Debug)]
pub struct URL {
    /// A [URL]’s scheme is an ASCII string that identifies the type of URL
    /// and can be used to dispatch a URL for further processing after parsing.
    /// It is initially the empty string.
    pub(crate) scheme: String,

    /// A [URL]’s username is an ASCII string identifying a username.
    /// It is initially the empty string.
    pub(crate) username: String,

    /// A [URL]’s password is an ASCII string identifying a password.
    /// It is initially the empty string.
    pub(crate) password: String,

    /// A [URL]’s host is [None](Option::None) or a [host](Host).
    /// It is initially [None](Option::None).
    pub(crate) host: Option<Host>,

    /// A [URL]’s port is either [None](Option::None) or a 16-bit unsigned integer that identifies a networking port.
    /// It is initially [None](Option::None).
    pub(crate) port: Option<Port>,

    pub(crate) path: Vec<String>,

    /// A [URL]’s query is either [None](Option::None) or an ASCII string.
    /// It is initially [None](Option::None).
    pub(crate) query: Option<String>,

    /// A URL’s fragment is either [None](Option::None) or an ASCII string
    /// that can be used for further processing on the resource the URL’s other components identify.
    /// It is initially [None](Option::None).
    pub(crate) fragment: Option<String>,
}

#[derive(Clone, Copy, Debug)]
pub struct URLParseError;

/// Whether or not the fragment of an [URL] should be excluded during serialization.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ExcludeFragment {
    Yes,
    #[default]
    No,
}

impl URL {
    pub fn scheme(&self) -> Scheme<'_> {
        Scheme(&self.scheme)
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn host(&self) -> Option<&Host> {
        self.host.as_ref()
    }

    pub fn port(&self) -> Option<Port> {
        self.port
    }

    pub fn path(&self) -> &[String] {
        &self.path
    }

    pub fn query(&self) -> Option<&str> {
        self.query.as_deref()
    }

    pub fn fragment(&self) -> Option<&str> {
        self.fragment.as_deref()
    }

    /// [Specification](https://url.spec.whatwg.org/#concept-basic-url-parser)
    pub fn parse_with_base(
        mut input: &str,
        base: Option<URL>,
        given_url: Option<URL>,
        state_override: Option<URLParserState>,
    ) -> Result<Self, URLParseError> {
        let url = match given_url {
            Some(url) => url,
            None => {
                // If url is not given:
                // Set url to a new URL.
                let url = Self::default();

                // If input contains any leading or trailing C0 control or space, validation error.

                // Remove any leading and trailing C0 control or space from input.
                input = input
                    .trim_start_matches(util::is_c0_or_space)
                    .trim_end_matches(util::is_c0_or_space);
                url
            },
        };

        // If input contains any ASCII tab or newline, validation error.

        // Remove all ASCII tab or newline from input.
        // TODO https://doc.rust-lang.org/std/string/struct.String.html#method.remove_matches
        // would be nice here, but it's not stabilized yet
        let filtered_input: String = input
            .chars()
            .filter(|c| !util::is_ascii_tab_or_newline(*c))
            .collect();

        // Let state be state override if given, or scheme start state otherwise.
        let state = state_override.unwrap_or(URLParserState::SchemeStart);

        // Set encoding to the result of getting an output encoding from encoding.

        // Let buffer be the empty string.
        let buffer = String::new();

        // Let atSignSeen, insideBrackets, and passwordTokenSeen be false.
        let at_sign_seen = false;
        let inside_brackets = false;
        let password_token_seen = false;

        let ptr = 0;

        let state_machine = URLParser {
            url: url,
            state: state,
            buffer: buffer,
            ptr: ptr,
            base: base,
            input: &filtered_input,
            state_override: state_override,
            at_sign_seen: at_sign_seen,
            inside_brackets: inside_brackets,
            password_token_seen: password_token_seen,
        };

        let parsed_url = state_machine
            .run_to_completion()
            .map_err(|_| URLParseError)?
            .url;
        Ok(parsed_url)
    }

    /// [Specification](https://url.spec.whatwg.org/#concept-basic-url-parser)
    pub fn parse(input: &str) -> Result<Self, URLParseError> {
        Self::parse_with_base(input, None, None, None)
    }

    /// [Specification](https://url.spec.whatwg.org/#include-credentials)
    ///
    /// A [URL] includes credentials if its  [username](URL::username) or [password](URL::password) is not the empty string.
    #[must_use]
    pub fn includes_credentials(&self) -> bool {
        !self.username.is_empty() || !self.password.is_empty()
    }

    /// [Specification](https://url.spec.whatwg.org/#is-special)
    ///
    /// A [URL] is special if its scheme is a special scheme. A [URL] is not special if its scheme is not a special scheme.
    #[must_use]
    pub fn is_special(&self) -> bool {
        self.scheme().is_special()
    }

    /// [Specification](https://url.spec.whatwg.org/#url-opaque-path)
    ///
    /// A [URL] has an opaque path if it only consists of a single string
    #[must_use]
    pub fn has_opaque_path(&self) -> bool {
        self.path.len() == 1
    }

    /// [Specification](https://url.spec.whatwg.org/#shorten-a-urls-path)
    pub(crate) fn shorten_path(&mut self) {
        // Assert: url does not have an opaque path.
        assert!(!self.has_opaque_path());

        // Let path be url’s path.
        let path = &mut self.path;

        // If url’s scheme is "file", path’s size is 1, and path[0] is a normalized Windows drive letter,
        if self.scheme == "file"
            && path.len() == 1
            && util::is_normalized_windows_drive_letter(&path[0])
        {
            // then return.
            return;
        }

        // Remove path’s last item, if any.
        path.pop();
    }

    /// <https://url.spec.whatwg.org/#url-serializing>
    pub fn serialize(&self, exclude_fragment: ExcludeFragment) -> String {
        // 1. Let output be url’s scheme and U+003A (:) concatenated.
        let mut output = format!("{}:", self.scheme);

        // 2. If url’s host is non-null:
        if let Some(host) = &self.host {
            // 1. Append "//" to output.
            output.push_str("//");

            // 2. If url includes credentials, then:
            if self.includes_credentials() {
                // 1. Append url’s username to output.
                output.push_str(&self.username);

                // 2. If url’s password is not the empty string, then append U+003A (:), followed by url’s password, to output.
                if !self.password.is_empty() {
                    output.push_str(&format!(":{}", self.password));
                }

                // 3. Append U+0040 (@) to output.
                output.push('@');
            }

            // 3. Append url’s host, serialized, to output.
            output.push_str(&host.to_string());

            // 4. If url’s port is non-null, append U+003A (:) followed by url’s port, serialized, to output.
            if let Some(port) = self.port {
                output.push_str(&format!(":{port}"));
            }
        }

        // 3. If url’s host is null, url does not have an opaque path, url’s path’s size is greater than 1, and url’s path[0] is the empty string, then append U+002F (/) followed by U+002E (.) to output.
        if self.host.is_none()
            && !self.has_opaque_path()
            && self.path.len() > 1
            && self.path[0].is_empty()
        {
            output.push_str("/.");
        }

        // 4. Append the result of URL path serializing url to output.
        output.push_str(&self.path_serialize());

        // 5. If url’s query is non-null, append U+003F (?), followed by url’s query, to output.
        if let Some(query) = &self.query {
            output.push_str(&format!("?{query}"));
        }

        // 6. If exclude fragment is false and url’s fragment is non-null, then append U+0023 (#), followed by url’s fragment, to output.
        if exclude_fragment == ExcludeFragment::No {
            if let Some(fragment) = &self.fragment {
                output.push_str(&format!("?{fragment}"));
            }
        }

        // 7. Return output.
        output
    }

    /// <https://url.spec.whatwg.org/#url-path-serializer>
    fn path_serialize(&self) -> String {
        // If url has an opaque path, then return url’s path.
        if self.has_opaque_path() {
            return self.path[0].clone();
        }

        // Let output be the empty string.
        // For each segment of url’s path: append U+002F (/) followed by segment to output.
        // Return output.
        if !self.path.is_empty() {
            format!("/{}", self.path.join("/"))
        } else {
            String::new()
        }
    }
}

impl TryFrom<&str> for URL {
    type Error = URLParseError;

    fn try_from(from: &str) -> Result<Self, Self::Error> {
        Self::parse(from)
    }
}

impl ToString for URL {
    fn to_string(&self) -> String {
        self.serialize(ExcludeFragment::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_url() {
        let url = URL::parse("https://google.com").unwrap();

        assert_eq!(url.scheme, "https");
        assert_eq!(url.username, "");
        assert_eq!(url.password, "");
        assert_eq!(url.host, Some(Host::OpaqueHost("google.com".to_string())));
        assert_eq!(url.path, vec![""]);
        assert_eq!(url.query, None);
        assert_eq!(url.fragment, None);
    }

    #[test]
    fn test_with_query() {
        let url = URL::parse("https://google.com?a=b").unwrap();

        assert_eq!(url.scheme, "https");
        assert_eq!(url.username, "");
        assert_eq!(url.password, "");
        assert_eq!(url.host, Some(Host::OpaqueHost("google.com".to_string())));
        assert_eq!(url.path, vec![""]);
        assert_eq!(url.query.as_deref(), Some("a=b"));
        assert_eq!(url.fragment, None);
    }

    #[test]
    fn test_with_fragment() {
        let url = URL::parse("https://google.com#foo").unwrap();

        assert_eq!(url.scheme, "https");
        assert_eq!(url.username, "");
        assert_eq!(url.password, "");
        assert_eq!(url.host, Some(Host::OpaqueHost("google.com".to_string())));
        assert_eq!(url.path, vec![""]);
        assert_eq!(url.query, None);
        assert_eq!(url.fragment.as_deref(), Some("foo"));
    }

    #[test]
    fn test_with_credentials() {
        let url = URL::parse("https://user:password@google.com").unwrap();

        assert_eq!(url.scheme, "https");
        assert_eq!(url.username, "user");
        assert_eq!(url.password, "password");
        assert_eq!(url.host, Some(Host::OpaqueHost("google.com".to_string())));
        assert_eq!(url.path, vec![""]);
        assert_eq!(url.query, None);
        assert_eq!(url.fragment, None);
    }
}
