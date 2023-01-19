use std::{env, path::Path, process::Command};

const DICT_URL: &str = "https://github.com/google/brotli/raw/master/c/common/dictionary.bin";

fn main() {
    // Only rerun if this build script changes
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("dictionary.bin");

    // Download the static brotli dictionary
    Command::new("wget")
        .args(["-nv", "-O", dest_path.to_str().unwrap(), DICT_URL])
        .spawn()
        .expect("failed to download brotli dictionary");
}
