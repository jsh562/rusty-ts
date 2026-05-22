//! Phase 3 MVP smoke test — exercises the default-format pipeline end-to-end
//! via `assert_cmd`. Verifies the SC-001 user-observable behavior: stdin lines
//! emerge on stdout with a moreutils-default-format timestamp prefix followed
//! by two spaces.
//!
//! This is NOT the full byte-level moreutils-parity snapshot suite (that
//! lives in `tests/compat_default.rs` and is populated per the Phase 3 task
//! list — T032, T033, T036, etc.). This file only proves the MVP runs.

use assert_cmd::Command;
use predicates::prelude::*;

/// Default-format moreutils ts output looks like `%b %d %H:%M:%S`:
/// three-letter month, space, two-digit day, space, HH:MM:SS, then two
/// spaces, then the payload. We assert the regex shape rather than a literal
/// string so the test is deterministic regardless of when it runs.
const DEFAULT_PREFIX_REGEX: &str = r"^[A-Z][a-z]{2} [ 0-9]\d \d{2}:\d{2}:\d{2}  (.*)$";

#[test]
fn default_format_two_lines() {
    let mut cmd = Command::cargo_bin("rusty-ts").expect("rusty-ts binary");
    cmd.env("TZ", "UTC")
        .env("LC_ALL", "C.UTF-8")
        .env_remove("RUSTY_TS_FORMAT")
        .write_stdin("hello\nworld\n");

    let output = cmd.assert().success();
    let stdout = String::from_utf8(output.get_output().stdout.clone())
        .expect("stdout is utf-8 in this fixture");

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        2,
        "expected exactly two output lines, got {lines:?}"
    );

    let re = regex::Regex::new(DEFAULT_PREFIX_REGEX).expect("regex compiles");
    assert!(
        re.is_match(lines[0]),
        "line 0 does not match default-format prefix shape: {:?}",
        lines[0],
    );
    assert!(
        re.is_match(lines[1]),
        "line 1 does not match default-format prefix shape: {:?}",
        lines[1],
    );

    // Payload preserved.
    assert!(
        lines[0].ends_with("  hello"),
        "line 0 payload: {:?}",
        lines[0]
    );
    assert!(
        lines[1].ends_with("  world"),
        "line 1 payload: {:?}",
        lines[1]
    );
}

#[test]
fn empty_stdin_clean_exit() {
    let mut cmd = Command::cargo_bin("rusty-ts").expect("rusty-ts binary");
    cmd.env("TZ", "UTC")
        .env("LC_ALL", "C.UTF-8")
        .env_remove("RUSTY_TS_FORMAT")
        .write_stdin("");

    cmd.assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

#[test]
fn partial_final_line_emits_with_prefix() {
    let mut cmd = Command::cargo_bin("rusty-ts").expect("rusty-ts binary");
    cmd.env("TZ", "UTC")
        .env("LC_ALL", "C.UTF-8")
        .env_remove("RUSTY_TS_FORMAT")
        .write_stdin("incomplete"); // no trailing newline

    let assertion = cmd.assert().success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone())
        .expect("stdout is utf-8 in this fixture");

    assert!(
        stdout.ends_with("  incomplete"),
        "expected payload preserved without added newline; got {stdout:?}",
    );
    assert!(
        !stdout.ends_with('\n'),
        "expected no trailing newline since input had none; got {stdout:?}",
    );
}
