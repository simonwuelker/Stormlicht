//! Manages loaded fonts within Stormlicht

use std::sync::LazyLock;

use font::Font;

pub static FONT_CACHE: LazyLock<FontCache> = LazyLock::new(|| {
    log::info!("Initializing font cache");
    FontCache::initial()
});

/// Eventually, this should manage the loaded/unloaded fonts
///
/// For now, it simply holds the one supported font face.
pub struct FontCache {
    /// The [Font] that should be used if no other matching font could be found
    fallback_font: Font,
}

impl FontCache {
    /// Construct the initial cache
    ///
    /// This isn't implemented using [Default], to make
    /// it impossible for users to construct their own font cache.
    fn initial() -> Self {
        Self {
            fallback_font: Font::default(),
        }
    }

    #[inline]
    #[must_use]
    pub fn fallback(&self) -> &Font {
        &self.fallback_font
    }
}
