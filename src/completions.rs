//! `completions <shell>` subcommand implementation.
//!
//! Per FR-028 and `plan.md` AD-002: `Cli::command()` from `src/cli.rs` is
//! the single source of truth; `clap_complete::generate` walks it for any
//! supported shell. The output is written to the supplied `Write` (stdout
//! when invoked from the CLI; a `Vec<u8>` buffer when invoked from the
//! drift test in `tests/completions_drift.rs`).
//!
//! Gated behind the `cli` cargo feature (transitively via the
//! `clap_complete` dep) so library consumers building with
//! `--no-default-features` do not pull in the completions surface.

#![cfg(feature = "cli")]

use crate::cli::Cli;
use clap::CommandFactory;
use clap_complete::{Shell, generate};
use std::io::Write;

/// Write a completion script for the given shell to the supplied writer.
/// Returns the number of bytes written semantics via `Write`.
pub fn emit_completions<W: Write>(shell: Shell, writer: &mut W) -> std::io::Result<()> {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    generate(shell, &mut cmd, bin_name, writer);
    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap_complete::Shell;

    #[test]
    fn bash_completion_script_is_non_empty() {
        let mut buf = Vec::new();
        emit_completions(Shell::Bash, &mut buf).expect("emit ok");
        let s = String::from_utf8(buf).expect("utf-8");
        assert!(s.contains("rusty-ts"));
        assert!(!s.is_empty());
    }

    #[test]
    fn all_four_shells_emit_non_empty() {
        for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell] {
            let mut buf = Vec::new();
            emit_completions(shell, &mut buf).expect("emit ok");
            assert!(!buf.is_empty(), "shell {shell:?} produced empty output");
        }
    }
}
