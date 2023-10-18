use std::{cmp, collections::HashSet, mem::Discriminant};

use crate::dom::{dom_objects::Element, DOMPtr};

use super::{
    properties::{BackgroundColorValue, DisplayValue, Important},
    selectors::{Selector, Specificity},
    values::{color::Color, AutoOr, Length, PercentageOr},
    Origin, StyleProperty, StylePropertyDeclaration, StyleRule, Stylesheet,
};

#[derive(Clone, Copy, Debug)]
pub struct StyleComputer<'a> {
    stylesheets: &'a [Stylesheet],
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
    pub fn get_computed_style(&self, element: DOMPtr<Element>) -> ComputedStyle {
        let mut matched_properties = self.collect_matched_properties(element);

        // Sort matching rules in cascade order, see
        // https://drafts.csswg.org/css-cascade-4/#cascade-sort for more info
        matched_properties.sort_unstable_by(MatchingProperty::compare_in_cascade_order);
        let mut computed_style = ComputedStyle::default();

        // Add properties in reverse order (most important first)
        for matched_property in matched_properties.iter().rev() {
            computed_style.add_property(matched_property.into_property());
        }

        computed_style
    }
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

#[derive(Clone, Debug, Default)]
pub struct ComputedStyle {
    properties_set: HashSet<Discriminant<StyleProperty>>,
    properties: Vec<StyleProperty>,
}

macro_rules! add_property_lookup {
    ($fn_name: ident, $value_type: ty, $variant_name: ident) => {
        pub fn $fn_name(&self) -> $value_type {
            for property in &self.properties {
                if let StyleProperty::$variant_name(v) = property {
                    return *v;
                }
            }
            <$value_type>::default()
        }
    };
}

macro_rules! add_property_lookup_with_default {
    ($fn_name: ident, $value_type: ty, $variant_name: ident, $default: expr) => {
        pub fn $fn_name(&self) -> $value_type {
            for property in &self.properties {
                if let StyleProperty::$variant_name(v) = property {
                    return *v;
                }
            }
            $default
        }
    };
}

impl ComputedStyle {
    const MARGIN_DEFAULT: AutoOr<PercentageOr<Length>> =
        AutoOr::NotAuto(PercentageOr::NotPercentage(Length::ZERO));

    /// Adds a property to the set of computed values
    ///
    /// If the property is already present, it is not updated and the
    /// old value is retained.
    pub fn add_property(&mut self, property: StyleProperty) {
        let discriminant = std::mem::discriminant(&property);
        if self.properties_set.insert(discriminant) {
            self.properties.push(property);
        }
    }

    add_property_lookup!(display, DisplayValue, Display);
    add_property_lookup_with_default!(
        margin_top,
        AutoOr<PercentageOr<Length>>,
        MarginTop,
        Self::MARGIN_DEFAULT
    );
    add_property_lookup_with_default!(
        margin_right,
        AutoOr<PercentageOr<Length>>,
        MarginRight,
        Self::MARGIN_DEFAULT
    );
    add_property_lookup_with_default!(
        margin_bottom,
        AutoOr<PercentageOr<Length>>,
        MarginBottom,
        Self::MARGIN_DEFAULT
    );
    add_property_lookup_with_default!(
        margin_left,
        AutoOr<PercentageOr<Length>>,
        MarginLeft,
        Self::MARGIN_DEFAULT
    );
    add_property_lookup!(padding_top, PercentageOr<Length>, PaddingTop);
    add_property_lookup!(padding_right, PercentageOr<Length>, PaddingRight);
    add_property_lookup!(padding_bottom, PercentageOr<Length>, PaddingBottom);
    add_property_lookup!(padding_left, PercentageOr<Length>, PaddingLeft);
    add_property_lookup!(width, AutoOr<PercentageOr<Length>>, Width);
    add_property_lookup!(height, AutoOr<PercentageOr<Length>>, Height);
    add_property_lookup!(background_color, BackgroundColorValue, BackgroundColor);
    add_property_lookup_with_default!(color, Color, Color, Color::BLACK);
}
