#!/bin/sh
# BoxMux's suite on a Linux or FreeBSD guest: installs what it needs (once per guest or cache),
# then checks formatting and lints and runs every test, the live terminal tests included.
# Each test's outcome goes to out/results.jsonl; the full output to out/test.log.
set -eu

cd "$MG_TEST_WORKSPACE"
mkdir -p out

die() { echo "mg-test/test.sh: $*" >&2; exit 1; }

[ -n "${MG_TEST_CACHE_RUST:-}" ] || die "the rust cache is not mounted; the plan must declare it"

# Rust is built by QuickRust, and QuickRust is the one thing installed for it: it installs and
# manages the toolchain itself and decides where builds go. Its home is the rust cache, so the
# toolchain, the crates it downloaded and its build cache outlive the guest. QuickRust starts
# nothing in the background, so no process of it is left running, writing into the cache, when
# the run ends: the toolchain and QuickRust are brought up to date and the cache is collected
# below, before the build.
system_path="$PATH"
export QRUST_HOME="$MG_TEST_CACHE_RUST/qrust"
export QRUST_TOOLCHAIN_AUTO_UPDATE=off QRUST_SELF_UPDATE_CHECK=off QRUST_CACHE_GC=off
export PATH="$QRUST_HOME/bin:$PATH" CARGO_TERM_COLOR=never

# QuickRust publishes Linux builds: the release channel's latest archive for this machine, from
# the release manifest, checked against the manifest's SHA-256 and size before anything in it
# runs.
install_quickrust_release() {
    case "$(uname -m)" in
        x86_64) platform=linux-x86_64 ;;
        aarch64|arm64) platform=linux-arm64 ;;
        *) die "QuickRust publishes no build for $(uname -m) Linux" ;;
    esac
    scratch="$(mktemp -d)"
    curl -fsSL --retry 3 https://release.machinefabric.com/quickrust/manifest -o "$scratch/manifest" \
        || die "the QuickRust release manifest could not be fetched"
    archive="$(jq -er --arg platform "$platform" '
        .channels.release as $release
        | ($release.versions[$release.latestVersion].platforms[$platform]
           // error("QuickRust \($release.latestVersion) has no \($platform) build"))
        | [.installers[] | select(.type == "targz")]
        | if length == 1 then .[0] else error("the manifest lists \(length) tar.gz archives for \($platform), not one") end
        | "\(.path) \(.sha256) \(.size)"' "$scratch/manifest")" \
        || die "the QuickRust manifest names no archive for $platform"
    set -- $archive
    echo "installing QuickRust from $1"
    curl -fsSL --retry 3 "$1" -o "$scratch/qrust.tar.gz" || die "$1 could not be fetched"
    [ "$(stat -c %s "$scratch/qrust.tar.gz")" = "$3" ] \
        || die "$1 is $(stat -c %s "$scratch/qrust.tar.gz") bytes, not the $3 the manifest says"
    echo "$2  $scratch/qrust.tar.gz" | sha256sum -c --quiet \
        || die "$1 does not have the manifest's SHA-256 $2"
    tar xzf "$scratch/qrust.tar.gz" -C "$scratch" || die "$1 could not be unpacked"
    "$scratch/quickrust/qrust" setup --no-modify-path || die "qrust setup failed"
    rm -rf "$scratch"
}

