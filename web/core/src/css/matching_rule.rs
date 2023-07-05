use std::cmp::Ordering;

use super::StyleRule;

#[derive(Clone, Copy, Debug)]
pub struct MatchingRule<'a> {
    stylesheet_index: usize,
    rule_index: usize,
    /// The referenced rule
    rule: &'a StyleRule,
}

impl<'a> MatchingRule<'a> {
    pub fn new(stylesheet_index: usize, rule_index: usize, rule: &'a StyleRule) -> Self {
        Self {
            stylesheet_index,
            rule_index,
            rule,
        }
    }

    pub fn rule(&self) -> &'a StyleRule {
        self.rule
    }
}

impl<'a> PartialEq for MatchingRule<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.stylesheet_index.eq(&other.stylesheet_index) && self.rule_index.eq(&other.rule_index)
    }
}

impl<'a> Eq for MatchingRule<'a> {}

impl<'a> PartialOrd for MatchingRule<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for MatchingRule<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        let stylesheet_ordering = self.stylesheet_index.cmp(&other.stylesheet_index);

        if stylesheet_ordering == Ordering::Equal {
            self.rule_index.cmp(&other.rule_index)
        } else {
            stylesheet_ordering
        }
    }
}
