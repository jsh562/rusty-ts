//! # rusty-ts
//!
//! A Rust port of the moreutils `ts` utility: prefix each line of stdin with
//! a timestamp. The CLI binary is the primary user-facing surface; this
//! library exposes the same line-timestamping logic for programmatic reuse.
//!
//! ## Quick example — library API
//!
//! ```no_run
//! use rusty_ts::{TimestamperBuilder, Format, TimezoneSource};
//! use std::io::{BufReader, Cursor};
//!
//! let mut ts = TimestamperBuilder::new()
//!     .format(Format::Strftime("%Y-%m-%d %H:%M:%S".into()))
//!     .timezone(TimezoneSource::Utc)
//!     .build()
//!     .expect("valid configuration");
//!
//! let input = BufReader::new(Cursor::new(b"hello\nworld\n".to_vec()));
//! for chunk in ts.prefix_lines(input) {
//!     let bytes = chunk.expect("io ok");
//!     print!("{}", String::from_utf8_lossy(&bytes));
//! }
//! ```
//!
//! ## Library-without-binary
//!
//! ```toml
//! [dependencies]
//! rusty-ts = { version = "0.1", default-features = false }
//! ```
//!
//! Disabling `default-features` drops the `cli` feature and skips `clap`,
//! `clap_complete`, and `anyhow` from the dependency closure.
//!
//! ## License
//!
//! Licensed under either of Apache-2.0 or MIT at your option.

#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "cli")]
pub mod compat_matrix;
#[cfg(feature = "cli")]
pub mod completions;
pub mod error;
pub mod mode;
pub mod pipeline;
pub mod relative;
pub mod time;

pub use error::Error;
pub use mode::{CompatibilityMode, ExplicitChoice};
pub use time::tz::TimezoneSource;

use crate::pipeline::{PrefixConfig, PrefixSource};
use crate::time::clock::{Clock, Wall};
use crate::time::format;

// ───────────────────────── Public Library API ──────────────────────────────

/// Strftime format selector for `TimestamperBuilder`. `#[non_exhaustive]` so
/// future variants (e.g., a precompiled format) can be added in minor
/// releases without breaking semver.
#[non_exhaustive]
#[derive(Debug, Default, Clone)]
pub enum Format {
    /// Use the moreutils default format (`%b %d %H:%M:%S`).
    #[default]
    Default,
    /// Use the supplied strftime spec, including `%.S` / `%.s` fractional
    /// extensions.
    Strftime(String),
}

/// Elapsed-time anchor selector. `Absolute` is the default; the other
/// variants correspond to the CLI `-i` / `-s` flags.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default)]
pub enum ElapsedAnchor {
    /// Absolute wall-clock time (no elapsed anchor). Default.
    #[default]
    Absolute,
    /// Elapsed since the previous input line (`-i`).
    SincePreviousLine,
    /// Elapsed since program start (`-s`).
    SinceProgramStart,
}

/// Builder for [`Timestamper`]. Every chain method is `#[must_use]` so silent
/// misuse is caught at compile time. `build()` performs post-configuration
/// validation and returns the same typed errors the CLI's post-parse
/// validation produces (e.g., `Error::InvalidUtcWithNamedTz` for FR-020
/// mirrored at the library layer).
#[derive(Debug, Clone, Default)]
pub struct TimestamperBuilder {
    format: Format,
    /// Mirrors the CLI `-u` / `--utc` flag.
    utc_requested: bool,
    /// Mirrors the CLI `--tz=<IANA>` flag.
    named_tz: Option<String>,
    /// Direct enum override; lower-level escape hatch. When set, supersedes
    /// `utc_requested` and `named_tz` (advanced consumers only).
    timezone_override: Option<TimezoneSource>,
    compat: CompatibilityMode,
    elapsed: ElapsedAnchor,
}

impl TimestamperBuilder {
    /// Start a new builder with all-default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the strftime format selector.
    #[must_use]
    pub fn format(mut self, format: Format) -> Self {
        self.format = format;
        self
    }

