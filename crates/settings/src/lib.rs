//! Responsible for creating and managing the global stormlicht settings

mod cli;

use std::{net, sync::LazyLock};

use clap::Parser;
use url::URL;

/// The global settings singleton
pub static SETTINGS: LazyLock<Settings> = LazyLock::new(Settings::init);

const WELCOME_PAGE: &str = concat!(
    "file://localhost/",
    env!("CARGO_MANIFEST_DIR"),
    "/../pages/welcome.html"
);

/// Holds all the configurable information for a stormlicht instance
#[derive(Debug)]
pub struct Settings {
    pub disable_javascript: bool,

    /// URL to load initially
    pub url: URL,

    /// Proxy for networking
    pub proxy: Option<net::SocketAddr>,
}

impl Settings {
    #[must_use]
    pub fn init() -> Self {
        let mut settings = Self::default();

        let args = cli::Arguments::parse();

        args.update_settings(&mut settings);

        settings
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            disable_javascript: false,
            url: WELCOME_PAGE.parse().expect("welcome page is a valid url"),
            proxy: None,
        }
    }
}
