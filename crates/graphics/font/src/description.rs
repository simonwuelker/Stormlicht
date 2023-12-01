#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Family {
    Specific(String),
    Generic(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct FontCacheKey {
    pub family: Family,
    pub style: Style,
}

#[derive(Clone, Copy, Debug)]
pub struct Properties {
    pub style: Style,
    pub weight: Weight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Style {
    Normal,
    Italic,
    /// Font is slanted by the specified number of degrees
    Oblique(i8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Weight(pub u16);

impl Weight {
    pub const THIN: Self = Self(100);
    pub const EXTRA_LIGHT: Self = Self(200);
    pub const LIGHT: Self = Self(300);
    pub const NORMAL: Self = Self(400);
    pub const MEDIUM: Self = Self(500);
    pub const SEMI_BOLD: Self = Self(600);
    pub const BOLD: Self = Self(700);
    pub const EXTRA_BOLD: Self = Self(800);
    pub const BLACK: Self = Self(900);
    pub const EXTRA_BLACK: Self = Self(950);
}

impl Family {
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Generic(generic_name) => generic_name,
            Self::Specific(specific_name) => specific_name,
        }
    }
}
