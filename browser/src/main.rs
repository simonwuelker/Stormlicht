mod browser_application;

use anyhow::{anyhow, Result};
use browser_application::BrowserApplication;

use cli::CommandLineArgumentParser;
use widgets::application::Application;

#[derive(Debug, Default, CommandLineArgumentParser)]
struct ArgumentParser {
    #[argument(optional, short_name = u, long_name = url)]
    _url: Option<String>,
}

#[cfg(target_os = "linux")]
#[link(name = "c")]
extern "C" {
    fn geteuid() -> u32;
}

pub fn main() -> Result<()> {
    #[cfg(target_os = "linux")]
    if unsafe { geteuid() } == 0 {
        return Err(anyhow!("Refusing to run as root"));
    }

    let mut application = BrowserApplication::default();
    application.run()
}
