//! Implements <https://url.spec.whatwg.org>

use crate::{
    host::Host,
    urlparser::{URLParser, URLParserState},
    util,
};

pub(crate) fn scheme_is_special(scheme: &str) -> bool {
    match scheme {
        "ftp" | "file" | "http" | "https" | "ws" | "wss" => true,
        _ => false,
    }
}

pub(crate) fn scheme_default_port(scheme: &str) -> Option<Port> {
    match scheme {
        "ftp" => Some(21),
        "http" | "ws" => Some(80),
        "https" | "wss" => Some(443),
        _ => None,
    }
}

pub type Port = u16;

pub type Path = Vec<String>;

/// A **U**niform **R**esource **L**ocator
///
/// [Specification](https://url.spec.whatwg.org/#concept-url)
#[derive(Default, Debug)]
pub struct URL {
    /// A [URL]’s scheme is an ASCII string that identifies the type of URL
    /// and can be used to dispatch a URL for further processing after parsing.
    /// It is initially the empty string.
    pub scheme: String,

    /// A [URL]’s username is an ASCII string identifying a username.
    /// It is initially the empty string.
    pub username: String,

    /// A [URL]’s password is an ASCII string identifying a password.
    /// It is initially the empty string.
    pub password: String,

    /// A [URL]’s host is [None](Option::None) or a [host](Host).
    /// It is initially [None](Option::None).
    pub host: Option<Host>,

    /// A [URL]’s port is either [None](Option::None) or a 16-bit unsigned integer that identifies a networking port.
    /// It is initially [None](Option::None).
    pub port: Option<Port>,

    pub path: Path,

    /// A [URL]’s query is either [None](Option::None) or an ASCII string.
    /// It is initially [None](Option::None).
    pub query: Option<String>,

    /// A URL’s fragment is either [None](Option::None) or an ASCII string
    /// that can be used for further processing on the resource the URL’s other components identify.
    /// It is initially [None](Option::None).
    pub fragment: Option<String>,
}

impl URL {
    // https://url.spec.whatwg.org/#concept-basic-url-parser
    pub fn parse_with_base(
        mut input: &str,
        base: Option<URL>,
        given_url: Option<URL>,
        state_override: Option<URLParserState>,
    ) -> Self {
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

        let mut state_machine = URLParser {
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
        _ = state_machine.run();
        state_machine.url
    }

    /// [Specification](https://url.spec.whatwg.org/#concept-basic-url-parser)
    pub fn parse(input: &str) -> Self {
        Self::parse_with_base(input, None, None, None)
    }

    /// [Specification](https://url.spec.whatwg.org/#include-credentials)
    ///
    /// A [URL] includes credentials if its  [username](URL::username) or [password](URL::password) is not the empty string.
    pub fn includes_credentials(&self) -> bool {
        !self.username.is_empty() || !self.password.is_empty()
    }

    /// [Specification](https://url.spec.whatwg.org/#is-special)
    ///
    /// A [URL] is special if its scheme is a special scheme. A [URL] is not special if its scheme is not a special scheme.
    pub fn is_special(&self) -> bool {
        scheme_is_special(&self.scheme)
    }

    /// [Specification](https://url.spec.whatwg.org/#url-opaque-path)
    ///
    /// A [URL] has an opaque path if it only consists of a single string
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
}

impl From<&str> for URL {
    fn from(from: &str) -> Self {
        Self::parse(from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_url() {
        let url = URL::parse("https://google.com");

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
        let url = URL::parse("https://google.com?a=b");

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
        let url = URL::parse("https://google.com#foo");

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
        let url = URL::parse("https://user:password@google.com");

        assert_eq!(url.scheme, "https");
        assert_eq!(url.username, "user");
        assert_eq!(url.password, "password");
        assert_eq!(url.host, Some(Host::OpaqueHost("google.com".to_string())));
        assert_eq!(url.path, vec![""]);
        assert_eq!(url.query, None);
        assert_eq!(url.fragment, None);
    }
}
