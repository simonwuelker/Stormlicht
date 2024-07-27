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
    parser::{self, Parser},
    percent_encode::percent_decode,
    util::{self, is_normalized_windows_drive_letter},
    PathSegments,
};

pub type Port = u16;

/// We refuse to parser urls longer than this
const MAX_URL_LEN: usize = 0x10000;

#[derive(Debug)]
pub enum Error {
    /// The length of the URL exceeds [MAX_URL_LEN]
    TooLong,
    Io(io::Error),
    Parser(parser::Error),
}

#[cfg_attr(
    feature = "serialize",
    derive(serialize::Serialize, serialize::Deserialize)
)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct UrlOffsets {
    // index of `:`
    pub scheme_end: usize,

    pub query_start: Option<usize>,
    pub fragment_start: Option<usize>,
    pub username_start: usize,
    pub password_start: usize,
    pub host_start: usize,
    pub path_start: usize,
}

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
#[cfg_attr(
    feature = "serialize",
    derive(serialize::Serialize, serialize::Deserialize)
)]
#[derive(Default, Clone, Debug, Hash, PartialEq, Eq)]
pub struct URL {
    /// A [URL]’s host is [None] or a [host](Host).
    ///
    /// It is initially [None].
    pub(crate) host: Option<Host>,

    /// A [URL]’s port is either [None] or a 16-bit unsigned integer that identifies a networking port.
    ///
    /// It is initially [None].
    pub(crate) port: Option<Port>,

    pub(crate) serialization: ascii::String,
    pub(crate) offsets: UrlOffsets,
}

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

    #[inline]
    #[must_use]
    pub fn scheme(&self) -> &ascii::Str {
        &self.serialization[..self.offsets.scheme_end]
    }

    #[inline]
    #[must_use]
    pub fn username(&self) -> &ascii::Str {
        let username_start = self.offsets.username_start;
        let password_start = self.offsets.password_start;

        if username_start == password_start {
            ascii::Str::EMPTY
        } else {
            &self.serialization[username_start..password_start - 1]
        }
    }

    #[inline]
    #[must_use]
    pub fn password(&self) -> &ascii::Str {
        let password_start = self.offsets.password_start;
        let host_start = self.offsets.host_start;

        if password_start == host_start {
            ascii::Str::EMPTY
        } else {
            &self.serialization[password_start..host_start - 1]
        }
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
        if let Some(end) = self.offsets.query_start.or(self.offsets.fragment_start) {
            &self.serialization[self.offsets.path_start..end - 1]
        } else {
            &self.serialization[self.offsets.path_start..]
        }
    }

    #[inline]
    #[must_use]
    pub fn path_segments(&self) -> PathSegments<'_> {
        PathSegments::new(self.path())
    }

    #[inline]
    #[must_use]
    pub fn query(&self) -> Option<&ascii::Str> {
        let query_start = self.offsets.query_start?;

        let query = if let Some(fragment_start) = self.offsets.fragment_start {
            &self.serialization[query_start..fragment_start - 1]
        } else {
            &self.serialization[query_start..]
        };

        Some(query)
    }

    #[inline]
    #[must_use]
    pub fn fragment(&self) -> Option<&ascii::Str> {
        let fragment_start = self.offsets.fragment_start?;

        Some(&self.serialization[fragment_start..])
    }

    pub fn from_user_input(input: &str) -> Result<Self, Error> {
        let base_url = match Self::cwd() {
            Ok(url) => url,
            Err(error) => {
                log::error!("Failed to access current working directory: {error}");
                return Err(Error::Io(error));
            },
        };

        Self::parse_with_base(input, Some(&base_url), None)
            .or_else(|_| format!("http://{input}").parse())
    }

    #[cfg(unix)]
    pub fn as_file_path(&self) -> Result<path::PathBuf, InvalidFilePath> {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let mut bytes = vec![];

        for segment in self.path().split(ascii::Char::Solidus) {
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

        Self::try_from(cwd.as_path()).map_err(|_| io::Error::other("path contains non-ascii data"))
    }

    /// [Specification](https://url.spec.whatwg.org/#concept-basic-url-parser)
    pub fn parse_with_base(
        mut input: &str,
        base: Option<&URL>,
        given_url: Option<URL>,
    ) -> Result<Self, Error> {
        if input.len() > MAX_URL_LEN {
            log::error!("Refusing to parse url with length {:x}", input.len());
            return Err(Error::TooLong);
        }

        let url = match given_url {
            Some(url) => url,
            None => {
                input = input.trim_matches(util::is_c0_or_space);

                let url = Self {
                    // This should be a reasonable approximation
                    serialization: ascii::String::with_capacity(input.len()),
                    offsets: UrlOffsets::default(),
                    host: None,
                    port: None,
                };

                url
            },
        };

        let filtered_input: String = input
            .chars()
            .filter(|c| !util::is_ascii_tab_or_newline(*c))
            .collect();

        let mut state_machine = Parser {
            url,
            input: ReversibleCharIterator::new(&filtered_input),
        };

        state_machine.parse_complete(base)?;

        Ok(state_machine.url)
    }

    /// [Specification](https://url.spec.whatwg.org/#include-credentials)
    ///
    /// A [URL] includes credentials if its  [username](URL::username) or [password](URL::password) is not the empty string.
    #[must_use]
    pub fn includes_credentials(&self) -> bool {
        self.offsets.username_start != self.offsets.password_start
            || self.offsets.password_start != self.offsets.host_start
    }

    /// [Specification](https://url.spec.whatwg.org/#url-opaque-path)
    ///
    /// A [URL] has an opaque path if it only consists of a single string
    #[must_use]
    pub fn has_opaque_path(&self) -> bool {
        self.offsets.scheme_end + 1 == self.offsets.host_start
    }

    /// <https://url.spec.whatwg.org/#shorten-a-urls-path>
    ///
    /// This implementation also gets rid of anything after the path (query, fragment),
    /// so it should only be called during parsing
    pub(crate) fn shorten_path(&mut self) {
        if self.scheme() == "file" {
            let mut segments = self.path_segments();

            if segments
                .next()
                .is_some_and(|s| is_normalized_windows_drive_letter(s.as_str()))
                && segments.next().is_none()
            {
                return;
            }
        }

        let last_slash = self
            .serialization
            .rfind(ascii::Char::Solidus)
            .unwrap_or(self.offsets.path_start);

        // FIXME: do we need to adjust query/fragment offsets here?
        self.serialization.truncate(last_slash)
    }

    /// <https://url.spec.whatwg.org/#url-serializing>
    pub fn serialize(&self, exclude_fragment: ExcludeFragment) -> &ascii::Str {
        let end = if exclude_fragment == ExcludeFragment::Yes {
            self.offsets
                .fragment_start
                .map(|v| v - 1)
                .unwrap_or(self.serialization.len())
        } else {
            self.serialization.len()
        };

        &self.serialization[..end]
    }
}

