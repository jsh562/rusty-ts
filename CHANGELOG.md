# Changelog

All notable changes to `rusty-ts` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-05-22

Initial release. Inaugural port in the [Rusty portfolio](https://github.com/jsh562).

### Added

- CLI binary `rusty-ts`: prefix each line of stdin with a timestamp (Rust port of moreutils `ts`).
- Default strftime format matching moreutils (`%b %d %H:%M:%S`), single-space separator.
- Custom format strings via positional argument, including the moreutils
  fractional-second extensions (`%.S` and `%.s`) at microsecond precision.
- Elapsed-time modes: `-i` (since previous line), `-s` (since program start),
  `-m` (monotonic clock).
- Explicit timezone control via `-u` / `--utc` and `--tz=<IANA>` flags
  (Rusty extensions, not present in moreutils — rejected in Strict mode).
- Relative-mode `-r` rewriter recognizing ISO-8601, RFC-3339, and Unix epoch
  timestamps in Default mode; Strict mode expands recognition to include
  human-readable date-time formats.
- Strict moreutils-compatibility mode via the `--strict` flag, the
  `RUSTY_TS_STRICT` env var, or invocation as `ts` (via the `ts-alias` cargo
  feature, a symlink, or a shell alias). In Strict mode, Rusty-only flags
  are rejected with a stderr diagnostic byte-equal to moreutils' own
  `Unknown option: <flag>\nusage: ts [-r] [format]\n` format.
- Optional `ts` binary alias, gated behind the `ts-alias` cargo feature.
  Default `cargo install rusty-ts` installs only `rusty-ts`;
  `cargo install rusty-ts --features ts-alias` adds the moreutils-name alias.
- `completions <shell>` subcommand emitting shell-completion scripts for
  bash, zsh, fish, and PowerShell.
- `RUSTY_TS_FORMAT` env var providing an implicit default format string
  (honored in Default mode; ignored in Strict mode per FR-027).
- Public Rust library API: `TimestamperBuilder` (with `#[must_use]` chain
  methods and FR-020 mutex enforcement at the library layer) →
  `Timestamper::prefix_lines(impl BufRead) -> impl Iterator<Item = io::Result<Vec<u8>>>`
  as the byte-typed canonical surface, plus a `prefix_string_lines`
  convenience adapter for UTF-8 callers.
- Library-without-binary build: `default-features = false` drops `clap`,
  `clap_complete`, and `anyhow` from the dependency closure.
- README Compatibility Matrix at `docs/COMPATIBILITY.md`, generated from
  the canonical CLI definition and drift-tested in CI on every PR.
- Cross-platform binary distribution: Linux x86_64/aarch64, macOS
  x86_64/aarch64, Windows x86_64 via `cargo-binstall` metadata pointing at
  GitHub Release archives.

### Behavioral parity with moreutils — verified byte-equal

Snapshot tests under `tests/compat_default.rs` and `tests/compat_strict.rs`
assert byte-equal output against captured moreutils `ts` output for:

- Default-format timestamping (3 fixtures)
- Custom strftime format strings (3 fixtures)
- Fractional-second tokens `%.S` and `%.s` at microsecond precision
  (2 fixtures)
- Non-UTF-8 byte payload passthrough (1 fixture)
- Strict-mode stderr rejection of Rusty-only flags (2 fixtures)

Fixtures captured under pinned `TZ=UTC` and `LC_ALL=C.UTF-8` from the
moreutils `ts` Perl script (madx/moreutils mirror, master HEAD as of
2026-05-22). Capture protocol and sidecar metadata documented in
`fixtures/README.md` and per-category `CAPTURE.json` files.

### Known limitations at v0.1.0

- `-r` relative-mode behavioral parity is verified by Rusty-side unit tests
  only, not byte-equal against moreutils. The moreutils `ts -r` code path
  requires the Perl `Time::Duration` and `Date::Parse` modules, which were
  not available in the WSL2 Ubuntu 24.04 baseline used to capture other
  fixtures. Byte-equal verification deferred to a follow-up release.
- The `ts-alias` cargo feature ships a second binary named `ts`. Users with
  moreutils already installed may experience PATH-order conflicts — by
  design, the user chooses which `ts` wins via their PATH.

### Verified

- 124 tests passing on Rust 1.85 (MSRV) and current stable (1.92).
- Clippy strict (`-D warnings`) clean.
- rustfmt clean.
- `cargo audit` clean.
- Library API consumable with `default-features = false`.

### Compatibility statement

A full Compatibility Matrix mapping every moreutils `ts` flag and every
Rusty-added flag to Default-mode and Strict-mode behavior lives at
[`docs/COMPATIBILITY.md`](docs/COMPATIBILITY.md). The file is generated
from the canonical CLI definition and CI fails on drift.

[Unreleased]: https://github.com/jsh562/rusty-ts/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/jsh562/rusty-ts/releases/tag/v0.1.0
