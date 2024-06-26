use dom_derive::inherit;
use url::URL;

use super::Node;

/// <https://dom.spec.whatwg.org/#interface-document>
#[inherit(Node)]
pub struct Document {
    /// <https://dom.spec.whatwg.org/#concept-document-url>
    url: URL,

    charset: String,
}

impl Document {
    #[must_use]
    pub fn charset(&self) -> &str {
        &self.charset
    }

    pub fn url(&self) -> &URL {
        &self.url
    }

    pub fn set_url(&mut self, url: URL) {
        self.url = url;
    }
}
