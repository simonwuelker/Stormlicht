use std::net;
use url::URL;

use crate::Settings;

#[derive(clap::Parser, Debug)]
#[command(name = "Stormlicht", version, about="A modern browser engine", long_about = None)]
pub struct Arguments {
    /// Disable javascript execution
    #[clap(
        long,
        action = clap::ArgAction::SetTrue,
    )]
    disable_javascript: bool,

    /// URL to load initially
    #[arg(value_parser = parse_url, value_hint = clap::ValueHint::Url)]
    url: Option<URL>,

    /// Proxy for http requests
    #[arg(long, value_parser = parse_socketaddr)]
    proxy: Option<net::SocketAddr>,
}

impl Arguments {
    pub(crate) fn update_settings(self, settings: &mut Settings) {
        settings.disable_javascript = settings.disable_javascript;

        if let Some(url) = self.url {
            settings.url = url;
        }

        if let Some(proxy) = self.proxy {
            settings.proxy = Some(proxy);
        }
    }
}

fn parse_url(s: &str) -> Result<URL, String> {
    s.parse().map_err(|e: url::Error| format!("{e:?}"))
}

fn parse_socketaddr(s: &str) -> Result<net::SocketAddr, String> {
    s.parse()
        .map_err(|e: <net::SocketAddr as std::str::FromStr>::Err| format!("{e}"))
}
