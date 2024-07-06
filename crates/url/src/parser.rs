use sl_std::{
    ascii::{self, AsciiCharExt},
    chars::{ReversibleCharIterator, State},
};

use crate::{
    default_port_for_scheme,
    host::{self, host_parse_with_special, Host, HostParseError},
    is_special_scheme,
    percent_encode::{
        is_c0_percent_encode_set, is_fragment_percent_encode_set, is_path_percent_encode_set,
        is_query_percent_encode_set, is_special_query_percent_encode_set,
        is_userinfo_percent_encode_set, percent_encode,
    },
    set::AsciiSet,
    util::{self, is_url_codepoint},
    ValidationError, ValidationErrorHandler, URL,
};

#[derive(Clone, Copy, Debug)]
pub enum Error {
    /// Generic Error
    Failure,

    InvalidScheme,

    MissingSchemeNonRelativeURL,

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

pub(crate) struct URLParser<'a, H> {
    pub(crate) url: URL,
    pub(crate) base: Option<URL>,
    pub(crate) input: ReversibleCharIterator<&'a str>,
    pub(crate) state: URLParserState,

    /// A temporary character buffer used during parsing
    ///
    /// Notably, unlike everything in a URL, this can contain unicode data
    pub(crate) buffer: String,
    pub(crate) at_sign_seen: bool,
    pub(crate) inside_brackets: bool,
    pub(crate) password_token_seen: bool,
    pub(crate) state_override: Option<URLParserState>,
    pub error_handler: H,
}

const SCHEME_CODE_POINTS: AsciiSet = AsciiSet::ALPHANUMERIC
    .add(ascii::Char::PlusSign)
    .add(ascii::Char::HyphenMinus)
    .add(ascii::Char::FullStop);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StartOver {
    Yes,
    No,
}

