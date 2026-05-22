//! Compatibility Matrix generator.
//!
//! Per `spec.md` FR-030 and `plan.md` AD-006: walks `Cli::command()` and
//! emits the README Compatibility Matrix as Markdown. The committed
//! `docs/COMPATIBILITY.md` file is asserted to match the generator's
//! output by `tests/compat_matrix.rs` so the matrix cannot drift from
//! the CLI definition.
//!
//! The matrix is the canonical contract surface for both consumers (the
//! README points at it) and tests (`compat_matrix` integration test).

#![cfg(feature = "cli")]

use crate::cli::Cli;
use clap::CommandFactory;
use std::fmt::Write;

/// Render the Compatibility Matrix as a Markdown document. Suitable for
/// writing to `docs/COMPATIBILITY.md`.
pub fn generate_matrix() -> String {
    let mut out = String::with_capacity(4096);

    writeln!(
        out,
        "# rusty-ts Compatibility Matrix\n\n\
         This file is **generated** from the CLI definition by \
         `cargo test --test compat_matrix`. Do not edit by hand — any \
         change must come from `src/cli.rs`. CI fails on drift.\n"
    )
    .ok();

    writeln!(
        out,
        "## TZ-pinning disclosure\n\n\
         Byte-level fidelity against moreutils `ts` is verified under \
         `TZ=UTC` and `LC_ALL=C.UTF-8` (see `fixtures/README.md` for the \
         full capture protocol). Snapshot tests refuse to run if these \
         env vars are not pinned.\n"
    )
    .ok();

    writeln!(out, "## Flags\n").ok();
    writeln!(
        out,
        "| Flag | Default mode | Strict mode |\n\
         |------|--------------|-------------|"
    )
    .ok();

    let cmd = Cli::command();
    let mut rows: Vec<(String, String, String)> = Vec::new();

    for arg in cmd.get_arguments() {
        let id = arg.get_id().as_str();
        // Skip the implicit `help` / `version` clap auto-args; we list
        // them in the Subcommands / `--help` discussion separately.
        if id == "help" || id == "version" {
            continue;
        }

        let flag_label = format_flag_label(arg);
        let (default_behavior, strict_behavior) = match id {
            "incremental" => (
                "Elapsed since previous line (FR-005).",
                "Same — matches moreutils `-i`.",
            ),
            "since_start" => (
                "Elapsed since program start (FR-006).",
                "Same — matches moreutils `-s`.",
            ),
            "monotonic" => (
                "Monotonic clock for elapsed modes (FR-007).",
                "Same — matches moreutils `-m`.",
            ),
            "relative" => (
                "Recognized set: ISO-8601, RFC-3339, Unix epoch (FR-009).",
                "Recognized set: full moreutils set (FR-025).",
            ),
            "utc" => (
                "Force UTC rendering (FR-018, Rusty extension).",
                "**Rejected** — moreutils-only flag surface (FR-026).",
            ),
            "tz" => (
                "Render in named IANA zone (FR-019, Rusty extension).",
                "**Rejected** — moreutils-only flag surface (FR-026).",
            ),
            "strict" => (
                "Switch into Strict mode for the invocation (FR-021).",
                "Treated as already-consumed (no-op).",
            ),
            "no_strict" => (
                "Force Default mode, overriding env/argv[0] (FR-021).",
                "Treated as already-consumed (no-op).",
            ),
            "format" => (
                "Positional strftime; wins over `RUSTY_TS_FORMAT` env (FR-004, FR-027).",
                "Positional strftime only; env var ignored (FR-027).",
            ),
            _ => ("(undocumented)", "(undocumented)"),
        };

        rows.push((
            flag_label,
            default_behavior.to_string(),
            strict_behavior.to_string(),
        ));
    }

    for (flag, default_b, strict_b) in &rows {
        writeln!(out, "| `{flag}` | {default_b} | {strict_b} |").ok();
    }

    writeln!(out, "\n## Subcommands\n").ok();
    writeln!(
        out,
        "| Subcommand | Default mode | Strict mode |\n\
         |------------|--------------|-------------|\n\
         | `completions <shell>` | Writes shell completion script to stdout \
         (FR-028). | **Rejected** — moreutils-only flag surface (FR-026). |"
    )
    .ok();

    writeln!(out, "\n## Environment variables\n").ok();
    writeln!(
        out,
        "| Variable | Default mode | Strict mode |\n\
         |----------|--------------|-------------|\n\
         | `TZ` | Honored via system local time (FR-017). | Same. |\n\
         | `RUSTY_TS_STRICT` | `1`/`true`/`yes` enables Strict mode (FR-022). \
         | Same. |\n\
         | `RUSTY_TS_FORMAT` | Implicit format when no positional arg \
         (FR-027). Empty = unset (default format). | **Ignored** (FR-027). |"
    )
    .ok();

    writeln!(out, "\n## Exit codes\n").ok();
    writeln!(
        out,
        "| Path | Default mode | Strict mode |\n\
         |------|--------------|-------------|\n\
         | Clean stdin EOF | `0` | `0` |\n\
         | Flag parse error | non-zero (clap default = `2`) | non-zero (clap default = `2`) |\n\
         | IO error on stdin/stdout (non-broken-pipe) | `1` | `1` |\n\
         | Broken-pipe on stdout | `0` (clean exit per HINT-004) | `0` |\n\
         | Unknown IANA name (`--tz=...`) | `2` | n/a (flag rejected) |\n\
         | `-u` + `--tz=...` mutex conflict | `2` (clap-enforced) | n/a (flags rejected) |\n\
         | Unknown flag (Rusty-only flag in Strict) | n/a | `2` (FR-026) |"
    )
    .ok();

    out
}

fn format_flag_label(arg: &clap::Arg) -> String {
    let short = arg.get_short();
    let long = arg.get_long();
    match (short, long) {
        (Some(s), Some(l)) => format!("-{s}, --{l}"),
        (Some(s), None) => format!("-{s}"),
        (None, Some(l)) => format!("--{l}"),
        (None, None) => arg.get_id().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_is_non_empty_and_lists_known_flags() {
        let m = generate_matrix();
        assert!(m.contains("Compatibility Matrix"));
        // Every expected flag appears.
        for needle in [
            "-i",
            "-s",
            "-m",
            "-r",
            "-u",
            "--tz",
            "--strict",
            "completions",
        ] {
            assert!(
                m.contains(needle),
                "expected matrix to mention {needle:?}; matrix={m}",
            );
        }
        // TZ-pinning disclosure required by FR-015.
        assert!(m.contains("TZ=UTC"));
        assert!(m.contains("LC_ALL=C.UTF-8"));
        // SC-023 exit-code rows.
        assert!(m.contains("Clean stdin EOF"));
        assert!(m.contains("Flag parse error"));
        assert!(m.contains("IO error"));
        assert!(m.contains("Broken-pipe"));
        assert!(m.contains("Unknown IANA name"));
        assert!(m.contains("mutex conflict"));
    }

    #[test]
    fn matrix_is_deterministic() {
        let a = generate_matrix();
        let b = generate_matrix();
        assert_eq!(a, b, "matrix generation must be deterministic");
    }
}
