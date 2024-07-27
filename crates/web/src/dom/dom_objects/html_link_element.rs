use dom_derive::inherit;
use url::URL;

use crate::{html::links, static_interned};

use super::HtmlElement;

/// <https://html.spec.whatwg.org/multipage/semantics.html#the-link-element>
#[inherit(HtmlElement)]
pub struct HtmlLinkElement {}

impl HtmlLinkElement {
    pub fn new(html_element: HtmlElement) -> Self {
        Self {
            __parent: html_element,
        }
    }

    #[must_use]
    pub fn relationship(&self) -> links::Relationship {
        self.attributes()
            .get(&static_interned!("rel"))
            .map(|s| s.to_string().to_ascii_lowercase())
            .map(|value| links::Relationship::from(value.as_str()))
            .unwrap_or(links::Relationship::Invalid)
    }

    #[must_use]
    pub fn url(&self) -> Option<URL> {
        let document = self.owning_document().expect("must have a document");

        self.attributes()
            .get(&static_interned!("href"))
            .map(|value| value.to_string())
            .as_ref()
            .map(String::as_str)
            .map(|value| URL::parse_with_base(value, Some(document.borrow().url()), None))
            .map(Result::ok)
            .flatten()
    }
}
