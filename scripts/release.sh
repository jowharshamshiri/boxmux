#!/bin/sh
# Releases the version in Cargo.toml: checks that everything is ready, builds the release
# binaries and packages on mg-test (plan mg-test/release.yaml), then, after confirmation,
# tags the release, publishes the crate and creates the GitHub release with the packages.
#
# Usage: scripts/release.sh [--dry-run]
#   --dry-run  stop after building; nothing is tagged, pushed or published.
#
# Needs: git, cargo, mg-test (logged in), gh (authenticated) and sha256sum or shasum.
set -eu

dry_run=false
case "${1:-}" in
    --dry-run) dry_run=true ;;
    "") ;;
    *) echo "usage: $0 [--dry-run]" >&2; exit 2 ;;
esac

die() {
    echo "release: $*" >&2
    exit 1
}

cd "$(dirname "$0")/.."
mg_test=${MG_TEST:-mg-test}
version=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)
[ -n "$version" ] || die "no version in Cargo.toml"
tag="v$version"

# The release is made from a clean, pushed main.
[ "$(git rev-parse --abbrev-ref HEAD)" = main ] || die "not on main"
[ -z "$(git status --porcelain)" ] || die "the working tree has changes"
git fetch --quiet origin main
[ "$(git rev-parse HEAD)" = "$(git rev-parse origin/main)" ] || die "main is not the same as origin/main"
if git rev-parse -q --verify "refs/tags/$tag" >/dev/null; then
    die "$tag already exists"
fi
grep -q "^## \[$version\]" CHANGELOG.md || die "CHANGELOG.md has no section for $version"

echo "== checks"
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo package --locked --list >/dev/null
cargo publish --locked --dry-run

echo "== building on mg-test"
run=$($mg_test run mg-test.yaml --plan release --detach | tail -n 1)
echo "run $run"
$mg_test attach "$run" || true
$mg_test runs "$run" | head -n 1 | grep -q ' passed ' || die "the release build $run did not pass"
rm -rf dist
mkdir -p dist
$mg_test artifacts "$run" --products --download dist
# The products keep their task directories; flatten them.
find dist -mindepth 2 -type f -exec mv {} dist/ \;
find dist -mindepth 1 -type d -empty -delete
(
    cd dist
    if command -v sha256sum >/dev/null; then sha256sum -- * >SHA256SUMS; else shasum -a 256 -- * >SHA256SUMS; fi
)
ls -l dist

notes=$(mktemp)
trap 'rm -f "$notes"' EXIT
# The changelog section of this version, without its heading.
awk -v v="$version" '
    $0 ~ "^## \\[" v "\\]" { found = 1; next }
    found && /^## \[/ { exit }
    found { print }
' CHANGELOG.md >"$notes"

if $dry_run; then
    echo "dry run: dist/ holds the packages; nothing was published"
    exit 0
fi

printf 'Publish %s (tag, crates.io, GitHub release)? [y/N] ' "$version"
read -r answer
[ "$answer" = y ] || die "not published"

git tag -a "$tag" -m "BoxMux $version"
git push origin "$tag"
cargo publish --locked
gh release create "$tag" dist/* --title "BoxMux $version" --notes-file "$notes"
echo "released $version"
