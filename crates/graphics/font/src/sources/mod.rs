use crate::{Language, SystemFont};

/// Used to abstract over font source backends (fontconfig etc)
pub trait FontStore {
    const NAME: &'static str;

    fn enumerate_system_fonts() -> Vec<SystemFont>;
}

cfg_match! {
    cfg(target_os = "linux") => {
        mod fontconfig;

        pub type SystemSource = fontconfig::FontConfig;
    }
    _ => {
        use crate::Weight;

        use std::path::PathBuf;

        pub struct Dummy;

        const FALLBACK_PATH: &str = concat!(
                    env!("DOWNLOAD_DIR"),
                    "/fonts/roboto/Roboto-Medium.ttf"
                );

        impl FontStore for Dummy {
            const NAME: &'static str = "Dummy";

            fn enumerate_system_fonts() -> Vec<SystemFont> {
                let fallback_font = SystemFont {
                    path: PathBuf::from(FALLBACK_PATH),
                    name: "Roboto-Medium".to_string(),
                    languages: vec![Language::English],
                    weight_range: (Weight::NORMAL, Weight::NORMAL),
                };

                vec![
                    fallback_font
                ]
            }
        }

        pub type SystemSource = Dummy;
    }
}
