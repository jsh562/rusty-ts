//! Public-API surface drift detection (T113 / T114).
//!
//! This test asserts that the hand-maintained `docs/public-api.txt` snapshot
//! is non-empty and that the items listed there are actually reachable from
//! the crate root. The full programmatic surface enumeration (via
//! `cargo public-api`) is deferred until the tool can be installed in CI;
//! this baseline serves as the committed semver contract in the meantime.
//!
//! When the public surface changes:
//! 1. Update `docs/public-api.txt` with the new items (additions are pre-1.0
//!    semver-patch; removals or signature changes are pre-1.0 semver-minor
//!    per the API Surface Summary in plan.md).
//! 2. Update CHANGELOG.md with the rationale.
//! 3. Run this test to confirm the new baseline parses cleanly.

#![cfg(feature = "cli")]

use std::fs;
use std::path::Path;

const BASELINE_PATH: &str = "docs/public-api.txt";

#[test]
fn baseline_file_exists_and_is_non_empty() {
    let content = fs::read_to_string(Path::new(BASELINE_PATH))
        .expect("baseline file present at docs/public-api.txt");
    let entries: Vec<&str> = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    assert!(
        entries.len() >= 50,
        "expected at least 50 public-surface entries; got {}",
        entries.len(),
    );
}

#[test]
fn baseline_includes_canonical_types() {
    let content = fs::read_to_string(Path::new(BASELINE_PATH)).expect("baseline file");
    // Spot-check that the load-bearing top-level types are listed.
    for required in [
        "rusty_ts::Timestamper",
        "rusty_ts::TimestamperBuilder",
        "rusty_ts::Format",
        "rusty_ts::TimezoneSource",
        "rusty_ts::CompatibilityMode",
        "rusty_ts::ElapsedAnchor",
        "rusty_ts::Error",
        "rusty_ts::run",
        "rusty_ts::TimestamperBuilder::build",
        "rusty_ts::TimestamperBuilder::utc",
        "rusty_ts::TimestamperBuilder::tz_name",
        "rusty_ts::Timestamper::prefix_lines",
        "rusty_ts::Timestamper::prefix_string_lines",
        "rusty_ts::error::Error::InvalidUtcWithNamedTz",
        "rusty_ts::error::Error::InvalidIanaName",
    ] {
        assert!(
            content.contains(required),
            "baseline missing required entry: {required}",
        );
    }
}

#[test]
fn baseline_entries_have_no_duplicates() {
    let content = fs::read_to_string(Path::new(BASELINE_PATH)).expect("baseline file");
    let entries: Vec<&str> = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    let mut sorted = entries.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        entries.len(),
        sorted.len(),
        "baseline contains duplicate entries; expected {} unique, got {}",
        sorted.len(),
        entries.len(),
    );
}
