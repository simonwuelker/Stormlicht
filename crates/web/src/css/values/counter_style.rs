/// <https://drafts.csswg.org/css-counter-styles-3/#typedef-counter-style-name>
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CounterStyle {
    /// <https://drafts.csswg.org/css-counter-styles-3/#decimal>
    Decimal,

    /// <https://drafts.csswg.org/css-counter-styles-3/#disc>
    Disc,

    /// <https://drafts.csswg.org/css-counter-styles-3/#square>
    Square,

    /// <https://drafts.csswg.org/css-counter-styles-3/#disclosure-open>
    DisclosureOpen,

    /// <https://drafts.csswg.org/css-counter-styles-3/#disclosure-closed>
    DisclosureClosed,
}

impl CounterStyle {
    #[must_use]
    pub fn as_str(&self) -> String {
        // FIXME: I don't think the spacing between
        //        the ::marker element and the list-item
        //        is specified - valid to use NBSP (U+00A0) here?
        match self {
            Self::Decimal | // FIXME: implement decimal
            Self::Disc => String::from("•\u{00A0}"),
            Self::Square => String::from("▪\u{00A0}"),
            Self::DisclosureOpen => String::from("▾\u{00A0}"),
            // FIXME: This should respect the writing type
            Self::DisclosureClosed => String::from("▸\u{00A0}")
        }
    }
}
