use std::fmt;

use crate::{
    css::{selectors::Specificity, syntax::Token, CSSParse, ParseError, Parser},
    dom::{dom_objects::Element, DomPtr},
    static_interned, InternedString,
};

use super::{
    type_selector, AttributeSelector, Combinator, PseudoClassSelector, TypeSelector,
    WellQualifiedName,
};

pub trait CSSValidateSelector {
    /// <https://drafts.csswg.org/selectors-4/#invalid-selector>
    fn is_valid(&self) -> bool;
}

#[derive(Clone, Debug)]
pub struct Selector {
    components: Box<[SelectorComponentOrCombinator]>,
}

#[derive(Clone, Debug)]
pub enum SelectorComponentOrCombinator {
    SelectorComponent(SelectorComponent),
    Combinator(Combinator),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectorComponent {
    /// Match an element by id (`#foo`)
    ///
    /// <https://drafts.csswg.org/selectors-4/#id-selectors>
    Id(InternedString),

    /// Match an element by class (`.foo`)
    ///
    /// <https://drafts.csswg.org/selectors-4/#class-html>
    Class(InternedString),

    /// Match an elements attribute
    ///
    /// <https://drafts.csswg.org/selectors-4/#attribute-selectors>
    Attribute(AttributeSelector),

    /// Matches an element on some other property
    ///
    /// <https://drafts.csswg.org/selectors-4/#typedef-pseudo-class-selector>
    PseudoClass(PseudoClassSelector),

    /// Match an element by type (`foo`)
    ///
    /// <https://drafts.csswg.org/selectors-4/#type-selectors>
    Type(TypeSelector),
}

impl<'a> CSSParse<'a> for Selector {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let mut components = vec![];

        loop {
            let component = SelectorComponent::parse(parser)?;
            components.push(component.into());

            if let Some(combinator) = Combinator::next_combinator(parser)? {
                components.push(combinator.into());
            };

            if matches!(
                parser.peek_token_ignoring_whitespace(0),
                Some(Token::CurlyBraceOpen | Token::Comma) | None
            ) {
                break;
            }
        }

        let selector = Self {
            components: components.into_boxed_slice(),
        };

        Ok(selector)
    }
}

impl<'a> CSSParse<'a> for SelectorComponent {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let component = match parser.peek_token_ignoring_whitespace(0) {
            Some(Token::Hash(ident, ..)) => {
                let ident = *ident;
                _ = parser.next_token_ignoring_whitespace();

                Self::Id(ident)
            },
            Some(Token::Delim('.')) => {
                _ = parser.next_token_ignoring_whitespace();

                // No whitespace allowed here
                let Some(Token::Ident(ident)) = parser.next_token() else {
                    return Err(ParseError);
                };

                Self::Class(ident)
            },
            Some(Token::BracketOpen) => {
                // Attribute selector
                let attribute_selector = AttributeSelector::parse(parser)?;

                Self::Attribute(attribute_selector)
            },
            Some(Token::Colon) => {
                let pseudo_class_selector = PseudoClassSelector::parse(parser)?;

                Self::PseudoClass(pseudo_class_selector)
            },
            Some(Token::Delim('*')) => {
                _ = parser.next_token_ignoring_whitespace();

                let type_selector = TypeSelector::Universal(None);

                Self::Type(type_selector)
            },
            Some(Token::Ident(_) | Token::Delim('|')) => {
                let type_selector = TypeSelector::parse(parser)?;

                Self::Type(type_selector)
            },
            _ => return Err(ParseError),
        };

        Ok(component)
    }
}

/// An iterator over the components of a [Selector]
///
/// Refer to [Selector::components] for more information
pub struct ComponentIterator<'a> {
    index: usize,
    components: &'a [SelectorComponentOrCombinator],
}

impl Selector {
    #[inline]
    #[must_use]
    pub fn components<'a>(&'a self) -> ComponentIterator<'a> {
        ComponentIterator {
            index: 0,
            components: &self.components,
        }
    }

    #[must_use]
    pub fn specificity(&self) -> Specificity {
        let mut specificity = Specificity::default();

        let mut components = self.components();
        loop {
            for simple_selector in &mut components {
                specificity += simple_selector.specificity();
            }

            if components.next_component().is_none() {
                break;
            }
        }

        specificity
    }

    #[must_use]
    pub fn matches(&self, element: &DomPtr<Element>) -> bool {
        let mut components = self.components();

        loop {
            if components.all(|selector| selector.matches(element)) {
                return true;
            }

            let Some(combinator) = components.next_component() else {
                break;
            };

            // FIXME: use the combinator
            _ = combinator;
        }

        false
    }
}

