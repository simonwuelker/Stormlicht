use std::{collections::HashSet, iter, mem::Discriminant};

use crate::dom::{dom_objects::Element, DOMPtr};

use super::{
    properties::{BackgroundColorValue, DisplayValue, Important},
    selectors::Selector,
    values::{color::Color, AutoOr, Length, PercentageOr},
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
                if rule
                    .selectors()
                    .iter()
                    .any(|selector| selector.matches(&element))
                {
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
        computed_style.extend(filter_matching_rules(
            &matching_rules,
            Important::Yes,
            Origin::UserAgent,
        ));

        // 3. Important user declarations
        computed_style.extend(filter_matching_rules(
            &matching_rules,
            Important::Yes,
            Origin::User,
        ));

        // 4. Important author declarations
        computed_style.extend(filter_matching_rules(
            &matching_rules,
            Important::Yes,
            Origin::Author,
        ));

        // FIXME: 5. Animation declarations [css-animations-1]

        // 6. Normal author declarations
        computed_style.extend(filter_matching_rules(
            &matching_rules,
            Important::No,
            Origin::Author,
        ));

        // 7. Normal user declarations
        computed_style.extend(filter_matching_rules(
            &matching_rules,
            Important::No,
            Origin::User,
        ));

        // 8. Normal user agent declarations
        computed_style.extend(filter_matching_rules(
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

impl iter::Extend<StyleProperty> for ComputedStyle {
    fn extend<T: IntoIterator<Item = StyleProperty>>(&mut self, iter: T) {
        for elem in iter {
            self.add_property(elem)
        }
    }

    fn extend_one(&mut self, item: StyleProperty) {
        self.add_property(item);
    }

    fn extend_reserve(&mut self, additional: usize) {
        self.properties.reserve(additional);
        self.properties_set.reserve(additional);
    }
}

fn filter_matching_rules<'rules>(
    matching_rules: &'rules [MatchingRule],
    important: Important,
    origin: Origin,
) -> impl Iterator<Item = StyleProperty> + 'rules {
    matching_rules
        .iter()
        .filter(move |rule| rule.origin() == origin)
        .flat_map(|rule| rule.rule().properties())
        .filter(move |property| property.important == important)
        .map(|property| &property.value)
        .cloned()
}
