use crate::{
    host::{self, host_parse_with_special, Host},
    scheme_default_port, scheme_is_special,
    urlencode::{
        is_c0_percent_encode_set, is_fragment_percent_encode_set, is_path_percent_encode_set,
        is_query_percent_encode_set, is_special_query_percent_encode_set,
        is_userinfo_percent_encode_set, percent_encode, percent_encode_char,
    },
    util, URL,
};

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
    pub(crate) base: Option<URL>,
    pub(crate) input: &'a str,
    pub(crate) state: URLParserState,
    pub(crate) ptr: usize,
    pub(crate) buffer: String,
    pub(crate) at_sign_seen: bool,
    pub(crate) inside_brackets: bool,
    pub(crate) password_token_seen: bool,
    pub(crate) state_override: Option<URLParserState>,
}

// https://infra.spec.whatwg.org/#c0-control
pub(crate) fn is_c0_control(c: char) -> bool {
    matches!(c, '\u{0000}'..='\u{001F}')
}

impl<'a> URLParser<'a> {
    pub(crate) fn run_to_completion(mut self) -> Result<Self, ()> {
        loop {
            // Keep running the following state machine by switching on state.
            self.step()?;

            // If after a run pointer points to the EOF code point
            if self.c().is_none() {
                // go to the next step
                break;
            }
            // Otherwise,
            else {
                // increase pointer by 1 and continue with the state machine.
                self.ptr += 1;
            }
        }
        Ok(self)
    }

    fn c(&self) -> Option<char> {
        self.input.chars().nth(self.ptr)
    }

