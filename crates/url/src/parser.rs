//! URL parser implementation
//!
//! The spec defines a complicated state machine that unfortunately doesn't translate
//! well into actual code, which is why we don't adhere to the spec as closely here.

use sl_std::{ascii, chars::ReversibleCharIterator};

use crate::{
    host::{self, HostParseError},
    is_special_scheme,
    percent_encode::{
        is_c0_percent_encode_set, is_fragment_percent_encode_set, is_query_percent_encode_set,
        is_special_query_percent_encode_set, is_userinfo_percent_encode_set, percent_encode,
    },
    util::{is_double_dot_path_segment, is_single_dot_path_segment, is_windows_drive_letter},
    URL,
};

#[derive(Clone, Copy, Debug)]
pub enum Error {
    /// Generic Error
    Failure,

    MissingHost,
    InvalidPort,

    /// Failed to parse host
    HostParse(HostParseError),
}

#[derive(Clone, Copy, Debug)]
pub enum URLParserState {
    SchemeStart,
    Scheme,
    NoScheme,
    SpecialRelativeOrAuthority,
    PathOrAuthority,
    Relative,
    RelativeSlash,
    SpecialAuthoritySlashes,
    SpecialAuthorityIgnoreSlashes,
    Authority,
    Host,
    Hostname,
    Port,
    File,
    FileSlash,
    FileHost,
    PathStart,
    Path,
    OpaquePath,
    Query,
    Fragment,
}

pub(crate) struct URLParser<'a> {
    pub(crate) url: URL,
    pub(crate) input: ReversibleCharIterator<&'a str>,
}

impl<'a> URLParser<'a> {
    /// Consumes and returns the scheme from the input if there is any.
    /// If there is no scheme then the position of the input is undefined.
    ///
    /// <https://url.spec.whatwg.org/#scheme-state>
    fn parse_scheme(&mut self) -> bool {
        // First character must be ascii alpha
        if !self.input.next().is_some_and(|c| c.is_ascii_alphabetic()) {
            return false;
        }

        while let Some(c) = self.input.next() {
            if c == ':' {
                let scheme = &self.input.source()[..self.input.position() - 1];
                let ascii_scheme = scheme.try_into().expect("all scheme chars are ascii");
                self.url.serialization.push_str(ascii_scheme);
                self.url.serialization.make_lowercase();
                self.url.offsets.scheme_end = self.url.serialization.len();
                self.url.serialization.push(ascii::Char::Colon);
                return true;
            }

            if !matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '+' | '-' | '.') {
                return false;
            }
        }