    /// Request UTC rendering (mirrors the CLI `-u` / `--utc` flag).
    /// Conflicts with `tz_name`; combined use causes `build()` to return
    /// [`Error::InvalidUtcWithNamedTz`] per FR-020.
    #[must_use]
    pub fn utc(mut self, utc: bool) -> Self {
        self.utc_requested = utc;
        self
    }

    /// Request rendering in a named IANA timezone (mirrors the CLI
    /// `--tz=<IANA>` flag). Conflicts with `utc(true)`. The name is
    /// resolved at `build()` time; an unrecognised name produces
    /// [`Error::InvalidIanaName`].
    #[must_use]
    pub fn tz_name(mut self, name: impl Into<String>) -> Self {
        self.named_tz = Some(name.into());
        self
    }

    /// Low-level escape hatch: set the timezone source directly. When set,
    /// overrides `utc()` and `tz_name()` configuration. Most callers should
    /// prefer the structured `utc()` / `tz_name()` methods, which mirror the
    /// CLI flag surface and provide the FR-020 mutex enforcement.
    #[must_use]
    pub fn timezone(mut self, tz: TimezoneSource) -> Self {
        self.timezone_override = Some(tz);
        self
    }

    /// Set the compatibility mode. Default is `CompatibilityMode::Default`.
    #[must_use]
    pub fn compat(mut self, mode: CompatibilityMode) -> Self {
        self.compat = mode;
        self
    }

    /// Set the elapsed-time anchor. Default is `Absolute`.
    #[must_use]
    pub fn elapsed(mut self, anchor: ElapsedAnchor) -> Self {
        self.elapsed = anchor;
        self
    }

    /// Finalize the builder. Returns a configured [`Timestamper`] or an
    /// [`Error`] if the configuration is invalid.
    ///
    /// Validation:
    /// - `utc(true)` + `tz_name(...)` together → [`Error::InvalidUtcWithNamedTz`]
    ///   (library-layer mirror of FR-020).
    /// - `tz_name("...")` with unrecognised IANA name → [`Error::InvalidIanaName`].
    /// - `timezone(...)` low-level override bypasses the above and uses
    ///   whatever variant was supplied directly.
    pub fn build(self) -> Result<Timestamper, Error> {
        // FR-020 library-layer mirror: utc + named tz is invalid.
        if self.utc_requested {
            if let Some(name) = &self.named_tz {
                return Err(Error::InvalidUtcWithNamedTz { tz: name.clone() });
            }
        }

        // Resolve the timezone source from the configured fields. The
        // low-level `timezone_override` wins if supplied.
        let timezone = if let Some(direct) = self.timezone_override {
            direct
        } else if self.utc_requested {
            TimezoneSource::Utc
        } else if let Some(name) = self.named_tz {
            TimezoneSource::named(&name)?
        } else {
            TimezoneSource::Local
        };

        Ok(Timestamper {
            format: self.format,
            timezone,
            compat: self.compat,
            elapsed: self.elapsed,
        })
    }
}

/// Configured line-timestamping engine. Cheap to construct (no IO until
/// `prefix_lines` is called). Marked `#[non_exhaustive]` for future evolution.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Timestamper {
    format: Format,
    timezone: TimezoneSource,
    compat: CompatibilityMode,
    elapsed: ElapsedAnchor,
}

impl Timestamper {
    /// Drive a [`BufRead`](std::io::BufRead) line source through the
    /// timestamper. Returns an iterator over byte-typed output chunks
    /// (`Vec<u8>`); non-UTF-8 payload bytes pass through unchanged per
    /// FR-011.
    ///
    /// The returned iterator yields one `Result<Vec<u8>, io::Error>` per
    /// input line. Each chunk is the timestamp prefix followed by a
    /// two-space separator followed by the payload (including any
    /// trailing newline that was present in the input).
    pub fn prefix_lines<R: std::io::BufRead>(
        &self,
        reader: R,
    ) -> impl Iterator<Item = Result<Vec<u8>, std::io::Error>> {
        TimestampingIterator {
            reader,
            clock: Wall,
            timestamper: self.clone(),
            program_start: None,
            previous_line_at: None,
        }
    }