    fn remaining(&self) -> &'a str {
        &self.input[self.ptr + 1..]
    }

    fn set_state(&mut self, new_state: URLParserState) {
        self.state = new_state;
    }

    fn step(&mut self) -> Result<(), ()> {
        match self.state {
            // https://url.spec.whatwg.org/#scheme-start-state
            URLParserState::SchemeStart => {
                // If c is an ASCII alpha,
                if self.c().is_some() && self.c().unwrap().is_ascii_alphabetic() {
                    // Append c, lowercased, to buffer,
                    self.buffer.push(self.c().unwrap().to_ascii_lowercase());

                    // and set state to scheme state.
                    self.set_state(URLParserState::Scheme);
                }
                // Otherwise, if state override is not given
                else if self.state_override.is_none() {
                    // Set state to no scheme state
                    self.set_state(URLParserState::NoScheme);

                    // and decrease pointer by 1.
                    self.ptr -= 1;
                }
                // Otherwise,
                else {
                    // validation error, return failure.
                    return Err(());
                }
            },
            // https://url.spec.whatwg.org/#scheme-state
            URLParserState::Scheme => {
                let c = self.c();

                // If c is an ASCII alphanumeric, U+002B (+), U+002D (-), or U+002E (.),
                if matches!(c, Some('a'..='z' | 'A'..='Z' | '0'..='9' | '+' | '-' | '.')) {
                    // Append c, lowercased, to buffer
                    self.buffer.push(c.unwrap().to_ascii_lowercase());
                }
                // Otherwise, if c is U+003A (:), then:
                else if c == Some(':') {
                    // If state override is given, then:
                    if self.state_override.is_some() {
                        // If url’s scheme is a special scheme and buffer is not a special scheme
                        if scheme_is_special(&self.url.scheme) && !scheme_is_special(&self.buffer) {
                            // then return.
                            return Ok(());
                        }
                        // If url’s scheme is not a special scheme and buffer is a special scheme
                        if !scheme_is_special(&self.url.scheme) && scheme_is_special(&self.buffer) {
                            // then return.
                            return Ok(());
                        }

                        // If url includes credentials or has a non-null port, and buffer is "file"
                        if (self.url.includes_credentials() || self.url.port.is_some())
                            && self.buffer == "file"
                        {
                            // then return.
                            return Ok(());
                        }

                        // If url’s scheme is "file" and its host is an empty host
                        if self.url.scheme == "file" && self.url.host == Some(Host::EmptyHost) {
                            // then return.
                            return Ok(());
                        }
                    }

                    // Set url’s scheme to buffer.
                    self.url.scheme = self.buffer.clone();

                    // If state override is given, then:
                    if self.state_override.is_some() {
                        // If url’s port is url’s scheme’s default port
                        if self.url.port == scheme_default_port(&self.url.scheme) {
                            // then set url’s port to null.
                            self.url.port = None;
                        }

                        // Return.
                        return Ok(());
                    }

                    // Set buffer to the empty string.
                    self.buffer.clear();

                    // If url’s scheme is "file", then:
                    if self.url.scheme == "file" {
                        // If remaining does not start with "//"
                        if !self.remaining().starts_with("//") {
                            // validation error.
                            return Err(());
                        }

                        // Set state to file state.
                        self.set_state(URLParserState::File);
                    }
                    // Otherwise, if url is special, base is non-null, and base’s scheme is url’s scheme:
                    else if self.url.is_special()
                        && self.base.is_some()
                        && self.base.as_ref().unwrap().scheme == self.url.scheme
                    {
                        // Assert: base is is special (and therefore does not have an opaque path).
                        assert!(self.base.as_ref().unwrap().is_special());

                        // Set state to special relative or authority state.
                        self.set_state(URLParserState::SpecialRelativeOrAuthority);
                    }
                    // Otherwise, if url is special
                    else if self.url.is_special() {
                        // set state to special authority slashes state.
                        self.set_state(URLParserState::SpecialAuthoritySlashes);
                    }
                    // Otherwise, if remaining starts with an U+002F (/)
                    else if self.remaining().starts_with('/') {
                        // set state to path or authority state and increase pointer by 1.
                        self.set_state(URLParserState::PathOrAuthority);
                        self.ptr += 1;
                    }
                    // Otherwise,
                    else {
                        // set url’s path to the empty string
                        self.url.path = vec![String::new()];

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
                    self.ptr = 0;
                }
                // Otherwise,
                else {
                    // validation error, return failure.
                    return Err(());
                }
            },
            // https://url.spec.whatwg.org/#no-scheme-state
            URLParserState::NoScheme => {
                // If base is null, or base has an opaque path and c is not U+0023 (#),
                if self.base.is_none()
                    || (self.base.as_ref().unwrap().has_opaque_path() && self.c() != Some('#'))
                {
                    // validation error, return failure.
                    return Err(());
                }
                let base = self.base.as_ref().unwrap();

                // Otherwise, if base has an opaque path and c is U+0023 (#)
                if base.has_opaque_path() && self.c() == Some('#') {
                    // set url’s scheme to base’s scheme,
                    self.url.scheme = base.scheme.clone();

                    // url’s path to base’s path,
                    self.url.path = base.path.clone();

                    // url’s query to base’s query,
                    self.url.query = base.query.clone();

                    // url’s fragment to the empty string,
                    self.url.fragment = Some(String::new());

                    // and set state to fragment state.
                    self.set_state(URLParserState::Fragment);
                }
                // Otherwise, if base’s scheme is not "file"
                else if base.scheme != "file" {
                    // set state to relative state
                    self.set_state(URLParserState::Relative);

                    // and decrease pointer by 1.
                    self.ptr -= 1;
                }
                // Otherwise,
                else {
                    // set state to file state
                    self.set_state(URLParserState::File);

                    // and decrease pointer by 1.
                    self.ptr -= 1;
                }
            },
            // https://url.spec.whatwg.org/#special-relative-or-authority-state
            URLParserState::SpecialRelativeOrAuthority => {
                // If c is U+002F (/) and remaining starts with U+002F (/)
                if self.c() == Some('/') && self.remaining().starts_with('/') {
                    // then set state to special authority ignore slashes state
                    self.set_state(URLParserState::SpecialAuthoritySlashes);

                    // and increase pointer by 1.
                    self.ptr += 1;
                }
                // Otherwise,
                else {
                    // validation error,
                    // set state to relative state
                    self.set_state(URLParserState::Relative);

                    // and decrease pointer by 1.
                    self.ptr -= 1;
                }
            },
            // https://url.spec.whatwg.org/#path-or-authority-state
            URLParserState::PathOrAuthority => {
                // if c is U+002F (/),
                if self.c() == Some('/') {
                    // then set state to authority state.
                    self.set_state(URLParserState::Authority);
                }
                // Otherwise,
                else {
                    // set state to path state,
                    self.set_state(URLParserState::Path);

                    // and decrease pointer by 1.
                    self.ptr -= 1;
                }
            },
            // https://url.spec.whatwg.org/#relative-state
            URLParserState::Relative => {
                // Assert: base’s scheme is not "file".
                assert!(self.base.is_some());
                let base = self.base.as_ref().unwrap();
                assert!(base.scheme != "file");

                // Set url’s scheme to base’s scheme.
                self.url.scheme = base.scheme.clone();

                // If c is U+002F (/)
                // clippy doesn't account for the validation error
                #[allow(clippy::if_same_then_else)]
                if self.c() == Some('/') {
                    // then set state to relative slash state.
                    self.set_state(URLParserState::RelativeSlash);
                }
                // Otherwise, if url is special and c is U+005C (\)
                else if self.url.is_special() && self.c() == Some('\\') {
                    // validation error
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
                    if self.c() == Some('?') {
                        // then set url’s query to the empty string,
                        self.url.query = Some(String::new());

                        // and state to query state.
                        self.set_state(URLParserState::Query);
                    }
                    // Otherwise, if c is U+0023 (#)
                    else if self.c() == Some('#') {
                        // set url’s fragment to the empty string
                        self.url.fragment = Some(String::new());

                        // and state to fragment state.
                        self.set_state(URLParserState::Fragment);
                    }
                }
            },
            // https://url.spec.whatwg.org/#relative-slash-state
            URLParserState::RelativeSlash => {
                // If url is special and c is U+002F (/) or U+005C (\), then:
                if self.url.is_special() && (self.c() == Some('/') || self.c() == Some('\\')) {
                    // If c is U+005C (\)
                    if self.c() == Some('\\') {
                        // validation error.
                    }

                    // Set state to special authority ignore slashes state.
                    self.set_state(URLParserState::SpecialAuthorityIgnoreSlashes);
                }
                // Otherwise, if c is U+002F (/)
                else if self.c() == Some('/') {
                    // then set state to authority state.
                    self.set_state(URLParserState::Authority);
                }
                // Otherwise
                else {
                    let base = self.base.as_ref().unwrap();
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
                    self.ptr -= 1;
                }
            },
            // https://url.spec.whatwg.org/#special-authority-slashes-state
            URLParserState::SpecialAuthoritySlashes => {
                // If c is U+002F (/) and remaining starts with U+002F (/)
                if self.c() == Some('/') && self.remaining().starts_with('/') {
                    // then set state to special authority ignore slashes state
                    self.set_state(URLParserState::SpecialAuthorityIgnoreSlashes);

                    // and increase pointer by 1.
                    self.ptr += 1;
                }
                // Otherwise
                else {
                    // validation error,
                    // set state to special authority ignore slashes state
                    self.set_state(URLParserState::SpecialAuthorityIgnoreSlashes);

                    // and decrease pointer by 1.
                    self.ptr -= 1;
                }
            },
            // https://url.spec.whatwg.org/#special-authority-ignore-slashes-state
            URLParserState::SpecialAuthorityIgnoreSlashes => {
                // If c is neither U+002F (/) nor U+005C (\)
                if self.c() != Some('/') && self.c() != Some('\\') {
                    // then set state to authority state
                    self.set_state(URLParserState::Authority);

                    // and decrease pointer by 1.
                    self.ptr -= 1;
                }
                // Otherwise
                else {
                    // validation error.
                }
            },
            // https://url.spec.whatwg.org/#authority-state
            URLParserState::Authority => {
                // If c is U+0040 (@), then:
                if self.c() == Some('@') {
                    // Validation error.
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

                        // Let encodedCodePoints be the result of running
                        // UTF-8 percent-encode codePoint using
                        // the userinfo percent-encode set.
                        let encoded_codepoints =
                            percent_encode_char(code_point, is_userinfo_percent_encode_set);

                        // If passwordTokenSeen is true
                        if self.password_token_seen {
                            // then append encodedCodePoints to url’s password.
                            self.url.password.push_str(encoded_codepoints.as_str());
                        }
                        // Otherwise
                        else {
                            // append encodedCodePoints to url’s username.
                            self.url.username.push_str(encoded_codepoints.as_str());
                        }
                    }

                    // Set buffer to the empty string.
                    self.buffer.clear();
                }
                // Otherwise, if one of the following is true:
                // * c is the EOF code point, U+002F (/), U+003F (?), or U+0023 (#)
                // * url is special and c is U+005C (\)
                else if (self.c().is_none() || self.c() == Some('/') || self.c() == Some('#'))
                    || (self.url.is_special() && self.c() == Some('\\'))
                {
                    // If atSignSeen is true and buffer is the empty string
                    if self.at_sign_seen && self.buffer.is_empty() {
                        // validation error,
                        // return failure.
                        return Err(());
                    }

                    // Decrease pointer by the number of code points in buffer plus one,
                    self.ptr -= self.buffer.chars().count() + 1;

                    // set buffer to the empty string,
                    self.buffer.clear();

                    // and set state to host state.
                    self.set_state(URLParserState::Host);
                }
                // Otherwise
                else {
                    // append c to buffer.
                    self.buffer.push(self.c().unwrap());
                }
            },
            // https://url.spec.whatwg.org/#host-state
            // https://url.spec.whatwg.org/#hostname-state
            URLParserState::Host | URLParserState::Hostname => {
                // If state override is given and url’s scheme is "file"
                if self.state_override.is_some() && self.url.scheme == "file" {
                    // then decrease pointer by 1
                    self.ptr -= 1;

                    // and set state to file host state.
                    self.set_state(URLParserState::FileHost);
                }
                // Otherwise, if c is U+003A (:) and insideBrackets is false
                else if self.c() == Some(':') && !self.inside_brackets {
                    // If buffer is the empty string
                    if self.buffer.is_empty() {
                        // validation error,
                        // return failure.
                        return Err(());
                    }

                    // If state override is given and state override is hostname state
                    if let Some(URLParserState::Hostname) = self.state_override {
                        // then return.
                        return Ok(());
                    }

                    // Let host be the result of host parsing buffer with url is not special.
                    let host_or_failure = host::host_parse_with_special(self.buffer.as_str(), true);

                    // If host is failure, then return failure.
                    let host = host_or_failure.map_err(|_| ())?; // FIXME: proper error handling

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
                else if (self.c().is_none()
                    || self.c() == Some('/')
                    || self.c() == Some('?')
                    || self.c() == Some('#'))
                    || (self.url.is_special() && self.c() == Some('\\'))
                {
                    // then decrease pointer by 1,
                    self.ptr -= 1;

                    // and then:
                    // If url is special and buffer is the empty string
                    if self.url.is_special() && self.buffer.is_empty() {
                        // validation error, return failure.
                        return Err(());
                    }
                    // Otherwise, if state override is given, buffer is the empty string,
                    // and either url includes credentials or url’s port is non-null
                    else if self.state_override.is_some()
                        && self.buffer.is_empty()
                        && (self.url.includes_credentials() || self.url.port.is_some())
                    {
                        //  return.
                        return Ok(());
                    }

                    // Let host be the result of host parsing buffer with url is not special.
                    let host_or_failure = host::host_parse_with_special(self.buffer.as_str(), true);

                    // If host is failure, then return failure.
                    let host = host_or_failure.map_err(|_| ())?; // FIXME: proper error handling

                    // Set url’s host to host,
                    self.url.host = Some(host);

                    // buffer to the empty string,
                    self.buffer.clear();

                    // and state to path start state.
                    self.set_state(URLParserState::PathStart);

                    // If state override is given, then return.
                    if self.state_override.is_some() {
                        return Ok(());
                    }
                }
                // Otherwise:
                else {
                    // If c is U+005B ([),
                    if self.c() == Some('[') {
                        // then set insideBrackets to true.
                        self.inside_brackets = true;
                    }
                    // If c is U+005D (])
                    else if self.c() == Some(']') {
                        // then set insideBrackets to false.
                        self.inside_brackets = false;
                    }

                    // Append c to buffer.
                    self.buffer.push(self.c().unwrap());
                }
            },
            // https://url.spec.whatwg.org/#port-state
            URLParserState::Port => {
                if self.c().is_some() && self.c().unwrap().is_ascii_digit() {
                    // append c to buffer.
                }
                // Otherwise, if one of the following is true:
                // * c is the EOF code point, U+002F (/), U+003F (?), or U+0023 (#)
                // * url is special and c is U+005C (\)
                // * state override is given
                else if (self.c().is_none()
                    || self.c() == Some('/')
                    || self.c() == Some('?')
                    || self.c() == Some('#'))
                    || (self.url.is_special() && self.c() == Some('\\'))
                    || (self.state_override.is_some())
                {
                    // If buffer is not the empty string, then:
                    if !self.buffer.is_empty() {
                        // Let port be the mathematical integer value that is
                        // represented by buffer in radix-10 using ASCII digits
                        // for digits with values 0 through 9.

                        // If port is greater than 2^16 − 1
                        // validation error, return failure.
                        let port = str::parse(&self.buffer).map_err(|_| ())?;

                        // Set url’s port to null, if port is url’s scheme’s default port; otherwise to port.
                        if scheme_default_port(&self.url.scheme) == Some(port) {
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
                        return Ok(());
                    }

                    // Set state to path start state
                    self.set_state(URLParserState::PathStart);

                    // and decrease pointer by 1.
                    self.ptr -= 1;
                }
                // Otherwise
                else {
                    // validation error, return failure.
                    return Err(());
                }
            },
            // https://url.spec.whatwg.org/#file-state
            URLParserState::File => {
                // Set url’s scheme to "file".
                self.url.scheme = "file".to_string();

                // Set url’s host to the empty string.
                self.url.host = Some(Host::OpaqueHost("".to_string()));

                // If c is U+002F (/) or U+005C (\), then:
                if self.c() == Some('/') || self.c() == Some('\\') {
                    // If c is U+005C (\),
                    if self.c() == Some('\\') {
                        // validation error.
                    }

                    // Set state to file slash state.
                    self.set_state(URLParserState::FileSlash);
                }
                // Otherwise, if base is non-null and base’s scheme is "file":
                else if self.base.is_some() && self.base.as_ref().unwrap().scheme == "file" {
                    let base = self.base.as_ref().unwrap();

                    // Set url’s host to base’s host,
                    self.url.host = base.host.clone();

                    // url’s path to a clone of base’s path,
                    self.url.path = base.path.clone();

                    // and url’s query to base’s query.
                    self.url.query = base.query.clone();

                    // If c is U+003F (?)
                    if self.c() == Some('?') {
                        // then set url’s query to the empty string
                        self.url.query = Some(String::new());

                        // and state to query state.
                        self.set_state(URLParserState::Query);
                    }
                    // Otherwise, if c is U+0023 (#)
                    else if self.c() == Some('#') {
                        // set url’s fragment to the empty string
                        self.url.fragment = Some(String::new());

                        // and state to fragment state.
                        self.set_state(URLParserState::Fragment);
                    }
                    // Otherwise, if c is not the EOF code point:
                    else if self.c().is_some() {
                        // Set url’s query to null.
                        self.url.query = None;
                    }

                    // If the code point substring from pointer to the end of input
                    // does not start with a Windows drive letter,
                    if !util::starts_with_windows_drive_letter(self.remaining()) {
                        // then shorten url’s path.
                        self.url.shorten_path();
                    }
                    // Otherwise:
                    else {
                        // Validation error.

                        // Set url’s path to an empty list.
                        self.url.path = vec![];
                    }

                    // Set state to path state
                    self.set_state(URLParserState::Path);

                    // and decrease pointer by 1.
                    self.ptr -= 1;
                }
                // Otherwise
                else {
                    // set state to path state,
                    self.set_state(URLParserState::Path);

                    // and decrease pointer by 1.
                    self.ptr -= 1;
                }
            },
            // https://url.spec.whatwg.org/#file-slash-state
            URLParserState::FileSlash => {
                // If c is U+002F (/) or U+005C (\), then:
                if self.c() == Some('/') || self.c() == Some('\\') {
                    // If c is U+005C (\)
                    if self.c() == Some('\\') {
                        // validation error.
                    }

                    // Set state to file host state.
                    self.set_state(URLParserState::FileHost);
                }
                // Otherwise:
                else {
                    // If base is non-null and base’s scheme is "file", then:
                    if self.base.is_some() && self.base.as_ref().unwrap().scheme == "file" {
                        let base = self.base.as_ref().unwrap();

                        // Set url’s host to base’s host.
                        self.url.host = base.host.clone();

                        // If the code point substring from pointer to the end of input
                        // does not start with a Windows drive letter
                        // and base’s path[0] is a normalized Windows drive letter
                        if util::starts_with_windows_drive_letter(self.remaining())
                            && util::is_normalized_windows_drive_letter(base.path[0].as_str())
                        {
                            // then append base’s path[0] to url’s path.
                            self.url.path.push(base.path[0].clone());
                        }
                    }

                    // Set state to path state,
                    self.set_state(URLParserState::Path);

                    // and decrease pointer by 1.
                    self.ptr -= 1;
                }
            },
            // https://url.spec.whatwg.org/#file-host-state
            URLParserState::FileHost => {
                // If c is the EOF code point, U+002F (/), U+005C (\), U+003F (?), or U+0023 (#)
                if self.c().is_none()
                    || self.c() == Some('/')
                    || self.c() == Some('\\')
                    || self.c() == Some('?')
                    || self.c() == Some('#')
                {
                    // then decrease pointer by 1 and then:
                    self.ptr -= 1;

                    // If state override is not given and buffer is a Windows drive letter
                    if self.state_override.is_none() && util::is_windows_drive_letter(&self.buffer)
                    {
                        // validation error,
                        // set state to path state.
                        self.set_state(URLParserState::Path);
                    }
                    // Otherwise, if buffer is the empty string, then:
                    else if !self.buffer.is_empty() {
                        // Set url’s host to the empty string.
                        self.url.host = Some(Host::OpaqueHost(String::new()));

                        // If state override is given,
                        if self.state_override.is_some() {
                            // then return.
                            return Ok(());
                        }

                        // Set state to path start state.
                        self.set_state(URLParserState::PathStart);
                    }
                    // Otherwise, run these steps:
                    else {
                        // Let host be the result of host parsing buffer with url is not special.
                        // If host is failure, then return failure.
                        let mut host =
                            host_parse_with_special(&self.buffer, false).map_err(|_| ())?; // FIXME: proper error handling

                        // If host is "localhost", then set host to the empty string.
                        if Host::OpaqueHost("localhost".to_string()) == host {
                            host = Host::OpaqueHost(String::new());
                        }

                        // Set url’s host to host.
                        self.url.host = Some(host);

                        // If state override is given,
                        if self.state_override.is_some() {
                            // then return.
                            return Ok(());
                        }

                        // Set buffer to the empty string
                        self.buffer.clear();

                        // and state to path start state.
                        self.set_state(URLParserState::PathStart);
                    }
                }
                // Otherwise, append c to buffer.
                else {
                    self.buffer.push(self.c().unwrap());
                }
            },

            // https://url.spec.whatwg.org/#path-start-state
            URLParserState::PathStart => {
                // If url is special, then:
                if self.url.is_special() {
                    // If c is U+005C (\),
                    if self.c() == Some('\\') {
                        // validation error.
                    }

                    // Set state to path state.
                    self.set_state(URLParserState::Path);

                    // If c is neither U+002F (/) nor U+005C (\)
                    if self.c() != Some('/') && self.c() != Some('\\') {
                        // then decrease pointer by 1.
                        self.ptr -= 1;
                    }
                }
                // Otherwise, if state override is not given and c is U+003F (?)
                else if self.state_override.is_none() && self.c() == Some('?') {
                    // set url’s query to the empty string
                    self.url.query = Some(String::new());

                    // and state to query state.
                    self.set_state(URLParserState::Query);
                }
                // Otherwise, if state override is not given and c is U+0023 (#)
                else if self.state_override.is_none() && self.c() == Some('#') {
                    // set url’s fragment to the empty string
                    self.url.fragment = Some(String::new());

                    // and state to fragment state.
                    self.set_state(URLParserState::Fragment);
                }
                // Otherwise, if c is not the EOF code point:
                else if self.c().is_some() {
                    // Set state to path state.
                    self.set_state(URLParserState::Path);

                    // If c is not U+002F (/),
                    if self.c() != Some('/') {
                        // then decrease pointer by 1.
                        self.ptr -= 1;
                    }
                }
                // Otherwise, if state override is given and url’s host is null,
                else if self.state_override.is_some() && self.url.host.is_none() {
                    // append the empty string to url’s path.
                    self.url.path.push(String::new());
                }
            },
            // https://url.spec.whatwg.org/#path-state
            URLParserState::Path => {
                // If one of the following is true:
                // * c is the EOF code point or U+002F (/)
                // * url is special and c is U+005C (\)
                // * state override is not given and c is U+003F (?) or U+0023 (#)
                if (self.c().is_none() || self.c() == Some('/'))
                    || (self.url.is_special() && self.c() == Some('\\'))
                    || (self.state_override.is_none()
                        && (self.c() == Some('?') || self.c() == Some('#')))
                {
                    // If url is special and c is U+005C (\)
                    if self.url.is_special() && self.c() == Some('\\') {
                        // validation error.
                    }

                    // If buffer is a double-dot path segment, then:
                    if util::is_double_dot_path_segment(&self.buffer) {
                        // Shorten url’s path.
                        self.url.shorten_path();

                        // If neither c is U+002F (/), nor url is special and c is U+005C (\)
                        if self.c() != Some('/')
                            && !(self.url.is_special() && self.c() == Some('\\'))
                        {
                            // append the empty string to url’s path.
                            self.url.path.push(String::new());
                        }
                    }
                    // Otherwise, if buffer is a single-dot path segment
                    // and if neither c is U+002F (/), nor url is special and c is U+005C (\)
                    else if util::is_single_dot_path_segment(&self.buffer)
                        && self.c() != Some('/')
                        && !(self.url.is_special() && self.c() == Some('\\'))
                    {
                        // append the empty string to url’s path.
                        self.url.path.push(String::new());
                    }
                    // Otherwise, if buffer is not a single-dot path segment, then:
                    else if !util::is_single_dot_path_segment(&self.buffer) {
                        // If url’s scheme is "file", url’s path is empty, and buffer is a Windows drive letter
                        if self.url.scheme == "file"
                            && self.url.path.is_empty()
                            && util::is_windows_drive_letter(&self.buffer)
                        {
                            // then replace the second code point in buffer with U+003A (:).
                            // NOTE: unicode codepoint replace is nontrivial because the length is not known
                            // https://stackoverflow.com/questions/66661118/how-do-i-change-characters-at-a-specific-index-within-a-string-in-rust
                            self.buffer.replace_range(
                                self.buffer
                                    .char_indices()
                                    .nth(2)
                                    .map(|(pos, ch)| (pos..pos + ch.len_utf8()))
                                    .unwrap(),
                                ":",
                            );
                        }

                        // Append buffer to url’s path.
                        self.url.path.push(self.buffer.clone());
                    }

                    // Set buffer to the empty string.
                    self.buffer.clear();

                    // If c is U+003F (?)
                    if self.c() == Some('?') {
                        // then set url’s query to the empty string
                        self.url.query = Some(String::new());

                        // and state to query state.
                        self.set_state(URLParserState::Query);
                    }

                    // If c is U+0023 (#)
                    if self.c() == Some('#') {
                        // then set url’s fragment to the empty string
                        self.url.fragment = Some(String::new());

                        // and state to fragment state.
                        self.set_state(URLParserState::Fragment);
                    }
                }
                // Otherwise, run these steps:
                else {
                    // If c is not a URL code point and not U+0025 (%),
                    if !util::is_url_codepoint(self.c().unwrap()) && self.c() != Some('%') {
                        // validation error.
                    }

                    // If c is U+0025 (%) and remaining does not start with two ASCII hex digits, validation error.

                    // UTF-8 percent-encode c using the path percent-encode set and append the result to buffer.
                    let result = percent_encode_char(self.c().unwrap(), is_path_percent_encode_set);
                    self.buffer.push_str(&result);
                }
            },
            // https://url.spec.whatwg.org/#cannot-be-a-base-url-path-state
            URLParserState::OpaquePath => {
                // If c is U+003F (?)
                if self.c() == Some('?') {
                    //  then set url’s query to the empty string
                    self.url.query = Some(String::new());

                    // and state to query state.
                    self.set_state(URLParserState::Query);
                }
                // Otherwise, if c is U+0023 (#)
                else if self.c() == Some('#') {
                    // then set url’s fragment to the empty string
                    self.url.fragment = Some(String::new());

                    // and state to fragment state.
                    self.set_state(URLParserState::Fragment);
                }
                // Otherwise:
                else {
                    // If c is not the EOF code point, not a URL code point, and not U+0025 (%), validation error.

                    // If c is U+0025 (%) and remaining does not start with two ASCII hex digits, validation error.

                    // If c is not the EOF code point
                    if self.c().is_some() {
                        //  UTF-8 percent-encode c using the C0 control percent-encode set
                        let result =
                            percent_encode_char(self.c().unwrap(), is_c0_percent_encode_set);

                        // and append the result to url’s path.
                        self.url.path.push(result);
                    }
                }
            },
            // https://url.spec.whatwg.org/#query-state
            URLParserState::Query => {
                // If encoding is not UTF-8 and one of the following is true:
                // * url is not special
                // * url’s scheme is "ws" or "wss"

                // We don't support non-utf8 encoding

                // If one of the following is true:
                // * state override is not given and c is U+0023 (#)
                // * c is the EOF code point
                if (self.state_override.is_none() && self.c() == Some('#')) || self.c().is_none() {
                    // Let queryPercentEncodeSet be the special-query percent-encode set
                    // if url is special; otherwise the query percent-encode set.
                    let query_percent_encode_set = if self.url.is_special() {
                        is_special_query_percent_encode_set
                    } else {
                        is_query_percent_encode_set
                    };

                    // Percent-encode after encoding, with encoding, buffer, and queryPercentEncodeSet,
                    let result = percent_encode(&self.buffer, query_percent_encode_set);

                    // and append the result to url’s query.
                    self.url.query.as_mut().unwrap().push_str(&result);

                    // Set buffer to the empty string.
                    self.buffer.clear();

                    // If c is U+0023 (#),
                    if self.c() == Some('#') {
                        // then set url’s fragment to the empty string
                        self.url.fragment = Some(String::new());

                        // and state to fragment state.
                        self.set_state(URLParserState::Fragment);
                    }
                }
                // Otherwise, if c is not the EOF code point:
                else if self.c().is_some() {
                    // If c is not a URL code point and not U+0025 (%), validation error.

                    // If c is U+0025 (%) and remaining does not start with two ASCII hex digits, validation error.

                    // Append c to buffer.
                    self.buffer.push(self.c().unwrap())
                }
            },

            // https://url.spec.whatwg.org/#fragment-state
            URLParserState::Fragment => {
                // If c is not the EOF code point, then:
                if let Some(c) = self.c() {
                    // If c is not a URL code point and not U+0025 (%), validation error.

                    // If c is U+0025 (%) and remaining does not start with two ASCII hex digits, validation error.

                    // UTF-8 percent-encode c using the fragment percent-encode set
                    let result = percent_encode_char(c, is_fragment_percent_encode_set);

                    // and append the result to url’s fragment.
                    self.url.fragment.as_mut().unwrap().push_str(&result);
                }
            },
        }
        Ok(())
    }
}