        // EOF in scheme
        false
    }

    pub fn parse_complete(&mut self, base: Option<&URL>) -> Result<(), Error> {
        let has_scheme = self.parse_scheme();

        if has_scheme {
            let scheme = self.url.scheme().as_str();

            if scheme == "file" {
                self.parse_file()
            } else if is_special_scheme(scheme) {
                if let Some(base) = base
                    && base.scheme() == scheme
                {
                    if self.input.remaining().starts_with("//") {
                        self.parse_special_authority_slashes()
                    } else {
                        self.parse_relative(base)
                    }
                } else {
                    self.parse_special_authority_slashes()
                }
            } else if self.input.remaining().starts_with('/') {
                self.input.next();
                self.parse_path_or_authority()
            } else {
                self.parse_opaque_path()
            }
        } else {
            self.input.set_position(0);
            self.parse_no_scheme(base)
        }
    }

    /// <https://url.spec.whatwg.org/#special-authority-slashes-state>
    /// <https://url.spec.whatwg.org/#special-authority-ignore-slashes-state>
    fn parse_special_authority_slashes(&mut self) -> Result<(), Error> {
        while self.input.current() == Some('/') {
            self.input.next();
        }

        self.parse_authority()
    }

    /// <https://url.spec.whatwg.org/#authority-state>
    fn parse_authority(&mut self) -> Result<(), Error> {
        self.url.serialization.push_str(ascii!("//"));

        let mut at_sign_seen = false;
        let mut password_token_seen = false;

        // When parsing authority, we don't know whether what we're parsing is a username
        // or a host before we see either a @ or a /. FIXME: Don't parse stuff twice here!
        let mut host_start_in_input = self.input.position();
        let mut host_start_in_serialization = self.url.serialization.len();

        let is_special_url = self.url.is_special();
        let terminates_authority =
            |c| matches!(c, '/' | '?' | '#') || (is_special_url && c == '\\');

        self.url.offsets.username_start = self.url.serialization.len();
        self.url.offsets.password_start = self.url.serialization.len();

        while let Some(c) = self.input.current().filter(|&c| !terminates_authority(c)) {
            self.input.next();

            if c == '@' {
                if at_sign_seen {
                    self.url.serialization.push_str(ascii!("%40"));
                } else {
                    self.url.serialization.push(ascii::Char::CommercialAt);
                    at_sign_seen = true;
                }

                self.url.offsets.host_start = self.url.serialization.len();

                host_start_in_input = self.input.position();
                host_start_in_serialization = self.url.serialization.len();
            } else if c == ':' {
                self.url.serialization.push(ascii::Char::Colon);

                if !password_token_seen {
                    self.url.offsets.password_start = self.url.serialization.len();
                    password_token_seen = true;
                }
            } else {
                let mut buffer = [0; 4];
                c.encode_utf8(&mut buffer);
                percent_encode(
                    &buffer[..c.len_utf8()],
                    is_userinfo_percent_encode_set,
                    &mut self.url.serialization,
                );
            }
        }

        if at_sign_seen && host_start_in_input == self.input.position() {
            return Err(Error::MissingHost);
        }

        self.input.set_position(host_start_in_input);
        self.url.serialization.truncate(host_start_in_serialization);

        self.parse_host()
    }

    /// <https://url.spec.whatwg.org/#host-state>
    fn parse_host(&mut self) -> Result<(), Error> {
        let host_start = self.input.position();
        self.url.offsets.host_start = self.url.serialization.len();
        let is_special_url = self.url.is_special();
        let mut inside_brackets = false;
        let terminates_host = |c| matches!(c, '/' | '?' | '#') || (is_special_url && c == '\\');

        while let Some(c) = self.input.current().filter(|&c| !terminates_host(c)) {
            self.input.next();

            if c == ':' && !inside_brackets {
                let host_buffer = &self.input.source()[host_start..self.input.position() - 1];
                let host = host::parse_with_special(host_buffer, !is_special_url)?;
                let host_serialization: ascii::String =
                    ascii::String::try_from(format!("{host}")).expect("is ascii");
                self.url.serialization.push_str(&host_serialization);
                self.url.host = Some(host);
                return self.parse_port();
            }

            if c == '[' {
                inside_brackets = true;
            } else if c == ']' {
                inside_brackets = false;
            }
        }

        if is_special_url && self.input.position() == host_start {
            return Err(Error::MissingHost);
        }

        let host_buffer = &self.input.source()[host_start..self.input.position()];
        let host = host::parse_with_special(host_buffer, !is_special_url)?;
        let host_serialization: ascii::String =
            ascii::String::try_from(format!("{host}")).expect("is ascii");
        self.url.serialization.push_str(&host_serialization);
        self.url.host = Some(host);

        self.parse_path_start()
    }

    fn parse_port(&mut self) -> Result<(), Error> {
        let is_special = self.url.is_special();
        let terminates_port = |c| matches!(c, '/' | '?' | '#') || (is_special && c == '\\');

        let end_of_port = self
            .input
            .remaining()
            .chars()
            .position(terminates_port)
            .unwrap_or(self.input.remaining().len());
        let port_str = &self.input.remaining()[..end_of_port];

        let port: u16 = port_str.parse().map_err(|_| Error::InvalidPort)?;
        self.url.port = Some(port);

        let port_str: &ascii::Str = port_str
            .try_into()
            .expect("port numbers are always valid ascii");

        self.url.serialization.push_str(port_str);

        self.input
            .set_position(self.input.position() + port_str.len());

        self.parse_path_start()
    }

    /// <https://url.spec.whatwg.org/#path-start-state>
    fn parse_path_start(&mut self) -> Result<(), Error> {
        self.url.offsets.path_start = self.url.serialization.len();

        if self.url.is_special() {
            if self.input.current() == Some('/') {
                self.input.next();
                self.url.serialization.push(ascii::Char::Solidus);
            }

            self.parse_path()
        } else {
            match self.input.current() {
                Some('?') => {
                    self.input.next();
                    self.parse_query()
                },
                Some('#') => {
                    self.input.next();
                    self.parse_fragment()
                },
                Some('/') => {
                    self.input.next();
                    self.parse_path()
                },
                Some(_) => self.parse_path(),
                None => Ok(()),
            }
        }
    }

    /// <https://url.spec.whatwg.org/#path-state>
    fn parse_path(&mut self) -> Result<(), Error> {
        let is_special = self.url.is_special();

        let mut terminating_character = None;
        loop {
            let path_segment_start = self.url.serialization.len();
            let mut is_last_segment = false;

            // Inner loop parses a single path segment
            loop {
                let Some(c) = self.input.next() else {
                    is_last_segment = true;
                    break;
                };

                if c == '?' || c == '#' {
                    terminating_character = Some(c);
                    is_last_segment = true;
                    break;
                }
                if c == '/' || (is_special && c == '\\') {
                    self.url.serialization.push(ascii::Char::Solidus);
                    break;
                }

                let mut buffer = [0; 4];
                c.encode_utf8(&mut buffer);
                percent_encode(
                    &buffer[..c.len_utf8()],
                    is_userinfo_percent_encode_set,
                    &mut self.url.serialization,
                );
            }

            let segment = &self.url.serialization[path_segment_start..];

            if is_double_dot_path_segment(segment.as_str()) {
                self.url.serialization.truncate(path_segment_start - 1);
                self.url.shorten_path();
            } else if is_single_dot_path_segment(segment.as_str()) {
                if is_last_segment {
                    self.url.serialization.push(ascii::Char::Solidus);
                } else {
                    self.url.serialization.push(ascii::Char::FullStop);
                }
            } else {
                if self.url.scheme() == "file" && is_windows_drive_letter(segment.as_str()) {
                    self.url.serialization[path_segment_start + 1] = ascii::Char::Colon;
                }

                self.url.serialization.push(ascii::Char::Solidus);
            }

            if is_last_segment {
                break;
            }
        }

        match terminating_character {
            Some('?') => self.parse_query(),
            Some('#') => self.parse_fragment(),
            _ => Ok(()),
        }
    }

    fn parse_path_or_authority(&mut self) -> Result<(), Error> {
        if self.input.current() == Some('/') {
            self.url.serialization.push(ascii::Char::Colon);
            self.input.next();
            self.parse_authority()
        } else {
            self.parse_path()
        }
    }

    /// <https://url.spec.whatwg.org/#cannot-be-a-base-url-path-state>
    fn parse_opaque_path(&mut self) -> Result<(), Error> {
        self.url.offsets.path_start = self.url.serialization.len();
        while let Some(c) = self.input.next() {
            if c == '?' {
                return self.parse_query();
            } else if c == '#' {
                return self.parse_fragment();
            } else {
                let mut buffer = [0; 4];
                c.encode_utf8(&mut buffer);
                percent_encode(
                    &buffer[..c.len_utf8()],
                    is_c0_percent_encode_set,
                    &mut self.url.serialization,
                );
            }
        }

        // EOF in path
        Ok(())
    }

    /// <https://url.spec.whatwg.org/#no-scheme-state>
    fn parse_no_scheme(&mut self, base: Option<&URL>) -> Result<(), Error> {
        let Some(base) = base else {
            return Err(Error::Failure);
        };

        let c = self.input.current();

        if base.has_opaque_path() {
            if c != Some('#') {
                return Err(Error::Failure);
            }
            self.input.next();

            let base_fragment_start = base
                .offsets
                .fragment_start
                .unwrap_or(base.serialization.len());
            self.url.serialization.clear();
            self.url
                .serialization
                .push_str(&base.serialization[..base_fragment_start]);

            self.url.offsets = base.offsets;
            self.url.offsets.fragment_start = Some(self.url.serialization.len());
            self.url.serialization.push(ascii::Char::NumberSign);

            // and set state to fragment state.
            return self.parse_fragment();
        } else if base.scheme() == "file" {
            return self.parse_file();
        } else {
            return self.parse_relative(base);
        }
    }

    fn parse_file(&mut self) -> Result<(), Error> {
        // FIXME
        Ok(())
    }

    /// <https://url.spec.whatwg.org/#query-state>
    ///
    /// This expects the starting `?` to have already been consumed (but not serialized)
    fn parse_query(&mut self) -> Result<(), Error> {
        self.url.serialization.push(ascii::Char::QuestionMark);
        self.url.offsets.query_start = Some(self.url.serialization.len());

        let percent_encode_set = if self.url.is_special() {
            is_special_query_percent_encode_set
        } else {
            is_query_percent_encode_set
        };

        let query_start = self.input.position();
        while let Some(c) = self.input.next() {
            if c == '#' {
                let buffer = &self.input.source()[query_start..self.input.position()];

                percent_encode(
                    buffer.as_bytes(),
                    percent_encode_set,
                    &mut self.url.serialization,
                );
                return self.parse_fragment();
            }
        }

        // EOF in query
        let buffer = &self.input.source()[query_start..self.input.position()];

        percent_encode(
            buffer.as_bytes(),
            percent_encode_set,
            &mut self.url.serialization,
        );

        Ok(())
    }

    /// <https://url.spec.whatwg.org/#fragment-state>
    ///
    /// This expects the starting `#` to have already been consumed (but not serialized)
    fn parse_fragment(&mut self) -> Result<(), Error> {
        self.url.serialization.push(ascii::Char::NumberSign);
        self.url.offsets.fragment_start = Some(self.url.serialization.len());

        let buffer = self.input.remaining();

        percent_encode(
            buffer.as_bytes(),
            is_fragment_percent_encode_set,
            &mut self.url.serialization,
        );

        Ok(())
    }

    /// <https://url.spec.whatwg.org/#relative-state>
    fn parse_relative(&mut self, base: &URL) -> Result<(), Error> {
        self.url.serialization.clear();
        self.url.serialization.push_str(&base.scheme());
        self.url.offsets.scheme_end = base.offsets.scheme_end;
        self.url.serialization.push(ascii::Char::Colon);

        let c = self.input.current();
        if c == Some('/') || (self.url.is_special() && c == Some('\\')) {
            return self.parse_relative_slash();
        }

        let base_fragment_end = base
            .offsets
            .fragment_start
            .unwrap_or(base.serialization.len());
        let username_to_query = &base.serialization[base.offsets.scheme_end..base_fragment_end];
        self.url.serialization.push_str(username_to_query);

        match c {
            Some('?') => {
                if let Some(query_start) = self.url.offsets.query_start {
                    self.url.serialization.truncate(query_start);
                }

                self.parse_query()
            },
            Some('#') => self.parse_fragment(),
            Some(_) => {
                self.url.shorten_path();
                self.parse_path()
            },
            _ => Ok(()),
        }
    }

    fn parse_relative_slash(&mut self) -> Result<(), Error> {
        // FIXME
        Ok(())
    }
}

impl From<HostParseError> for Error {
    fn from(value: HostParseError) -> Self {
        Self::HostParse(value)
    }
}
