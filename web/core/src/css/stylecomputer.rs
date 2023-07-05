use crate::dom::{dom_objects::Element, DOMPtr};

use super::{selectors::Selector, MatchingRule, Stylesheet};

pub fn collect_matching_rules(
    element: DOMPtr<Element>,
    stylesheets: &[Stylesheet],
) -> Vec<MatchingRule<'_>> {
    let mut matching_rules = vec![];

    for (stylesheet_index, stylesheet) in stylesheets.iter().enumerate() {
        for (rule_index, rule) in stylesheet.rules().iter().enumerate() {
            if rule.selector().matches(&element) {
                matching_rules.push(MatchingRule::new(stylesheet_index, rule_index, rule))
            }
        }
    }
    matching_rules
}
