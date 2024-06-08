use crate::{
    css::{
        layout::Pixels,
        style::{computed, StyleContext, ToComputedStyle},
        syntax::Token,
        values::{Number, Percentage},
        CSSParse, ParseError, Parser,
    },
    static_interned, InternedString,
};

use std::ops::Mul;

/// <https://www.w3.org/TR/css-values-4/#length-value>
#[derive(Clone, Copy, Debug)]
pub struct Length {
    value: f32,
    unit: Unit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Unit {
    // Font-relative units
    /// <https://www.w3.org/TR/css-values-4/#em>
    Em,
    /// <https://www.w3.org/TR/css-values-4/#rem>
    Rem,
    /// <https://www.w3.org/TR/css-values-4/#ex>
    Ex,
    /// <https://www.w3.org/TR/css-values-4/#rex>
    Rex,
    /// <https://www.w3.org/TR/css-values-4/#cap>
    Cap,
    /// <https://www.w3.org/TR/css-values-4/#rcap>
    Rcap,
    /// <https://www.w3.org/TR/css-values-4/#ch>
    Ch,
    /// <https://www.w3.org/TR/css-values-4/#rch>
    Rch,
    /// <https://www.w3.org/TR/css-values-4/#ic>
    Ic,
    /// <https://www.w3.org/TR/css-values-4/#ric>
    Ric,
    /// <https://www.w3.org/TR/css-values-4/#lh>
    Lh,
    /// <https://www.w3.org/TR/css-values-4/#rlh>
    Rlh,

    // Viewport-relative units
    /// <https://www.w3.org/TR/css-values-4/#vw>
    Vw,
    /// <https://www.w3.org/TR/css-values-4/#svw>
    Svw,
    /// <https://www.w3.org/TR/css-values-4/#lvw>
    Lvw,
    /// <https://www.w3.org/TR/css-values-4/#dvw>
    Dvw,
    /// <https://www.w3.org/TR/css-values-4/#vh>
    Vh,
    /// <https://www.w3.org/TR/css-values-4/#svh>
    Svh,
    /// <https://www.w3.org/TR/css-values-4/#lvh>
    Lvh,
    /// <https://www.w3.org/TR/css-values-4/#dvh>
    Dvh,
    /// <https://www.w3.org/TR/css-values-4/#vi>
    Vi,
    /// <https://www.w3.org/TR/css-values-4/#svi>
    Svi,
    /// <https://www.w3.org/TR/css-values-4/#lvi>
    Lvi,
    /// <https://www.w3.org/TR/css-values-4/#dvi>
    Dvi,
    /// <https://www.w3.org/TR/css-values-4/#vb>
    Vb,
    /// <https://www.w3.org/TR/css-values-4/#svb>
    Svb,
    /// <https://www.w3.org/TR/css-values-4/#lvb>
    Lvb,
    /// <https://www.w3.org/TR/css-values-4/#dvb>
    Dvb,
    /// <https://www.w3.org/TR/css-values-4/#vmin>
    Vmin,
    /// <https://www.w3.org/TR/css-values-4/#svmin>
    Svmin,
    /// <https://www.w3.org/TR/css-values-4/#lvmin>
    Lvmin,
    /// <https://www.w3.org/TR/css-values-4/#dvmin>
    Dvmin,
    /// <https://www.w3.org/TR/css-values-4/#vmax>
    Vmax,
    /// <https://www.w3.org/TR/css-values-4/#svmax>
    Svmax,
    /// <https://www.w3.org/TR/css-values-4/#lvmax>,
    Lvmax,
    /// <https://www.w3.org/TR/css-values-4/#dvmax>
    Dvmax,

    // Absolute units
    /// <https://www.w3.org/TR/css-values-4/#cm>
    Cm,
    /// <https://www.w3.org/TR/css-values-4/#mm>
    Mm,
    /// <https://www.w3.org/TR/css-values-4/#Q>
    Q,
    /// <https://www.w3.org/TR/css-values-4/#in>
    In,
    /// <https://www.w3.org/TR/css-values-4/#pc>
    Pc,
    /// <https://www.w3.org/TR/css-values-4/#pt>
    Pt,
    /// <https://www.w3.org/TR/css-values-4/#px>
    Px,
}

impl Length {
    pub const ZERO: Self = Self {
        value: 0.,
        unit: Unit::Px,
    };

    #[must_use]
    pub const fn pixels(pixels: Pixels) -> Self {
        Self {
            value: pixels.0,
            unit: Unit::Px,
        }
    }

    pub fn from_dimension(value: Number, unit_name: InternedString) -> Result<Self, ParseError> {
        let length = Self {
            value: value.into(),
            unit: Unit::try_from(unit_name)?,
        };

        Ok(length)
    }
}

impl From<Pixels> for Length {
    fn from(value: Pixels) -> Self {
        Self {
            value: value.0,
            unit: Unit::Px,
        }
    }
}

impl TryFrom<InternedString> for Unit {
    type Error = ParseError;

