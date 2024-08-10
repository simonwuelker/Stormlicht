use clap::Parser;
use url::URL;

use crate::Settings;

#[derive(Parser, Debug)]
#[command(name = "Stormlicht")]
#[command(version, about, long_about = None)]
pub struct Arguments {
    /// Disable javascript execution
    #[arg(long)]
    disable_javascript: Option<bool>,

    /// URL to load initially
    #[arg(value_parser = parse_url)]
    url: Option<URL>,
}

impl Arguments {
    pub(crate) fn update_settings(self, settings: &mut Settings) {
        if let Some(disable_javascript) = self.disable_javascript {
            settings.disable_javascript = disable_javascript
        }

        if let Some(url) = self.url {
            settings.url = url;
        }
    }
}

fn parse_url(s: &str) -> Result<URL, String> {
    s.parse().map_err(|e: url::Error| format!("{e:?}"))
}
