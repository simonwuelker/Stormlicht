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
    #[inline]
    #[must_use]
    pub fn comment_data(&self) -> &str {
        &self.content
    }

    #[inline]
    #[must_use]
    pub fn content_mut(&mut self) -> &mut String {
        &mut self.content
    }
}

impl DOMDisplay for Comment {
    fn format<W: fmt::Write>(&self, mut writer: &mut W) -> fmt::Result {
        write!(writer, "<!-- ")?;
        self.format_text(&mut writer, &self.content)?;
        write!(writer, " -->")?;
        Ok(())
    }
}
