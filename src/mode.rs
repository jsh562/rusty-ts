//! Compatibility Mode resolution.
//!
//! Per `plan.md` AD-004 and `spec.md` FR-021..FR-023:
//!
//! Resolves the `CompatibilityMode` for an invocation once at startup at
//! zero per-line cost. Precedence (highest first):
//!
//! 1. Explicit `--no-strict` / `--no-moreutils-compat` flag → `Default`
//! 2. Explicit `--strict` / `--moreutils-compat` flag → `Strict`
//! 3. `RUSTY_TS_STRICT` env var (`1`/`true`/`yes` = on; anything else = off)
//! 4. argv[0] basename auto-detect: `ts` (or `ts.exe` stripped on Windows)
//!    → `Strict`
//! 5. Default
//!
//! Encoded as a single pure function `resolve(...)` so the precedence is
//! testable in isolation (HINT-002 — the precedence ladder lives in exactly
//! one place).

/// The resolved compatibility-mode posture for the invocation.
///
/// Marked `#[non_exhaustive]` so future modes (e.g., explicit moreutils
/// version pinning) can be added in minor versions.
///
/// # Example
///
/// ```
/// use rusty_ts::{CompatibilityMode, TimestamperBuilder};
///
/// // Default mode — Rusty extensions active.
/// let ts = TimestamperBuilder::new()
///     .compat(CompatibilityMode::Default)
///     .build()
///     .unwrap();
///
/// // Strict mode — byte-identical moreutils behavior.
/// let ts = TimestamperBuilder::new()
///     .compat(CompatibilityMode::Strict)
///     .build()
///     .unwrap();
/// # let _ = ts;
/// ```
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompatibilityMode {
    /// Rusty enhancements active: `-u`, `--tz`, `RUSTY_TS_FORMAT`, `-r`
    /// subset, completions subcommand, Rusty-flavored `--help`.
    #[default]
    Default,
    /// Byte-identical moreutils behavior: Rusty-only flags rejected,
    /// `-r` expanded to full moreutils set, `RUSTY_TS_FORMAT` ignored,
    /// `--help` / `--version` mirror moreutils layout.
    Strict,
}

/// Explicit user choice from the CLI flag layer.
///
/// `None` if neither `--strict` nor `--no-strict` was supplied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplicitChoice {
    /// User passed `--strict` or `--moreutils-compat`.
    Strict,
    /// User passed `--no-strict` or `--no-moreutils-compat`.
    Default,
}

/// Resolve the compatibility mode from the three inputs.
///
/// - `explicit_flag` — `Some(_)` if the user passed `--strict` / `--no-strict`.
/// - `env_strict` — value of the `RUSTY_TS_STRICT` env var if present.
/// - `argv0_basename` — argv[0] basename with any platform extension stripped.
///
/// Per FR-021 precedence, `--no-strict` beats `--strict`, which beats
/// `RUSTY_TS_STRICT`, which beats argv[0] auto-detect.
pub fn resolve(
    explicit_flag: Option<ExplicitChoice>,
    env_strict: Option<&str>,
    argv0_basename: Option<&str>,
) -> CompatibilityMode {
    match explicit_flag {
        Some(ExplicitChoice::Default) => return CompatibilityMode::Default,
        Some(ExplicitChoice::Strict) => return CompatibilityMode::Strict,
        None => {}
    }

    if let Some(value) = env_strict {
        if env_var_is_truthy(value) {
            return CompatibilityMode::Strict;
        }
    }

    if let Some(name) = argv0_basename {
        if name.eq_ignore_ascii_case("ts") {
            return CompatibilityMode::Strict;
        }
    }

    CompatibilityMode::Default
}

