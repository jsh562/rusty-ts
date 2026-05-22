//! CLI surface — clap-derive `Cli` struct, post-parse validation, and
//! mode-aware dispatch helpers.
//!
//! Per `plan.md` AD-002: clap derive with the `env` feature for
//! `RUSTY_TS_FORMAT` honoring; same `Cli::command()` is consumed by
//! `clap_complete` (completions) and `compat_matrix` (drift test). Single
//! source of truth.

use crate::error::Error;
use crate::mode::ExplicitChoice;
use clap::{Parser, Subcommand};

/// `rusty-ts` — prefix each line of stdin with a timestamp. A Rust port of
/// moreutils `ts`.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "rusty-ts",
    version,
    about = "Prefix each line of stdin with a timestamp (Rust port of moreutils ts)",
    long_about = None,
)]
pub struct Cli {
    // ─────────────────────── Elapsed-time mode flags ───────────────────────
    /// Render elapsed time since the previous input line instead of absolute time.
    #[arg(
        short = 'i',
        long = "incremental",
        conflicts_with = "since_start",
        action = clap::ArgAction::SetTrue
    )]
    pub incremental: bool,

    /// Render elapsed time since program start instead of absolute time.
    #[arg(
        short = 's',
        long = "since-start",
        conflicts_with = "incremental",
        action = clap::ArgAction::SetTrue
    )]
    pub since_start: bool,

    /// Use a monotonic clock source for elapsed-time calculations.
    /// Has no effect unless `-i` or `-s` is also present.
    #[arg(short = 'm', long = "monotonic", action = clap::ArgAction::SetTrue)]
    pub monotonic: bool,

    // ─────────────────────── Relative-mode flag ────────────────────────────
    /// Convert recognized in-line timestamps to relative form rather than
    /// prefixing new timestamps. Default mode recognizes ISO-8601, RFC-3339,
    /// and Unix epoch; Strict mode expands to the full moreutils set.
    #[arg(short = 'r', long = "relative", action = clap::ArgAction::SetTrue)]
    pub relative: bool,

    // ─────────────────────── Timezone control ──────────────────────────────
    /// Force timestamps to be rendered in UTC, overriding system local time
    /// and the `TZ` env var. Rejected in Strict mode.
    #[arg(
        short = 'u',
        long = "utc",
        conflicts_with = "tz",
        action = clap::ArgAction::SetTrue
    )]
    pub utc: bool,

    /// Render timestamps in the named IANA timezone (e.g., `America/New_York`).
    /// Resolved once at startup; per-line render cost is a fixed-offset
    /// conversion. Rejected in Strict mode.
    #[arg(long = "tz", value_name = "IANA-NAME", conflicts_with = "utc")]
    pub tz: Option<String>,

    // ─────────────────────── Compatibility-mode toggles ────────────────────
    /// Switch into Strict moreutils Compatibility Mode. Rejects `-u`, `--tz`,
    /// and other Rusty-only flags; expands `-r` to the full moreutils set;
    /// mirrors moreutils `--help` / `--version` layout; ignores
    /// `RUSTY_TS_FORMAT`.
    #[arg(
        long = "strict",
        alias = "moreutils-compat",
        conflicts_with = "no_strict",
        action = clap::ArgAction::SetTrue
    )]
    pub strict: bool,

    /// Force Default mode, overriding `RUSTY_TS_STRICT` env var and argv[0]
    /// auto-detection.
    #[arg(
        long = "no-strict",
        alias = "no-moreutils-compat",
        conflicts_with = "strict",
        action = clap::ArgAction::SetTrue
    )]
    pub no_strict: bool,

    // ─────────────────────── Positional format ─────────────────────────────
    /// Optional strftime format string. If omitted, uses the moreutils
    /// default format (`%b %d %H:%M:%S`) or the `RUSTY_TS_FORMAT` env var
    /// (Default mode only). A positional argument always wins over the env var.
    #[arg(value_name = "FORMAT")]
    pub format: Option<String>,

    // ─────────────────────── Subcommands ──────────────────────────────────
    #[command(subcommand)]
    pub subcommand: Option<CliCommand>,
}

