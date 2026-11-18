//! Stamps the commit BoxMux is built from into `boxmux --version` (`BOXMUX_COMMIT`), and bakes
//! in the release line a release build is given (`BOXMUX_BAKED_RELEASE_LINE`), which
//! `boxmux upgrade` follows.

#[path = "build/commit.rs"]
mod commit;
#[path = "build/release_line.rs"]
mod release_line;

use std::path::PathBuf;

fn main() {
    let root = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("cargo sets it"));
    let commit = commit::describe(&root).unwrap_or_else(|e| panic!("{}", e));
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=build/commit.rs");
    println!("cargo:rerun-if-changed=build/release_line.rs");
    for path in &commit.watch {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    println!("cargo:rustc-env=BOXMUX_COMMIT={}", commit.stamp);

    for name in release_line::INPUTS {
        println!("cargo:rerun-if-env-changed={}", name);
    }
    let line =
        release_line::read(&|name| std::env::var(name).ok()).unwrap_or_else(|e| panic!("{}", e));
    if let Some(line) = line {
        println!("cargo:rustc-env={}={}", release_line::BAKED, line);
    }
}
