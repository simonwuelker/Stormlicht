#![feature(panic_update_hook, cfg_match)]

mod chrome;

use std::{process::ExitCode, sync::LazyLock};

use settings::SETTINGS;

#[cfg(all(target_os = "linux", not(miri)))]
#[link(name = "c")]
extern "C" {
    fn geteuid() -> u32;
}

pub fn main() -> ExitCode {
    // Register a custom panic handler
    std::panic::update_hook(move |prev, info| {
        eprintln!(
            "The browser has panicked. This is a bug. Please open an issue at {}, including the debug information below. Thanks!\n", 
            env!("CARGO_PKG_REPOSITORY")
        );
        prev(info);
    });

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    #[cfg(all(target_os = "linux", not(miri)))]
    if unsafe { geteuid() } == 0 {
        log::error!("Refusing to run as root");
        return ExitCode::FAILURE;
    }

    // Initialize settings object
    LazyLock::force(&SETTINGS);

    chrome::run()
}