/// Subcommands. Currently just `completions`; future ports may add more.
#[derive(Subcommand, Debug, Clone)]
pub enum CliCommand {
    /// Generate shell-completion scripts for bash, zsh, fish, or powershell.
    /// Writes to stdout.
    Completions {
        /// Target shell.
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

impl Cli {
    /// Compute the explicit-choice signal for the mode resolver. Returns
    /// `None` if neither `--strict` nor `--no-strict` was supplied.
    pub fn explicit_compat_choice(&self) -> Option<ExplicitChoice> {
        match (self.strict, self.no_strict) {
            (true, false) => Some(ExplicitChoice::Strict),
            (false, true) => Some(ExplicitChoice::Default),
            (false, false) => None,
            // clap's `conflicts_with` prevents this pair, but defend in
            // depth.
            (true, true) => None,
        }
    }

    /// Post-parse validation per FR-020 (defense in depth alongside clap's
    /// `conflicts_with`). Returns `Error::InvalidUtcWithNamedTz` if both `-u`
    /// and `--tz=...` were supplied.
    pub fn validate(&self) -> Result<(), Error> {
        if self.utc {
            if let Some(tz) = &self.tz {
                return Err(Error::InvalidUtcWithNamedTz { tz: tz.clone() });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_command_factory_builds_without_panic() {
        // Smoke test: clap derive metadata compiles and yields a valid
        // Command tree. Used by `completions` and `compat_matrix`.
        let _cmd = Cli::command();
    }

    #[test]
    fn explicit_choice_signals_strict_when_strict_flag_set() {
        let cli = Cli::parse_from(["rusty-ts", "--strict"]);
        assert_eq!(cli.explicit_compat_choice(), Some(ExplicitChoice::Strict));
    }

    #[test]
    fn explicit_choice_signals_default_when_no_strict_flag_set() {
        let cli = Cli::parse_from(["rusty-ts", "--no-strict"]);
        assert_eq!(cli.explicit_compat_choice(), Some(ExplicitChoice::Default));
    }

    #[test]
    fn explicit_choice_none_when_neither_flag() {
        let cli = Cli::parse_from(["rusty-ts"]);
        assert_eq!(cli.explicit_compat_choice(), None);
    }

    #[test]
    fn positional_format_captured() {
        let cli = Cli::parse_from(["rusty-ts", "%Y-%m-%d %H:%M:%S"]);
        assert_eq!(cli.format.as_deref(), Some("%Y-%m-%d %H:%M:%S"));
    }

    #[test]
    fn utc_flag_parsed() {
        let cli = Cli::parse_from(["rusty-ts", "-u"]);
        assert!(cli.utc);
        assert!(cli.tz.is_none());
    }

    #[test]
    fn tz_flag_parsed() {
        let cli = Cli::parse_from(["rusty-ts", "--tz=Asia/Tokyo"]);
        assert_eq!(cli.tz.as_deref(), Some("Asia/Tokyo"));
        assert!(!cli.utc);
    }

    #[test]
    fn utc_and_tz_rejected_by_clap_conflict() {
        let err = Cli::try_parse_from(["rusty-ts", "-u", "--tz=Asia/Tokyo"]);
        assert!(
            err.is_err(),
            "clap should reject -u + --tz=... via conflicts_with"
        );
    }

    #[test]
    fn incremental_and_since_start_rejected_by_clap_conflict() {
        let err = Cli::try_parse_from(["rusty-ts", "-i", "-s"]);
        assert!(
            err.is_err(),
            "clap should reject -i + -s via conflicts_with"
        );
    }

    #[test]
    fn validate_passes_when_only_utc() {
        let cli = Cli::parse_from(["rusty-ts", "-u"]);
        assert!(cli.validate().is_ok());
    }

    #[test]
    fn completions_subcommand_parsed() {
        let cli = Cli::parse_from(["rusty-ts", "completions", "bash"]);
        assert!(matches!(
            cli.subcommand,
            Some(CliCommand::Completions { shell: _ })
        ));
    }
}
