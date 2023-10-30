use std::path;

/// Used to abstract over font source backends (fontconfig etc)
///
/// The actual backend used is known at compile time, but we need to
/// build all backends anyways to make sure they compile. This trait
/// aims to guarantee that all backends behave in the same way
pub trait FontStore {
    fn new() -> Self;
    fn lookup(&self, query: FontQuery<'_, Self>) -> MatchedFont;
}

pub struct GenericSystemStore<T> {
    store: T,
}

cfg_match! {
    cfg(target_os = "linux") => {
        mod fontconfig;

        pub use fontconfig::FontConfig;

        pub type SystemStore = GenericSystemStore<FontConfig>;
    }
    _ => {
        pub struct DummyStore;

        impl FontStore for DummyStore {
            fn new() -> Self {
                log::warn!("Failed to find font store for system, using fallback font instead");
                Self
            }

            fn lookup(&self, _query: FontQuery<'_, Self>) -> MatchedFont {
                MatchedFont::fallback()
            }
        }

        pub type SystemStore = GenericSystemStore<DummyStore>;
    }
}

impl<T: FontStore> GenericSystemStore<T> {
    pub fn query(&self) -> FontQuery<'_, T> {
        FontQuery {
            font_store: &self.store,
            name: None,
            weight: None,
            style: None,
        }
    }
}

impl<T: FontStore> Default for GenericSystemStore<T> {
    fn default() -> Self {
        Self { store: T::new() }
    }
}

#[derive(Clone, Debug)]
pub struct FontQuery<'a, T>
where
    T: ?Sized,
{
    font_store: &'a T,
    name: Option<String>,
    weight: Option<Weight>,
    style: Option<Style>,
}

impl<'a, T> FontQuery<'a, T>
where
    T: FontStore,
{
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn weight(&self) -> Option<Weight> {
        self.weight
    }

    pub fn style(&self) -> Option<Style> {
        self.style
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_weight(mut self, weight: Weight) -> Self {
        self.weight = Some(weight);
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    pub fn resolve(self) -> MatchedFont {
        self.font_store.lookup(self)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum Weight {
    #[default]
    Normal,
    Bold,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum Style {
    #[default]
    Normal,
    Italic,
}

/// The result of a FontQuery
pub struct MatchedFont {
    pub file: path::PathBuf,
}

impl MatchedFont {
    pub(crate) fn fallback() -> Self {
        const FALLBACK_FONT: &str =
            concat!(env!("DOWNLOAD_DIR"), "/fonts/roboto/Roboto-Medium.ttf");

        Self {
            file: path::PathBuf::from(FALLBACK_FONT),
        }
    }
}
