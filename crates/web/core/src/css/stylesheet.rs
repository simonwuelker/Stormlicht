use std::fmt;

use super::{
    selectors::{serialize_selector_list, SelectorList},
    Parser, StylePropertyDeclaration,
};

/// <https://drafts.csswg.org/css-cascade-4/#cascading-origins>
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

    /// A number describing the order of appearance of different stylesheets
    index: usize,
}

impl Stylesheet {
    #[inline]
    #[must_use]
    pub fn new(origin: Origin, rules: Vec<StyleRule>, index: usize) -> Self {
        Self {
            origin,
            rules,
            index,
        }
    }

    #[inline]
    #[must_use]
    pub fn user_agent_rules() -> Self {
        let default_css = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/default.css"));
        Parser::new(default_css, Origin::UserAgent)
            .parse_stylesheet(usize::MAX)
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

    pub fn index(&self) -> usize {
        self.index
    }
}

#[derive(Clone)]
pub struct StyleRule {
    selectors: SelectorList,
    properties: Vec<StylePropertyDeclaration>,
}

impl StyleRule {
    pub fn new(selectors: SelectorList, properties: Vec<StylePropertyDeclaration>) -> Self {
        Self {
            selectors,
            properties,
        }
    }

    #[must_use]
    pub fn selectors(&self) -> &SelectorList {
        &self.selectors
    }

    #[must_use]
    pub fn properties(&self) -> &[StylePropertyDeclaration] {
        &self.properties
    }
}

impl fmt::Debug for StyleRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut selectors = String::new();
        serialize_selector_list(&self.selectors, &mut selectors)
            .expect("Serializing into a string can never fail");

        f.debug_struct("StyleRule")
            .field("selector", &selectors)
            .field("properties", &self.properties)
            .finish()
    }
}
