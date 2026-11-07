# BoxMux

CONTRIBUTING.md is the guide to the code layout, the build, the tests, mg-test and releasing.
Read it before changing code. This file adds what an agent working on the repository needs on
top of it.

## Testing

- `cargo test` runs everything on one machine. Other operating systems are covered by the
  mg-test plans in `mg-test.yaml` and `mg-test/` (hosts, arm64, freebsd, release), as
  CONTRIBUTING.md describes.
- The repository has no GitHub Actions workflows and gets none: cross-platform runs go through
  mg-test.
- There are no macOS machines; macOS runs the Unix code path untested.

## Versions

The pre-commit hook rewrites `version.txt`, `Cargo.toml` and `Cargo.lock` with a new version on
every commit (major version 0, set in `.wsb/repo.json`). Do not edit the version by hand, and
commit through git with the hook in place so every commit carries its version.

## Board

Work is tracked as draft cards on the GitHub project jowharshamshiri/14 (named in
`.wsb/project.json`). A card that waits on the owner goes to Suspended while other work
continues. Board changes go through `gh api graphql` with these ids:

- Project `PVT_kwHOAHykxc4Bnx6x`
- Status `PVTSSF_lAHOAHykxc4Bnx6xzhmU8V0`: Backlog `f75ad846`, Ready `61e4505c`,
  In progress `47fc9ee4`, In review `df73e18b`, Done `98236657`, Suspended `87536d2f`,
  Postponed `c5014e8d`
- Priority `PVTSSF_lAHOAHykxc4Bnx6xzhmU8bU`: P0 `79628723`, P1 `0a877460`, P2 `da944a9c`
- Size `PVTSSF_lAHOAHykxc4Bnx6xzhmU8bY`: XS `6c6483d2`, S `f784b110`, M `7515a9f1`,
  L `817d0097`, XL `db339eb2`
