#!/bin/sh
# Builds the release binary on a Linux guest and packages it as a tarball, a .deb and an .rpm
# under out/dist, named for the version and the platform.
set -eu

cd "$MG_TEST_WORKSPACE"
export RUSTUP_HOME="$MG_TEST_CACHE_TOOLS/rustup" CARGO_HOME="$MG_TEST_CACHE_TOOLS/cargo"
export CARGO_TARGET_DIR="$MG_TEST_CACHE_TARGET" CARGO_TERM_COLOR=never
export PATH="$CARGO_HOME/bin:$PATH"
mkdir -p out/dist

if ! command -v cc >/dev/null || ! command -v curl >/dev/null; then
    sudo -n apt-get update
    sudo -n env DEBIAN_FRONTEND=noninteractive apt-get install -y build-essential curl
fi
[ -x "$CARGO_HOME/bin/cargo" ] || curl -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path \
    --profile minimal --default-toolchain stable
rustup update stable --no-self-update

version=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)
arch=$(uname -m)
{
    cargo build --release --locked
    command -v cargo-deb >/dev/null || cargo install --locked cargo-deb
    command -v cargo-generate-rpm >/dev/null || cargo install --locked cargo-generate-rpm
    # cargo-deb and cargo-generate-rpm read the binary from target/release.
    rm -rf target && ln -s "$CARGO_TARGET_DIR" target
    cargo deb --no-build --output out/dist/
    cargo generate-rpm --output out/dist/
} > out/build.log 2>&1

stage="out/boxmux-$version-linux-$arch"
mkdir -p "$stage"
cp "$CARGO_TARGET_DIR/release/boxmux" README.md LICENSE "$stage/"
tar -czf "out/dist/boxmux-$version-linux-$arch.tar.gz" -C out "boxmux-$version-linux-$arch"
ls -l out/dist