    /// Convenience adapter for callers who already have UTF-8 `String`
    /// lines. Returns prefixed `String` chunks. Non-UTF-8 input is impossible
    /// at this surface — use `prefix_lines` if you need byte fidelity.
    pub fn prefix_string_lines<I>(&self, lines: I) -> impl Iterator<Item = String>
    where
        I: IntoIterator<Item = String>,
    {
        let clock = Wall;
        let program_start = clock.now();
        let format_spec = self.format_spec().to_owned();
        let tz = self.timezone.clone();
        let elapsed = self.elapsed;
        let mut previous_line_at = program_start;

        lines.into_iter().map(move |line| {
            let now = clock.now();
            let prefix = match elapsed {
                ElapsedAnchor::Absolute => format::format_with(&format_spec, now, &tz),
                ElapsedAnchor::SincePreviousLine => {
                    let delta = (now - previous_line_at).to_std().unwrap_or_default();
                    previous_line_at = now;
                    elapsed_string(&format_spec, delta)
                }
                ElapsedAnchor::SinceProgramStart => {
                    let delta = (now - program_start).to_std().unwrap_or_default();
                    elapsed_string(&format_spec, delta)
                }
            };
            format!("{prefix}  {line}")
        })
    }

    /// Return the effective strftime format spec — either the builder's
    /// supplied spec or the moreutils default. Used by the iterator
    /// implementations.
    pub fn format_spec(&self) -> &str {
        match &self.format {
            Format::Default => format::DEFAULT_FORMAT,
            Format::Strftime(s) => s.as_str(),
        }
    }

    /// Return the configured compatibility mode.
    pub fn compat(&self) -> CompatibilityMode {
        self.compat
    }

    /// Return a reference to the configured timezone source.
    pub fn timezone(&self) -> &TimezoneSource {
        &self.timezone
    }

    /// Return the configured elapsed-time anchor.
    pub fn elapsed_anchor(&self) -> ElapsedAnchor {
        self.elapsed
    }
}

struct TimestampingIterator<R: std::io::BufRead> {
    reader: R,
    clock: Wall,
    timestamper: Timestamper,
    program_start: Option<chrono::DateTime<chrono::Utc>>,
    previous_line_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl<R: std::io::BufRead> Iterator for TimestampingIterator<R> {
    type Item = Result<Vec<u8>, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = Vec::with_capacity(256);
        match self.reader.read_until(b'\n', &mut line) {
            Ok(0) => None, // EOF
            Ok(_) => {
                let now = self.clock.now();
                let prog_start = *self.program_start.get_or_insert(now);
                let prev = *self.previous_line_at.get_or_insert(prog_start);

                let prefix = match self.timestamper.elapsed {
                    ElapsedAnchor::Absolute => format::format_with(
                        self.timestamper.format_spec(),
                        now,
                        &self.timestamper.timezone,
                    ),
                    ElapsedAnchor::SincePreviousLine => {
                        let delta = (now - prev).to_std().unwrap_or_default();
                        self.previous_line_at = Some(now);
                        elapsed_string(self.timestamper.format_spec(), delta)
                    }
                    ElapsedAnchor::SinceProgramStart => {
                        let delta = (now - prog_start).to_std().unwrap_or_default();
                        elapsed_string(self.timestamper.format_spec(), delta)
                    }
                };

                let mut out = Vec::with_capacity(prefix.len() + 2 + line.len());
                out.extend_from_slice(prefix.as_bytes());
                out.extend_from_slice(b"  ");
                out.extend_from_slice(&line);
                Some(Ok(out))
            }
            Err(err) => Some(Err(err)),
        }
    }
}

