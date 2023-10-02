use std::process::Command;

fn main() {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("Failed to read git version");
    let git_hash = String::from_utf8(output.stdout).expect("git version contains non-utf8 data");

    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
    println!(
        "cargo:rustc-env=TARGET_TRIPLE={}",
        std::env::var("TARGET").unwrap()
    );
    println!(
        "cargo:rustc-env=RUSTC_VERSION={}",
        std::env::var("RUSTC").unwrap()
    );
}
