//! Implements <https://url.spec.whatwg.org>

use std::{
    fmt::Display,
    io,
    path::{self, Path},
    str::FromStr,
};

use sl_std::{ascii, chars::ReversibleCharIterator};

use crate::{
    host::Host,
    parser::{URLParser, URLParserState},
    percent_encode::percent_decode,
    util, IgnoreValidationErrors,
};

pub type Port = u16;

/// <https://url.spec.whatwg.org/#special-scheme>
pub(crate) fn is_special_scheme(scheme: &str) -> bool {
    matches!(scheme, "ftp" | "file" | "http" | "https" | "ws" | "wss")
}

/// <https://url.spec.whatwg.org/#default-port>
pub(crate) fn default_port_for_scheme(scheme: &ascii::Str) -> Option<Port> {
    match scheme.as_str() {
        "ftp" => Some(21),
        "http" | "ws" => Some(80),
        "https" | "wss" => Some(443),
        _ => None,
    }
}

/// A **U**niform **R**esource **L**ocator
///
/// [Specification](https://url.spec.whatwg.org/#concept-url)
///
/// # Segments
/// A url typically consists of the following segments:
/// ```text, ignore
/// https://username:password@example.com/foobar?foo=bar#baz               
/// │       │        │        │           │      │       │                 
/// └───────┼────────┼────────┼───────────┼──────┼───────┼─► Scheme start  
///         │        │        │           │      │       │                 
///         └────────┼────────┼───────────┼──────┼───────┼─► Username start
///                  │        │           │      │       │                 
///                  └────────┼───────────┼──────┼───────┼─► Password start
///                           │           │      │       │                 
///                           └───────────┼──────┼───────┼─► Host start    
///                                       │      │       │                 
///                                       └──────┼───────┼─► Path start    
///                                              │       │                 
///                                              └───────┼─► Query start   
///                                                      │                 
///                                                      └─► Fragment start
/// ```
#[cfg_attr(
    feature = "serialize",
    derive(serialize::Serialize, serialize::Deserialize)
)]
#[derive(Default, Clone, Debug, Hash, PartialEq, Eq)]
pub struct URL {
    pub(crate) port: Option<u16>,
    pub(crate) host: Option<Host>,

    /// The serialized URL, stored for efficiency
    pub(crate) serialization: ascii::String,

    pub(crate) scheme_end: usize,
    pub(crate) username_start: usize,
    pub(crate) password_start: usize,
    pub(crate) host_start: usize,
    pub(crate) path_start: usize,
    pub(crate) query_start: Option<usize>,
    pub(crate) fragment_start: Option<usize>,
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
    /// [Specification](https://url.spec.whatwg.org/#is-special)
    #[inline]
    #[must_use]
    pub fn is_special(&self) -> bool {
        is_special_scheme(self.scheme().as_str())
    }

    #[inline]
    #[must_use]
    pub fn default_port(&self) -> Option<Port> {
        default_port_for_scheme(&self.scheme())
    }

    /// A [URL]’s scheme is an ASCII string that identifies the type of URL
    /// and can be used to dispatch a URL for further processing after parsing.
    #[inline]
    #[must_use]
    pub fn scheme(&self) -> &ascii::Str {
        &self.serialization[..self.scheme_end]
    }

    #[inline]
    #[must_use]
    pub fn username(&self) -> &ascii::Str {
        &self.serialization[self.username_start..self.password_start - 1]
    }

    #[inline]
    #[must_use]
    pub fn password(&self) -> &ascii::Str {
        &self.serialization[self.password_start..self.host_start - 1]
    }

    #[inline]
    #[must_use]
    pub fn host(&self) -> Option<&Host> {
        self.host.as_ref()
    }

    #[inline]
    #[must_use]
    pub fn port(&self) -> Option<Port> {
        self.port
    }

    #[inline]
    #[must_use]
    pub fn path(&self) -> &ascii::Str {
        let path_end = self
            .query_start
            .or(self.fragment_start)
            .unwrap_or(self.serialization.len());
        &self.serialization[self.path_start..path_end]
    }

