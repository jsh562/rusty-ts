//! Completions drift test per FR-028 / T106.
//!
//! Regenerates each shell's completion script from the canonical
//! `Cli::command()` and asserts equality with the committed file under
//! `completions/`. CI fails on drift.
//!
//! To accept a CLI change, run `UPDATE_COMPLETIONS=1 cargo test --test
//! completions_drift` to regenerate the four shell files in-place.

#![cfg(feature = "cli")]

use clap_complete::Shell;
use std::fs;
use std::path::PathBuf;

fn committed_path(shell: Shell) -> PathBuf {
    let filename = match shell {
        Shell::Bash => "rusty-ts.bash",
        Shell::Zsh => "_rusty-ts",
        Shell::Fish => "rusty-ts.fish",
        Shell::PowerShell => "rusty-ts.ps1",
        _ => unreachable!("unsupported shell"),
    };
    PathBuf::from("completions").join(filename)
}

fn generated_for(shell: Shell) -> Vec<u8> {
    let mut buf = Vec::new();
    rusty_ts::completions::emit_completions(shell, &mut buf).expect("emit");
    buf
}

#[test]
fn bash_completion_matches_committed() {
    check_shell(Shell::Bash);
}

#[test]
fn zsh_completion_matches_committed() {
    check_shell(Shell::Zsh);
}

#[test]
fn fish_completion_matches_committed() {
    check_shell(Shell::Fish);
}

#[test]
fn powershell_completion_matches_committed() {
    check_shell(Shell::PowerShell);
}

fn check_shell(shell: Shell) {
    let generated = generated_for(shell);
    let path = committed_path(shell);

    if std::env::var("UPDATE_COMPLETIONS").is_ok() {
        fs::write(&path, &generated).expect("write committed completion");
        eprintln!(
            "UPDATE_COMPLETIONS=1 set — overwrote {}. Review the diff in PR.",
            path.display()
        );
        return;
    }

    let committed = fs::read(&path).unwrap_or_else(|err| {
        panic!(
            "could not read {}: {err}. Run \
             `UPDATE_COMPLETIONS=1 cargo test --test completions_drift` \
             to regenerate.",
            path.display(),
        );
    });

    // Normalize CRLF on Windows checkouts: generator emits LF; the committed
    // file may have CRLF after git's autocrlf. Compare LF-normalized bytes.
    let normalize = |bytes: &[u8]| -> Vec<u8> {
        let s = String::from_utf8_lossy(bytes);
        s.replace("\r\n", "\n").into_bytes()
    };

    assert_eq!(
        normalize(&committed),
        normalize(&generated),
        "Completion for {shell:?} drifted from CLI definition. Run \
         `UPDATE_COMPLETIONS=1 cargo test --test completions_drift` \
         to regenerate, then commit the diff at {}.",
        path.display(),
    );
}
