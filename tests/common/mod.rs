//! Shared test harness — env-pin verification and fixture loaders.
//!
//! Per `plan.md` HINT-003 and STF-001: every snapshot/integration test
//! invokes `assert_pinned_env()` at start. The three determinism inputs
//! (`TZ=UTC`, `LC_ALL=C.UTF-8`, `RUSTY_TS_FORMAT` empty) must be jointly
//! pinned before any snapshot comparison; mismatch fails the test before
//! the snapshot compare even happens.

#![allow(dead_code)]

use std::env;

/// Assert the snapshot-determinism env vars are pinned per the capture
/// protocol. Call at the start of every test that depends on byte-level
/// timestamp output.
///
/// On Windows runners that lack `C.UTF-8`, the harness accepts `LANG=C`
/// or `LC_ALL=C` (ASCII-only) and the affected snapshot fixtures are
/// expected to use ASCII-only format tokens.
pub fn assert_pinned_env() {
    let tz = env::var("TZ").unwrap_or_default();
    assert_eq!(
        tz, "UTC",
        "TZ env var must be set to 'UTC' for deterministic snapshot tests; got {tz:?}",
    );

    let lc_all = env::var("LC_ALL").unwrap_or_default();
    let lang = env::var("LANG").unwrap_or_default();
    let locale_ok = matches!(
        lc_all.as_str(),
        "C.UTF-8" | "C" | "POSIX",
    ) || matches!(lang.as_str(), "C.UTF-8" | "C" | "POSIX");
    assert!(
        locale_ok,
        "LC_ALL or LANG must be C.UTF-8 (preferred), C, or POSIX for deterministic snapshot tests; \
         LC_ALL={lc_all:?} LANG={lang:?}",
    );

    let rusty_format = env::var("RUSTY_TS_FORMAT").unwrap_or_default();
    assert!(
        rusty_format.is_empty(),
        "RUSTY_TS_FORMAT must be unset/empty for deterministic snapshot tests; got {rusty_format:?}",
    );
}

/// Set the snapshot-determinism env vars within the current process. Used by
/// tests that want to opt into pinned-env behavior without requiring the
/// test runner itself to have them set.
///
/// Tests that need different env values must use a separate harness (e.g.,
/// `assert_cmd` with `.env(...)`).
pub fn pin_env_in_process() {
    // SAFETY: set_var is technically unsafe in recent Rust versions because
    // env mutation is racy if other threads are reading env vars
    // concurrently. In single-threaded test setup before `assert_pinned_env`
    // calls this is fine; multi-threaded tests should use assert_cmd's env
    // override instead.
    // Safety: see comment above.
    unsafe {
        env::set_var("TZ", "UTC");
        env::set_var("LC_ALL", "C.UTF-8");
        env::remove_var("RUSTY_TS_FORMAT");
    }
}
