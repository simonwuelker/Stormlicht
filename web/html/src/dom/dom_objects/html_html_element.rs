//! <https://html.spec.whatwg.org/multipage/semantics.html#the-html-element>

use dom_derive::inherit;

use super::HTMLElement;

#[inherit(HTMLElement)]
pub struct HTMLHtmlElement {}