    #[inline]
    #[must_use]
    pub fn query(&self) -> Option<&ascii::Str> {
        let query_start = self.query_start?;
        let query_end = self.fragment_start.unwrap_or(self.serialization.len());

        Some(&self.serialization[query_start..query_end])
    }

    #[inline]
    #[must_use]
    pub fn fragment(&self) -> Option<&ascii::Str> {
        self.fragment_start
            .map(|start| &self.serialization[start..])
    }

    pub fn from_user_input(input: &str) -> Result<Self, URLParseError> {
        let base_url = match Self::cwd() {
            Ok(url) => url,
            Err(error) => {
                log::error!("Failed to access current working directory: {error}");
                return Err(URLParseError);
            },
        };

        Self::parse_with_base(input, Some(base_url), None, None)
            .or_else(|_| format!("http://{input}").parse())
    }

    #[cfg(unix)]
    pub fn as_file_path(&self) -> Result<path::PathBuf, InvalidFilePath> {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let mut bytes = vec![];

        for segment in &self.path {
            bytes.push(b'/');
            bytes.extend_from_slice(&percent_decode(segment));
        }

        let path = path::PathBuf::from(OsStr::from_bytes(&bytes));
        debug_assert!(path.is_absolute());

        Ok(path)
    }

    #[cfg(windows)]
    pub fn as_file_path(&self) -> Result<path::PathBuf, InvalidFilePath> {
        let mut segments = self.path().iter();

        // Make sure that the first segment is a valid start of a absolute
        // windows path
        let first_segment = segments.next().ok_or(InvalidFilePath)?;
        let mut result: String = match first_segment.len() {
            2 => {
                // Drive letter
                if first_segment[0].to_char().is_ascii_alphabetic()
                    && first_segment[1] == ascii::Char::Colon
                {
                    first_segment.as_str().to_owned()
                } else {
                    return Err(InvalidFilePath);
                }
            },
            _ => return Err(InvalidFilePath),
        };

        for segment in segments {
            result.push(path::MAIN_SEPARATOR);
            result.push_str(&String::from_utf8_lossy(&percent_decode(segment)));
        }

        let path = path::PathBuf::from(result);
        debug_assert!(
            path.is_absolute(),
            "to_file_path() failed to produce an absolute Path"
        );
        Ok(path)
    }

    pub fn cwd() -> Result<Self, io::Error> {
        let cwd = std::env::current_dir()?;

        let mut serialization: ascii::String = "file:///".try_into().expect("is ascii");
        for part in cwd.iter().skip(1) {
            let bytes = part.as_encoded_bytes();
            let Some(ascii_str) = ascii::Str::from_bytes(bytes) else {
                let error = io::Error::other(format!(
                    "Path to cwd ({}) contains non-ascii data",
                    cwd.display()
                ));
                return Err(error);
            };

            serialization.push_str(ascii_str);
        }

        // Since we are referring to a directory (which ends with a slash)
        // the last path segment is empty
        serialization.push(ascii::Char::Solidus);

        let url = Self {
            port: None,
            host: None,
            serialization,
            scheme_end: 4,
            username_start: 4,
            password_start: 4,
            host_start: 5,
            path_start: 6,
            query_start: None,
            fragment_start: None,
        };
        Ok(url)
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

        let state_machine = URLParser {
            url,
            state,
            buffer,
            base,
            input: ReversibleCharIterator::new(&filtered_input),
            state_override,
            at_sign_seen,
            inside_brackets,
            password_token_seen,
            error_handler: IgnoreValidationErrors,
        };

        let parsed_url = state_machine
            .run_to_completion()
            .map_err(|_| URLParseError)?
            .url;
        Ok(parsed_url)
    }

    /// [Specification](https://url.spec.whatwg.org/#include-credentials)
    ///
    /// A [URL] includes credentials if its  [username](URL::username) or [password](URL::password) is not the empty string.
    #[must_use]
    pub fn includes_credentials(&self) -> bool {
        !self.username().is_empty() || !self.password().is_empty()
    }

    /// [Specification](https://url.spec.whatwg.org/#url-opaque-path)
    ///
    /// A [URL] has an opaque path if it only consists of a single string
    #[must_use]
    pub fn has_opaque_path(&self) -> bool {
        self.scheme_end + 1 == self.username_start
    }

