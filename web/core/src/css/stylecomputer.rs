use std::{collections::HashSet, mem::Discriminant};

use crate::dom::{dom_objects::Element, DOMPtr};

use super::{
    properties::{DisplayValue, Important},
    selectors::Selector,
    values::{AutoOr, Length, PercentageOr},
    MatchingRule, Origin, StyleProperty, Stylesheet,
};

#[derive(Clone, Copy, Debug)]
pub struct StyleComputer<'a> {
    stylesheets: &'a [Stylesheet],
}

impl<'a> StyleComputer<'a> {
    pub fn new(stylesheets: &'a [Stylesheet]) -> Self {
        Self { stylesheets }
    }

    // Find all the [StyleRules](super::StyleRule) that apply to an [Element]
    fn collect_matching_rules(&self, element: DOMPtr<Element>) -> Vec<MatchingRule<'_>> {
        let mut matching_rules = vec![];

        for stylesheet in self.stylesheets {
            for rule in stylesheet.rules() {
                if rule.selector().matches(&element) {
                    matching_rules.push(MatchingRule::new(stylesheet.origin(), rule))
                }
            }
        }
        matching_rules
    }

    /// Find all the [StyleProperties](StyleProperty) that apply to an [Element].
    /// This includes cascading values.
    pub fn get_computed_style(&self, element: DOMPtr<Element>) -> ComputedStyle {
        let matching_rules = self.collect_matching_rules(element);

        let mut computed_style = ComputedStyle::default();

        // A rule's origin and importance defines its priority in the cascade, see
        // https://drafts.csswg.org/css-cascade-4/#cascade-sort for more info

        // FIXME: 1. Transition declarations [css-transitions-1]

        // 2. Important user agent declarations
        computed_style.add_properties(&filter_matching_rules(
            &matching_rules,
            Important::Yes,
            Origin::UserAgent,
        ));

        // 3. Important user declarations
        computed_style.add_properties(&filter_matching_rules(
            &matching_rules,
            Important::Yes,
            Origin::User,
        ));

        // 4. Important author declarations
        computed_style.add_properties(&filter_matching_rules(
            &matching_rules,
            Important::Yes,
            Origin::Author,
        ));

        // FIXME: 5. Animation declarations [css-animations-1]

        // 6. Normal author declarations
        computed_style.add_properties(&filter_matching_rules(
            &matching_rules,
            Important::No,
            Origin::Author,
        ));

        // 7. Normal user declarations
        computed_style.add_properties(&filter_matching_rules(
            &matching_rules,
            Important::No,
            Origin::User,
        ));

        // 8. Normal user agent declarations
        computed_style.add_properties(&filter_matching_rules(
            &matching_rules,
            Important::No,
            Origin::UserAgent,
        ));

        computed_style
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

impl ComputedStyle {
    pub fn add_properties(&mut self, properties: &[StyleProperty]) {
        for property in properties {
            let discriminant = std::mem::discriminant(property);
            if self.properties_set.insert(discriminant) {
                self.properties.push(property.clone());
            }
        }
    }

    add_property_lookup!(display, DisplayValue, Display);
    add_property_lookup!(margin_top, AutoOr<PercentageOr<Length>>, MarginTop);
    add_property_lookup!(margin_right, AutoOr<PercentageOr<Length>>, MarginRight);
    add_property_lookup!(margin_bottom, AutoOr<PercentageOr<Length>>, MarginBottom);
    add_property_lookup!(margin_left, AutoOr<PercentageOr<Length>>, MarginLeft);
    add_property_lookup!(width, AutoOr<PercentageOr<Length>>, Width);
    add_property_lookup!(height, AutoOr<PercentageOr<Length>>, Height);
}

fn filter_matching_rules(
    matching_rules: &[MatchingRule],
    important: Important,
    origin: Origin,
) -> Vec<StyleProperty> {
    let mut properties = vec![];
    for matching_rule in matching_rules {
        if matching_rule.origin() == origin {
            for property_declaration in matching_rule.rule().properties() {
                if property_declaration.important == important {
                    properties.push(property_declaration.value.clone())
                }
            }
        }
    }
    properties
}