fn elapsed_string(spec: &str, elapsed: std::time::Duration) -> String {
    let secs = elapsed.as_secs() as i64;
    let nsecs = elapsed.subsec_nanos();
    let synthetic = chrono::DateTime::<chrono::Utc>::from_timestamp(secs, nsecs)
        .unwrap_or_else(|| chrono::DateTime::<chrono::Utc>::from_timestamp(0, 0).unwrap());
    format::format_with(spec, synthetic, &TimezoneSource::Utc)
}

// ───────────────────────── CLI Entry Point ─────────────────────────────────

/// Binary entry point shared by `src/main.rs` and `src/bin/ts.rs`.
///
/// Resolves [`CompatibilityMode`] once at startup from CLI flag → env var →
/// argv[0] basename per FR-021. Dispatches to the appropriate pipeline
/// based on the resolved flags (relative mode, elapsed modes, absolute
/// default). In Strict mode, Rusty-only flags are rejected with a
/// moreutils-style diagnostic.
#[cfg(feature = "cli")]
pub fn run() -> std::process::ExitCode {
    use clap::Parser;
    use std::io::{Write, stderr, stdin, stdout};

    let cli = match cli::Cli::try_parse() {
        Ok(c) => c,
        Err(err) => err.exit(), // clap handles its own help/version exit codes; never returns
    };

    // Resolve compatibility mode once at startup (FR-021..023).
    let argv0 = std::env::args_os().next();
    let argv0_basename = argv0
        .as_ref()
        .and_then(|s| mode::argv0_basename(s.as_os_str()));
    let env_strict = std::env::var("RUSTY_TS_STRICT").ok();
    let compat = mode::resolve(
        cli.explicit_compat_choice(),
        env_strict.as_deref(),
        argv0_basename.as_deref(),
    );

    // In Strict mode, reject Rusty-only flags (FR-026).
    if compat == CompatibilityMode::Strict
        && (cli.utc || cli.tz.is_some() || cli.subcommand.is_some())
    {
        let _ = writeln!(
            stderr(),
            "rusty-ts: unknown flag in --strict mode (rejecting Rusty-only extensions; \
             see README Compatibility Matrix)"
        );
        return std::process::ExitCode::from(2);
    }

    // Defense-in-depth: validate -u + --tz mutex (clap also enforces this).
    if let Err(err) = cli.validate() {
        let _ = writeln!(stderr(), "rusty-ts: {err}");
        return std::process::ExitCode::from(2);
    }

    // Dispatch subcommands first.
    if let Some(cli::CliCommand::Completions { shell }) = cli.subcommand {
        let mut out = stdout().lock();
        if let Err(err) = completions::emit_completions(shell, &mut out) {
            if err.kind() == std::io::ErrorKind::BrokenPipe {
                return std::process::ExitCode::SUCCESS;
            }
            let _ = writeln!(stderr(), "rusty-ts: {err}");
            return std::process::ExitCode::from(1);
        }
        return std::process::ExitCode::SUCCESS;
    }

    // Resolve timezone source.
    let tz = if cli.utc {
        TimezoneSource::Utc
    } else if let Some(name) = &cli.tz {
        match TimezoneSource::named(name) {
            Ok(t) => t,
            Err(err) => {
                let _ = writeln!(stderr(), "rusty-ts: {err}");
                return std::process::ExitCode::from(2);
            }
        }
    } else {
        TimezoneSource::Local
    };

    // Resolve format spec — positional argument wins over RUSTY_TS_FORMAT.
    // RUSTY_TS_FORMAT is ignored in Strict mode (FR-027).
    let format_spec: String = if let Some(spec) = &cli.format {
        spec.clone()
    } else if compat == CompatibilityMode::Default {
        std::env::var("RUSTY_TS_FORMAT")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format::DEFAULT_FORMAT.to_string())
    } else {
        format::DEFAULT_FORMAT.to_string()
    };

    let stdin = stdin();
    let stdout = stdout();
    let stdin_locked = stdin.lock();
    let mut stdout_locked = stdout.lock();

    let result: std::io::Result<()> = if cli.relative {
        let rewriter = relative::RelativeRewriter::for_mode(compat);
        let clock = Wall;
        let cfg = pipeline::RelativeConfig {
            rewriter: &rewriter,
            reference: clock.now(),
        };
        pipeline::run_relative(stdin_locked, &mut stdout_locked, &cfg)
    } else {
        let clock = Wall;
        let source = if cli.incremental {
            PrefixSource::SincePreviousLine
        } else if cli.since_start {
            PrefixSource::SinceProgramStart
        } else {
            PrefixSource::Absolute
        };
        let cfg = PrefixConfig {
            format: &format_spec,
            tz: &tz,
            clock: &clock,
            source,
        };
        pipeline::run_prefix(stdin_locked, &mut stdout_locked, &cfg)
    };

    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(err) => {
            let _ = writeln!(stderr(), "rusty-ts: {err}");
            std::process::ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_default_yields_absolute_default_format() {
        let ts = TimestamperBuilder::new().build().expect("builds");
        assert_eq!(ts.format_spec(), format::DEFAULT_FORMAT);
        assert!(matches!(ts.elapsed_anchor(), ElapsedAnchor::Absolute));
    }

    #[test]
    fn builder_custom_format_round_trips() {
        let ts = TimestamperBuilder::new()
            .format(Format::Strftime("%H:%M:%S".into()))
            .build()
            .expect("builds");
        assert_eq!(ts.format_spec(), "%H:%M:%S");
    }

    #[test]
    fn prefix_lines_byte_typed_iterator() {
        let ts = TimestamperBuilder::new()
            .format(Format::Strftime("[%H:%M:%S]".into()))
            .timezone(TimezoneSource::Utc)
            .build()
            .expect("builds");
        let input = std::io::Cursor::new(b"hello\nworld\n".to_vec());
        let chunks: Vec<Vec<u8>> = ts
            .prefix_lines(input)
            .collect::<Result<Vec<_>, _>>()
            .expect("io ok");
        assert_eq!(chunks.len(), 2);
        // Each chunk ends with "  hello\n" or "  world\n".
        assert!(chunks[0].ends_with(b"  hello\n"), "got {:?}", chunks[0]);
        assert!(chunks[1].ends_with(b"  world\n"), "got {:?}", chunks[1]);
    }

    #[test]
    fn prefix_string_lines_utf8_convenience() {
        let ts = TimestamperBuilder::new()
            .format(Format::Strftime("[%H:%M:%S]".into()))
            .timezone(TimezoneSource::Utc)
            .build()
            .expect("builds");
        let lines = vec!["hello\n".to_string(), "world\n".to_string()];
        let out: Vec<String> = ts.prefix_string_lines(lines).collect();
        assert_eq!(out.len(), 2);
        assert!(out[0].ends_with("  hello\n"));
        assert!(out[1].ends_with("  world\n"));
    }

    /// Send + !Sync compile-time assertion per `plan.md` AD-008.
    #[test]
    fn timestamper_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Timestamper>();
    }

    #[test]
    fn timestamper_builder_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TimestamperBuilder>();
    }

    #[test]
    fn non_utf8_payload_preserved_through_byte_iterator() {
        let ts = TimestamperBuilder::new()
            .format(Format::Strftime("[%H:%M:%S]".into()))
            .timezone(TimezoneSource::Utc)
            .build()
            .expect("builds");
        // Payload with 0xFF byte (invalid UTF-8).
        let input: &[u8] = b"hello\xff\nworld\n";
        let chunks: Vec<Vec<u8>> = ts
            .prefix_lines(std::io::Cursor::new(input.to_vec()))
            .collect::<Result<Vec<_>, _>>()
            .expect("io ok");
        assert!(
            chunks[0].contains(&0xFF),
            "expected 0xFF byte preserved in first chunk; got {:?}",
            chunks[0],
        );
    }
}
