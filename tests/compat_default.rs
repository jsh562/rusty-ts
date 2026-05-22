//! Byte-equal snapshot tests against captured moreutils `ts` output.
//!
//! Per FR-001 / SC-001 / T032+T033: each fixture pair under
//! `fixtures/inputs/default/` and `fixtures/moreutils_outputs/default/`
//! is fed through rusty-ts with the clock pinned to the timestamp moreutils
//! emitted at capture time. rusty-ts output is compared byte-for-byte to
//! the captured moreutils output. Failure means behavioral divergence
//! from moreutils `ts` for the documented default-format surface.
//!
//! Capture protocol (per fixtures/README.md): moreutils source pinned at
//! the madx mirror master HEAD as of 2026-05-22; TZ=UTC; LC_ALL=C.UTF-8.
//! Captured timestamps documented in
//! `fixtures/moreutils_outputs/default/CAPTURE.json`.

#![cfg(feature = "cli")]

use assert_cmd::Command;
use std::fs;

mod common {
    pub fn fixture_envs(cmd: &mut assert_cmd::Command) {
        cmd.env("TZ", "UTC")
            .env("LC_ALL", "C.UTF-8")
            .env_remove("RUSTY_TS_FORMAT")
            .env_remove("RUSTY_TS_STRICT");
    }
}

/// Run rusty-ts with the fixed-clock env var pinned to the captured-at
/// timestamp, then assert byte-equal output against the moreutils output.
fn assert_byte_equal_against_moreutils(name: &str, captured_at_iso: Option<&str>) {
    let input_path = format!("fixtures/inputs/default/{name}.txt");
    let expected_path = format!("fixtures/moreutils_outputs/default/{name}.txt");

    let input = fs::read(&input_path).unwrap_or_else(|e| panic!("read {input_path}: {e}"));
    let expected = fs::read(&expected_path).unwrap_or_else(|e| panic!("read {expected_path}: {e}"));

    let mut cmd = Command::cargo_bin("rusty-ts").unwrap();
    common::fixture_envs(&mut cmd);
    if let Some(iso) = captured_at_iso {
        cmd.env("RUSTY_TS_TEST_FIXED_CLOCK", iso);
    }
    cmd.arg("-u"); // force UTC to match the capture env

    let assertion = cmd.write_stdin(input).assert().success();
    let actual = assertion.get_output().stdout.clone();

    assert_eq!(
        actual,
        expected,
        "rusty-ts output does not byte-match moreutils fixture {name:?}\n  \
         actual ({} bytes):   {:?}\n  \
         expected ({} bytes): {:?}",
        actual.len(),
        String::from_utf8_lossy(&actual),
        expected.len(),
        String::from_utf8_lossy(&expected),
    );
}

#[test]
fn default_format_three_lines_byte_equal() {
    assert_byte_equal_against_moreutils("three_lines", Some("2026-05-22T20:05:37Z"));
}

#[test]
fn default_format_single_line_byte_equal() {
    assert_byte_equal_against_moreutils("single_line", Some("2026-05-22T20:05:37Z"));
}

#[test]
fn default_format_empty_input_byte_equal() {
    // Empty input → empty output, regardless of clock.
    assert_byte_equal_against_moreutils("empty", None);
}