impl FromStr for URL {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // https://url.spec.whatwg.org/#concept-basic-url-parser
        Self::parse_with_base(s, None, None)
    }
}

impl Display for URL {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.serialize(ExcludeFragment::default()))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InvalidFilePath;

impl TryFrom<&Path> for URL {
    type Error = InvalidFilePath;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let absolute_path = value.canonicalize().expect("Failed to canonicalize path");

        let mut serialization = ascii!("file:///").to_owned();
        let mut offsets = UrlOffsets::default();
        offsets.scheme_end = "file".len();
        offsets.path_start = "file:///".len();

        for part in absolute_path.iter().skip(1) {
            let bytes = part.as_encoded_bytes();
            let Some(ascii_str) = ascii::Str::from_bytes(bytes) else {
                log::error!(
                    "Failed to create file URL for path containing non-ascii data: {}",
                    absolute_path.display()
                );
                return Err(InvalidFilePath);
            };

            serialization.push_str(ascii_str);
            serialization.push(ascii::Char::Solidus);
        }

        Ok(Self {
            host: Some(Host::OpaqueHost(ascii::String::default())),
            port: None,
            serialization,
            offsets,
        })
    }
}

impl From<parser::Error> for Error {
    fn from(value: parser::Error) -> Self {
        Self::Parser(value)
    }
}

#[cfg(test)]
mod tests {
    use sl_std::ascii;

    use super::*;

    #[test]
    fn test_simple_url() {
        let url: URL = "https://google.com".parse().unwrap();

        assert_eq!(url.scheme(), "https");
        assert_eq!(url.username(), "");
        assert_eq!(url.password(), "");
        assert_eq!(
            url.host,
            Some(Host::Domain(ascii!("google.com").to_owned()))
        );
        assert_eq!(url.path(), "/");
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
            Some(Host::Domain(ascii!("google.com").to_owned()))
        );
        assert_eq!(url.path(), "/");
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
            Some(Host::Domain(ascii!("google.com").to_owned()))
        );
        assert_eq!(url.path(), "/");
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
            Some(Host::Domain(ascii!("google.com").to_owned()))
        );
        assert_eq!(url.path(), "/");
        assert_eq!(url.query(), None);
        assert_eq!(url.fragment(), None);
    }

    #[test]
    fn opaque_path() {
        let url: URL = "data:text/html,Hello World".parse().unwrap();
        assert_eq!(url.path(), "text/html,Hello World");
    }

    #[test]
    fn dont_parse_very_long_url() {
        // This is a valid, but way too long url
        let url_str = format!("https://example.com{}", " ".repeat(MAX_URL_LEN));

        let url: Result<URL, _> = url_str.parse();

        assert!(url.is_err());
    }

    #[test]
    fn filename_with_base_url() {
        let base: URL = "https://soju.im/".parse().unwrap();

        let name = "style.css";

        let url = URL::parse_with_base(name, Some(&base), None).unwrap();

        assert_eq!(url.scheme(), "https");
        assert_eq!(url.path(), "/style.css");
        assert_eq!(url.serialization, "https://soju.im/style.css");
    }
}
