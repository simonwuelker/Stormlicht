//! Implements https://url.spec.whatwg.org

pub type Port = u16;
// https://url.spec.whatwg.org/#ip-address
#[derive(PartialEq, Clone)]
pub enum IP {
    IPv4(u32),
    IPv6(u128),
}

// https://url.spec.whatwg.org/#concept-host
#[derive(PartialEq, Clone)]
pub enum Host {
    Domain(String),
    IP(IP),
    OpaqueHost(String),
    EmptyHost,
}

#[derive(Clone)]
pub enum Path {
    Opaque(String),
    NotOpaque(Vec<String>),
}

// https://url.spec.whatwg.org/#concept-url
#[derive(Default)]
pub struct URL {
    scheme: String,
    username: String,
    password: String,
    host: Option<Host>,
    port: Option<Port>,
    path: Path,
    query: Option<String>,
    fragment: Option<String>,
}

// https://infra.spec.whatwg.org/#c0-control
fn is_c0_or_space(c: char) -> bool {
    match c {
        '\u{0000}'..='\u{001F}' | '\u{0020}' => true,
        _ => false,
    }
}

fn is_ascii_tab_or_newline(c: char) -> bool {
    match c {
        '\u{0009}' | '\u{000A}' | '\u{000D}' => true,
        _ => false,
    }
}

fn is_special_scheme(scheme: &str) -> (bool, Option<Port>) {
    match scheme {
        "ftp" => (true, Some(21)),
        "file" => (true, None),
        "http" | "ws" => (true, Some(80)),
        "https" | "wss" => (true, Some(443)),
        _ => (false, None),
    }
}

impl URL {
    // https://url.spec.whatwg.org/#concept-basic-url-parser
    pub fn parse_with_base(
        mut input: String,
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
                    .trim_start_matches(is_c0_or_space)
                    .trim_end_matches(is_c0_or_space)
                    .to_string();
                url
            },
        };

        // If input contains any ASCII tab or newline, validation error.

        // Remove all ASCII tab or newline from input.
        // TODO https://doc.rust-lang.org/std/string/struct.String.html#method.remove_matches
        // would be nice here, but it's not stabilized yet
        input = input
            .chars()
            .filter(|c| !is_ascii_tab_or_newline(*c))
            .collect();

        // Let state be state override if given, or scheme start state otherwise.
        let state = state_override.unwrap_or(URLParserState::Scheme);

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
            input: &input,
            state_override: state_override,
            at_sign_seen: at_sign_seen,
            inside_brackets: inside_brackets,
            password_token_seen: password_token_seen,
        };
        _ = state_machine.run();
        state_machine.url
    }

    // https://url.spec.whatwg.org/#concept-basic-url-parser
    pub fn parse(input: String) -> Self {
        Self::parse_with_base(input, None, None, None)
    }

    // https://url.spec.whatwg.org/#include-credentials
    fn includes_credentials(&self) -> bool {
        !self.username.is_empty() || !self.password.is_empty()
    }

    // https://url.spec.whatwg.org/#is-special
    fn is_special(&self) -> bool {
        is_special_scheme(&self.scheme).0
    }

    // https://url.spec.whatwg.org/#url-opaque-path
    fn has_opaque_path(&self) -> bool {
        match self.path {
            Path::Opaque(_) => true,
            _ => false,
        }
    }
}

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

pub struct URLParser<'a> {
    url: URL,
    base: Option<URL>,
    input: &'a str,
    state: URLParserState,
    ptr: usize,
    buffer: String,
    at_sign_seen: bool,
    inside_brackets: bool,
    password_token_seen: bool,
    state_override: Option<URLParserState>,
}

// https://url.spec.whatwg.org/#c0-control-percent-encode-set
fn is_c0_percent_encode_set(c: char) -> bool {
    match c {
        '\u{0000}'..='\u{001F}' | '\u{007F}'.. => true,
        _ => false,
    }
}

// https://url.spec.whatwg.org/#query-percent-encode-set
fn is_query_percent_encode_set(c: char) -> bool {
    is_c0_percent_encode_set(c)
        | match c {
            ' ' | '"' | '#' | '<' | '>' => true,
            _ => false,
        }
}

// https://url.spec.whatwg.org/#path-percent-encode-set
fn is_path_percent_encode_set(c: char) -> bool {
    is_query_percent_encode_set(c)
        | match c {
            '?' | '`' | '{' | '}' => true,
            _ => false,
        }
}

// https://url.spec.whatwg.org/#userinfo-percent-encode-set
fn is_userinfo_percent_encode_set(c: char) -> bool {
    is_path_percent_encode_set(c)
        | match c {
            '/' | ':' | ';' | '=' | '@' | '['..='^' | '|' => true,
            _ => false,
        }
}
// https://url.spec.whatwg.org/#string-percent-encode-after-encoding
fn percent_encode<F: Fn(char) -> bool>(c: char, in_encode_set: F) -> String {
    let mut out = String::new();
    let mut buffer = [0; 4];
    let encoded = c.encode_utf8(&mut buffer);
    for b in encoded.chars() {
        if in_encode_set(b) {
            // percent-encode byte and append the result to output.
            out.push('%');
            out.push_str(b.to_string().as_str());
        } else {
            out.push(b);
        }
    }
    out
}

impl<'a> URLParser<'a> {
    fn run(&mut self) -> Result<(), ()> {
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
                            if is_special_scheme(&self.url.scheme).0
                                && !is_special_scheme(&self.buffer).0
                            {
                                // then return.
                                return Ok(());
                            }
                            // If url’s scheme is not a special scheme and buffer is a special scheme
                            if !is_special_scheme(&self.url.scheme).0
                                && is_special_scheme(&self.buffer).0
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
                            if self.url.port == is_special_scheme(&self.url.scheme).1 {
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
                            percent_encode(code_point, is_userinfo_percent_encode_set);

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
                }
            },
            _ => todo!(),
        }
        Ok(())
    }
}

impl Default for Path {
    fn default() -> Self {
        Self::Opaque(String::new())
    }
}