/// Parse an env-var value the way Unix-y tools usually do.
///
/// `1`, `true`, `yes`, `on` (any case) → enabled. Everything else → disabled.
fn env_var_is_truthy(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

/// Extract the argv[0] basename, stripping a `.exe` extension on Windows.
///
/// Returns `None` if argv is empty or the basename is unrecognizable. Used
/// by FR-023 to auto-detect Strict mode when the binary is invoked as `ts`.
pub fn argv0_basename(argv0: &std::ffi::OsStr) -> Option<String> {
    use std::path::Path;
    Path::new(argv0)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_no_strict_wins_over_everything() {
        let mode = resolve(Some(ExplicitChoice::Default), Some("1"), Some("ts"));
        assert_eq!(mode, CompatibilityMode::Default);
    }

    #[test]
    fn explicit_strict_wins_over_env_and_argv() {
        let mode = resolve(Some(ExplicitChoice::Strict), Some("0"), Some("rusty-ts"));
        assert_eq!(mode, CompatibilityMode::Strict);
    }

    #[test]
    fn env_truthy_enables_strict() {
        for truthy in ["1", "true", "TRUE", "yes", "Yes", "on", "  1  "] {
            assert_eq!(
                resolve(None, Some(truthy), Some("rusty-ts")),
                CompatibilityMode::Strict,
                "env {truthy:?} should enable Strict",
            );
        }
    }

    #[test]
    fn env_falsy_or_unset_falls_through() {
        for falsy in ["0", "false", "no", "off", "", "  "] {
            assert_eq!(
                resolve(None, Some(falsy), Some("rusty-ts")),
                CompatibilityMode::Default,
                "env {falsy:?} should not enable Strict",
            );
        }
        assert_eq!(
            resolve(None, None, Some("rusty-ts")),
            CompatibilityMode::Default,
        );
    }

    #[test]
    fn argv0_ts_enables_strict() {
        assert_eq!(resolve(None, None, Some("ts")), CompatibilityMode::Strict);
    }

    #[test]
    fn argv0_ts_case_insensitive() {
        assert_eq!(resolve(None, None, Some("TS")), CompatibilityMode::Strict);
        assert_eq!(resolve(None, None, Some("Ts")), CompatibilityMode::Strict);
    }

    #[test]
    fn argv0_rusty_ts_stays_default() {
        assert_eq!(
            resolve(None, None, Some("rusty-ts")),
            CompatibilityMode::Default,
        );
    }

    #[test]
    fn argv0_basename_strips_exe() {
        use std::ffi::OsStr;
        assert_eq!(argv0_basename(OsStr::new("ts.exe")).as_deref(), Some("ts"));
        assert_eq!(argv0_basename(OsStr::new("ts")).as_deref(), Some("ts"));
        assert_eq!(argv0_basename(OsStr::new("./ts")).as_deref(), Some("ts"),);
    }

    #[test]
    fn argv0_basename_handles_path_components() {
        use std::ffi::OsStr;
        // On Unix this becomes "ts"; on Windows, paths like "C:\\bin\\ts.exe"
        // are tested via the matching path-separator handling in std::path.
        assert_eq!(
            argv0_basename(OsStr::new("/usr/local/bin/ts")).as_deref(),
            Some("ts"),
        );
    }

    /// Exhaustive truth table covering the FR-021 precedence ladder.
    #[test]
    fn precedence_table() {
        // (explicit, env, argv0_basename) -> expected
        type Row = (
            Option<ExplicitChoice>,
            Option<&'static str>,
            Option<&'static str>,
            CompatibilityMode,
        );
        let cases: &[Row] = &[
            // Default everywhere
            (None, None, None, CompatibilityMode::Default),
            (None, None, Some("rusty-ts"), CompatibilityMode::Default),
            // argv[0] = ts
            (None, None, Some("ts"), CompatibilityMode::Strict),
            // env truthy
            (None, Some("1"), Some("rusty-ts"), CompatibilityMode::Strict),
            (None, Some("true"), None, CompatibilityMode::Strict),
            // env falsy
            (None, Some("0"), Some("ts"), CompatibilityMode::Strict), // argv wins over falsy env
            (
                None,
                Some("0"),
                Some("rusty-ts"),
                CompatibilityMode::Default,
            ),
            // explicit --strict
            (
                Some(ExplicitChoice::Strict),
                Some("0"),
                Some("rusty-ts"),
                CompatibilityMode::Strict,
            ),
            (
                Some(ExplicitChoice::Strict),
                None,
                Some("ts"),
                CompatibilityMode::Strict,
            ),
            // explicit --no-strict beats env and argv
            (
                Some(ExplicitChoice::Default),
                Some("1"),
                Some("ts"),
                CompatibilityMode::Default,
            ),
            (
                Some(ExplicitChoice::Default),
                None,
                Some("ts"),
                CompatibilityMode::Default,
            ),
        ];

        for (i, (explicit, env, argv, expected)) in cases.iter().enumerate() {
            let actual = resolve(*explicit, *env, *argv);
            assert_eq!(
                actual, *expected,
                "case {i}: explicit={explicit:?} env={env:?} argv={argv:?} expected {expected:?} got {actual:?}",
            );
        }
    }
}