    /// <https://url.spec.whatwg.org/#url-serializing>
    pub fn serialize(&self, exclude_fragment: ExcludeFragment) -> &ascii::Str {
        match self.fragment_start {
            Some(offset) if exclude_fragment == ExcludeFragment::Yes => {
                &self.serialization[..offset - 1]
            },
            _ => &self.serialization,
        }
    }
}

impl FromStr for URL {
    type Err = URLParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // https://url.spec.whatwg.org/#concept-basic-url-parser
        Self::parse_with_base(s, None, None, None)
    }
}

impl Display for URL {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.serialize(ExcludeFragment::default()))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InvalidFilePath;

impl From<&Path> for URL {
    fn from(value: &Path) -> Self {
        let absolute_path = value.canonicalize().expect("Failed to canonicalize path");

        let mut path_segments = vec![];
        for part in absolute_path.iter().skip(1) {
            let bytes = part.as_encoded_bytes();

            let Some(ascii_str) = ascii::Str::from_bytes(bytes) else {
                panic!(
                    "Path contains non-ascii data: {:?}",
                    absolute_path.display()
                );
            };

            path_segments.push(ascii_str.to_owned());
        }

        Self {
            scheme: ascii::String::try_from("file").unwrap(),
            username: ascii::String::new(),
            password: ascii::String::new(),
            host: Some(Host::OpaqueHost(ascii::String::new())),
            port: None,
            path: path_segments,
            query: None,
            fragment: None,
            serialization: todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use sl_std::ascii;

    use super::{Host, URL};

    #[test]
    fn test_simple_url() {
        let url: URL = "https://google.com".parse().unwrap();

        assert_eq!(url.scheme(), "https");
        assert_eq!(url.username(), "");
        assert_eq!(url.password(), "");
        assert_eq!(
            url.host,
            Some(Host::OpaqueHost(
                ascii::Str::from_bytes(b"google.com").unwrap().to_owned()
            ))
        );
        assert_eq!(url.path, vec![""]);
        assert_eq!(url.query(), None);
        assert_eq!(url.fragment(), None);
    }

    #[test]
    fn test_with_query() {
        let url: URL = "https://google.com?a=b".parse().unwrap();

        assert_eq!(url.scheme(), "https");
        assert_eq!(url.username(), "");
        assert_eq!(url.password(), "");
        assert_eq!(
            url.host,
            Some(Host::OpaqueHost(
                ascii::Str::from_bytes(b"google.com").unwrap().to_owned()
            ))
        );
        assert_eq!(url.path, vec![""]);
        assert_eq!(url.query().as_deref().map(ascii::Str::as_str), Some("a=b"));
        assert_eq!(url.fragment(), None);
    }

    #[test]
    fn test_with_fragment() {
        let url: URL = "https://google.com#foo".parse().unwrap();

        assert_eq!(url.scheme(), "https");
        assert_eq!(url.username(), "");
        assert_eq!(url.password(), "");
        assert_eq!(
            url.host,
            Some(Host::OpaqueHost(
                ascii::Str::from_bytes(b"google.com").unwrap().to_owned()
            ))
        );
        assert_eq!(url.path, vec![ascii::String::new()]);
        assert_eq!(url.query(), None);
        assert_eq!(
            url.fragment().as_deref().map(ascii::Str::as_str),
            Some("foo")
        );
    }

    #[test]
    fn test_with_credentials() {
        let url: URL = "https://user:password@google.com".parse().unwrap();

        assert_eq!(url.scheme(), "https");
        assert_eq!(url.username(), "user");
        assert_eq!(url.password(), "password");
        assert_eq!(
            url.host,
            Some(Host::OpaqueHost(
                ascii::Str::from_bytes(b"google.com").unwrap().to_owned()
            ))
        );
        assert_eq!(url.path, vec![""]);
        assert_eq!(url.query(), None);
        assert_eq!(url.fragment(), None);
    }

    #[test]
    fn opaque_path() {
        let url: URL = "data:text/html,Hello World".parse().unwrap();
        assert_eq!(url.path(), &["text/html,Hello World"]);
    }
}
