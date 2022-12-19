use crate::{
    host::{self, Host},
    scheme_default_port, scheme_is_special,
    urlencode::{is_userinfo_percent_encode_set, percent_encode_char},
    Path, URL,
};

#[derive(Clone, Copy)]
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
    match c {
        '\u{0000}'..='\u{001F}' => true,
        _ => false,
    }
}

impl<'a> URLParser<'a> {
    pub(crate) fn run(&mut self) -> Result<(), ()> {
        while self.ptr < self.input.len() {
            self.step()?;
        }
        Ok(())
    }

    fn c(&self) -> Option<char> {
        self.input.chars().nth(self.ptr)
    }

    fn remaining(&self) -> &'a str {
        &self.input[self.ptr + 1..]
    }

    fn step(&mut self) -> Result<(), ()> {
        match self.state {
            // https://url.spec.whatwg.org/#scheme-start-state
            URLParserState::SchemeStart => {
                match self.c() {
                    Some(c @ 'A'..='Z') => {
                        // Append c, lowercased, to buffer, and set state to scheme state.
                        self.buffer.push(c.to_ascii_lowercase());
                        self.state = URLParserState::Scheme;
                    },
                    _ => {
                        // Otherwise, if state override is not given
                        if self.state_override.is_none() {
                            // Set state to no scheme state and decrease pointer by 1.
                            self.state = URLParserState::NoScheme;
                            self.ptr -= 1;
                        } else {
                            // Otherwise, validation error, return failure.
                            return Err(());
                        }
                    },
                }
            },
            // https://url.spec.whatwg.org/#scheme-state
            URLParserState::Scheme => {
                match self.c() {
                    Some(c @ ('A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '+' | '.')) => {
                        // Append c, lowercased, to buffer
                        self.buffer.push(c.to_ascii_lowercase());
                    },
                    Some(':') => {
                        // If state override is given, then:
                        if self.state_override.is_some() {
                            // If url’s scheme is a special scheme and buffer is not a special scheme
                            if scheme_is_special(&self.url.scheme)
                                && !scheme_is_special(&self.buffer)
                            {
                                // then return.
                                return Ok(());
                            }
                            // If url’s scheme is not a special scheme and buffer is a special scheme
                            if !scheme_is_special(&self.url.scheme)
                                && scheme_is_special(&self.buffer)
                            {
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
                        self.buffer = String::new();

                        // If url’s scheme is "file", then:
                        if self.url.scheme == "file" {
                            // If remaining does not start with "//"
                            if !self.remaining().starts_with("//") {
                                // validation error.
                                return Err(());
                            }

                            // Set state to file state.
                            self.state = URLParserState::File;
                        }
                        // Otherwise, if url is special, base is non-null, and base’s scheme is url’s scheme:
                        else if self.url.is_special()
                            && self.base.is_some()
                            && self.base.as_ref().unwrap().scheme == self.url.scheme
                        {
                            // Assert: base is is special (and therefore does not have an opaque path).
                            assert!(self.base.as_ref().unwrap().is_special());

                            // Set state to special relative or authority state.
                            self.state = URLParserState::SpecialRelativeOrAuthority;
                        }
                        // Otherwise, if url is special
                        else if self.url.is_special() {
                            // set state to special authority slashes state.
                            self.state = URLParserState::SpecialAuthoritySlashes;
                        }
                        // Otherwise, if remaining starts with an U+002F (/)
                        else if self.remaining().starts_with('/') {
                            // set state to path or authority state and increase pointer by 1.
                            self.state = URLParserState::PathOrAuthority;
                            self.ptr += 1;
                        }
                        // Otherwise,
                        else {
                            // set url’s path to the empty string and set state to opaque path state.
                            self.url.path = Path::Opaque(String::new());
                            self.state = URLParserState::OpaquePath;
                        }
                    },
                    _ => {
                        // Otherwise, if state override is not given
                        if self.state_override.is_none() {
                            // set buffer to the empty string,
                            self.buffer = String::new();

                            // state to no scheme state,
                            self.state = URLParserState::NoScheme;

                            // and start over (from the first code point in input).
                            self.ptr = 0;
                        } else {
                            // Otherwise, validation error, return failure.
                        }
                    },
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
                    self.state = URLParserState::Fragment;
                }
                // Otherwise, if base’s scheme is not "file"
                else if base.scheme != "file" {
                    // set state to relative state
                    self.state = URLParserState::Relative;

                    // and decrease pointer by 1.
                    self.ptr -= 1;
                }
                // Otherwise,
                else {
                    // set state to file state
                    self.state = URLParserState::File;

                    // and decrease pointer by 1.
                    self.ptr -= 1;
                }
            },
            // https://url.spec.whatwg.org/#special-relative-or-authority-state
            URLParserState::SpecialRelativeOrAuthority => {
                // If c is U+002F (/) and remaining starts with U+002F (/)
                if self.c() == Some('/') && self.remaining().starts_with('/') {
                    // then set state to special authority ignore slashes state
                    self.state = URLParserState::SpecialAuthorityIgnoreSlashes;

                    // and increase pointer by 1.
                    self.ptr += 1;
                }
                // Otherwise,
                else {
                    // validation error,
                    // set state to relative state
                    self.state = URLParserState::Relative;

                    // and decrease pointer by 1.
                    self.ptr -= 1;
                }
            },
            // https://url.spec.whatwg.org/#path-or-authority-state
            URLParserState::PathOrAuthority => {
                // if c is U+002F (/),
                if self.c() == Some('/') {
                    // then set state to authority state.
                    self.state = URLParserState::Authority;
                }
                // Otherwise,
                else {
                    // set state to path state,
                    self.state = URLParserState::Path;

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
                if self.c() == Some('/') {
                    // then set state to relative slash state.
                    self.state = URLParserState::RelativeSlash;
                }
                // Otherwise, if url is special and c is U+005C (\)
                else if self.url.is_special() && self.c() == Some('\\') {
                    // validation error
                    // set state to relative slash state.
                    self.state = URLParserState::RelativeSlash;
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
                        self.state = URLParserState::Query;
                    }
                    // Otherwise, if c is U+0023 (#)
                    else if self.c() == Some('#') {
                        // set url’s fragment to the empty string
                        self.url.fragment = Some(String::new());

                        // and state to fragment state.
                        self.state = URLParserState::Fragment;
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
                    self.state = URLParserState::SpecialAuthorityIgnoreSlashes;
                }
                // Otherwise, if c is U+002F (/)
                else if self.c() == Some('/') {
                    // then set state to authority state.
                    self.state = URLParserState::Authority;
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
                    self.state = URLParserState::Path;

                    // and then, decrease pointer by 1.
                    self.ptr -= 1;
                }
            },
            // https://url.spec.whatwg.org/#special-authority-slashes-state
            URLParserState::SpecialAuthoritySlashes => {
                // If c is U+002F (/) and remaining starts with U+002F (/)
                if self.c() == Some('/') && self.remaining().starts_with('/') {
                    // then set state to special authority ignore slashes state
                    self.state = URLParserState::SpecialAuthorityIgnoreSlashes;

                    // and increase pointer by 1.
                    self.ptr += 1;
                }
                // Otherwise
                else {
                    // validation error,
                    // set state to special authority ignore slashes state
                    self.state = URLParserState::SpecialAuthorityIgnoreSlashes;

                    // and decrease pointer by 1.
                    self.ptr -= 1;
                }
            },
            // https://url.spec.whatwg.org/#special-authority-ignore-slashes-state
            URLParserState::SpecialAuthorityIgnoreSlashes => {
                // If c is neither U+002F (/) nor U+005C (\)
                if self.c() != Some('/') && self.c() != Some('\\') {
                    // then set state to authority state
                    self.state = URLParserState::Authority;

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
                    self.buffer = String::new();
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
                    self.buffer = String::new();

                    // and set state to host state.
                    self.state = URLParserState::Host;
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
                    self.state = URLParserState::FileHost;
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
                    let _ = host::host_parse_with_special(self.buffer.as_str(), true);
                }
            },
            _ => todo!(),
        }
        Ok(())
    }
}
