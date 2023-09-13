use super::{selectors::SelectorList, Parser, StylePropertyDeclaration};

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
    origin: Origin,

    /// The rules contained in the stylesheet
    rules: Vec<StyleRule>,
}

impl Stylesheet {
    #[inline]
    #[must_use]
    pub fn new(origin: Origin, rules: Vec<StyleRule>) -> Self {
        Self { origin, rules }
    }

    #[inline]
    #[must_use]
    pub fn user_agent_rules() -> Self {
        let default_css = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/default.css"));
        Parser::new(default_css, Origin::UserAgent)
            .parse_stylesheet()
            .expect("Parsing user agent CSS should never fail")
    }

    #[inline]
    #[must_use]
    pub fn origin(&self) -> Origin {
        self.origin
    }

    #[inline]
    #[must_use]
    pub fn rules(&self) -> &[StyleRule] {
        &self.rules
    }
}

#[derive(Clone, Debug)]
pub struct StyleRule {
    selector: SelectorList,
    properties: Vec<StylePropertyDeclaration>,
}

impl StyleRule {
    pub fn new(selector: SelectorList, properties: Vec<StylePropertyDeclaration>) -> Self {
        Self {
            selector,
            properties,
        }
    }

    #[must_use]
    pub fn selector(&self) -> &SelectorList {
        &self.selector
    }

    #[must_use]
    pub fn properties(&self) -> &[StylePropertyDeclaration] {
        &self.properties
    }
}
