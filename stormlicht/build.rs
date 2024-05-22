use std::{env, process};

fn main() {
    // Register "chrome" cfg
    println!("cargo::rustc-check-cfg=cfg(chrome, values(\"glazier\", \"gtk\"))");

    get_environment_info();

    #[cfg(chrome = "gtk")]
    compile_glib_resources();
}

fn get_environment_info() {
    let output = process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("Failed to read git version");
    let git_hash = String::from_utf8(output.stdout).expect("git version contains non-utf8 data");

    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
    println!(
        "cargo:rustc-env=TARGET_TRIPLE={}",
        env::var("TARGET").unwrap()
    );
    println!(
        "cargo:rustc-env=RUSTC_VERSION={}",
        env::var("RUSTC").unwrap()
    );
}

#[cfg(chrome = "gtk")]
fn compile_glib_resources() {
    glib_build_tools::compile_resources(
        &["resources"],
        "resources/gresources.xml",
        "composite_template.gresource",
    );
}
