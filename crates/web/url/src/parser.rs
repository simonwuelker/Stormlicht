use sl_std::{
    ascii,
    chars::{ReversibleCharIterator, State},
};

use crate::{
    default_port_for_scheme,
    host::{self, host_parse_with_special, Host},
    is_special_scheme,
    percent_encode::{
        is_c0_percent_encode_set, is_fragment_percent_encode_set, is_path_percent_encode_set,
        is_query_percent_encode_set, is_special_query_percent_encode_set,
        is_userinfo_percent_encode_set, percent_encode,
    },
    util, ValidationError, ValidationErrorHandler, URL,
};

#[derive(Clone, Copy, Debug)]
pub struct Failure;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StartOver {
    Yes,
    No,
}

impl<'a, H> URLParser<'a, H>
where
    H: ValidationErrorHandler,
{
    pub(crate) fn run_to_completion(mut self) -> Result<Self, Failure> {
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

    fn step(&mut self) -> Result<StartOver, Failure> {
        match self.state {
            // https://url.spec.whatwg.org/#scheme-start-state
            URLParserState::SchemeStart => {
                // If c is an ASCII alpha,
                if let Some(c) = self.input.current()
                    && c.is_ascii_alphabetic()
                {
                    // Append c, lowercased, to buffer,
                    self.buffer.push(c.to_ascii_lowercase());

                    // and set state to scheme state.
                    self.set_state(URLParserState::Scheme);
                }
                // Otherwise, if state override is not given
                else if self.state_override.is_none() {
                    // Set state to no scheme state
                    self.set_state(URLParserState::NoScheme);

                    // and decrease pointer by 1.
                    self.input.go_back();
                }
                // Otherwise,
                else {
                    // return failure.
                    return Err(Failure);
                }
            },
            // https://url.spec.whatwg.org/#scheme-state
            URLParserState::Scheme => {
                let c = self.input.current();

                // If c is an ASCII alphanumeric, U+002B (+), U+002D (-), or U+002E (.),
                if let Some(c) = c
                    && matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '+' | '-' | '.')
                {
                    // Append c, lowercased, to buffer
                    self.buffer.push(c.to_ascii_lowercase());
                }
                // Otherwise, if c is U+003A (:), then:
                else if c == Some(':') {
                    // If state override is given, then:
                    if self.state_override.is_some() {
                        // If url’s scheme is a special scheme and buffer is not a special scheme
                        if self.url.is_special() && !is_special_scheme(&self.buffer) {
                            // then return.
                            return Ok(StartOver::No);
                        }
                        // If url’s scheme is not a special scheme and buffer is a special scheme
                        if !&self.url.is_special() && is_special_scheme(&self.buffer) {
                            // then return.
                            return Ok(StartOver::No);
                        }

                        // If url includes credentials or has a non-null port, and buffer is "file"
                        if (self.url.includes_credentials() || self.url.port().is_some())
                            && self.buffer == "file"
                        {
                            // then return.
                            return Ok(StartOver::No);
                        }

                        // If url’s scheme is "file" and its host is an empty host
                        if self.url.scheme == "file" && self.url.host == Some(Host::EmptyHost) {
                            // then return.
                            return Ok(StartOver::No);
                        }
                    }

                    // Set url’s scheme to buffer.
                    self.url.scheme = ascii::String::try_from(self.buffer.as_str())
                        .expect("buffer cannot contain non-ascii data during scheme state");

                    // If state override is given, then:
                    if self.state_override.is_some() {
                        // If url’s port is url’s scheme’s default port
                        if self.url.port == default_port_for_scheme(&self.url.scheme) {
                            // then set url’s port to null.
                            self.url.port = None;
                        }

                        // Return.
                        return Ok(StartOver::No);
                    }

                    // Set buffer to the empty string.
                    self.buffer.clear();

                    // If url’s scheme is "file", then:
                    if self.url.scheme == "file" {
                        // If remaining does not start with "//"
                        if !self.input.remaining().starts_with("//") {
                            // special-scheme-missing-following-solidus validation error.
                            self.error_handler.validation_error(
                                ValidationError::SpecialSchemeMissingFollowingSolidus,
                            );
                        }

                        // Set state to file state.
                        self.set_state(URLParserState::File);
                    }
                    // Otherwise, if url is special, base is non-null, and base’s scheme is url’s scheme:
                    else if self.url.is_special()
                        && self
                            .base
                            .as_ref()
                            .map(URL::scheme)
                            .is_some_and(|scheme| scheme == self.url.scheme())
                    {
                        // Assert: base is is special (and therefore does not have an opaque path).
                        assert!(self.base.as_ref().is_some_and(URL::is_special));

                        // Set state to special relative or authority state.
                        self.set_state(URLParserState::SpecialRelativeOrAuthority);
                    }
                    // Otherwise, if url is special
                    else if self.url.is_special() {
                        // set state to special authority slashes state.
                        self.set_state(URLParserState::SpecialAuthoritySlashes);
                    }
                    // Otherwise, if remaining starts with an U+002F (/)
                    else if self.input.remaining().starts_with('/') {
                        // set state to path or authority state and increase pointer by 1.
                        self.set_state(URLParserState::PathOrAuthority);
                        self.input.next();
                    }
                    // Otherwise,
                    else {
                        // set url’s path to the empty string
                        self.url.path = vec![ascii::String::new()];

                        // and set state to opaque path state.
                        self.set_state(URLParserState::OpaquePath);
                    }
                }
                // Otherwise, if state override is not given
                else if self.state_override.is_none() {
                    // set buffer to the empty string,
                    self.buffer.clear();

                    // state to no scheme state,
                    self.set_state(URLParserState::NoScheme);

                    // and start over (from the first code point in input).
                    return Ok(StartOver::Yes);
                }
                // Otherwise,
                else {
                    // return failure.
                    return Err(Failure);
                }
            },
            // https://url.spec.whatwg.org/#no-scheme-state
            URLParserState::NoScheme => {
                let c = self.input.current();

                // If base is null, or base has an opaque path and c is not U+0023 (#),
                if self.base.is_none()
                    || (self.base.as_ref().is_some_and(URL::has_opaque_path) && c != Some('#'))
                {
                    // missing-scheme-non-relative-URL validation error
                    self.error_handler
                        .validation_error(ValidationError::MissingSchemeNonRelativeURL);

                    // return failure.
                    return Err(Failure);
                }
                let base = self
                    .base
                    .as_ref()
                    .expect("if base is none then we returned a validation error earlier");

                // Otherwise, if base has an opaque path and c is U+0023 (#)
                if base.has_opaque_path() && c == Some('#') {
                    // set url’s scheme to base’s scheme,
                    self.url.scheme = base.scheme.clone();

                    // url’s path to base’s path,
                    self.url.path = base.path.clone();

                    // url’s query to base’s query,
                    self.url.query = base.query.clone();

                    // url’s fragment to the empty string,
                    self.url.fragment = Some(ascii::String::new());

                    // and set state to fragment state.
                    self.set_state(URLParserState::Fragment);
                }
                // Otherwise, if base’s scheme is not "file"
                else if base.scheme != "file" {
                    // set state to relative state
                    self.set_state(URLParserState::Relative);

                    // and decrease pointer by 1.
                    self.input.go_back();
                }
                // Otherwise,
                else {
                    // set state to file state
                    self.set_state(URLParserState::File);

                    // and decrease pointer by 1.
                    self.input.go_back();
                }
            },
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
                    Some(url) if url.scheme != "file" => url,
                    _ => panic!("base must exist and have a scheme other than none"),
                };

                // Set url’s scheme to base’s scheme.
                self.url.scheme = base.scheme.clone();

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
                    self.url.username = base.username.clone();

                    // url’s password to base’s password
                    self.url.password = base.password.clone();

                    // url’s host to base’s host
                    self.url.host = base.host.clone();

                    // url’s port to base’s port
                    self.url.port = base.port;

                    // url’s path to a clone of base’s path
                    self.url.path = base.path.clone();

                    // and url’s query to base’s query.
                    self.url.query = base.query.clone();

                    // If c is U+003F (?)
                    if c == Some('?') {
                        // then set url’s query to the empty string,
                        self.url.query = Some(ascii::String::new());

                        // and state to query state.
                        self.set_state(URLParserState::Query);
                    }
                    // Otherwise, if c is U+0023 (#)
                    else if c == Some('#') {
                        // set url’s fragment to the empty string
                        self.url.fragment = Some(ascii::String::new());

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
                    self.url.username = base.username.clone();

                    // url’s password to base’s password,
                    self.url.password = base.password.clone();

                    // url’s host to base’s host,
                    self.url.host = base.host.clone();

                    // url’s port to base’s port,
                    self.url.port = base.port;

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
                        return Err(Failure);
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
                if self.state_override.is_some() && self.url.scheme == "file" {
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
                        return Err(Failure);
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
                    let host = host_or_failure.map_err(|_| Failure)?;

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
                        return Err(Failure);
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
                    let host = host_or_failure.map_err(|_| Failure)?;

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
                                return Err(Failure);
                            },
                        };

                        // Set url’s port to null, if port is url’s scheme’s default port; otherwise to port.
                        if default_port_for_scheme(&self.url.scheme) == Some(port) {
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
                    return Err(Failure);
                }
            },
            // https://url.spec.whatwg.org/#file-state
            URLParserState::File => {
                // Set url’s scheme to "file".
                self.url.scheme = "file"
                    .to_string()
                    .try_into()
                    .expect("\"file\" is valid ascii");

                // Set url’s host to the empty string.
                self.url.host = Some(Host::OpaqueHost(ascii::String::default()));

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
                    && base.scheme == "file"
                {
                    // Set url’s host to base’s host,
                    self.url.host = base.host.clone();

                    // url’s path to a clone of base’s path,
                    self.url.path = base.path.clone();

                    // and url’s query to base’s query.
                    self.url.query = base.query.clone();

                    // If c is U+003F (?)
                    if c == Some('?') {
                        // then set url’s query to the empty string
                        self.url.query = Some(ascii::String::new());

                        // and state to query state.
                        self.set_state(URLParserState::Query);
                    }
                    // Otherwise, if c is U+0023 (#)
                    else if c == Some('#') {
                        // set url’s fragment to the empty string
                        self.url.fragment = Some(ascii::String::new());

                        // and state to fragment state.
                        self.set_state(URLParserState::Fragment);
                    }
                    // Otherwise, if c is not the EOF code point:
                    else if c.is_some() {
                        // Set url’s query to null.
                        self.url.query = None;
                    }

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
                        self.url.path = vec![];
                    }

                    // Set state to path state
                    self.set_state(URLParserState::Path);

                    // and decrease pointer by 1.
                    self.input.go_back();
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
                        && base.scheme == "file"
                    {
                        // Set url’s host to base’s host.
                        self.url.host = base.host.clone();

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
                            )
                            .map_err(|_| Failure)?;

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
                    self.url.query = Some(ascii::String::new());

                    // and state to query state.
                    self.set_state(URLParserState::Query);
                }
                // Otherwise, if state override is not given and c is U+0023 (#)
                else if self.state_override.is_none() && c == Some('#') {
                    // set url’s fragment to the empty string
                    self.url.fragment = Some(ascii::String::new());

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
                    self.url.path.push(ascii::String::new());
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
                            self.url.path.push(ascii::String::new());
                        }
                    }
                    // Otherwise, if buffer is a single-dot path segment
                    // and if neither c is U+002F (/), nor url is special and c is U+005C (\)
                    else if util::is_single_dot_path_segment(&self.buffer)
                        && c != Some('/')
                        && !(self.url.is_special() && c == Some('\\'))
                    {
                        // append the empty string to url’s path.
                        self.url.path.push(ascii::String::new());
                    }
                    // Otherwise, if buffer is not a single-dot path segment, then:
                    else if !util::is_single_dot_path_segment(&self.buffer) {
                        // If url’s scheme is "file", url’s path is empty, and buffer is a Windows drive letter
                        if self.url.scheme == "file"
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
                        self.url.query = Some(ascii::String::new());

                        // and state to query state.
                        self.set_state(URLParserState::Query);
                    }

                    // If c is U+0023 (#)
                    if c == Some('#') {
                        // then set url’s fragment to the empty string
                        self.url.fragment = Some(ascii::String::new());

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
            // https://url.spec.whatwg.org/#cannot-be-a-base-url-path-state
            URLParserState::OpaquePath => {
                let c = self.input.current();

                // If c is U+003F (?)
                if c == Some('?') {
                    //  then set url’s query to the empty string
                    self.url.query = Some(ascii::String::new());

                    // and state to query state.
                    self.set_state(URLParserState::Query);
                }
                // Otherwise, if c is U+0023 (#)
                else if c == Some('#') {
                    // then set url’s fragment to the empty string
                    self.url.fragment = Some(ascii::String::new());

                    // and state to fragment state.
                    self.set_state(URLParserState::Fragment);
                }
                // Otherwise:
                else {
                    // If c is not the EOF code point, not a URL code point, and not U+0025 (%)
                    if c.is_some() && !c.is_some_and(|c| c == '%' || util::is_url_codepoint(c)) {
                        // invalid-URL-unit validation error.
                        self.error_handler
                            .validation_error(ValidationError::InvalidURLUnit);
                    }

                    // If c is U+0025 (%) and remaining does not start with two ASCII hex digits
                    // NOTE: technically this check is incorrect as remaining() could have less than two characters
                    //       But there's probably more important issues to worry about...
                    if c == Some('%')
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

                    // If c is not the EOF code point
                    if let Some(c) = c {
                        //  UTF-8 percent-encode c using the C0 control percent-encode set and append the result to url’s path.
                        let mut buffer = [0; 4];
                        c.encode_utf8(&mut buffer);
                        percent_encode(
                            &buffer[..c.len_utf8()],
                            is_c0_percent_encode_set,
                            &mut self.url.path[0],
                        );
                    }
                }
            },
            // https://url.spec.whatwg.org/#query-state
            URLParserState::Query => {
                // If encoding is not UTF-8 and one of the following is true:
                // * url is not special
                // * url’s scheme is "ws" or "wss"
                // NOTE: We don't support non-utf8 encoding

                // If one of the following is true:
                // * state override is not given and c is U+0023 (#)
                // * c is the EOF code point
                let c = self.input.current();
                if (self.state_override.is_none() && c == Some('#')) || c.is_none() {
                    // Let queryPercentEncodeSet be the special-query percent-encode set
                    // if url is special; otherwise the query percent-encode set.
                    let query_percent_encode_set = if self.url.is_special() {
                        is_special_query_percent_encode_set
                    } else {
                        is_query_percent_encode_set
                    };

                    // Percent-encode after encoding, with encoding, buffer, and queryPercentEncodeSet,
                    // and append the result to url’s query.
                    let query = self.url.query.get_or_insert_default();
                    percent_encode(self.buffer.as_bytes(), query_percent_encode_set, query);

                    // Set buffer to the empty string.
                    self.buffer.clear();

                    // If c is U+0023 (#),
                    if c == Some('#') {
                        // then set url’s fragment to the empty string
                        self.url.fragment = Some(ascii::String::new());

                        // and state to fragment state.
                        self.set_state(URLParserState::Fragment);
                    }
                }
                // Otherwise, if c is not the EOF code point:
                else if let Some(c) = c {
                    // If c is not a URL code point and not U+0025 (%)
                    if c != '%' && !util::is_url_codepoint(c) {
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

                    // Append c to buffer.
                    self.buffer.push(c)
                };
            },

            // https://url.spec.whatwg.org/#fragment-state
            URLParserState::Fragment => {
                // If c is not the EOF code point, then:
                if let Some(c) = self.input.current() {
                    // If c is not a URL code point and not U+0025 (%)
                    if c != '%' && !util::is_url_codepoint(c) {
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

                    // UTF-8 percent-encode c using the fragment percent-encode set
                    // and append the result to url’s fragment.
                    let fragment = self.url.fragment.get_or_insert_default();

                    let mut buffer = [0; 4];
                    c.encode_utf8(&mut buffer);
                    percent_encode(
                        &buffer[..c.len_utf8()],
                        is_fragment_percent_encode_set,
                        fragment,
                    );
                }
            },
        }
        Ok(StartOver::No)
    }
}