impl SelectorComponent {
    #[must_use]
    fn specificity(&self) -> Specificity {
        match self {
            Self::Id(_) => Specificity::new(1, 0, 0),
            Self::Class(_) => Specificity::new(0, 1, 0),
            Self::Attribute(_) => Specificity::new(0, 1, 0),
            Self::PseudoClass(_) => Specificity::new(0, 1, 0),
            Self::Type(type_selector) => type_selector.specificity(),
        }
    }

    #[must_use]
    pub fn matches(&self, element: &DomPtr<Element>) -> bool {
        match self {
            Self::Id(id) => element
                .borrow()
                .attributes()
                .get(&static_interned!("id"))
                .is_some_and(|attr| attr == id),
            Self::Class(_) => {
                // FIXME: implement class selector
                false
            },
            Self::PseudoClass(_) => {
                // FIXME: implement pseudo class selectors
                false
            },
            Self::Attribute(attribute_selector) => attribute_selector.matches(element),
            Self::Type(type_selector) => type_selector.matches(element),
        }
    }
}

impl<'a> ComponentIterator<'a> {
    /// Resumes the iterator, moving on to the next component
    ///
    /// Returns the iterator between the previous and the next component,
    /// or `None` if the current component is not finished or there are no
    /// more components
    #[must_use]
    pub fn next_component(&mut self) -> Option<Combinator> {
        match self.components.get(self.index) {
            Some(SelectorComponentOrCombinator::Combinator(combinator)) => {
                self.index += 1;
                Some(*combinator)
            },
            _ => None,
        }
    }
}

impl<'a> Iterator for ComponentIterator<'a> {
    type Item = &'a SelectorComponent;

    fn next(&mut self) -> Option<Self::Item> {
        match self.components.get(self.index) {
            Some(SelectorComponentOrCombinator::SelectorComponent(component)) => {
                self.index += 1;
                Some(component)
            },
            _ => None,
        }
    }
}

impl From<SelectorComponent> for SelectorComponentOrCombinator {
    fn from(value: SelectorComponent) -> Self {
        Self::SelectorComponent(value)
    }
}