impl<'a, H> URLParser<'a, H>
where
    H: ValidationErrorHandler,
{
    pub(crate) fn run_to_completion(mut self) -> Result<Self, Error> {
        loop {
            // Keep running the following state machine by switching on state.
            let start_over = self.step()?;

            if start_over == StartOver::Yes {
                self.input.set_position(0);
                continue;
            }

            // If after a run pointer points to the EOF code point
            if matches!(self.input.state(), State::AfterEnd(_)) {
                // go to the next step
                break;
            }
            // Otherwise,
            else {
                // increase pointer by 1 and continue with the state machine.
                self.input.next();
            }
        }
        Ok(self)
    }

    #[inline]
    fn set_state(&mut self, new_state: URLParserState) {
        self.state = new_state;
    }

    /// Parse the scheme of a URL
    ///
    /// * <https://url.spec.whatwg.org/#scheme-start-state>
    /// * <https://url.spec.whatwg.org/#scheme-state>
    fn parse_scheme(&mut self) -> Result<bool, Error> {
        // First character must be a valid scheme char
        if !self
            .input
            .current()
            .is_some_and(|c| c.is_ascii_alphabetic())
        {
            return Err(Error::InvalidScheme);
        }

        while let Some(c) = self.input.next() {
            if let Some(c) = c.as_ascii()
                && SCHEME_CODE_POINTS.contains(c)
            {
                self.url.serialization.push(c.to_lowercase());
            } else if c == ':' {
                // Set url’s scheme to buffer.
                self.url.scheme_end = self.url.serialization.len();
                self.url.serialization.push(ascii::Char::Colon);
            }
        }

        // EOF before scheme end -> no scheme
        Ok(false)
    }

    /// <https://url.spec.whatwg.org/#no-scheme-state>
    fn parse_no_scheme(&mut self, base: Option<&URL>) -> Result<(), Error> {
        let Some(base) = base else {
            self.error_handler
                .validation_error(ValidationError::MissingSchemeNonRelativeURL);

            return Err(Error::MissingSchemeNonRelativeURL);
        };

        if base.has_opaque_path() {
            if self.input.current() == Some('#') {
                // This URL only contains a fragment
                self.parse_fragment_only(&base)?;
            } else {
                self.error_handler
                    .validation_error(ValidationError::MissingSchemeNonRelativeURL);

                return Err(Error::MissingSchemeNonRelativeURL);
            }
        }

        if base.scheme() == "file" {
            self.parse_file(Some(base))
        } else {
            self.parse_relative()
        }
    }

    fn parse_fragment_only(&mut self, base: &URL) -> Result<(), Error> {
        let base_up_until_fragment = match base.fragment_start {
            Some(offset) => &base.serialization[..offset],
            None => &base.serialization,
        };

        self.url.serialization.clear();
        self.url.serialization.push_str(base_up_until_fragment);
        self.parse_fragment()
    }

    /// https://url.spec.whatwg.org/#file-state
    fn parse_file(&mut self, base: Option<&URL>) -> Result<(), Error> {
        self.url.serialization.clear(); // Everyone copies stuff into here anyways
        let base = base.filter(|url| url.scheme() == "file");

        if self.input.remaining().starts_with("//") {
            self.error_handler
                .validation_error(ValidationError::SpecialSchemeMissingFollowingSolidus);
        }

        if matches!(self.input.current(), Some('/' | '\\')) {
            if self.input.current() == Some('\\') {
                self.error_handler
                    .validation_error(ValidationError::InvalidReverseSolidus);
            }
            self.input.next();

            // File slash state
            if matches!(self.input.current(), Some('/' | '\\')) {
                if self.input.current() == Some('\\') {
                    self.error_handler
                        .validation_error(ValidationError::InvalidReverseSolidus);
                }
                self.input.next();

                return self.parse_file_host();
            } else {
                if let Some(base) = base {
                    // Copy everything up to the host
                    let host_str = &base.serialization[..base.path_start];
                    self.url.serialization.push_str(host_str);

                    // Windows drive letter quirk
                    if let Some(first_segment) = base.path_segments() {
                        todo!()
                    }
                }

                return self.parse_path();
            }
        }

        if let Some(base) = base {
            // Copy everything up to the query
            todo!()
        }

        self.parse_path()
    }

    /// <https://url.spec.whatwg.org/#path-state>
    fn parse_path(&mut self) -> Result<(), Error> {
        let allow_backslash = self.url.is_special();

        while let Some(c) = self.input.next() {
            match c {
                '?' => return self.parse_query(),
                '#' => return self.parse_query(),
                '/' => {
                    todo!();
                },
                '\\' if allow_backslash => {
                    self.error_handler
                        .validation_error(ValidationError::InvalidReverseSolidus);
                },
                other => {},
            }
        }

        Ok(())
    }

    /// <https://url.spec.whatwg.org/#cannot-be-a-base-url-path-state>
    fn parse_opaque_path(&mut self) -> Result<(), Error> {
        while let Some(c) = self.input.next() {
            match c {
                '?' => return self.parse_query(),
                '#' => return self.parse_fragment(),
                _ => {
                    let mut buffer = [0; 4];
                    c.encode_utf8(&mut buffer);
                    percent_encode(
                        &buffer[..c.len_utf8()],
                        is_c0_percent_encode_set,
                        &mut self.url.serialization,
                    );
                },
            }
        }

        Ok(())
    }

    /// <https://url.spec.whatwg.org/#query-state>
    fn parse_query(&mut self) -> Result<(), Error> {
        self.url.serialization.push(ascii::Char::QuestionMark);
        self.url.query_start = Some(self.url.serialization.len());

        let query_start = self.input.position();
        let has_fragment = self.input.find(|&c| c == '#').is_some();

        let query_slice = &self.input.source()[query_start..self.input.position()];

        let encode_set = if self.url.is_special() {
            is_special_query_percent_encode_set
        } else {
            is_query_percent_encode_set
        };
        percent_encode(
            query_slice.as_bytes(),
            encode_set,
            &mut self.url.serialization,
        );

        if has_fragment {
            self.parse_fragment()?;
        }

        Ok(())
    }

    /// <https://url.spec.whatwg.org/#fragment-state>
    fn parse_fragment(&mut self) -> Result<(), Error> {
        self.url.serialization.push(ascii::Char::NumberSign);
        self.url.fragment_start = Some(self.url.serialization.len());

        while let Some(c) = self.input.next() {
            if !is_url_codepoint(c) && c != '%' {
                self.error_handler
                    .validation_error(ValidationError::InvalidURLUnit);
            }

            // FIXME: If c is U+0025 (%) and remaining does not start with two ASCII hex digits, invalid-URL-unit validation error.

            let mut buffer = [0; 4];
            c.encode_utf8(&mut buffer);
            percent_encode(
                &buffer[..c.len_utf8()],
                is_fragment_percent_encode_set,
                &mut self.url.serialization,
            );
        }

        // Done
        Ok(())
    }

    /// <https://url.spec.whatwg.org/#relative-state>
    fn parse_relative(&mut self) -> Result<(), Error> {
        todo!()
    }

    /// <https://url.spec.whatwg.org/#concept-basic-url-parser>
    fn parse_complete(&mut self) -> Result<(), Error> {
        let has_scheme = self.parse_scheme()?;
        if has_scheme {
            match self.url.scheme().as_str() {
                "file" => self.parse_file(),
                "ftp" | "http" | "https" | "ws" | "wss" => {
                    // "special authority slashes" or "special relative or authority state"
                    todo!()
                },
                _ => todo!(),
            }
        } else {
            // Start over without a scheme
            self.input.set_position(0);
            self.parse_no_scheme()
        }
    }

    fn step(&mut self) -> Result<StartOver, Error> {
        match self.state {
            // https://url.spec.whatwg.org/#special-relative-or-authority-state
            URLParserState::SpecialRelativeOrAuthority => {
                // If c is U+002F (/) and remaining starts with U+002F (/)
                if self.input.current() == Some('/') && self.input.remaining().starts_with('/') {
                    // then set state to special authority ignore slashes state
                    self.set_state(URLParserState::SpecialAuthoritySlashes);

                    // and increase pointer by 1.
                    self.input.next();
                }
                // Otherwise,
                else {
                    // special-scheme-missing-following-solidus validation error
                    self.error_handler
                        .validation_error(ValidationError::SpecialSchemeMissingFollowingSolidus);

                    // set state to relative state
                    self.set_state(URLParserState::Relative);

                    // and decrease pointer by 1.
                    self.input.go_back();
                }
            },
            // https://url.spec.whatwg.org/#path-or-authority-state
            URLParserState::PathOrAuthority => {
                // if c is U+002F (/),
                if self.input.current() == Some('/') {
                    // then set state to authority state.
                    self.set_state(URLParserState::Authority);
                }
                // Otherwise,
                else {
                    // set state to path state,
                    self.set_state(URLParserState::Path);

                    // and decrease pointer by 1.
                    self.input.go_back();
                }
            },
            // https://url.spec.whatwg.org/#relative-state
            URLParserState::Relative => {
                // Assert: base’s scheme is not "file".
                let base = match &self.base {
                    Some(url) if url.scheme() != "file" => url,
                    _ => panic!("base must exist and have a scheme other than none"),
                };

                // Set url’s scheme to base’s scheme.
                self.url.serialization.clear();
                self.url.serialization.push_str(base.scheme());
                self.url.scheme_end = self.url.serialization.len();

                let c = self.input.current();
                // If c is U+002F (/)
                if c == Some('/') {
                    // then set state to relative slash state.
                    self.set_state(URLParserState::RelativeSlash);
                }
                // Otherwise, if url is special and c is U+005C (\)
                else if self.url.is_special() && c == Some('\\') {
                    // invalid-reverse-solidus validation error
                    self.error_handler
                        .validation_error(ValidationError::InvalidReverseSolidus);

                    // set state to relative slash state.
                    self.set_state(URLParserState::RelativeSlash);
                }
                // Otherwise:
                else {
                    // Set url’s username to base’s username
                    // url’s password to base’s password
                    // url’s host to base’s host
                    // url’s port to base’s port
                    // url’s path to a clone of base’s path
                    // and url’s query to base’s query.
                    self.url.serialization.clear();
                    self.url
                        .serialization
                        .push_str(base.serialization[base.quer]);

                    // If c is U+003F (?)
                    if c == Some('?') {
                        // then set url’s query to the empty string,
                        self.url.query_start = self.url.serialization.len();

                        // and state to query state.
                        self.set_state(URLParserState::Query);
                    }
                    // Otherwise, if c is U+0023 (#)
                    else if c == Some('#') {
                        // set url’s fragment to the empty string
                        self.url.fragment_start = self.url.serialization.len();

                        // and state to fragment state.
                        self.set_state(URLParserState::Fragment);
                    }
                }
            },
            // https://url.spec.whatwg.org/#relative-slash-state
            URLParserState::RelativeSlash => {
                let c = self.input.current();

                // If url is special and c is U+002F (/) or U+005C (\), then:
                if self.url.is_special() && matches!(c, Some('/' | '\\')) {
                    // If c is U+005C (\)
                    if c == Some('\\') {
                        // invalid-reverse-solidus validation error.
                        self.error_handler
                            .validation_error(ValidationError::InvalidReverseSolidus);
                    }

                    // Set state to special authority ignore slashes state.
                    self.set_state(URLParserState::SpecialAuthorityIgnoreSlashes);
                }
                // Otherwise, if c is U+002F (/)
                else if c == Some('/') {
                    // then set state to authority state.
                    self.set_state(URLParserState::Authority);
                }
                // Otherwise
                else {
                    let base = self.base.as_ref().expect("no base url");

                    // set url’s username to base’s username,
                    self.url.username.clone_from(&base.username);

                    // url’s password to base’s password,
                    self.url.password.clone_from(&base.password);

                    // url’s host to base’s host,
                    self.url.host.clone_from(&base.host);

                    // url’s port to base’s port,
                    self.url.port.clone_from(&base.port);

                    // state to path state,
                    self.set_state(URLParserState::Path);

                    // and then, decrease pointer by 1.
                    self.input.go_back();
                }
            },
            // https://url.spec.whatwg.org/#special-authority-slashes-state
            URLParserState::SpecialAuthoritySlashes => {
                // If c is U+002F (/) and remaining starts with U+002F (/)
                if self.input.current() == Some('/') && self.input.remaining().starts_with('/') {
                    // then set state to special authority ignore slashes state
                    self.set_state(URLParserState::SpecialAuthorityIgnoreSlashes);

                    // and increase pointer by 1.
                    self.input.next();
                }
                // Otherwise
                else {
                    // special-scheme-missing-following-solidus validation error
                    self.error_handler
                        .validation_error(ValidationError::SpecialSchemeMissingFollowingSolidus);

                    // set state to special authority ignore slashes state
                    self.set_state(URLParserState::SpecialAuthorityIgnoreSlashes);

                    // and decrease pointer by 1.
                    self.input.go_back();
                }
            },
            // https://url.spec.whatwg.org/#special-authority-ignore-slashes-state
            URLParserState::SpecialAuthorityIgnoreSlashes => {
                // If c is neither U+002F (/) nor U+005C (\)
                if !matches!(self.input.current(), Some('/' | '\\')) {
                    // then set state to authority state
                    self.set_state(URLParserState::Authority);

                    // and decrease pointer by 1.
                    self.input.go_back();
                }
                // Otherwise
                else {
                    // special-scheme-missing-following-solidus validation error.
                    self.error_handler
                        .validation_error(ValidationError::SpecialSchemeMissingFollowingSolidus);
                }
            },
            // https://url.spec.whatwg.org/#authority-state
            URLParserState::Authority => {
                let c = self.input.current();

                // If c is U+0040 (@), then:
                if c == Some('@') {
                    // Invalid-credentials validation error.
                    self.error_handler
                        .validation_error(ValidationError::InvalidCredentials);

                    // If atSignSeen is true,
                    if self.at_sign_seen {
                        // then prepend "%40" to buffer.
                        self.buffer.insert_str(0, "%40");
                    }

                    // Set atSignSeen to true.
                    self.at_sign_seen = true;

                    // For each codePoint in buffer:
                    for code_point in self.buffer.chars() {
                        // If codePoint is U+003A (:) and passwordTokenSeen is false
                        if code_point == ':' && !self.password_token_seen {
                            // then set passwordTokenSeen to true and continue.
                            self.password_token_seen = true;
                            continue;
                        }

                        // NOTE: We group the following two steps together to avoid unnecessary
                        // allocations
                        // Let encodedCodePoints be the result of running
                        // UTF-8 percent-encode codePoint using
                        // the userinfo percent-encode set.

                        // If passwordTokenSeen is true
                        let append_to = if self.password_token_seen {
                            // then append encodedCodePoints to url’s password.
                            &mut self.url.password
                        } else {
                            // Otherwise, append encodedCodePoints to url’s username.
                            &mut self.url.username
                        };

                        let mut buffer = [0; 4];
                        code_point.encode_utf8(&mut buffer);
                        percent_encode(
                            &buffer[..code_point.len_utf8()],
                            is_userinfo_percent_encode_set,
                            append_to,
                        );
                    }

                    // Set buffer to the empty string.
                    self.buffer.clear();
                }
                // Otherwise, if one of the following is true:
                // * c is the EOF code point, U+002F (/), U+003F (?), or U+0023 (#)
                // * url is special and c is U+005C (\)
                else if matches!(c, None | Some('/' | '?' | '#'))
                    || (self.url.is_special() && c == Some('\\'))
                {
                    // If atSignSeen is true and buffer is the empty string
                    if self.at_sign_seen && self.buffer.is_empty() {
                        // Invalid-credentials validation error.
                        self.error_handler
                            .validation_error(ValidationError::InvalidCredentials);

                        // return failure.
                        return Err(Error::Failure);
                    }

                    // Decrease pointer by the number of code points in buffer plus one,
                    self.input.go_back_n(self.buffer.chars().count() + 1);

                    // set buffer to the empty string,
                    self.buffer.clear();

                    // and set state to host state.
                    self.set_state(URLParserState::Host);
                }
                // Otherwise
                else {
                    // append c to buffer.
                    self.buffer
                        .push(c.expect("the previous step catches EOF codepoints"));
                }
            },
            // https://url.spec.whatwg.org/#host-state
            // https://url.spec.whatwg.org/#hostname-state
            URLParserState::Host | URLParserState::Hostname => {
                let c = self.input.current();
                // If state override is given and url’s scheme is "file"
                if self.state_override.is_some() && self.url.scheme() == "file" {
                    // then decrease pointer by 1
                    self.input.go_back();

                    // and set state to file host state.
                    self.set_state(URLParserState::FileHost);
                }
                // Otherwise, if c is U+003A (:) and insideBrackets is false
                else if c == Some(':') && !self.inside_brackets {
                    // If buffer is the empty string
                    if self.buffer.is_empty() {
                        // host-missing validation error
                        self.error_handler
                            .validation_error(ValidationError::HostMissing);

                        // return failure.
                        return Err(Error::Failure);
                    }

                    // If state override is given and state override is hostname state
                    if let Some(URLParserState::Hostname) = self.state_override {
                        // then return.
                        return Ok(StartOver::No);
                    }

                    // Let host be the result of host parsing buffer with url is not special.
                    let host_or_failure = host::host_parse_with_special(
                        self.buffer.as_str(),
                        true,
                        &mut self.error_handler,
                    );

                    // If host is failure, then return failure.
                    let host = host_or_failure?;

                    // Set url’s host to host,
                    self.url.host = Some(host);

                    // buffer to the empty string,
                    self.buffer.clear();

                    // and state to port state.
                    self.set_state(URLParserState::Port);
                }
                // Otherwise, if one of the following is true:
                // * c is the EOF code point, U+002F (/), U+003F (?), or U+0023 (#)
                // * url is special and c is U+005C (\)
                else if matches!(c, None | Some('/' | '?' | '#'))
                    || (self.url.is_special() && c == Some('\\'))
                {
                    // then decrease pointer by 1,
                    self.input.go_back();

                    // and then:
                    // If url is special and buffer is the empty string
                    if self.url.is_special() && self.buffer.is_empty() {
                        // host-missing validation error
                        self.error_handler
                            .validation_error(ValidationError::HostMissing);

                        // return failure.
                        return Err(Error::Failure);
                    }
                    // Otherwise, if state override is given, buffer is the empty string,
                    // and either url includes credentials or url’s port is non-null
                    else if self.state_override.is_some()
                        && self.buffer.is_empty()
                        && (self.url.includes_credentials() || self.url.port.is_some())
                    {
                        // return.
                        return Ok(StartOver::No);
                    }

                    // Let host be the result of host parsing buffer with url is not special.
                    let host_or_failure = host::host_parse_with_special(
                        self.buffer.as_str(),
                        true,
                        &mut self.error_handler,
                    );

                    // If host is failure, then return failure.
                    let host = host_or_failure?;

                    // Set url’s host to host,
                    self.url.host = Some(host);

                    // buffer to the empty string,
                    self.buffer.clear();

                    // and state to path start state.
                    self.set_state(URLParserState::PathStart);

                    // If state override is given
                    if self.state_override.is_some() {
                        // then return.
                        return Ok(StartOver::No);
                    }
                }
                // Otherwise:
                else {
                    // If c is U+005B ([),
                    if c == Some('[') {
                        // then set insideBrackets to true.
                        self.inside_brackets = true;
                    }
                    // If c is U+005D (])
                    else if c == Some(']') {
                        // then set insideBrackets to false.
                        self.inside_brackets = false;
                    }

                    // Append c to buffer.
                    self.buffer
                        .push(c.expect("the previous step checks for EOF codepoints"));
                }
            },
            // https://url.spec.whatwg.org/#port-state
            URLParserState::Port => {
                let c = self.input.current();

                // If c is an ASCII digit
                if let Some(ascii_digit) = c.filter(char::is_ascii_digit) {
                    // append c to buffer.
                    self.buffer.push(ascii_digit);
                }
                // Otherwise, if one of the following is true:
                // * c is the EOF code point, U+002F (/), U+003F (?), or U+0023 (#)
                // * url is special and c is U+005C (\)
                // * state override is given
                else if matches!(c, None | Some('/' | '?' | '#'))
                    || (self.url.is_special() && c == Some('\\'))
                    || (self.state_override.is_some())
                {
                    // If buffer is not the empty string, then:
                    if !self.buffer.is_empty() {
                        // Let port be the mathematical integer value that is
                        // represented by buffer in radix-10 using ASCII digits
                        // for digits with values 0 through 9.

                        // If port is greater than 2^16 − 1
                        let port = match str::parse(&self.buffer) {
                            Ok(port) => port,
                            Err(_) => {
                                // port-out-of-range validation error
                                self.error_handler
                                    .validation_error(ValidationError::PortOutOfRange);

                                // return failure.
                                return Err(Error::Failure);
                            },
                        };

                        // Set url’s port to null, if port is url’s scheme’s default port; otherwise to port.
                        if default_port_for_scheme(&self.url.scheme()) == Some(port) {
                            self.url.port = None;
                        } else {
                            self.url.port = Some(port);
                        }

                        // Set buffer to the empty string.
                        self.buffer.clear();
                    }

                    // If state override is given
                    if self.state_override.is_some() {
                        // then return.
                        return Ok(StartOver::No);
                    }

                    // Set state to path start state
                    self.set_state(URLParserState::PathStart);

                    // and decrease pointer by 1.
                    self.input.go_back();
                }
                // Otherwise
                else {
                    // port-invalid validation error
                    self.error_handler
                        .validation_error(ValidationError::PortInvalid);

                    // return failure.
                    return Err(Error::Failure);
                }
            },
            // https://url.spec.whatwg.org/#file-state
            URLParserState::File => {
                // Set url’s scheme to "file".
                self.url.serialization = "file".try_into().expect("\"file\" is valid ascii");
                self.url.scheme_end = self.url.serialization.len();

                // Set url’s host to the empty string.
                self.url.host_start = self.url.serialization.len();
                self.url.username_start = self.url.serialization.len();
                self.url.self.url.host = Some(Host::OpaqueHost(ascii::String::default()));

                // If c is U+002F (/) or U+005C (\), then:
                let c = self.input.current();
                if matches!(c, Some('/' | '\\')) {
                    // If c is U+005C (\),
                    if c == Some('\\') {
                        // invalid-reverse-solidus validation error.
                        self.error_handler
                            .validation_error(ValidationError::InvalidReverseSolidus);
                    }

                    // Set state to file slash state.
                    self.set_state(URLParserState::FileSlash);
                }
                // Otherwise, if base is non-null and base’s scheme is "file":
                else if let Some(base) = &self.base
                    && base.scheme() == "file"
                {
                    // Set url’s host to base’s host,
                    self.url.host.clone_from(&base.host);

                    // url’s path to a clone of base’s path,
                    self.url.path.clone_from(&base.path);

                    // and url’s query to base’s query.
                    self.url.query.clone_from(&base.query);

                    // If c is U+003F (?)
                    if c == Some('?') {
                        // then set url’s query to the empty string
                        self.url.serialization.push(ascii::Char::QuestionMark);
                        self.url.query_start = self.url.serialization.len();

                        // and state to query state.
                        self.set_state(URLParserState::Query);
                    }
                    // Otherwise, if c is U+0023 (#)
                    else if c == Some('#') {
                        // set url’s fragment to the empty string
                        self.url.serialization.push(ascii::Char::NumberSign);
                        self.url.fragment_start = self.url.serialization.len();

                        // and state to fragment state.
                        self.set_state(URLParserState::Fragment);
                    }
                    // Otherwise, if c is not the EOF code point:
                    else if c.is_some() {
                        // Set url’s query to null.
                        self.url.query = None;

                        // If the code point substring from pointer to the end of input
                        // does not start with a Windows drive letter,
                        if !util::starts_with_windows_drive_letter(self.input.remaining()) {
                            // then shorten url’s path.
                            self.url.shorten_path();
                        }
                        // Otherwise:
                        else {
                            // File-invalid-Windows-drive-letter validation error.
                            self.error_handler
                                .validation_error(ValidationError::FileInvalidWindowsDriveLetter);

                            // Set url’s path to an empty list.
                            self.url.serialization.truncate(self.url.path_start);
                        }

                        // Set state to path state
                        self.set_state(URLParserState::Path);

                        // and decrease pointer by 1.
                        self.input.go_back();
                    }
                }
                // Otherwise
                else {
                    // set state to path state,
                    self.set_state(URLParserState::Path);

                    // and decrease pointer by 1.
                    self.input.go_back();
                }
            },
            // https://url.spec.whatwg.org/#file-slash-state
            URLParserState::FileSlash => {
                // If c is U+002F (/) or U+005C (\), then:
                let c = self.input.current();
                if matches!(c, Some('/' | '\\')) {
                    // If c is U+005C (\)
                    if c == Some('\\') {
                        // invalid-reverse-solidus validation error.
                        self.error_handler
                            .validation_error(ValidationError::InvalidReverseSolidus);
                    }

                    // Set state to file host state.
                    self.set_state(URLParserState::FileHost);
                }
                // Otherwise:
                else {
                    // If base is non-null and base’s scheme is "file", then:
                    if let Some(base) = &self.base
                        && base.scheme() == "file"
                    {
                        // Set url’s host to base’s host.
                        self.url.host.clone_from(&base.host);

                        // If the code point substring from pointer to the end of input
                        // does not start with a Windows drive letter
                        // and base’s path[0] is a normalized Windows drive letter
                        if util::starts_with_windows_drive_letter(self.input.remaining())
                            && util::is_normalized_windows_drive_letter(base.path[0].as_str())
                        {
                            // then append base’s path[0] to url’s path.
                            self.url.path.push(base.path[0].clone());
                        }
                    }

                    // Set state to path state,
                    self.set_state(URLParserState::Path);

                    // and decrease pointer by 1.
                    self.input.go_back();
                }
            },
            // https://url.spec.whatwg.org/#file-host-state
            URLParserState::FileHost => {
                match self.input.current() {
                    // If c is the EOF code point, U+002F (/), U+005C (\), U+003F (?), or U+0023 (#)
                    None | Some('/' | '\\' | '?' | '#') => {
                        // then decrease pointer by 1 and then:
                        self.input.go_back();

                        // If state override is not given and buffer is a Windows drive letter
                        if self.state_override.is_none()
                            && util::is_windows_drive_letter(&self.buffer)
                        {
                            // file-invalid-Windows-drive-letter-host validation error
                            self.error_handler.validation_error(
                                ValidationError::FileInvalidWindowsDriveLetterHost,
                            );

                            // set state to path state.
                            self.set_state(URLParserState::Path);
                        }
                        // Otherwise, if buffer is the empty string, then:
                        else if self.buffer.is_empty() {
                            // Set url’s host to the empty string.
                            self.url.host = Some(Host::OpaqueHost(ascii::String::default()));

                            // If state override is given,
                            if self.state_override.is_some() {
                                // then return.
                                return Ok(StartOver::No);
                            }

                            // Set state to path start state.
                            self.set_state(URLParserState::PathStart);
                        }
                        // Otherwise, run these steps:
                        else {
                            // Let host be the result of host parsing buffer with url is not special.
                            // If host is failure, then return failure.
                            let mut host = host_parse_with_special(
                                &self.buffer,
                                false,
                                &mut self.error_handler,
                            )?;

                            // If host is "localhost", then set host to the empty string.
                            if let Host::Domain(domain) = &host
                                && domain.as_str() == "localhost"
                            {
                                host = Host::OpaqueHost(ascii::String::default());
                            }

                            // Set url’s host to host.
                            self.url.host = Some(host);

                            // If state override is given,
                            if self.state_override.is_some() {
                                // then return.
                                return Ok(StartOver::No);
                            }

                            // Set buffer to the empty string
                            self.buffer.clear();

                            // and state to path start state.
                            self.set_state(URLParserState::PathStart);
                        }
                    },
                    // Otherwise, append c to buffer.
                    Some(c) => self.buffer.push(c),
                }
            },

            // https://url.spec.whatwg.org/#path-start-state
            URLParserState::PathStart => {
                let c = self.input.current();

                // If url is special, then:
                if self.url.is_special() {
                    // If c is U+005C (\),
                    if c == Some('\\') {
                        // invalid-reverse-solidus validation error.
                        self.error_handler
                            .validation_error(ValidationError::InvalidReverseSolidus);
                    }

                    // Set state to path state.
                    self.set_state(URLParserState::Path);

                    // If c is neither U+002F (/) nor U+005C (\)
                    if !matches!(c, Some('/' | '\\')) {
                        // then decrease pointer by 1.
                        self.input.go_back();
                    }
                }
                // Otherwise, if state override is not given and c is U+003F (?)
                else if self.state_override.is_none() && c == Some('?') {
                    // set url’s query to the empty string
                    self.url.serialization.push(ascii::Char::QuestionMark);
                    self.url.query_start = self.url.serialization.len();

                    // and state to query state.
                    self.set_state(URLParserState::Query);
                }
                // Otherwise, if state override is not given and c is U+0023 (#)
                else if self.state_override.is_none() && c == Some('#') {
                    // set url’s fragment to the empty string
                    self.url.serialization.push(ascii::Char::NumberSign);
                    self.url.fragment_start = self.url.serialization.len();

                    // and state to fragment state.
                    self.set_state(URLParserState::Fragment);
                }
                // Otherwise, if c is not the EOF code point:
                else if c.is_some() {
                    // Set state to path state.
                    self.set_state(URLParserState::Path);

                    // If c is not U+002F (/),
                    if c != Some('/') {
                        // then decrease pointer by 1.
                        self.input.go_back();
                    }
                }
                // Otherwise, if state override is given and url’s host is null,
                else if self.state_override.is_some() && self.url.host.is_none() {
                    // append the empty string to url’s path.
                    self.url.serialization.push(ascii::Char::Solidus);
                }
            },
            // https://url.spec.whatwg.org/#path-state
            URLParserState::Path => {
                let c = self.input.current();

                // If one of the following is true:
                // * c is the EOF code point or U+002F (/)
                // * url is special and c is U+005C (\)
                // * state override is not given and c is U+003F (?) or U+0023 (#)
                if matches!(c, None | Some('/'))
                    || (self.url.is_special() && c == Some('\\'))
                    || (self.state_override.is_none() && matches!(c, Some('?' | '#')))
                {
                    // If url is special and c is U+005C (\)
                    if self.url.is_special() && c == Some('\\') {
                        // invalid-reverse-solidus validation error.
                        self.error_handler
                            .validation_error(ValidationError::InvalidReverseSolidus);
                    }

                    // If buffer is a double-dot path segment, then:
                    if util::is_double_dot_path_segment(&self.buffer) {
                        // Shorten url’s path.
                        self.url.shorten_path();

                        // If neither c is U+002F (/), nor url is special and c is U+005C (\)
                        if c != Some('/') && !(self.url.is_special() && c == Some('\\')) {
                            // append the empty string to url’s path.
                            self.url.serialization.push(ascii::Char::Solidus)
                        }
                    }
                    // Otherwise, if buffer is a single-dot path segment
                    // and if neither c is U+002F (/), nor url is special and c is U+005C (\)
                    else if util::is_single_dot_path_segment(&self.buffer)
                        && c != Some('/')
                        && !(self.url.is_special() && c == Some('\\'))
                    {
                        // append the empty string to url’s path.
                        self.url.serialization.push(ascii::Char::Solidus)
                    }
                    // Otherwise, if buffer is not a single-dot path segment, then:
                    else if !util::is_single_dot_path_segment(&self.buffer) {
                        // If url’s scheme is "file", url’s path is empty, and buffer is a Windows drive letter
                        if self.url.scheme() == "file"
                            && self.url.path.is_empty()
                            && util::is_windows_drive_letter(&self.buffer)
                        {
                            // then replace the second code point in buffer with U+003A (:).
                            // NOTE: The spec doesn't specify what to do if there is no second codepoint (maybe it can't happen?).
                            // If that happens, we simply do nothing.
                            if let Some(range) = self
                                .buffer
                                .char_indices()
                                .nth(2)
                                .map(|(pos, ch)| (pos..pos + ch.len_utf8()))
                            {
                                self.buffer.replace_range(range, ":");
                            }
                        }

                        // Append buffer to url’s path.
                        self.url.path.push(
                            ascii::String::try_from(self.buffer.as_str())
                                .expect("buffer cannot contain non-ascii data during path state"),
                        );
                    }

                    // Set buffer to the empty string.
                    self.buffer.clear();

                    // If c is U+003F (?)
                    if c == Some('?') {
                        // then set url’s query to the empty string
                        self.url.serialization.push(ascii::Char::QuestionMark);
                        self.url.query_start = self.url.serialization.len();

                        // and state to query state.
                        self.set_state(URLParserState::Query);
                    }

                    // If c is U+0023 (#)
                    if c == Some('#') {
                        // then set url’s fragment to the empty string
                        self.url.query_start = self.url.serialization.len();
                        self.url.serialization.push(ascii::Char::NumberSign);
                        self.url.fragment_start = self.url.serialization.len();

                        // and state to fragment state.
                        self.set_state(URLParserState::Fragment);
                    }
                }
                // Otherwise, run these steps:
                else {
                    let c = c.expect("The previous step checks for EOF code points");

                    // If c is not a URL code point and not U+0025 (%),
                    if !util::is_url_codepoint(c) && c != '%' {
                        // invalid-URL-unit validation error.
                        self.error_handler
                            .validation_error(ValidationError::InvalidURLUnit);
                    }

                    // If c is U+0025 (%) and remaining does not start with two ASCII hex digits
                    // NOTE: technically this check is incorrect as remaining() could have less than two characters
                    //       But there's probably more important issues to worry about...
                    if c == '%'
                        && self
                            .input
                            .remaining()
                            .chars()
                            .take(2)
                            .all(|c| c.is_ascii_hexdigit())
                    {
                        // invalid-URL-unit validation error.
                        self.error_handler
                            .validation_error(ValidationError::InvalidURLUnit);
                    }

                    // UTF-8 percent-encode c using the path percent-encode set and append the result to buffer.
                    let mut buffer = [0; 4];
                    c.encode_utf8(&mut buffer);
                    percent_encode(
                        &buffer[..c.len_utf8()],
                        is_path_percent_encode_set,
                        &mut self.buffer,
                    );
                }
            },
        }
        Ok(StartOver::No)
    }

    /// [Specification](https://url.spec.whatwg.org/#shorten-a-urls-path)
    ///
    /// This implementation assumes that no query/fragment exist (yet)
    pub(crate) fn shorten_path(&mut self) {
        debug_assert!(!self.url.has_opaque_path());

        // If path's size is not 1 then it contains and therefore cannot be a windows drive letter
        if self.url.scheme() == "file"
            && util::is_normalized_windows_drive_letter(self.url.path().as_str())
        {
            return;
        }

        if self.url.path_start == self.url.serialization.len() {
            return; // Path is empty
        }

        if let Some(last_path_segment) = self.url.serialization.rfind(ascii::Char::Solidus) {
            self.url.serialization.truncate(last_path_segment)
        }
    }
}

impl From<HostParseError> for Error {
    fn from(value: HostParseError) -> Self {
        Self::HostParse(value)
    }
}
