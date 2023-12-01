use crate::SystemFont;

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
        pub struct Dummy;

        const FALLBACK_PATH: &str = concat!(
                    env!("DOWNLOAD_DIR"),
                    "/fonts/roboto/Roboto-Medium.ttf"
                );

        impl FontStore for Dummy {
            fn enumerate_system_fonts() -> Vec<SystemFont> {
                let fallback_font = SystemFont {
                    path: PathBuf::from(FALLBACK_PATH),
                    name: "Roboto-Medium".to_string()
                };

                vec![
                    fallback_font
                ]
            }
        }

        pub type SystemSource = Dummy;
    }
}
