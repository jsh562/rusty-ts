# rusty-ts

A Rust port of the moreutils `ts` utility: prefix each line of stdin with a timestamp. Static binaries on Linux, macOS, and Windows; works with or without a Rust toolchain via `cargo install` or `cargo binstall`. Default mode adds a few niceties moreutils doesn't have (`-u`/`--utc`, `--tz=<IANA>`, env-var defaults, shell completions); Strict mode reverts every observable surface to byte-identical moreutils behavior for drop-in migration.

Part of the [Rusty portfolio](https://jsh562.github.io/rusty-portfolio) — a collection of small Rust ports of utilities missing from the Rust ecosystem.

[![crates.io](https://img.shields.io/crates/v/rusty-ts.svg)](https://crates.io/crates/rusty-ts)
[![docs.rs](https://docs.rs/rusty-ts/badge.svg)](https://docs.rs/rusty-ts)
[![license: MIT OR Apache-2.0](https://img.shields.io/crates/l/rusty-ts.svg)](#license)

## Install

### With a Rust toolchain

```sh
cargo install rusty-ts
```

To also install the `ts` binary alias (auto-enables Strict mode on invocation):

```sh
cargo install rusty-ts --features ts-alias
```

### Without a Rust toolchain (prebuilt binaries via cargo-binstall)

```sh
cargo binstall rusty-ts
```

### Direct download

Per-target archives are attached to each [GitHub Release](https://github.com/jsh562/rusty-ts/releases). Linux x86_64/aarch64, macOS x86_64/aarch64, Windows x86_64. Each archive contains the binary plus pre-generated shell-completion scripts for bash, zsh, fish, and PowerShell.

## Usage

```sh
# Default format (matches moreutils ts: `%b %d %H:%M:%S`)
some-command | rusty-ts

# Custom strftime format
some-command | rusty-ts '%Y-%m-%d %H:%M:%S'

# Elapsed time since previous line / since program start (monotonic clock)
some-command | rusty-ts -i
some-command | rusty-ts -s -m

# UTC or named IANA timezone
some-command | rusty-ts -u
some-command | rusty-ts --tz=America/New_York

# Convert pre-timestamped lines to relative form (Default-mode subset: ISO-8601, RFC-3339, Unix epoch)
cat logfile | rusty-ts -r

# Strict moreutils-compat mode (rejects -u/--tz, expands -r to full moreutils set, mirrors stderr layout)
some-command | rusty-ts --strict
RUSTY_TS_STRICT=1 some-command | rusty-ts
some-command | ts          # via the ts-alias feature or a symlink — argv[0] auto-detect

# Implicit default format via env var (Default mode only)
RUSTY_TS_FORMAT='[%H:%M:%S]' some-command | rusty-ts

# Shell completions
rusty-ts completions bash    # > ~/.bash_completion.d/rusty-ts
rusty-ts completions zsh     # > ~/.zfunc/_rusty-ts
rusty-ts completions fish    # > ~/.config/fish/completions/rusty-ts.fish
rusty-ts completions powershell
```

## Compatibility statement (vs moreutils ts)

Byte-level fidelity is verified by snapshot tests against captured moreutils-`ts` output under a pinned environment: `TZ=UTC` and `LC_ALL=C.UTF-8`. The snapshot reference is moreutils at a pinned upstream commit recorded in [`fixtures/README.md`](fixtures/README.md).

**Documented intentional divergences from moreutils ts** (also enumerated in [`docs/COMPATIBILITY.md`](docs/COMPATIBILITY.md) — generated from the CLI definition and drift-tested in CI):

1. **`-r` recognized-timestamp set is a subset in Default mode**: ISO-8601, RFC-3339, and Unix epoch (integer + fractional) only. `--strict` expands recognition to the full moreutils set.
2. **`-u` / `--utc` flag**: not present in moreutils. Default-mode addition; rejected in Strict mode.
3. **`--tz=<IANA>` flag**: not present in moreutils. Default-mode addition; rejected in Strict mode.
4. **`RUSTY_TS_FORMAT` env var**: not defined by moreutils. Honored in Default mode; ignored in Strict mode.
5. **`completions` subcommand**: not present in moreutils. Default-mode addition; rejected in Strict mode.

In Strict mode, exit codes, stderr diagnostic text, and `--help` / `--version` layouts match moreutils. See [`docs/COMPATIBILITY.md`](docs/COMPATIBILITY.md) for the full per-flag matrix and exit-code table.

## Library API

The crate exposes a public Rust API for programmatic use. The canonical surface is byte-typed (preserves non-UTF-8 payload bytes per FR-011); a `String`-typed convenience adapter is available for the common UTF-8 case.

```rust
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

To use the library without pulling in the CLI dependencies:

```toml
[dependencies]
rusty-ts = { version = "0.2", default-features = false }
```

## Cargo Features

`default` enables `full`, which (for this single-capability port) resolves to the `cli` umbrella. `ts-classic` reproduces v0.1.x bare-port behavior matching upstream moreutils `ts` 1:1. To strip the CLI surface use `default-features = false` or `--no-default-features` and then add the features you want.

rusty-ts is a **single-capability port** per spec 00011 §Scope Edge Cases — its sole documented capability is "prefix each line of stdin with a timestamp". Following the portfolio-wide convention's minimum-port rule, zero leaves are carved beyond the required umbrellas. See [`docs/feature-layout.md`](docs/feature-layout.md) for the per-capability rejection rationale.

### Feature matrix

| Feature | Description | Umbrella(s) |
|---|---|---|
| `cli` | All CLI-only dependencies (`clap`, `clap_complete`, `anyhow`) and the binary entry point. Library consumers strip this via `default-features = false`. | `full`, `ts-classic`, `ts-minimal`, `ts-alias` |
| `ts-alias` | Installs an additional `ts` binary alongside `rusty-ts`. Both share the same source; argv[0] auto-detect routes `ts` invocations into Strict mode (FR-023). | (standalone — implies `cli`) |
| `bench` | Pulls `criterion` and enables `benches/throughput.rs`. Dev-tooling only; outside the convention's leaf surface. Name preserved verbatim from v0.1.x. | (standalone) |

### Preset bundles

| Bundle | Composition | Use case |
|---|---|---|
| `ts-classic` | `cli` | Drop-in upstream moreutils `ts` replacement. Strict mode is invoked via `--strict`, the `RUSTY_TS_STRICT` env var, or `ts-alias` argv[0] auto-detect — no extra feature flag is required. |
| `ts-minimal` | `cli` | Explicit minimal-CLI alias for users who prefer the `<port>-minimal` naming convention seen across other portfolio ports (figlet-minimal, pwgen-minimal). Identical composition to `ts-classic`. |

### Keep-list workaround (Cargo features are union-only)

Cargo features cannot subtract from `default`. To get "everything except a specific feature," disable defaults and enumerate the features you want:

```sh
cargo install rusty-ts --no-default-features --features "cli"
# → bare CLI with no ts-alias binary, no bench tooling. Equivalent to
#   ts-classic / ts-minimal.

cargo install rusty-ts --no-default-features --features "cli ts-alias"
# → CLI + the ts alias binary; library consumers still get a clean strip
#   via `default-features = false`.
```

For the common cases the named [preset bundles](#preset-bundles) above are usually sufficient.

### Library-only consumers

```toml
[dependencies]
rusty-ts = { version = "0.2", default-features = false }
```

This strips `clap`, `clap_complete`, and `anyhow`. The resulting build pulls only `chrono`, `chrono-tz`, `regex`, and `thiserror`. The CI `test-no-default` job runs `cargo tree --no-default-features` on every PR and fails the build if any CLI-only dep leaks back in.

### Convention authority

This layout follows the portfolio-wide Cargo Features Convention. The "why" lives in [ADR-0006](https://github.com/jsh562/rustylib/blob/main/specs/adrs/0006-cargo-features-convention-for-portfolio-ports.md) (option analysis and rationale); the "what" lives in [`project-instructions.md` §Cargo Feature Surface](https://github.com/jsh562/rustylib/blob/main/project-instructions.md) (canonical rules per port). Every Rusty port from v0.2 onward exposes the same umbrella set (`default` / `full` / `cli` / `<port>-classic`), per-port leaves named in kebab-case, and 2 to 4 preset bundles. Single-capability ports like rusty-ts adopt the minimum convention (zero leaves; `<port>-classic` and `<port>-minimal` as the two required preset bundles per FR-007).

## Relationship to moreutils

`rusty-ts` is a **clean-room Rust reimplementation** of the moreutils `ts` utility. It contains **no source code from moreutils** — only a from-scratch Rust implementation that observes the documented behavior of moreutils `ts` and reproduces it.

The moreutils `ts` Perl script is © 2006 Joey Hess and licensed under the GNU GPL. That license governs the *Perl source code*. Behavioral interfaces (flag set, output format) are not copyrightable, so a clean-room reimplementation under a different license is well-established practice — the same posture as [`uutils/coreutils`](https://github.com/uutils/coreutils) (MIT-licensed reimplementation of GPL-licensed GNU coreutils).

`rusty-ts` does **not** distribute or derive from the moreutils source code. Snapshot tests in this repository compare `rusty-ts` *runtime output* against captured *moreutils ts runtime output* (captured by running moreutils against fixtures and recording bytes) — that is not source-code derivation either. The captured output bytes are facts, not creative expression.

If you want the original moreutils `ts`, install it via your platform's package manager (`apt install moreutils`, `brew install moreutils`, etc.) — that is unaffected by this port's existence.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE](LICENSE))

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
