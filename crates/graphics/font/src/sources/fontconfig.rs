use crate::{Language, SystemFont, Weight};

use super::FontStore;

pub struct FontConfig;

impl FontStore for FontConfig {
    const NAME: &'static str = "FontConfig";

    fn enumerate_system_fonts() -> Vec<SystemFont> {
        let config = fontconfig::Config::init();

        let pattern = fontconfig::Pattern::default();
        // FIXME: Remove this once we support all common font formats
        pattern.add_string(fontconfig::objects::FC_FONTFORMAT, "truetype");

        let mut os = fontconfig::ObjectSet::default();
        os.add_object(fontconfig::objects::FC_FAMILY);
        os.add_object(fontconfig::objects::FC_FILE);
        os.add_object(fontconfig::objects::FC_STYLE);
        os.add_object(fontconfig::objects::FC_FILE);
        os.add_object(fontconfig::objects::FC_WEIGHT);
        os.add_object(fontconfig::objects::FC_LANG);

        let system_base_path = config.system_root().unwrap_or_default();

        config
            .matching_fonts(pattern, os)
            .iter()
            .map(|pattern| {
                let path = system_base_path.join(
                    pattern
                        .get_string(fontconfig::objects::FC_FILE)
                        .expect("Could not read FC_FILE key"),
                );
                let name = pattern
                    .get_string(fontconfig::objects::FC_FAMILY)
                    .expect("Could not read FC_FAMILY key")
                    .to_owned();

                let lang_set = pattern
                    .get_lang_set(fontconfig::objects::FC_LANG)
                    .expect("Could not read FC_LANG key");
                let languages = fontconfig_lang_to_languages(lang_set);

                let weight = pattern
                    .get(fontconfig::objects::FC_WEIGHT)
                    .expect("Could not read FC_WEIGHT key");

                let weight_range = match weight {
                    fontconfig::Value::Double(double) => {
                        let supported_weight = Weight(double.round() as u16);
                        (supported_weight, supported_weight)
                    },
                    fontconfig::Value::Range(range) => {
                        let (begin, end) = range.get_bounds();

                        (Weight(begin.round() as u16), Weight(end.round() as u16))
                    },
                    other => {
                        panic!("Invalid font weight range: Expected Range or Double, got {other:?}")
                    },
                };

                SystemFont {
                    path,
                    name,
                    languages,
                    weight_range,
                }
            })
            .collect()
    }
}

fn fontconfig_lang_to_languages(lang_set: fontconfig::LangSet) -> Vec<Language> {
    let mut languages = vec![];

    // FIXME: This is very ad-hoc, we probably want to use a binding for
    //        https://fontconfig.pages.freedesktop.org/fontconfig/fontconfig-devel/fcgetlangs.html
    //        here

    if lang_set.contains_language("en") {
        languages.push(Language::English);
    }

    languages
}