impl From<Combinator> for SelectorComponentOrCombinator {
    fn from(value: Combinator) -> Self {
        Self::Combinator(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::css::selectors::{AttributeMatcher, AttributeModifier, NamespacePrefix};

    use super::*;

    #[test]
    fn parse_id_selector() {
        let selector = Selector::parse_from_str("#foo").unwrap();
        let mut components = selector.components();

        assert_eq!(
            components.next(),
            Some(SelectorComponent::Id("foo".into())).as_ref()
        );

        assert!(components.next().is_none());
        assert!(components.next_component().is_none());
    }

    #[test]
    fn parse_class_selector() {
        let selector = Selector::parse_from_str(".foo").unwrap();
        let mut components = selector.components();

        assert_eq!(
            components.next(),
            Some(SelectorComponent::Class("foo".into())).as_ref()
        );

        assert!(components.next().is_none());
        assert!(components.next_component().is_none());
    }

    #[test]
    fn parse_attribute_selector_exists() {
        let selector = Selector::parse_from_str("[foo]").unwrap();
        let mut components = selector.components();

        let reference = AttributeSelector::Exists {
            attribute_name: WellQualifiedName {
                prefix: None,
                ident: "foo".into(),
            },
        };

        assert_eq!(
            components.next(),
            Some(&SelectorComponent::Attribute(reference))
        );

        assert!(components.next().is_none());
        assert!(components.next_component().is_none());
    }

    #[test]
    fn parse_attribute_selector_starts_with() {
        let selector = Selector::parse_from_str("[foo ^= bar]").unwrap();
        let mut components = selector.components();

        let reference = AttributeSelector::Matches {
            attribute_name: WellQualifiedName {
                prefix: None,
                ident: "foo".into(),
            },
            matcher: AttributeMatcher::StartsWith,
            value: "bar".into(),
            modifier: AttributeModifier::CaseSensitive,
        };

        assert_eq!(
            components.next(),
            Some(&SelectorComponent::Attribute(reference))
        );

        assert!(components.next().is_none());
        assert!(components.next_component().is_none());
    }

    #[test]
    fn parse_attribute_selector_ends_with() {
        let selector = Selector::parse_from_str("[foo $= bar]").unwrap();
        let mut components = selector.components();

        let reference = AttributeSelector::Matches {
            attribute_name: WellQualifiedName {
                prefix: None,
                ident: "foo".into(),
            },
            matcher: AttributeMatcher::EndsWith,
            value: "bar".into(),
            modifier: AttributeModifier::CaseSensitive,
        };

        assert_eq!(
            components.next(),
            Some(&SelectorComponent::Attribute(reference))
        );

        assert!(components.next().is_none());
        assert!(components.next_component().is_none());
    }

    #[test]
    fn parse_attribute_selector_equal() {
        let selector = Selector::parse_from_str("[foo = bar]").unwrap();
        let mut components = selector.components();

        let reference = AttributeSelector::Matches {
            attribute_name: WellQualifiedName {
                prefix: None,
                ident: "foo".into(),
            },
            matcher: AttributeMatcher::EqualTo,
            value: "bar".into(),
            modifier: AttributeModifier::CaseSensitive,
        };

        assert_eq!(
            components.next(),
            Some(&SelectorComponent::Attribute(reference))
        );

        assert!(components.next().is_none());
        assert!(components.next_component().is_none());
    }

    #[test]
    fn parse_type_selector_with_namespace() {
        let selector = Selector::parse_from_str("foo|bar").unwrap();
        let mut components = selector.components();

        let reference = TypeSelector::Typename(WellQualifiedName {
            prefix: Some(NamespacePrefix::Ident("foo".into())),
            ident: "bar".into(),
        });

        assert_eq!(components.next(), Some(&SelectorComponent::Type(reference)));

        assert!(components.next().is_none());
        assert!(components.next_component().is_none());
    }

    #[test]
    fn parse_universal_selector_with_namespace() {
        let selector = Selector::parse_from_str("foo|*").unwrap();
        let mut components = selector.components();

        let reference = TypeSelector::Universal(Some(NamespacePrefix::Ident("foo".into())));

        assert_eq!(components.next(), Some(&SelectorComponent::Type(reference)));

        assert!(components.next().is_none());
        assert!(components.next_component().is_none());
    }
    #[test]
    fn parse_universal_selector() {
        let selector = Selector::parse_from_str("*").unwrap();
        let mut components = selector.components();

        let reference = TypeSelector::Universal(None);

        assert_eq!(components.next(), Some(&SelectorComponent::Type(reference)));

        assert!(components.next().is_none());
        assert!(components.next_component().is_none());
    }

    #[test]
    fn fail_to_parse_invalid_type_selector() {
        // Whitespace between *any* of the components of a type selector
        // is not allowed
        assert!(Selector::parse_from_str("*| bar").is_err());
        assert!(Selector::parse_from_str("foo | *").is_err());
        assert!(Selector::parse_from_str("* | *").is_err());
    }

    #[test]
    fn parse_type_selector_with_empty_prefix_that_looks_like_combinator() {
        // This is a id selector(#foo), a descendant combinator (whitespace)
        // and a type selector (|bar)

        // The parser should not misinterpret the "|" as the beginning of a column combinator
        let source = "#foo |bar";

        let selector = Selector::parse_from_str(source).unwrap();
        let mut components = selector.components();

        let id_reference = SelectorComponent::Id("foo".into());
        let type_reference = SelectorComponent::Type(TypeSelector::Typename(
            WellQualifiedName::without_namespace("bar".into()),
        ));

        assert_eq!(components.next(), Some(&id_reference));
        assert!(components.next().is_none());

        assert_eq!(components.next_component(), Some(Combinator::Descendant));

        assert_eq!(components.next(), Some(&type_reference));
        assert!(components.next().is_none());
    }

    #[test]
    fn parse_pseudo_class_selector() {
        let source = ":foo";

        let selector = Selector::parse_from_str(source).unwrap();
        let mut components = selector.components();

        let reference = SelectorComponent::PseudoClass(PseudoClassSelector::Ident("foo".into()));

        assert_eq!(components.next(), Some(&reference));
        assert!(components.next().is_none());
        assert!(components.next_component().is_none());
    }
}
