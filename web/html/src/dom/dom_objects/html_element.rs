//! <https://html.spec.whatwg.org/multipage/dom.html#htmlelement>

use dom_derive::inherit;

use super::Element;

#[inherit(Element)]
pub struct HTMLElement {}
