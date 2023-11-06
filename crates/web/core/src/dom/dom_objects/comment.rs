use super::Node;

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
