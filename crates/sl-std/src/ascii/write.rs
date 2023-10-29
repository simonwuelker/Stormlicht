use super::{Char, Str, String};

use std::string::String as Utf8String;

pub trait Write {
    fn write_str(&mut self, s: &Str);

    fn write_char(&mut self, c: Char) {
        self.write_str(Str::from_ascii_chars(&[c]));
    }
}

impl Write for String {
    fn write_str(&mut self, s: &Str) {
        self.push_str(s);
    }

    fn write_char(&mut self, c: Char) {
        self.push(c);
    }
}

impl Write for Utf8String {
    fn write_str(&mut self, s: &Str) {
        self.push_str(s.as_str())
    }

    fn write_char(&mut self, c: Char) {
        self.push(c.to_char())
    }
}
