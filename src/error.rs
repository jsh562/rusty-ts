//! Library error type for `rusty-ts`.
//!
//! Per AD-009: library errors are typed via `thiserror`; the binary boundary
//! (`src/main.rs`) wraps these in `anyhow` for human-readable diagnostics.
//!
//! All public variants carry actionable context (offending input, source
//! error) rather than opaque strings. The enum is `#[non_exhaustive]` so new
//! variants can be added in minor versions without breaking semver per the
//! pre-1.0 evolution rules documented in `plan.md` §API Surface Summary.

use std::io;

/// Errors raised by the `rusty-ts` library API.
///
/// Marked `#[non_exhaustive]` to allow new variants in minor releases.
///
/// # Example
///
/// ```
/// use rusty_ts::{Error, TimestamperBuilder};
///
/// // Pattern-match on specific variants for actionable handling.
/// let result = TimestamperBuilder::new()
///     .utc(true)
///     .tz_name("Asia/Tokyo")
///     .build();
///
/// match result {
///     Err(Error::InvalidUtcWithNamedTz { tz }) => {
///         eprintln!("cannot combine -u with --tz={tz}");
///     }
///     Err(Error::InvalidIanaName(name)) => {
///         eprintln!("unknown IANA timezone: {name}");
///     }
///     Err(other) => eprintln!("error: {other}"),
///     Ok(_) => unreachable!("we configured a conflict"),
/// }
/// ```
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// `-u` (UTC) and `--tz=<name>` were both specified, which is invalid.
    /// Mirrors the CLI-layer `FR-020` mutual-exclusion check at the library
    /// layer so library consumers do not depend on the CLI to catch it.
    #[error("--utc and --tz=<name> are mutually exclusive; got --tz={tz}")]
    InvalidUtcWithNamedTz {
        /// The IANA name the caller supplied.
        tz: String,
    },

    /// The named IANA timezone could not be resolved (e.g., typo, removed
    /// zone). Carries the offending input.
    #[error("unknown IANA timezone: {0}")]
    InvalidIanaName(String),

    /// The strftime format string is malformed or unsupported. Carries the
    /// offending format string.
    #[error("invalid strftime format: {0}")]
    InvalidFormat(String),

    /// Underlying IO error, surfaced from `BufRead`/`Write` operations.
    #[error("io error: {0}")]
    Io(#[from] io::Error),
}
