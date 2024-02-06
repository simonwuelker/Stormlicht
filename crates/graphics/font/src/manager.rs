//! Manages available system fonts

use std::{fs, io, path::PathBuf, sync::LazyLock};

use crate::{
    sources::{FontStore, SystemSource},
    ttf::TTFParseError,
    Family, Font, Language, Properties, Weight,
};

#[derive(Clone, Debug)]
pub struct SystemFont {
    pub(crate) path: PathBuf,
    pub(crate) name: String,

    /// The languages targeted by this font
    pub(crate) languages: Vec<Language>,

    /// The range of weights supported by the font.
    ///
    /// The font might only support a single weight, in which case
    /// both values are identical
    pub(crate) weight_range: (Weight, Weight),
}

pub struct FontManager {
    system_fonts: Vec<SystemFont>,
}

impl FontManager {
    fn new<S>() -> Self
    where
        S: FontStore,
    {
        log::info!("Loading system fonts from store {:?}", S::NAME);
        let system_fonts = S::enumerate_system_fonts();
        log::info!("Loaded {} system fonts", system_fonts.len());
        Self { system_fonts }
    }

    pub fn lookup(&self, family: Family, properties: Properties) -> &SystemFont {
        let best_fit = self
            .system_fonts
            .iter()
            .max_by_key(|font| font.score(&family, properties))
            .expect("No font found");

        log::debug!(
            "Resolved family={family:?}, properties={properties:?} to {}",
            best_fit.path.display()
        );

        best_fit
    }
}

impl SystemFont {
    /// Ranks how well the font matches the requirements, higher is better
    fn score(&self, family: &Family, properties: Properties) -> usize {
        // FIXME: this is very ad-hoc
        let mut score = 0;

        if self
            .name
            .to_ascii_lowercase()
            .contains(&family.name().to_ascii_lowercase())
        {
            score += 10;
        }

        if self.weight_range.0 <= properties.weight && properties.weight <= self.weight_range.1 {
            score += 5;
        }

        if self.languages.contains(&properties.language) {
            score += 10;
        }

        score
    }

    pub fn try_load(&self) -> Result<Font, FontLoadError> {
        let bytes = fs::read(&self.path)?;
        let loaded_font = Font::new(&bytes)?;

        Ok(loaded_font)
    }
}

#[derive(Debug)]
pub enum FontLoadError {
    IO(io::Error),
    TrueType(TTFParseError),
}

impl From<io::Error> for FontLoadError {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<TTFParseError> for FontLoadError {
    fn from(value: TTFParseError) -> Self {
        Self::TrueType(value)
    }
}

pub static SYSTEM_FONTS: LazyLock<FontManager> = LazyLock::new(FontManager::new::<SystemSource>);
