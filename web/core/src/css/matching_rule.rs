use super::{Origin, StyleRule};

#[derive(Clone, Copy, Debug)]
pub struct MatchingRule<'a> {
    origin: Origin,

    /// The referenced rule
    rule: &'a StyleRule,
}

impl<'a> MatchingRule<'a> {
    pub fn new(origin: Origin, rule: &'a StyleRule) -> Self {
        Self { origin, rule }
    }

    pub fn origin(&self) -> Origin {
        self.origin
    }

    pub fn rule(&self) -> &'a StyleRule {
        self.rule
    }
}
