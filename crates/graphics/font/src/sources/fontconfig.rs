use super::{FontQuery, FontStore, MatchedFont, Style, Weight};

pub struct FontConfig {
    config: fontconfig::Config,
}

impl FontStore for FontConfig {
    fn new() -> Self {
        log::info!("Using fontconfig version {}", fontconfig::Version::get());

        Self {
            config: fontconfig::Config::init(),
        }
    }

    fn lookup(&self, query: FontQuery<'_, Self>) -> MatchedFont {
        // Build a fontconfig pattern from the query
        let pattern = fontconfig::Pattern::default();

        if let Some(name) = query.name() {
            pattern.add_string(fontconfig::objects::FC_FAMILY, name);
        }

        if let Some(weight) = query.weight() {
            let fc_value = match weight {
                Weight::Normal => fontconfig::consts::FC_WEIGHT_REGULAR,
                Weight::Bold => fontconfig::consts::FC_WEIGHT_BOLD,
            };
            pattern.add_int(fontconfig::objects::FC_WEIGHT, fc_value);
        }

        if let Some(style) = query.style() {
            let fc_value = match style {
                Style::Normal => fontconfig::consts::FC_SLANT_ROMAN,
                Style::Italic => fontconfig::consts::FC_SLANT_ITALIC,
            };
            pattern.add_int(fontconfig::objects::FC_SLANT, fc_value);
        }

        self.lookup_internal(pattern).unwrap_or_else(|error| {
            log::error!("Font lookup failed: {error:?}");
            MatchedFont::fallback()
        })
    }
}

impl FontConfig {
    fn lookup_internal(
        &self,
        pattern: fontconfig::Pattern,
    ) -> Result<MatchedFont, fontconfig::bindings::LookupError> {
        let matched_pattern = self.config.best_match_for_pattern(&pattern)?;
        let filename = matched_pattern.get_string(fontconfig::objects::FC_FILE)?;

        let mut base = self.config.system_root().unwrap_or_default();
        base.push(filename);

        Ok(MatchedFont { file: base })
    }
}