    fn try_from(value: InternedString) -> Result<Self, Self::Error> {
        match value {
            static_interned!("em") => Ok(Self::Em),
            static_interned!("rem") => Ok(Self::Rem),
            static_interned!("ex") => Ok(Self::Ex),
            static_interned!("rex") => Ok(Self::Rex),
            static_interned!("cap") => Ok(Self::Cap),
            static_interned!("rcap") => Ok(Self::Rcap),
            static_interned!("ch") => Ok(Self::Ch),
            static_interned!("rch") => Ok(Self::Rch),
            static_interned!("ic") => Ok(Self::Ic),
            static_interned!("ric") => Ok(Self::Ric),
            static_interned!("lh") => Ok(Self::Lh),
            static_interned!("rlh") => Ok(Self::Rlh),
            static_interned!("vw") => Ok(Self::Vw),
            static_interned!("svw") => Ok(Self::Svw),
            static_interned!("lvw") => Ok(Self::Lvw),
            static_interned!("dvw") => Ok(Self::Dvw),
            static_interned!("vh") => Ok(Self::Vh),
            static_interned!("svh") => Ok(Self::Svh),
            static_interned!("lvh") => Ok(Self::Lvh),
            static_interned!("dvh") => Ok(Self::Dvh),
            static_interned!("vi") => Ok(Self::Vi),
            static_interned!("svi") => Ok(Self::Svi),
            static_interned!("lvi") => Ok(Self::Lvi),
            static_interned!("dvi") => Ok(Self::Dvi),
            static_interned!("vb") => Ok(Self::Vb),
            static_interned!("svb") => Ok(Self::Svb),
            static_interned!("lvb") => Ok(Self::Lvb),
            static_interned!("dvb") => Ok(Self::Dvb),
            static_interned!("vmin") => Ok(Self::Vmin),
            static_interned!("svmin") => Ok(Self::Svmin),
            static_interned!("lvmin") => Ok(Self::Lvmin),
            static_interned!("dvmin") => Ok(Self::Dvmin),
            static_interned!("vmax") => Ok(Self::Vmax),
            static_interned!("svmax") => Ok(Self::Svmax),
            static_interned!("lvmax") => Ok(Self::Lvmax),
            static_interned!("dvmax") => Ok(Self::Dvmax),
            static_interned!("cm") => Ok(Self::Cm),
            static_interned!("mm") => Ok(Self::Mm),
            static_interned!("q") => Ok(Self::Q),
            static_interned!("in") => Ok(Self::In),
            static_interned!("pc") => Ok(Self::Pc),
            static_interned!("pt") => Ok(Self::Pt),
            static_interned!("px") => Ok(Self::Px),
            _ => {
                // Unknown length unit
                Err(ParseError)
            },
        }
    }
}

impl<'a> CSSParse<'a> for Length {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        match parser.next_token_ignoring_whitespace() {
            Some(Token::Dimension(number, unit_name)) => {
                let length = Self::from_dimension(number, unit_name)?;
                Ok(length)
            },
            Some(Token::Number(number)) if number.is_zero() => Ok(Self {
                value: 0.,
                unit: Unit::Px,
            }),
            _ => Err(ParseError),
        }
    }
}

impl Mul<Percentage> for Length {
    type Output = Self;

    fn mul(self, rhs: Percentage) -> Self::Output {
        Self {
            value: self.value * rhs.as_fraction(),
            unit: self.unit,
        }
    }
}

impl Default for Length {
    fn default() -> Self {
        Self::ZERO
    }
}

impl ToComputedStyle for Length {
    type Computed = computed::Length;

    fn to_computed_style(&self, context: StyleContext) -> Self::Computed {
        match self.unit {
            Unit::Cm => Pixels(self.value * 96. / 2.54),
            Unit::Mm => Pixels(self.value * 96. / 2.54 / 10.),
            Unit::Q => Pixels(self.value * 96. / 2.54 / 40.),
            Unit::In => Pixels(self.value * 96.),
            Unit::Pc => Pixels(self.value * 96. / 6.),
            Unit::Pt => Pixels(self.value * 96. / 72.),
            Unit::Px => Pixels(self.value),

            // Viewport-relative units
            Unit::Vw
            | Unit::Svw
            | Unit::Lvw
            | Unit::Dvw
            | Unit::Vi
            | Unit::Svi
            | Unit::Lvi
            | Unit::Dvi => context.viewport.width / 100. * self.value,
            Unit::Vh
            | Unit::Svh
            | Unit::Lvh
            | Unit::Dvh
            | Unit::Vb
            | Unit::Svb
            | Unit::Lvb
            | Unit::Dvb => context.viewport.height / 100. * self.value,
            Unit::Vmin | Unit::Svmin | Unit::Lvmin | Unit::Dvmin => {
                if context.viewport.width < context.viewport.height {
                    (context.viewport.width / 100.) * self.value
                } else {
                    (context.viewport.height / 100.) * self.value
                }
            },
            Unit::Vmax | Unit::Svmax | Unit::Lvmax | Unit::Dvmax => {
                if context.viewport.width < context.viewport.height {
                    (context.viewport.height / 100.) * self.value
                } else {
                    (context.viewport.width / 100.) * self.value
                }
            },

            // Font-relative units
            Unit::Em => context.font_size * self.value,
            Unit::Rem => context.root_font_size * self.value,
            _ => todo!("absolutize font-relative length: {self:?}"),
        }
    }
}
