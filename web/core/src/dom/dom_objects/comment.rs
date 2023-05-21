use super::Node;
use crate::dom::DOMDisplay;
use std::fmt;

use dom_derive::inherit;

/// <https://dom.spec.whatwg.org/#characterdata>
#[inherit(Node)]
pub struct Comment {
    content: String,
}

impl Comment {
    pub fn content_mut(&mut self) -> &mut String {
        &mut self.content
    }
}

impl DOMDisplay for Comment {
    fn format(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<!-- ")?;
        self.format_text(f, &self.content)?;
        write!(f, " -->")?;
        Ok(())
    }
}
