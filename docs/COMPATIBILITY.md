# rusty-ts Compatibility Matrix

This file is **generated** from the CLI definition by `cargo test --test compat_matrix`. Do not edit by hand — any change must come from `src/cli.rs`. CI fails on drift.

## TZ-pinning disclosure

Byte-level fidelity against moreutils `ts` is verified under `TZ=UTC` and `LC_ALL=C.UTF-8` (see `fixtures/README.md` for the full capture protocol). Snapshot tests refuse to run if these env vars are not pinned.

## Flags

| Flag | Default mode | Strict mode |
|------|--------------|-------------|
| `-i, --incremental` | Elapsed since previous line (FR-005). | Same — matches moreutils `-i`. |
| `-s, --since-start` | Elapsed since program start (FR-006). | Same — matches moreutils `-s`. |
| `-m, --monotonic` | Monotonic clock for elapsed modes (FR-007). | Same — matches moreutils `-m`. |
| `-r, --relative` | Recognized set: ISO-8601, RFC-3339, Unix epoch (FR-009). | Recognized set: full moreutils set (FR-025). |
| `-u, --utc` | Force UTC rendering (FR-018, Rusty extension). | **Rejected** — moreutils-only flag surface (FR-026). |
| `--tz` | Render in named IANA zone (FR-019, Rusty extension). | **Rejected** — moreutils-only flag surface (FR-026). |
| `--strict` | Switch into Strict mode for the invocation (FR-021). | Treated as already-consumed (no-op). |
| `--no-strict` | Force Default mode, overriding env/argv[0] (FR-021). | Treated as already-consumed (no-op). |
| `format` | Positional strftime; wins over `RUSTY_TS_FORMAT` env (FR-004, FR-027). | Positional strftime only; env var ignored (FR-027). |

## Subcommands

| Subcommand | Default mode | Strict mode |
|------------|--------------|-------------|
| `completions <shell>` | Writes shell completion script to stdout (FR-028). | **Rejected** — moreutils-only flag surface (FR-026). |

## Environment variables

| Variable | Default mode | Strict mode |
|----------|--------------|-------------|
| `TZ` | Honored via system local time (FR-017). | Same. |
| `RUSTY_TS_STRICT` | `1`/`true`/`yes` enables Strict mode (FR-022). | Same. |
| `RUSTY_TS_FORMAT` | Implicit format when no positional arg (FR-027). Empty = unset (default format). | **Ignored** (FR-027). |

## Exit codes

| Path | Default mode | Strict mode |
|------|--------------|-------------|
| Clean stdin EOF | `0` | `0` |
| Flag parse error | non-zero (clap default = `2`) | non-zero (clap default = `2`) |
| IO error on stdin/stdout (non-broken-pipe) | `1` | `1` |
| Broken-pipe on stdout | `0` (clean exit per HINT-004) | `0` |
| Unknown IANA name (`--tz=...`) | `2` | n/a (flag rejected) |
| `-u` + `--tz=...` mutex conflict | `2` (clap-enforced) | n/a (flags rejected) |
| Unknown flag (Rusty-only flag in Strict) | n/a | `2` (FR-026) |
