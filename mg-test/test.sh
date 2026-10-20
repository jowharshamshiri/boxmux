#!/bin/sh
# BoxMux's suite on a Linux or FreeBSD guest: installs what it needs (once per cache), then
# checks formatting and lints and runs every test, the live terminal tests included.
# Each test's outcome goes to out/results.jsonl; the full output to out/test.log.
set -eu

cd "$MG_TEST_WORKSPACE"
export RUSTUP_HOME="$MG_TEST_CACHE_TOOLS/rustup" CARGO_HOME="$MG_TEST_CACHE_TOOLS/cargo"
export CARGO_TARGET_DIR="$MG_TEST_CACHE_TARGET" CARGO_TERM_COLOR=never
export PATH="$CARGO_HOME/bin:$PATH"
mkdir -p out

case "$(uname -s)" in
    Linux)
        if ! command -v cc >/dev/null || ! command -v curl >/dev/null; then
            sudo -n apt-get update
            sudo -n env DEBIAN_FRONTEND=noninteractive apt-get install -y build-essential curl
        fi
        ;;
    FreeBSD)
        # The live terminal tests start boxmux through bash; rustup needs curl.
        command -v bash >/dev/null && command -v curl >/dev/null || pkg install -y bash curl
        ;;
    *)
        echo "mg-test/test.sh does not know how to prepare a $(uname -s) guest" >&2
        exit 1
        ;;
esac

[ -x "$CARGO_HOME/bin/cargo" ] || curl -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path \
    --profile minimal --default-toolchain stable -c clippy -c rustfmt
rustup update stable --no-self-update

status=0
{
    echo "== fmt"
    cargo fmt --all -- --check
    echo "== clippy"
    cargo clippy --all-targets --locked -- -D warnings
    echo "== tests"
    cargo test --locked --no-fail-fast
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
