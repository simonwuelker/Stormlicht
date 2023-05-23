use super::{CSSValidateSelector, Selector, SubClassSelector, TypeSelector};
use crate::{
    css::{CSSParse, ParseError, Parser},
    dom::{dom_objects::Element, DOMPtr},
};

/// <https://drafts.csswg.org/selectors-4/#simple>
#[derive(Clone, Debug, PartialEq)]
pub enum SimpleSelector<'a> {
    Type(TypeSelector<'a>),
    SubClass(SubClassSelector<'a>),
}

/// <https://drafts.csswg.org/selectors-4/#typedef-simple-selector-list>
pub type SimpleSelectorList<'a> = Vec<SimpleSelector<'a>>;

impl<'a> CSSParse<'a> for SimpleSelector<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-simple-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let start_state = parser.state();
        if let Ok(type_selector) = TypeSelector::parse(parser) {
            return Ok(SimpleSelector::Type(type_selector));
        }

        parser.set_state(start_state);
        if let Ok(subclass_selector) = SubClassSelector::parse(parser) {
            return Ok(SimpleSelector::SubClass(subclass_selector));
        }

        Err(ParseError)
    }
}

impl<'a> CSSParse<'a> for SimpleSelectorList<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-simple-selector-list>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        Ok(parser.parse_comma_seperated_list(SimpleSelector::parse))
    }
}

impl<'a> CSSValidateSelector for SimpleSelector<'a> {
    fn is_valid(&self) -> bool {
        match self {
            Self::Type(type_selector) => type_selector.is_valid(),
            Self::SubClass(subclass_selector) => subclass_selector.is_valid(),
        }
    }
}

impl<'a> Selector for SimpleSelector<'a> {
    fn matches(&self, element: &DOMPtr<Element>) -> bool {
        match self {
            Self::Type(type_selector) => type_selector.matches(element),
            Self::SubClass(subclass_selector) => subclass_selector.matches(element),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SimpleSelector;
    use crate::css::{
        selectors::{IDSelector, SubClassSelector, TypeSelector, WQName},
        CSSParse,
    };

    #[test]
    fn parse_simple_selector() {
        assert_eq!(
            SimpleSelector::parse_from_str("foo"),
            Ok(SimpleSelector::Type(TypeSelector::WQName(WQName {
                prefix: None,
                ident: "foo".into()
            })))
        );

        assert_eq!(
            SimpleSelector::parse_from_str("#foo"),
            Ok(SimpleSelector::SubClass(SubClassSelector::ID(IDSelector {
                ident: "foo".into()
            })))
        );
    }
}