# QuickRust publishes no FreeBSD build, so it is built from its source, at this commit of
# https://github.com/machinefabric/quickrust, with FreeBSD's own Rust package, which is removed
# again once `qrust setup` has copied QuickRust into its home. The package's cargo builds with the
# package's rustc: the system's PATH, without the QuickRust a cache may already hold. The commit
# built is kept beside the home: QuickRust is built again when this one changes.
QUICKRUST_SOURCE=https://github.com/machinefabric/quickrust
QUICKRUST_COMMIT=75d52f464b92e5e77db3077e414a0960079ab9c7
quickrust_built="$MG_TEST_CACHE_RUST/quickrust-commit"
install_quickrust_source() {
    echo "building QuickRust from $QUICKRUST_SOURCE at $QUICKRUST_COMMIT"
    pkg install -y rust || die "FreeBSD's rust package, which builds QuickRust, could not be installed"
    source="$MG_TEST_SCRATCH/quickrust"
    rm -rf "$source"
    git init -q "$source"
    git -C "$source" fetch -q --depth 1 "$QUICKRUST_SOURCE" "$QUICKRUST_COMMIT" \
        || die "$QUICKRUST_SOURCE has no commit $QUICKRUST_COMMIT"
    git -C "$source" checkout -q FETCH_HEAD
    (cd "$source" && PATH="$system_path" /usr/local/bin/cargo build --release --locked) || die "QuickRust did not build"
    "$source/target/release/qrust" setup --no-modify-path || die "qrust setup failed"
    echo "$QUICKRUST_COMMIT" > "$quickrust_built"
    rm -rf "$source"
    pkg delete -y rust || die "FreeBSD's rust package could not be removed after building QuickRust"
}

case "$(uname -s)" in
    Linux)
        # jq reads QuickRust's release manifest.
        if ! command -v cc >/dev/null || ! command -v curl >/dev/null || ! command -v jq >/dev/null; then
            sudo -n apt-get update
            sudo -n env DEBIAN_FRONTEND=noninteractive apt-get install -y build-essential curl jq
        fi
        if [ -x "$QRUST_HOME/bin/qrust" ]; then
            qrust self update || die "qrust self update failed"
        else
            install_quickrust_release
        fi
        ;;
    FreeBSD)
        # The live terminal tests start boxmux through bash, the version tests make git
        # repositories, and QuickRust's source is fetched with git.
        command -v bash >/dev/null && command -v git >/dev/null || pkg install -y bash git
        if ! [ -x "$QRUST_HOME/bin/qrust" ] || [ "$(cat "$quickrust_built" 2>/dev/null)" != "$QUICKRUST_COMMIT" ]; then
            install_quickrust_source
        fi
        ;;
    *)
        die "it does not know how to prepare a $(uname -s) guest"
        ;;
esac
[ "$(command -v cargo)" = "$QRUST_HOME/bin/cargo" ] || die "cargo on PATH is $(command -v cargo), not QuickRust's"
qrust toolchain update --channel stable || die "qrust toolchain update failed"
qrust gc || die "qrust gc failed"

echo "$(qrust --version), $(cargo --version)"

# Each check stops the run when it fails: chained, as `set -e` does not apply on the left of ||.
status=0
{
    echo "== fmt" && cargo fmt --all -- --check \
        && echo "== clippy" && cargo clippy --all-targets --locked -- -D warnings \
        && echo "== tests" && cargo test --locked --no-fail-fast
} > out/test.log 2>&1 || status=$?

awk -v os="$(uname -s)" '
    BEGIN { printf "{\"record\": \"run\", \"version\": \"boxmux\", \"host_os\": \"%s\", \"mode\": \"cargo-test\"}\n", os }
    /^== / { stage = $2 }
    /^ *Running / { suite = $2 }
    /^test .* \.\.\. (ok|FAILED|ignored)$/ {
        outcome = ($NF == "ok") ? "ok" : ($NF == "ignored" ? "skipped" : "failed")
        printf "{\"record\": \"item\", \"group\": \"%s\", \"item\": \"%s\", \"outcome\": \"%s\"}\n", suite, $2, outcome
        count[outcome]++
    }
    END {
        if (stage != "tests") {
            printf "{\"record\": \"item\", \"group\": \"checks\", \"item\": \"%s\", \"outcome\": \"failed\", \"fault\": \"workload\", \"log\": \"out/test.log\"}\n", stage
            count["failed"]++
        }
        printf "{\"record\": \"summary\", \"passed\": %d, \"failed\": %d, \"skipped\": %d}\n", count["ok"], count["failed"], count["skipped"]
    }
' out/test.log > out/results.jsonl
tail -n 40 out/test.log
exit "$status"
