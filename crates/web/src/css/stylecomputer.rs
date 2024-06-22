use std::cmp;

use crate::{
    css::{
        computed_style::ComputedStyle,
        properties::Important,
        selectors::{Selector, Specificity},
        syntax::RuleParser,
        Origin, Parser, StyleProperty, StylePropertyDeclaration, Stylesheet,
    },
    dom::{dom_objects::Element, DomPtr},
    static_interned,
};

use super::{
    font_metrics::DEFAULT_FONT_SIZE,
    layout::{Pixels, Size},
    style::{StyleContext, ToComputedStyle},
};

#[derive(Clone, Copy, Debug)]
pub struct StyleComputer<'a> {
    stylesheets: &'a [Stylesheet],
    root_font_size: Pixels,
    viewport_size: Size<Pixels>,
}

#[derive(Clone, Debug)]
pub struct MatchingProperty<'a> {
    /// The property that should be applied
    // FIXME: This could (and should) be a reference - but in the case of "style" attributes,
    //        there exists no stylesheet that could be referenced :/
    property: &'a StylePropertyDeclaration,

    /// The specificity of the selector of this rule that matched the element
    specificity: Specificity,

    /// The index of the matched rule within its parent [Stylesheet]
    rule_index: usize,

    /// The stylesheet that this property originated from
    stylesheet_index: usize,

    // The stylesheet origin
    origin: Origin,
}

impl<'a> MatchingProperty<'a> {
    pub fn new(
        property: &'a StylePropertyDeclaration,
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

    fn property(&self) -> StyleProperty {
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
    pub fn new(
        stylesheets: &'a [Stylesheet],
        root_font_size: Pixels,
        viewport_size: Size<Pixels>,
    ) -> Self {
        // Sort the list in cascade order:
        // https://drafts.csswg.org/css-cascade-4/#cascade-specificity
        // https://drafts.csswg.org/css-cascade-4/#cascade-order
        Self {
            stylesheets,
            root_font_size,
            viewport_size,
        }
    }

    // Find all the [StyleRules](super::StyleRule) that apply to an [Element]
    fn collect_matched_properties(&self, element: DomPtr<Element>) -> Vec<MatchingProperty<'_>> {
        let mut matched_properties = vec![];

        for stylesheet in self.stylesheets {
            for (rule_index, rule) in stylesheet.rules().iter().enumerate() {
                if rule.selectors().iter().any(|s| s.matches(&element)) {
                    let new_properties = rule.properties().iter().map(|prop| {
                        // FIXME: This should be the specificity of the most-specific matching selector,
                        //        not the sum
                        let specificity = rule.selectors().iter().map(Selector::specificity).sum();

                        MatchingProperty::new(
                            prop,
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

        matched_properties
    }

    /// Find all the [StyleProperties](StyleProperty) that apply to an [Element].
    /// This includes cascading values.
    pub fn get_computed_style(
        &self,
        element: DomPtr<Element>,
        parent_style: &ComputedStyle,
    ) -> ComputedStyle {
        // If the element has a "style" attribute, create a short-lived stylesheet
        // FIXME: Can we cache this somehow?
        let attribute_style = attribute_style_for_element(element.clone());

        // NOTE: The rule and stylesheet index don't matter
        //       because the specificy is already MAX
        let attribute_style = attribute_style.iter().map(|property| {
            MatchingProperty::new(property, Specificity::MAX, 0, 0, Origin::Author)
        });

        let mut matched_properties = self.collect_matched_properties(element);
        matched_properties.extend(attribute_style);

        // Sort matching rules in cascade order, see
        // https://drafts.csswg.org/css-cascade-4/#cascade-sort for more info
        matched_properties.sort_unstable_by(MatchingProperty::compare_in_cascade_order);

        // Find the font size of the element
        // This is done seperately, as only the font-size property refers to the font size
        // of the parent instead of the element itself
        let font_size = matched_properties
            .iter()
            .rev()
            .flat_map(|prop| {
                if let StyleProperty::FontSize(font_size) = prop.property() {
                    Some(font_size)
                } else {
                    None
                }
            })
            .map(|font_size| {
                let style_context = StyleContext {
                    font_size: *parent_style.font_size(),
                    root_font_size: self.root_font_size,
                    viewport: self.viewport_size,
                };

                font_size.to_computed_style(&style_context)
            })
            .next()
            .unwrap_or(DEFAULT_FONT_SIZE);

        let style_context = StyleContext {
            font_size,
            root_font_size: self.root_font_size,
            viewport: self.viewport_size,
        };

        // Add properties in logical order (least important first)
        // That way, more important rules can override less important ones
        let mut computed_style = parent_style.get_inherited();

        for matched_property in matched_properties.into_iter() {
            let property = matched_property.property();
            computed_style.set_property(property, &style_context);
        }

        computed_style
    }
}

// Don't want to put this on `Element` since the DOM doesn't really know about CSS
fn attribute_style_for_element(element: DomPtr<Element>) -> Vec<StylePropertyDeclaration> {
    element
        .borrow()
        .attributes()
        .get(&static_interned!("style"))
        .and_then(|inline_style_string| {
            // https://html.spec.whatwg.org/multipage/dom.html#the-style-attribute
            // https://drafts.csswg.org/css-style-attr/#style-attribute

            let css_source = inline_style_string.to_string();
            let mut rule_parser = RuleParser;

            // Properties from "style" attributes have
            // * Author origin
            // * A higher specificity than any other element
            // A parse error is simply ignored (treated as if no "style" attribute was present)
            rule_parser
                .parse_qualified_rule_block(&mut Parser::new(&css_source, Origin::Author))
                .ok()
        })
        .unwrap_or_default()
}
