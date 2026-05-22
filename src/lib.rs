//! # rusty-ts
//!
//! A Rust port of the moreutils `ts` utility: prefix each line of stdin with
//! a timestamp. The CLI binary is the primary user-facing surface; this
//! library exposes the same line-timestamping logic for programmatic reuse.
//!
//! ## Quick example
//!
//! ```no_run
//! use rusty_ts::time::{format::DEFAULT_FORMAT, tz::TimezoneSource};
//! use rusty_ts::time::clock::{Clock, Wall};
//! use rusty_ts::time::format::format_with;
//!
//! let clock = Wall;
//! let tz = TimezoneSource::Utc;
//! let line = format_with(DEFAULT_FORMAT, clock.now(), &tz);
//! println!("{line}  hello");
//! ```
//!
//! At v0.1.0 the public library surface exposes the modules below directly
//! (`time::format`, `time::tz`, `time::clock`, `mode`, `error`). The
//! richer `Timestamper` / `TimestamperBuilder` API surface promised by
//! FR-012 lands as part of US7 in a subsequent implementation pass and will
//! supersede direct module access as the canonical API.
//!
//! ## License
//!
//! Licensed under either of Apache License, Version 2.0 or MIT License at
//! your option.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod error;
pub mod mode;
pub mod time;

pub use error::Error;
pub use mode::{CompatibilityMode, ExplicitChoice};
pub use time::tz::TimezoneSource;

/// Stand-in entry point for the CLI binary. Promoted to a real argv-parsing
/// dispatcher in Phase 3 (US1) once `src/cli.rs` lands.
///
/// At Phase 2 scaffold completion this function reads stdin line-by-line and
/// prefixes each line with the moreutils default format using `chrono::Local`
/// (matching FR-017's default behavior). Subsequent phases swap this for the
/// full `Timestamper` builder pipeline that handles flags, env vars, Strict
/// mode, and the library API surface.
///
/// Gated behind the `cli` feature so library consumers depending on the crate
/// with `default-features = false` do not pull in the binary code path.
#[cfg(feature = "cli")]
pub fn run() -> std::process::ExitCode {
    use std::io::{BufRead, Write};

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut stdin_locked = stdin.lock();
    let mut stdout_locked = stdout.lock();
    let clock = time::clock::Wall;
    let tz = TimezoneSource::Local;

    let mut line = Vec::with_capacity(256);
    loop {
        line.clear();
        match stdin_locked.read_until(b'\n', &mut line) {
            Ok(0) => return std::process::ExitCode::SUCCESS, // clean EOF
            Ok(_) => {
                let prefix = time::format::format_default(
                    <time::clock::Wall as time::clock::Clock>::now(&clock),
                    &tz,
                );
                // Write prefix + two spaces (FR-001 moreutils-conventional
                // separator) + raw payload bytes (FR-011 byte passthrough).
                if let Err(err) = stdout_locked
                    .write_all(prefix.as_bytes())
                    .and_then(|_| stdout_locked.write_all(b"  "))
                    .and_then(|_| stdout_locked.write_all(&line))
                    .and_then(|_| stdout_locked.flush())
                {
                    // Broken-pipe on stdout is a clean exit per HINT-004.
                    if err.kind() == std::io::ErrorKind::BrokenPipe {
                        return std::process::ExitCode::SUCCESS;
                    }
                    eprintln!("rusty-ts: io error: {err}");
                    return std::process::ExitCode::from(1);
                }
            }
            Err(err) => {
                if err.kind() == std::io::ErrorKind::BrokenPipe {
                    return std::process::ExitCode::SUCCESS;
                }
                eprintln!("rusty-ts: stdin error: {err}");
                return std::process::ExitCode::from(1);
            }
        }
    }
}
