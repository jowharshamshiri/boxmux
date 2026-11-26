# Contributing to BoxMux

Thanks for helping. This guide covers how the code is organized, how to build and test it, and
how a release is built.

## Code of conduct

Be respectful and constructive. Harassment or personal attacks are not tolerated; maintainers may
remove comments, commits or contributors that break this rule.

## Getting started

You need a stable Rust toolchain (install it with [rustup](https://rustup.rs)), `git` (the build
reads the commit of the checkout for `boxmux --version`) and, to work on the documentation site,
Node.js 20 or newer.

```bash
git clone https://github.com/jowharshamshiri/boxmux.git
cd boxmux
cargo build
cargo run -- layouts/showcase_menu.yaml
```

Press `q` to quit. Run `cargo run -- --help` for the command line.

## Project structure

```
src/
  main.rs            command line: running an interface, and the control commands
  lib.rs             the library the binary and the tests use
  model/             configuration: app, layouts, boxes, choices, streams, positions,
                     appearance values, variables, in-place YAML editing
  validation.rs      the JSON schema check and the consistency checks
  render/            drawing: the screen buffer, styled text, the theme, box views
                     (used for drawing and for mouse hit-testing), tab bar, scrollbars
  runtime/           the event loop: input, script runs, the executor, PTY sessions,
                     socket commands, the terminal display
  socket/            the control socket: protocol, client and server
  platform/          Unix and Windows specifics: shells, process trees, signals
  ansi_processor.rs  the terminal emulator behind PTY boxes
  chart.rs, table.rs charts and tables drawn as text
  plugin.rs          loading plugins through the C ABI
  text.rs            cleaning script output for display
  upgrade/           `boxmux upgrade`: finding, verifying and installing the newest release
build.rs, build/commit.rs  the commit `boxmux --version` names (tested by tests/version.rs)
build/release_line.rs     the release line a release build bakes in (tested by tests/release_line.rs)
tools/boxmux-dist         builds a release archive of the platform it runs on
schemas/app_schema.json   the configuration schema (kept in sync with the model by a test)
api-spec.json             the control protocol schema (kept in sync by a test)
layouts/, examples/       configurations shipped as demos; every one must load
examples/plugins/gauge    an example plugin
tests/live_ux.rs          end-to-end tests of the real binary in a pseudo-terminal
mg-test.yaml, mg-test/    cross-platform test plans and runners
scripts/                  documentation deployment
docs/                     the documentation site (SvelteKit)
```

## Making changes

- Fix causes, not symptoms. Invalid configuration and impossible states are rejected with a
  clear error that names where the problem is; nothing is silently ignored or guessed.
- Keep one way of doing each thing. When behaviour changes, update the code, the schema, the
  shipped layouts, the tests and the documentation together.
- Every bug fix comes with a test that fails without the fix. Tests check behaviour (what the
  user sees, what the program does), not the shape of the implementation.
- Shipped layouts and documentation examples must load; `cargo test` checks the layouts.

### Code style

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
```

Both must pass. Write comments that explain why, in plain sentences; leave out comments that
restate the code.

### Commit messages

Use a short imperative summary line (`Fix tab bar scrolling past the last tab`), a blank line,
then what changed and why. One logical change per commit.

## Testing

```bash
cargo test                      # unit tests, runtime tests and the end-to-end tests
cargo test --test live_ux       # only the end-to-end tests
cargo test render::             # one module
```

- Unit tests live next to the code they test (`#[cfg(test)] mod tests`); tests that span modules
  are in `src/tests/`.
- `src/runtime/tests.rs` runs the real runtime against an in-memory display: configuration in,
  keyboard and mouse events in, frames out, with real scripts.
- `tests/live_ux.rs` runs the `boxmux` binary in a pseudo-terminal (ConPTY on Windows) and drives
  it with keys and mouse sequences. It reads the screen back through a small terminal emulator
  that shows each synchronized update whole, as WezTerm does, however the pseudo-terminal splits
  the output into reads. A test checks what boxmux shows or wrote only once it has waited for
  that whole state (`wait_for_text`, `wait_until`, `wait_for_screen`, `wait_for_exit`, which
  reads until the output ends), never after a fixed time: how soon output arrives, and in what
  pieces, differs between machines and with their load. Only a check that something does not
  happen waits a fixed time.
- Every test waits on the event it checks: a script's output or exit, a terminal's screen, a
  process going away. A wait fails only when its hang bound passes (two minutes: `HANG_BOUND`
  in `src/tests/mod.rs` and `tests/panic_cleanup.rs`, `TIMEOUT` in `tests/live_ux.rs`), and says
  what it waited for. The bound catches a hang, not a slow machine: a loaded machine can take
  seconds to start one process. A program a test needs running sleeps ten minutes, longer than
  the bound, so only being stopped ends it while the test looks.

### Cross-platform runs with mg-test

BoxMux is tested on several operating systems with mg-test, which creates virtual machines,
copies the tracked files of the working tree into them and runs the plans in `mg-test/`:

| Plan | Machines | What runs |
|------|----------|-----------|
| `hosts` | Ubuntu x86-64, Windows 11 x86-64 | formatting (Linux), clippy, all tests |
| `freebsd` | FreeBSD x86-64 | formatting, clippy, all tests |

There is no Linux arm64 plan: the only arm64 guest is emulated on x86-64, and too slow to run
the suite.

```bash
mg-test validate mg-test.yaml
mg-test run mg-test.yaml --plan hosts            # waits and reports
mg-test run mg-test.yaml --plan freebsd --detach # returns a run id
mg-test results RUN_ID
mg-test logs RUN_ID --task test-windows-x86-64
mg-test destroy RUN_ID                         # give the machines back
```

The runners are `mg-test/test.sh` (POSIX sh) and `mg-test/test.ps1` (PowerShell); each writes
one result per test to `out/results.jsonl`, and stops at the first check that fails (formatting,
then clippy), reporting it as a failed item.

The guests build with [QuickRust](https://github.com/machinefabric/quickrust), and nothing else
of Rust is installed on them: QuickRust installs and updates the stable toolchain itself and
chooses where builds go. Its home is each target's `rust` cache, so the toolchain, the crates and
the build output carry over from one run to the next; Windows keeps Git in its `tools` cache. On
Linux and Windows the runner installs the release channel's latest QuickRust from its release
manifest, checking the archive's SHA-256 and size, and has it update itself on later runs.
Every runner brings the toolchain up to date and collects QuickRust's cache before it builds
(not on FreeBSD, where QuickRust cannot collect it), and QuickRust starts nothing in the
background, so nothing of it is still running, writing into the cache or holding a log open,
when the run ends.
QuickRust publishes no FreeBSD build, so on FreeBSD the runner builds it once from its source, at
the commit `QUICKRUST_COMMIT` in `mg-test/test.sh` names, with FreeBSD's own Rust package, which
it removes again once QuickRust is set up. macOS is covered by the Unix code path but there are
no macOS machines to test on.

## Documentation

The site in `docs/` is built from Markdown in `docs/src/content/`.

```bash
cd docs
npm ci
npm run dev      # local preview
npm run build    # the static site in docs/build
```

`scripts/deploy-docs.sh` builds the site and publishes it to the `gh-pages` branch, which serves
boxmux.com.

## Releasing

BoxMux is released from bahramj.com's release line, as the other apps published there are: a
release builds the archive of each platform on a machine of that platform, signs the manifest
that lists them, makes the apt, rpm and Homebrew packages from those archives, and publishes the
crate to crates.io. The release's notes are written there too; this repository keeps no
changelog. What a release needs from this repository is `tools/boxmux-dist`:

```bash
cargo run --locked -p boxmux-dist -- release --platform linux-x86_64 --out-dir out/dist
```

It builds `boxmux` for the platform it runs on (`linux-x86_64`, `darwin-arm64`; `windows-x86_64`
once its release build is set up; Linux on arm64 is not released, and is built from source)
with the release line in its environment, the five variables `build/release_line.rs` names, and
writes
`boxmux-<version>-<platform>.tar.gz` (a `.zip` for Windows), whose one directory `boxmux/` holds
the program, `LICENSE`, `README.md` and the demo configurations (`layouts/*.yaml`,
`examples/*.yaml` and `examples/*.sh`), and nothing else: an AppleDouble `._` file among them
stops the release, naming it. It stops, saying why, unless
`version.txt` and `Cargo.toml` agree, the release line is given whole and the program carries
it, the program carries no path of the machine it was built on, and `boxmux --version` names the
version. The same files make the same archive, byte for byte.

On Linux the build links with the rust-lld the toolchain ships, through cc, whichever linker
rustc would pick for the target itself, and stops, naming the path, when the toolchain has no
`lib/rustlib/<triple>/bin/gcc-ld/ld.lld`. std reaches `pidfd_spawnp` and `pidfd_getpid` (glibc
2.39) weakly, and runs without them; rust-lld makes a glibc version needed only that way a weak
need, which an older glibc starts the program past, where GNU ld would make it a hard one.

A build given no release line, `cargo build` and `cargo install` included, is a build from
source: `boxmux upgrade` says it has no release line to follow.
