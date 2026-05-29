# rusty-ts

Prefix every line of stdin with a timestamp. Rust port of moreutils [`ts(1)`](https://joeyh.name/code/moreutils/).

[![crates.io](https://img.shields.io/crates/v/rusty-ts.svg)](https://crates.io/crates/rusty-ts)
[![docs.rs](https://docs.rs/rusty-ts/badge.svg)](https://docs.rs/rusty-ts)
[![CI](https://github.com/jsh562/rusty-ts/actions/workflows/ci.yml/badge.svg)](https://github.com/jsh562/rusty-ts/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-blue.svg)](#msrv)
[![license: MIT OR Apache-2.0](https://img.shields.io/crates/l/rusty-ts.svg)](#license)

Default mode adds the niceties moreutils doesn't have: `-u`/`--utc`, `--tz=<IANA>`, `RUSTY_TS_FORMAT` env-var default, & shell completions. Strict mode reverts every observable surface to byte-equal moreutils `ts` for drop-in migration. Prebuilt binaries for Linux, macOS, & Windows ship on every release.

Part of the [Rusty portfolio](https://jsh562.github.io/rusty-portfolio).

## Install

```sh
cargo install rusty-ts
# or, with prebuilt binaries:
cargo binstall rusty-ts
# or, download directly from GitHub Releases:
# https://github.com/jsh562/rusty-ts/releases
```

To also install a `ts` binary alias (argv[0] auto-detect routes into Strict mode):

```sh
cargo install rusty-ts --features ts-alias
```

## Usage

```sh
# Prefix every log line with a timestamp (moreutils-default format `%b %d %H:%M:%S`)
some-command | rusty-ts

# Custom strftime format for sortable timestamps
some-command | rusty-ts '%Y-%m-%d %H:%M:%S'

# Show elapsed time between lines (debug slow pipelines)
some-command | rusty-ts -i

# Show elapsed time since program start using the monotonic clock
some-command | rusty-ts -s -m

# Force UTC instead of local time (consistent across hosts)
some-command | rusty-ts -u

# Use a specific IANA timezone
some-command | rusty-ts --tz=America/New_York

# Convert already-stamped log lines to relative form
cat logfile | rusty-ts -r

# Implicit default format via env var (Default mode only)
RUSTY_TS_FORMAT='[%H:%M:%S]' some-command | rusty-ts

# Strict moreutils-compat mode (drop-in moreutils ts replacement)
some-command | rusty-ts --strict
RUSTY_TS_STRICT=1 some-command | rusty-ts
some-command | ts                          # via ts-alias feature or argv[0] symlink

# Shell completions
rusty-ts completions bash                   # > ~/.bash_completion.d/rusty-ts
rusty-ts completions zsh                    # > ~/.zfunc/_rusty-ts
rusty-ts completions fish                   # > ~/.config/fish/completions/rusty-ts.fish
rusty-ts completions powershell
```

## Library API

The crate exposes a byte-typed streaming surface. Non-UTF-8 input bytes round-trip unchanged. Use it inside a long-running daemon when you'd rather not shell out to a binary.

```rust,no_run
use rusty_ts::{TimestamperBuilder, Format, CompatibilityMode, TimezoneSource};
use std::io::{BufReader, Cursor};

let mut ts = TimestamperBuilder::new()
    .format(Format::Strftime("%Y-%m-%d %H:%M:%S".into()))
    .compat(CompatibilityMode::Default)
    .timezone(TimezoneSource::Utc)
    .build()?;

let input = BufReader::new(Cursor::new("hello\nworld\n"));
for line in ts.prefix_lines(input) {
    let bytes = line?;
    print!("{}", String::from_utf8_lossy(&bytes));
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

For library-only consumers without CLI deps see the [Cargo Features](#cargo-features) section.

## Cargo Features

`default` enables `full`, which (for this single-capability port) resolves to the `cli` umbrella. `ts-classic` reproduces v0.1.x bare-port behavior matching upstream moreutils `ts` 1:1. To strip the CLI surface use `default-features = false` or `--no-default-features` & then add what you want.

rusty-ts is a **single-capability port**: its one documented job is "prefix each line of stdin with a timestamp". No optional feature leaves are carved beyond the required umbrellas; see [`docs/feature-layout.md`](docs/feature-layout.md) for why.

### Feature matrix

| Feature | Description | Umbrella(s) |
|---|---|---|
| `cli` | All CLI-only dependencies (`clap`, `clap_complete`, `anyhow`) and the binary entry point. Library consumers strip via `default-features = false`. | `full`, `ts-classic`, `ts-minimal`, `ts-alias` |
| `ts-alias` | Installs an additional `ts` binary alongside `rusty-ts`. Both share source; argv[0] auto-detect routes `ts` invocations into Strict mode. | (standalone, implies `cli`) |
| `bench` | Pulls `criterion` and enables `benches/throughput.rs`. Dev-tooling only; outside the convention's leaf surface. Name preserved verbatim from v0.1.x. | (standalone) |

### Preset bundles

| Bundle | Composition | Use case |
|---|---|---|
| `ts-classic` | `cli` | Drop-in upstream moreutils `ts` replacement. Strict mode is invoked via `--strict`, `RUSTY_TS_STRICT`, or `ts-alias` argv[0] auto-detect. No extra feature flag is required. |
| `ts-minimal` | `cli` | Explicit minimal-CLI alias for users who prefer the `<port>-minimal` naming convention seen across other portfolio ports (figlet-minimal, pwgen-minimal). Identical composition to `ts-classic`. |

### Keep-list workaround (Cargo features are union-only)

Cargo features cannot subtract from `default`. To get "everything except a specific feature," disable defaults & enumerate the features you want:

```sh
cargo install rusty-ts --no-default-features --features "cli"
# → bare CLI with no ts-alias binary, no bench tooling.
#   Equivalent to ts-classic / ts-minimal.

cargo install rusty-ts --no-default-features --features "cli ts-alias"
# → CLI + the ts alias binary.
```

For the common cases the named [preset bundles](#preset-bundles) are usually sufficient.

### Library-only consumers

```toml
[dependencies]
rusty-ts = { version = "0.2", default-features = false }
```

This strips `clap`, `clap_complete`, & `anyhow`. The resulting build pulls only `chrono`, `chrono-tz`, `regex`, & `thiserror`. The CI `test-no-default` job runs `cargo tree --no-default-features` on every PR & fails the build if any CLI-only dep leaks back in.

### Convention authority

This layout follows the portfolio-wide Cargo Features Convention. The "why" lives in [ADR-0006](https://github.com/jsh562/rustylib/blob/main/specs/adrs/0006-cargo-features-convention-for-portfolio-ports.md); the "what" lives in [`project-instructions.md` §Cargo Feature Surface](https://github.com/jsh562/rustylib/blob/main/project-instructions.md). Every Rusty port from v0.2 onward exposes the same umbrella set (`default` / `full` / `cli` / `<port>-classic`), per-port leaves named in kebab-case, & 2 to 4 preset bundles.

## Compatibility

`rusty-ts` has two modes:

- **Default mode.** clap-styled flag parser. UTF-8 input. `-u`/`--utc`, `--tz=<IANA>`, `RUSTY_TS_FORMAT` env-var default, & the `completions` subcommand are all available. `-r` recognizes ISO-8601, RFC-3339, & Unix epoch (integer + fractional).
- **Strict mode** (activated by `--strict`, `RUSTY_TS_STRICT=1`, or invoking the binary as `ts`). Byte-equal stdout, stderr, exit codes, & `--help`/`--version` layouts against moreutils `ts` at the pinned upstream commit recorded in [`fixtures/README.md`](fixtures/README.md). `-u`, `--tz`, & `completions` MUST be rejected. `RUSTY_TS_FORMAT` MUST be ignored. `-r` expands to the full moreutils recognized-timestamp set.

Byte-level fidelity is verified by snapshot tests against captured moreutils-`ts` output under a pinned environment: `TZ=UTC` and `LC_ALL=C.UTF-8`.

### Documented intentional divergences

1. **`-r` recognized-timestamp set is a Default-mode subset**. ISO-8601, RFC-3339, & Unix epoch only. `--strict` expands to the full moreutils set.
2. **`-u` / `--utc`**. Default-mode addition; rejected in Strict.
3. **`--tz=<IANA>`**. Default-mode addition; rejected in Strict.
4. **`RUSTY_TS_FORMAT` env var**. Honored in Default; ignored in Strict.
5. **`completions` subcommand**. Default-mode addition; rejected in Strict.

See [`docs/COMPATIBILITY.md`](docs/COMPATIBILITY.md) for the full per-flag matrix & exit-code table.

## What's not shipped

- **The full moreutils `-r` recognized-timestamp set in Default mode.** Strict mode covers it; Default mode keeps the surface small (ISO-8601, RFC-3339, Unix epoch). Niche extended formats live behind `--strict`.
- **Source-code derivation from moreutils.** This is a clean-room reimplementation. The moreutils `ts` Perl script is GPL'd & untouched. Snapshot tests compare runtime output bytes only, which are facts, not creative expression. Same posture as [`uutils/coreutils`](https://github.com/uutils/coreutils).

If you want the original moreutils `ts`, install it via your platform package manager (`apt install moreutils`, `brew install moreutils`). It coexists fine with this port.

## MSRV

Rust **1.85** (edition 2024). Re-verified against the portfolio's stable-minus-two policy at each release.

## License

Dual-licensed under [MIT](LICENSE) or [Apache-2.0](LICENSE-APACHE) at your option.
