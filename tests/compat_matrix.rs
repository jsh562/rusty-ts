//! Compatibility Matrix drift test per FR-030 / AD-006 / SC-016.
//!
//! Regenerates the Compatibility Matrix from the canonical CLI definition
//! and asserts equality with the committed `docs/COMPATIBILITY.md` file.
//! Fails CI if the committed file drifts from the CLI surface.
//!
//! To accept a CLI change, run `UPDATE_COMPATIBILITY_MATRIX=1 cargo test
//! --test compat_matrix` which regenerates the file in-place; review the
//! diff in PR.

#![cfg(feature = "cli")]

use std::fs;
use std::path::Path;

const COMMITTED_PATH: &str = "docs/COMPATIBILITY.md";

#[test]
fn committed_compatibility_matrix_matches_cli_definition() {
    let generated = rusty_ts::compat_matrix::generate_matrix();

    if std::env::var("UPDATE_COMPATIBILITY_MATRIX").is_ok() {
        fs::write(COMMITTED_PATH, &generated).expect("write COMPATIBILITY.md");
        eprintln!(
            "UPDATE_COMPATIBILITY_MATRIX=1 set — overwrote {COMMITTED_PATH}. \
             Review the diff in PR."
        );
        return;
    }

    let committed = fs::read_to_string(Path::new(COMMITTED_PATH)).unwrap_or_else(|err| {
        panic!(
            "could not read {COMMITTED_PATH}: {err}. \
             Run `UPDATE_COMPATIBILITY_MATRIX=1 cargo test --test compat_matrix` \
             to regenerate.",
        );
    });

    // Normalize line endings: tests on Windows checkouts may convert LF to
    // CRLF on disk; the generator emits LF. Strip CR from both sides before
    // comparing so the drift check is platform-neutral.
    let normalize = |s: &str| s.replace("\r\n", "\n");
    assert_eq!(
        normalize(&committed),
        normalize(&generated),
        "Compatibility Matrix drifted from CLI definition. Run \
         `UPDATE_COMPATIBILITY_MATRIX=1 cargo test --test compat_matrix` \
         to regenerate, then commit the diff.",
    );
}

/// FR-015 / SC-001 / T112: the README MUST contain the TZ-pinning disclosure
/// verbatim so consumers can trust that "byte-level fidelity is verified
/// under TZ=UTC" is a testable contract, not implementation lore.
#[test]
fn readme_contains_tz_pinning_disclosure_verbatim() {
    let readme = fs::read_to_string("README.md").expect("read README");
    let disclosure = "Byte-level fidelity is verified by snapshot tests against captured \
                      moreutils-`ts` output under a pinned environment: `TZ=UTC` and \
                      `LC_ALL=C.UTF-8`.";
    let normalized_readme = readme.replace("\r\n", "\n");
    let normalized_disclosure = disclosure.replace("\r\n", "\n");
    assert!(
        normalized_readme.contains(&normalized_disclosure),
        "README MUST contain the FR-015 TZ-pinning disclosure verbatim. \
         Expected substring:\n{normalized_disclosure}\n\
         README content:\n{normalized_readme}",
    );
}
