# rusty-ts

A Rust port of the moreutils `ts` utility: prefix each line of stdin with a timestamp. Static binaries on Linux, macOS, and Windows; works with or without a Rust toolchain via `cargo install` or `cargo binstall`. Default mode adds a few niceties moreutils doesn't have (`-u`/`--utc`, `--tz=<IANA>`, env-var defaults, shell completions); Strict mode reverts every observable surface to byte-identical moreutils behavior for drop-in migration.

Part of the [Rusty portfolio](https://github.com/REPLACE_OWNER/rusty) — a collection of small Rust ports of utilities missing from the Rust ecosystem.

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

Per-target archives are attached to each [GitHub Release](https://github.com/REPLACE_OWNER/rusty-ts/releases). Linux x86_64/aarch64, macOS x86_64/aarch64, Windows x86_64. Each archive contains the binary plus pre-generated shell-completion scripts for bash, zsh, fish, and PowerShell.

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
rusty-ts = { version = "0.1", default-features = false }
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
