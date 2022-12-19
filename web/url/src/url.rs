//! Implements https://url.spec.whatwg.org

use crate::{
    host::Host,
    urlparser::{URLParser, URLParserState},
};

// https://url.spec.whatwg.org/#url-code-points
pub(crate) fn is_url_codepoint(c: char) -> bool {
    c.is_alphanumeric()
        | match c {
            '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | '-' | '.' | '/' | ':' | ';'
            | '=' | '?' | '@' | '_' | '~' => true,
            // range excludes surrogates and noncharacters
            '\u{00A0}'..='\u{D7FF}' | '\u{E000}'..='\u{10FFFD}' => {
                // check for noncharacters
                // return true if c is not a noncharacter
                match c {
                    '\u{FDD0}'..='\u{FDEF}'
                    | '\u{FFFE}'
                    | '\u{FFFF}'
                    | '\u{1FFFE}'
                    | '\u{1FFFF}'
                    | '\u{2FFFE}'
                    | '\u{2FFFF}'
                    | '\u{3FFFE}'
                    | '\u{3FFFF}'
                    | '\u{4FFFE}'
                    | '\u{4FFFF}'
                    | '\u{5FFFE}'
                    | '\u{5FFFF}'
                    | '\u{6FFFE}'
                    | '\u{6FFFF}'
                    | '\u{7FFFE}'
                    | '\u{7FFFF}'
                    | '\u{8FFFE}'
                    | '\u{8FFFF}'
                    | '\u{9FFFE}'
                    | '\u{9FFFF}'
                    | '\u{AFFFE}'
                    | '\u{AFFFF}'
                    | '\u{BFFFE}'
                    | '\u{BFFFF}'
                    | '\u{CFFFE}'
                    | '\u{CFFFF}'
                    | '\u{DFFFE}'
                    | '\u{DFFFF}'
                    | '\u{EFFFE}'
                    | '\u{EFFFF}'
                    | '\u{FFFFE}'
                    | '\u{FFFFF}'
                    | '\u{10FFFE}'
                    | '\u{10FFFF}' => false,
                    _ => true,
                }
            },
            _ => false,
        }
}

pub fn scheme_is_special(scheme: &str) -> bool {
    match scheme {
        "ftp" | "file" | "http" | "https" | "ws" | "wss" => true,
        _ => false,
    }
}

pub fn scheme_default_port(scheme: &str) -> Option<Port> {
    match scheme {
        "ftp" => Some(21),
        "http" | "ws" => Some(80),
        "https" | "wss" => Some(443),
        _ => None,
    }
}

pub type Port = u16;

#[derive(Clone)]
pub enum Path {
    Opaque(String),
    NotOpaque(Vec<String>),
}

// https://url.spec.whatwg.org/#concept-url
#[derive(Default)]
pub struct URL {
    pub scheme: String,
    pub username: String,
    pub password: String,
    pub host: Option<Host>,
    pub port: Option<Port>,
    pub path: Path,
    pub query: Option<String>,
    pub fragment: Option<String>,
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
    pub fn includes_credentials(&self) -> bool {
        !self.username.is_empty() || !self.password.is_empty()
    }

    // https://url.spec.whatwg.org/#is-special
    pub fn is_special(&self) -> bool {
        scheme_is_special(&self.scheme)
    }

    // https://url.spec.whatwg.org/#url-opaque-path
    pub fn has_opaque_path(&self) -> bool {
        match self.path {
            Path::Opaque(_) => true,
            _ => false,
        }
    }
}

impl Default for Path {
    fn default() -> Self {
        Self::Opaque(String::new())
    }
}
