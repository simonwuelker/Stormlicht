use std::{cmp, collections::HashSet, mem};

use crate::{
    css::{
        computed_style::ComputedStyle,
        properties::Important,
        selectors::{Selector, Specificity},
        syntax::RuleParser,
        Origin, Parser, StyleProperty, StylePropertyDeclaration, Stylesheet,
    },
    dom::{dom_objects::Element, DOMPtr},
    static_interned,
};

#[derive(Clone, Copy, Debug)]
pub struct StyleComputer<'a> {
    stylesheets: &'a [Stylesheet],
}

#[derive(Clone, Debug)]
pub struct MatchingProperty {
    /// The property that should be applied
    // FIXME: This could (and should) be a reference - but in the case of "style" attributes,
    //        there exists no stylesheet that could be referenced :/
    property: StylePropertyDeclaration,

    /// The specificity of the selector of this rule that matched the element
    specificity: Specificity,

    /// The index of the matched rule within its parent [Stylesheet]
    rule_index: usize,

    /// The stylesheet that this property originated from
    stylesheet_index: usize,

    // The stylesheet origin
    origin: Origin,
}

impl MatchingProperty {
    pub fn new(
        property: StylePropertyDeclaration,
        specificity: Specificity,
        rule_index: usize,
        stylesheet_index: usize,
        origin: Origin,
    ) -> Self {
        Self {
            property,
            specificity,
            rule_index,
            stylesheet_index,
            origin,
        }
    }

    fn into_property(self) -> StyleProperty {
        self.property.value.clone()
    }

    fn origin_and_importance_group(&self) -> u8 {
        match (self.property.important, self.origin) {
            // 1. FIXME: Transition declarations [css-transitions-1]

            // 2. Important user agent declarations
            (Important::Yes, Origin::UserAgent) => 2,

            // 3. Important user declarations
            (Important::Yes, Origin::User) => 3,

            // 4. Important author declarations
            (Important::Yes, Origin::Author) => 4,
            // 5. FIXME: Animation declarations [css-animations-1]

            // 6. Normal author declarations
            (Important::No, Origin::Author) => 6,

            // 7. Normal user declarations
            (Important::No, Origin::User) => 7,

            // 8. Normal user agent declarations
            (Important::No, Origin::UserAgent) => 8,
        }
    }

    /// Compare two properties according to <https://drafts.csswg.org/css-cascade-4/#cascade-sort>
    pub fn compare_in_cascade_order(&self, other: &Self) -> cmp::Ordering {
        // Origin and importance (https://drafts.csswg.org/css-cascade-4/#cascade-origin)
        // NOTE: We need to reverse the ordering here because the spec uses low numbers for the important groups
        let ordering = self
            .origin_and_importance_group()
            .cmp(&other.origin_and_importance_group())
            .reverse();

        // FIXME: Context (https://drafts.csswg.org/css-cascade-4/#cascade-context)

        // Specificity (https://drafts.csswg.org/css-cascade-4/#cascade-specificity)
        let ordering = ordering.then(self.specificity.cmp(&other.specificity));

        // Order of appearance (https://drafts.csswg.org/css-cascade-4/#cascade-order)
        ordering
            .then(self.stylesheet_index.cmp(&other.stylesheet_index))
            .then(self.rule_index.cmp(&other.rule_index))
    }
}

impl<'a> StyleComputer<'a> {
    pub fn new(stylesheets: &'a [Stylesheet]) -> Self {
        // Sort the list in cascade order:
        // https://drafts.csswg.org/css-cascade-4/#cascade-specificity
        // https://drafts.csswg.org/css-cascade-4/#cascade-order
        Self { stylesheets }
    }

    // Find all the [StyleRules](super::StyleRule) that apply to an [Element]
    fn collect_matched_properties(&self, element: DOMPtr<Element>) -> Vec<MatchingProperty> {
        let mut matched_properties = vec![];

        for stylesheet in self.stylesheets {
            for (rule_index, rule) in stylesheet.rules().iter().enumerate() {
                if rule
                    .selectors()
                    .iter()
                    .any(|selector| selector.matches(&element))
                {
                    let new_properties = rule.properties().iter().map(|prop| {
                        // FIXME: This should be the specificity of the most-specific matching selector,
                        //        not the sum
                        let specificity = rule.selectors().iter().map(Selector::specificity).sum();

                        MatchingProperty::new(
                            prop.clone(),
                            specificity,
                            rule_index,
                            stylesheet.index(),
                            stylesheet.origin(),
                        )
                    });
                    matched_properties.extend(new_properties);
                }
            }
        }

        // Add the inline style (from the "style" attribute), if any
        // FIXME: Can we cache this somehow?
        if let Some(inline_style) = element
            .borrow()
            .attributes()
            .get(&static_interned!("style"))
        {
            // https://html.spec.whatwg.org/multipage/dom.html#the-style-attribute
            // https://drafts.csswg.org/css-style-attr/#style-attribute

            let css_source = inline_style.to_string();
            let mut rule_parser = RuleParser;

            // Properties from "style" attributes have
            // * Author origin
            // * A higher specificity than any other element
            if let Ok(properties) = rule_parser
                .parse_qualified_rule_block(&mut Parser::new(&css_source, Origin::Author))
            {
                // NOTE: The rule and stylesheet index don't matter
                //       because the specificy is already MAX
                let inline_properties = properties.into_iter().map(|property| {
                    MatchingProperty::new(property, Specificity::MAX, 0, 0, Origin::Author)
                });
                matched_properties.extend(inline_properties);
            }
        }
        matched_properties
    }

    /// Find all the [StyleProperties](StyleProperty) that apply to an [Element].
    /// This includes cascading values.
    pub fn get_computed_style(
        &self,
        element: DOMPtr<Element>,
        parent_style: &ComputedStyle,
    ) -> ComputedStyle {
        let mut matched_properties = self.collect_matched_properties(element);

        // Sort matching rules in cascade order, see
        // https://drafts.csswg.org/css-cascade-4/#cascade-sort for more info
        matched_properties.sort_unstable_by(MatchingProperty::compare_in_cascade_order);
        let mut computed_style = parent_style.get_inherited();

        // Add properties in reverse order (most important first)
        let mut properties_added = HashSet::new();

        for matched_property in matched_properties.into_iter().rev() {
            let property = matched_property.into_property();

            if properties_added.insert(mem::discriminant(&property)) {
                computed_style.set_property(property);
            }
        }

        computed_style
    }
}
