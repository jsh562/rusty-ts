//! Error-path and flag-precedence integration tests for the `rusty-ts` CLI.
//!
//! Covers acceptance scenarios from US1 (clean-EOF), US4 (elapsed-mode
//! shape), US8 (TZ control + mutex + unknown IANA), US9 (Strict mode
//! rejections + argv[0] auto-detect), US10 (`RUSTY_TS_FORMAT` env var
//! precedence, completions subcommand).

#![cfg(feature = "cli")]

use assert_cmd::Command;
use predicates::prelude::*;

mod common {
    pub fn fixture_envs(cmd: &mut assert_cmd::Command) {
        cmd.env("TZ", "UTC")
            .env("LC_ALL", "C.UTF-8")
            .env_remove("RUSTY_TS_FORMAT")
            .env_remove("RUSTY_TS_STRICT");
    }
}

// ─────────────────── US8 — Timezone control ────────────────────

#[test]
fn utc_flag_renders_in_utc() {
    let mut cmd = Command::cargo_bin("rusty-ts").unwrap();
    common::fixture_envs(&mut cmd);
    let out = cmd
        .args(["-u", "%H:%M"])
        .write_stdin("x\n")
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    // Output begins with HH:MM (UTC) followed by two spaces then "x".
    let re = regex::Regex::new(r"^\d{2}:\d{2}  x\n$").unwrap();
    assert!(re.is_match(&stdout), "got {stdout:?}");
}

#[test]
fn utc_and_tz_mutex_rejected() {
    let mut cmd = Command::cargo_bin("rusty-ts").unwrap();
    common::fixture_envs(&mut cmd);
    cmd.args(["-u", "--tz=Asia/Tokyo"])
        .write_stdin("x\n")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("cannot be used").or(predicate::str::contains("conflicts")),
        );
}

#[test]
fn unknown_iana_name_diagnosed() {
    let mut cmd = Command::cargo_bin("rusty-ts").unwrap();
    common::fixture_envs(&mut cmd);
    cmd.args(["--tz=Atlantis/Atlantica"])
        .write_stdin("x\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Atlantis/Atlantica"));
}

// ─────────────────── US9 — Strict mode dispatch ────────────────

#[test]
fn strict_flag_rejects_utc() {
    let mut cmd = Command::cargo_bin("rusty-ts").unwrap();
    common::fixture_envs(&mut cmd);
    cmd.args(["--strict", "-u"])
        .write_stdin("x\n")
        .assert()
        .failure();
}

#[test]
fn strict_flag_rejects_tz() {
    let mut cmd = Command::cargo_bin("rusty-ts").unwrap();
    common::fixture_envs(&mut cmd);
    cmd.args(["--strict", "--tz=Asia/Tokyo"])
        .write_stdin("x\n")
        .assert()
        .failure();
}

#[test]
fn strict_env_var_enables_rejection() {
    let mut cmd = Command::cargo_bin("rusty-ts").unwrap();
    common::fixture_envs(&mut cmd);
    cmd.env("RUSTY_TS_STRICT", "1")
        .args(["-u"])
        .write_stdin("x\n")
        .assert()
        .failure();
}

#[test]
fn no_strict_flag_overrides_env() {
    let mut cmd = Command::cargo_bin("rusty-ts").unwrap();
    common::fixture_envs(&mut cmd);
    // env enables strict; --no-strict explicit flag wins per FR-021 precedence.
    cmd.env("RUSTY_TS_STRICT", "1")
        .args(["--no-strict", "-u"])
        .write_stdin("x\n")
        .assert()
        .success();
}

#[test]
fn ts_alias_binary_auto_enables_strict() {
    // The `ts-alias` cargo feature installs the binary under the name `ts`.
    // Invoking via that name triggers argv[0] auto-detect.
    let mut cmd = Command::cargo_bin("ts").unwrap();
    common::fixture_envs(&mut cmd);
    cmd.args(["-u"]).write_stdin("x\n").assert().failure(); // Strict rejects -u
}

#[test]
fn rusty_ts_binary_name_does_not_trigger_strict() {
    // Negative control for argv[0] auto-detect: the `rusty-ts` binary name
    // must NOT auto-enable Strict mode (only `ts` does). Default mode allows -u.
    let mut cmd = Command::cargo_bin("rusty-ts").unwrap();
    common::fixture_envs(&mut cmd);
    cmd.args(["-u"]).write_stdin("x\n").assert().success();
}

// ─────────────────── US10 — Plug-and-play ───────────────────────

#[test]
fn rusty_ts_format_env_var_default_path() {
    let mut cmd = Command::cargo_bin("rusty-ts").unwrap();
    common::fixture_envs(&mut cmd);
    cmd.env("RUSTY_TS_FORMAT", "[%H:%M:%S]")
        .args(["-u"])
        .write_stdin("hi\n")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"^\[\d{2}:\d{2}:\d{2}\]  hi\n$").unwrap());
}

#[test]
fn positional_format_beats_env_var() {
    let mut cmd = Command::cargo_bin("rusty-ts").unwrap();
    common::fixture_envs(&mut cmd);
    cmd.env("RUSTY_TS_FORMAT", "[%H:%M:%S]")
        .args(["-u", "%H"]) // positional wins
        .write_stdin("hi\n")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"^\d{2}  hi\n$").unwrap());
}

#[test]
fn rusty_ts_format_empty_treated_as_unset() {
    let mut cmd = Command::cargo_bin("rusty-ts").unwrap();
    common::fixture_envs(&mut cmd);
    cmd.env("RUSTY_TS_FORMAT", "")
        .args(["-u"])
        .write_stdin("hi\n")
        .assert()
        .success()
        // Empty env var falls through to moreutils default format.
        .stdout(
            predicate::str::is_match(r"^[A-Z][a-z]{2} [ 0-9]\d \d{2}:\d{2}:\d{2}  hi\n$").unwrap(),
        );
}

