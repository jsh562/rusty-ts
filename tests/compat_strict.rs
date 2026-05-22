//! Byte-equal compatibility tests covering custom format, fractional seconds,
//! binary passthrough, and Strict-mode error rejection — all against captured
//! moreutils `ts` output. Per FR-004, FR-008, FR-011, FR-026 / T036, T041,
//! T042, T075, T076, T077, T082, T083.
//!
//! Capture environment: moreutils `ts` Perl script from
//! https://raw.githubusercontent.com/madx/moreutils/master/ts (fetched
//! 2026-05-22), TZ=UTC, LC_ALL=C.UTF-8.

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

/// Helper: run rusty-ts with the supplied args + pinned clock, assert
/// stdout byte-matches the expected fixture.
fn assert_byte_equal(
    input_path: &str,
    expected_path: &str,
    fixed_clock_iso: &str,
    extra_args: &[&str],
) {
    let input = fs::read(input_path).unwrap_or_else(|e| panic!("read {input_path}: {e}"));
    let expected = fs::read(expected_path).unwrap_or_else(|e| panic!("read {expected_path}: {e}"));

    let mut cmd = Command::cargo_bin("rusty-ts").unwrap();
    common::fixture_envs(&mut cmd);
    cmd.env("RUSTY_TS_TEST_FIXED_CLOCK", fixed_clock_iso);
    cmd.arg("-u"); // force UTC to match capture env
    for arg in extra_args {
        cmd.arg(arg);
    }
    let assertion = cmd.write_stdin(input).assert().success();
    let actual = assertion.get_output().stdout.clone();
    assert_eq!(
        actual,
        expected,
        "byte mismatch for {input_path}\n  actual:   {:?}\n  expected: {:?}",
        String::from_utf8_lossy(&actual),
        String::from_utf8_lossy(&expected),
    );
}

// ────────────── T041 / T042 Custom format byte-equal ──────────────

#[test]
fn custom_format_iso_byte_equal() {
    assert_byte_equal(
        "fixtures/inputs/custom_format/iso.txt",
        "fixtures/moreutils_outputs/custom_format/iso.txt",
        "2026-05-22T23:20:23Z",
        &["%Y-%m-%d %H:%M:%S"],
    );
}

#[test]
fn custom_format_brackets_byte_equal() {
    assert_byte_equal(
        "fixtures/inputs/custom_format/brackets.txt",
        "fixtures/moreutils_outputs/custom_format/brackets.txt",
        "2026-05-22T23:20:23Z",
        &["[%H:%M:%S]"],
    );
}

#[test]
fn custom_format_with_year_byte_equal() {
    assert_byte_equal(
        "fixtures/inputs/custom_format/with_year.txt",
        "fixtures/moreutils_outputs/custom_format/with_year.txt",
        "2026-05-22T23:20:23Z",
        &["%Y %b %d %H:%M:%S"],
    );
}

// ────────────── T082 / T083 Fractional-second byte-equal ──────────────

#[test]
fn fractional_seconds_byte_equal() {
    // moreutils captured `23:20:23.090125 one` — pinned UTC instant with
    // microsecond precision so rusty-ts produces the same fractional digits.
    assert_byte_equal(
        "fixtures/inputs/fractional/seconds.txt",
        "fixtures/moreutils_outputs/fractional/seconds.txt",
        "2026-05-22T23:20:23.090125Z",
        &["%H:%M:%.S"],
    );
}

#[test]
fn fractional_epoch_byte_equal() {
    // moreutils captured `1779492023.104574 one` — different microsecond
    // because the captures ran at slightly different instants. Pin the
    // matching ISO so rusty-ts reproduces the exact bytes.
    assert_byte_equal(
        "fixtures/inputs/fractional/epoch.txt",
        "fixtures/moreutils_outputs/fractional/epoch.txt",
        "2026-05-22T23:20:23.104574Z",
        &["%.s"],
    );
}

// ────────────── T036 Binary passthrough byte-equal ──────────────

#[test]
fn binary_payload_passes_through_byte_equal() {
    assert_byte_equal(
        "fixtures/inputs/binary/passthrough.txt",
        "fixtures/moreutils_outputs/binary/passthrough.txt",
        "2026-05-22T23:20:23Z",
        &[], // default format
    );
}

// ────────────── T075 / T076 / T077 Strict-mode error byte-equal ──────────────
//
// In Strict mode, rusty-ts rejects Rusty-only flags with the exact
// moreutils stderr format: `Unknown option: <flag>\nusage: ts [-r] [format]\n`.

fn run_strict_rejection(flag: &str) -> (Vec<u8>, i32) {
    let mut cmd = Command::cargo_bin("rusty-ts").unwrap();
    common::fixture_envs(&mut cmd);
    cmd.env("RUSTY_TS_STRICT", "1");
    cmd.arg(flag);
    cmd.write_stdin("");
    // `.output()` returns std::process::Output without panicking on non-zero
    // exit (we *expect* non-zero for Strict-rejection). assert_cmd's
    // Command derefs to std::process::Command so this just works.
    let output = cmd.output().expect("run rusty-ts");
    (output.stderr, output.status.code().unwrap_or(0))
}

#[test]
fn strict_mode_rejects_dash_u_byte_equal() {
    let (stderr, code) = run_strict_rejection("-u");
    let expected =
        fs::read("fixtures/moreutils_outputs/strict_errors/unknown_u.txt").expect("read fixture");
    assert_eq!(
        stderr,
        expected,
        "stderr byte mismatch\n  actual:   {:?}\n  expected: {:?}",
        String::from_utf8_lossy(&stderr),
        String::from_utf8_lossy(&expected),
    );
    assert_ne!(code, 0, "Strict rejection must exit non-zero");
}

#[test]
fn strict_mode_rejects_tz_byte_equal() {
    let (stderr, code) = run_strict_rejection("--tz=Asia/Tokyo");
    let expected =
        fs::read("fixtures/moreutils_outputs/strict_errors/unknown_tz.txt").expect("read fixture");
    assert_eq!(
        stderr,
        expected,
        "stderr byte mismatch\n  actual:   {:?}\n  expected: {:?}",
        String::from_utf8_lossy(&stderr),
        String::from_utf8_lossy(&expected),
    );
    assert_ne!(code, 0, "Strict rejection must exit non-zero");
}
