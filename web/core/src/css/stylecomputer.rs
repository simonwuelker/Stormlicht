use std::{cmp, collections::HashSet, mem};

use crate::dom::{dom_objects::Element, DOMPtr};

use super::{
    computed_style::ComputedStyle,
    properties::Important,
    selectors::{Selector, Specificity},
    Origin, StyleProperty, StylePropertyDeclaration, StyleRule, Stylesheet,
};

#[derive(Clone, Copy, Debug)]
pub struct StyleComputer<'a> {
    stylesheets: &'a [Stylesheet],
}

#[derive(Clone, Copy, Debug)]
pub struct MatchingProperty<'stylesheets> {
    /// The property that should be applied
    property: &'stylesheets StylePropertyDeclaration,

    /// The rule that this property originated from
    rule: &'stylesheets StyleRule,

    /// The index of the matched rule within its parent [Stylesheet]
    rule_index: usize,

    /// The stylesheet that this property originated from
    stylesheet: &'stylesheets Stylesheet,
}

impl<'a> MatchingProperty<'a> {
    pub fn new(
        property: &'a StylePropertyDeclaration,
        rule: &'a StyleRule,
        rule_index: usize,
        stylesheet: &'a Stylesheet,
    ) -> Self {
        Self {
            property,
            rule,
            rule_index,
            stylesheet,
        }
    }

    fn into_property(self) -> StyleProperty {
        self.property.value.clone()
    }

    fn origin_and_importance_group(&self) -> u8 {
        match (self.property.important, self.stylesheet.origin()) {
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
        let my_specificity: Specificity = self
            .rule
            .selectors()
            .iter()
            .map(|sel| sel.specificity())
            .sum();
        let other_specificity = other
            .rule
            .selectors()
            .iter()
            .map(|sel| sel.specificity())
            .sum();
        let ordering = ordering.then(my_specificity.cmp(&other_specificity));

        // Order of appearance (https://drafts.csswg.org/css-cascade-4/#cascade-order)
        ordering
            .then(self.stylesheet.index().cmp(&other.stylesheet.index()))
            .then(self.rule_index.cmp(&other.rule_index))
    }
}

macro_rules! stylerule_to_fields {
    ($property: ident, $computed_style: ident, $(($variant: ident, $set_fn: ident)),*$(,)?) => {
        match $property {
            $(
                StyleProperty::$variant(value) => $computed_style.$set_fn(value),
            )*
        }
    };
}

impl<'a> StyleComputer<'a> {
    pub fn new(stylesheets: &'a [Stylesheet]) -> Self {
        // Sort the list in cascade order:
        // https://drafts.csswg.org/css-cascade-4/#cascade-specificity
        // https://drafts.csswg.org/css-cascade-4/#cascade-order
        Self { stylesheets }
    }

    // Find all the [StyleRules](super::StyleRule) that apply to an [Element]
    fn collect_matched_properties(&self, element: DOMPtr<Element>) -> Vec<MatchingProperty<'_>> {
        let mut matched_properties = vec![];

        for stylesheet in self.stylesheets {
            for (rule_index, rule) in stylesheet.rules().iter().enumerate() {
                if rule
                    .selectors()
                    .iter()
                    .any(|selector| selector.matches(&element))
                {
                    let new_properties = rule
                        .properties()
                        .iter()
                        .map(|prop| MatchingProperty::new(prop, rule, rule_index, stylesheet));
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

        for matched_property in matched_properties.iter().rev() {
            let property = matched_property.into_property();

            if properties_added.insert(mem::discriminant(&property)) {
                stylerule_to_fields!(
                    property,
                    computed_style,
                    (BackgroundColor, set_background_color),
                    (Color, set_color),
                    (Display, set_display),
                    (FontFamily, set_font_family),
                    (FontSize, set_font_size),
                    (Height, set_height),
                    (MarginBottom, set_margin_bottom),
                    (MarginLeft, set_margin_left),
                    (MarginRight, set_margin_right),
                    (MarginTop, set_margin_top),
                    (PaddingBottom, set_padding_bottom),
                    (PaddingLeft, set_padding_left),
                    (PaddingRight, set_padding_right),
                    (PaddingTop, set_padding_top),
                    (Width, set_width),
                    (Position, set_position)
                );
            }
        }

        computed_style
    }
}
