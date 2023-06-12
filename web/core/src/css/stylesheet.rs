use super::{selectors::SelectorList, StylePropertyDeclaration};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    /// CSS added by the browser
    /// <https://drafts.csswg.org/css-cascade-4/#cascade-origin-ua>
    UserAgent,

    /// CSS added by the user, for example via the developer tools
    /// <https://drafts.csswg.org/css-cascade-4/#cascade-origin-user>
    User,

    /// CSS added by the website, for example using `<style>` tags
    /// <https://drafts.csswg.org/css-cascade-4/#cascade-origin-author>
    Author,
}

#[derive(Clone, Debug)]
pub struct Stylesheet {
    /// Where the stylesheet came from
    pub origin: Origin,

    /// The rules contained in the stylesheet
    pub rules: Vec<StyleRule>,
}

#[derive(Clone, Debug)]
pub struct StyleRule {
    pub selectors: SelectorList,
    pub properties: Vec<StylePropertyDeclaration>,
}

impl StyleRule {
    #[must_use]
    pub fn properties(&self) -> &[StylePropertyDeclaration] {
        &self.properties
    }
}
