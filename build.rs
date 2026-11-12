//! Stamps the commit BoxMux is built from into `boxmux --version` (`BOXMUX_COMMIT`).

#[path = "build/commit.rs"]
mod commit;

use std::path::PathBuf;

fn main() {
    let root = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("cargo sets it"));
    let commit = commit::describe(&root).unwrap_or_else(|e| panic!("{}", e));
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=build/commit.rs");
    for path in &commit.watch {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    println!("cargo:rustc-env=BOXMUX_COMMIT={}", commit.stamp);
}
