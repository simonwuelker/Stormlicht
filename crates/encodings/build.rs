#![feature(exit_status_error)]

use buildutils::PYTHON;
use std::{env, path::PathBuf, process::Command};

pub fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let download_dir = env::var_os("DOWNLOAD_DIR").unwrap();

    println!("cargo:rerun-if-changed=templates");
    println!("cargo:rerun-if-changed=build.py");
    println!(
        "cargo:rerun-if-changed={}",
        PathBuf::from(&download_dir)
            .join("encodings.json")
            .display()
    );

    Command::new(PYTHON.as_str())
        .args(&["build.py".into(), out_dir, download_dir])
        .status()
        .expect("Failed to start python build script")
        .exit_ok()
        .expect("Failed to run python build script");
}