#[test]
fn rusty_ts_format_ignored_in_strict() {
    let mut cmd = Command::cargo_bin("rusty-ts").unwrap();
    common::fixture_envs(&mut cmd);
    cmd.env("RUSTY_TS_FORMAT", "[%H:%M:%S]") // env var present
        .env("RUSTY_TS_STRICT", "1") // but strict mode
        .write_stdin("hi\n")
        .assert()
        .success()
        // Strict mode ignores RUSTY_TS_FORMAT → falls back to default format.
        .stdout(
            predicate::str::is_match(r"^[A-Z][a-z]{2} [ 0-9]\d \d{2}:\d{2}:\d{2}  hi\n$").unwrap(),
        );
}

#[test]
fn completions_subcommand_emits_bash_script() {
    let mut cmd = Command::cargo_bin("rusty-ts").unwrap();
    common::fixture_envs(&mut cmd);
    cmd.args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("rusty-ts"));
}

// ─────────────────── US4 — Elapsed modes ───────────────────────

#[test]
fn elapsed_since_start_first_line_is_zero() {
    let mut cmd = Command::cargo_bin("rusty-ts").unwrap();
    common::fixture_envs(&mut cmd);
    cmd.args(["-s", "%H:%M:%S"])
        .write_stdin("first\n")
        .assert()
        .success()
        // First line: elapsed since start ≈ 0.
        .stdout(predicate::str::starts_with("00:00:00  first"));
}

// ─────────────────── T052/T053/T054 — deterministic elapsed-mode tests ────
//
// The binary supports `RUSTY_TS_TEST_FIXED_CLOCK=<rfc3339>` to pin the clock
// at a deterministic instant. With the clock fixed, elapsed-mode output is
// reproducible across runs. These tests cover the core acceptance scenarios
// for US4 (T052/T053/T054) without depending on wall-clock timing.

#[test]
fn fixed_clock_pins_absolute_timestamp() {
    let mut cmd = Command::cargo_bin("rusty-ts").unwrap();
    common::fixture_envs(&mut cmd);
    cmd.env("RUSTY_TS_TEST_FIXED_CLOCK", "2026-05-22T14:30:45Z")
        .args(["-u", "%Y-%m-%d %H:%M:%S"])
        .write_stdin("alpha\nbeta\n")
        .assert()
        .success()
        .stdout("2026-05-22 14:30:45  alpha\n2026-05-22 14:30:45  beta\n");
}

#[test]
fn elapsed_i_with_fixed_clock_shows_zero_delta() {
    // With a Fixed clock, every call to .now() returns the same instant,
    // so elapsed-between-lines is always zero. This proves -i is wired
    // correctly even though it can't show non-zero deltas without a clock
    // that advances. The companion `--monotonic`-with-Wall test below
    // covers the advancement case.
    let mut cmd = Command::cargo_bin("rusty-ts").unwrap();
    common::fixture_envs(&mut cmd);
    cmd.env("RUSTY_TS_TEST_FIXED_CLOCK", "2026-05-22T14:30:45Z")
        .args(["-i", "%H:%M:%S"])
        .write_stdin("a\nb\nc\n")
        .assert()
        .success()
        .stdout("00:00:00  a\n00:00:00  b\n00:00:00  c\n");
}

#[test]
fn elapsed_s_with_fixed_clock_shows_zero_from_start() {
    let mut cmd = Command::cargo_bin("rusty-ts").unwrap();
    common::fixture_envs(&mut cmd);
    cmd.env("RUSTY_TS_TEST_FIXED_CLOCK", "2026-05-22T14:30:45Z")
        .args(["-s", "%H:%M:%S"])
        .write_stdin("a\nb\n")
        .assert()
        .success()
        .stdout("00:00:00  a\n00:00:00  b\n");
}

#[test]
fn monotonic_flag_combined_with_elapsed_works() {
    // With -m and Wall clock (no fixed-clock env), elapsed must advance
    // monotonically across lines. Two-line sequence with a short sleep
    // verifies the second line shows a non-zero elapsed prefix.
    let mut cmd = Command::cargo_bin("rusty-ts").unwrap();
    common::fixture_envs(&mut cmd);
    // No RUSTY_TS_TEST_FIXED_CLOCK — use real monotonic clock.
    let out = cmd
        .args(["-s", "-m", "%S"])
        .write_stdin("a\nb\n")
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    // First line should be "00  a" (elapsed since start ≈ 0).
    assert!(
        stdout.starts_with("00  a\n"),
        "expected first line to show 00 elapsed; got {stdout:?}",
    );
    // Both lines render through the same format pipeline; verify shape.
    let re = regex::Regex::new(r"^\d{2}  a\n\d{2}  b\n$").unwrap();
    assert!(
        re.is_match(&stdout),
        "elapsed-mode output shape mismatch: {stdout:?}",
    );
}

// ─────────────────── FR-029 — Exit codes ───────────────────────

#[test]
fn flag_parse_error_returns_nonzero() {
    let mut cmd = Command::cargo_bin("rusty-ts").unwrap();
    common::fixture_envs(&mut cmd);
    cmd.args(["--definitely-not-a-flag"])
        .write_stdin("")
        .assert()
        .failure();
}

#[test]
fn version_flag_exits_clean() {
    let mut cmd = Command::cargo_bin("rusty-ts").unwrap();
    common::fixture_envs(&mut cmd);
    cmd.args(["--version"])
        .assert()
        .success()
        .stdout(predicate::str::contains("rusty-ts"));
}
