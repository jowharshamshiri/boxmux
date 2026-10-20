# Contributing to BoxMux

Thanks for helping. This guide covers how the code is organized, how to build and test it, and
how changes are released.

## Code of conduct

Be respectful and constructive. Harassment or personal attacks are not tolerated; maintainers may
remove comments, commits or contributors that break this rule.

## Getting started

You need a stable Rust toolchain (install it with [rustup](https://rustup.rs)) and, to work on the
documentation site, Node.js 20 or newer.

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
schemas/app_schema.json   the configuration schema (kept in sync with the model by a test)
api-spec.json             the control protocol schema (kept in sync by a test)
layouts/, examples/       configurations shipped as demos; every one must load
examples/plugins/gauge    an example plugin
tests/live_ux.rs          end-to-end tests of the real binary in a pseudo-terminal
mg-test.yaml, mg-test/    cross-platform test plans and runners
scripts/                  release and documentation deployment
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
  it with keys and mouse sequences.

### Cross-platform runs with mg-test

BoxMux is tested on several operating systems with mg-test, which creates virtual machines,
copies the tracked files of the working tree into them and runs the plans in `mg-test/`:

| Plan | Machines | What runs |
|------|----------|-----------|
| `hosts` | Ubuntu x86-64, Windows 11 x86-64 | formatting (Linux), clippy, all tests |
| `arm64` | Ubuntu arm64 | clippy, all tests |
| `freebsd` | FreeBSD x86-64 | clippy, all tests |
| `release` | Ubuntu x86-64 and arm64, Windows 11 | release builds and packages |

```bash
mg-test validate mg-test.yaml
mg-test run mg-test.yaml --plan hosts          # waits and reports
mg-test run mg-test.yaml --plan arm64 --detach # returns a run id
mg-test results RUN_ID
mg-test logs RUN_ID --task test-windows-x86-64
mg-test destroy RUN_ID                         # give the machines back
```

The runners are `mg-test/test.sh` (POSIX sh) and `mg-test/test.ps1` (PowerShell); each writes
one result per test to `out/results.jsonl`. macOS is covered by the Unix code path but there are
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

1. Set the version in `Cargo.toml` and add its section to `CHANGELOG.md`.
2. Commit and push to `main`.
3. Run `scripts/release.sh --dry-run` to run the checks and build the packages on mg-test
   (`mg-test/release.yaml`) into `dist/`.
4. Run `scripts/release.sh` to do the same, then, after you confirm, tag `vX.Y.Z`, publish the
   crate and create the GitHub release with the packages and their checksums.
