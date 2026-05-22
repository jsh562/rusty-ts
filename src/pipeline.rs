//! The stdin → format → stdout line pipeline.
//!
//! Per `spec.md` FR-001, FR-002, FR-005, FR-006, FR-007, FR-010, FR-011 and
//! `plan.md` HINT-004 (broken-pipe-on-stdout = clean exit 0).
//!
//! Two pipeline shapes are exported:
//!
//! - `run_prefix` — the canonical prefix-each-line pipeline. Handles
//!   absolute timestamps (default) and elapsed modes (`-i`, `-s`).
//! - `run_relative` — the `-r` rewriter pipeline that converts in-line
//!   timestamps to relative form via `relative::RelativeRewriter`.
//!
//! Both pipelines read `BufRead` line-by-line, write byte-faithful output
//! to `Write`, and treat broken-pipe-on-stdout as a clean EOF (exit 0).

use crate::relative::RelativeRewriter;
use crate::time::clock::Clock;
use crate::time::format;
use crate::time::tz::TimezoneSource;
use chrono::{DateTime, Utc};
use std::io::{BufRead, ErrorKind, Write};

/// Selects which time value is rendered into each line's prefix.
#[derive(Debug, Clone)]
pub enum PrefixSource {
    /// Absolute wall-clock time (FR-001, FR-003 default).
    Absolute,
    /// Elapsed since the previous input line (FR-005, `-i`). The "previous"
    /// anchor starts at program start for the first line, matching moreutils.
    SincePreviousLine,
    /// Elapsed since program start (FR-006, `-s`).
    SinceProgramStart,
}

/// Configuration for `run_prefix`.
pub struct PrefixConfig<'a> {
    /// strftime format spec.
    pub format: &'a str,
    /// Timezone source resolved per FR-017/018/019.
    pub tz: &'a TimezoneSource,
    /// Time source for absolute mode and the elapsed anchors.
    pub clock: &'a dyn Clock,
    /// Which prefix value to render.
    pub source: PrefixSource,
}

/// Configuration for `run_relative`.
pub struct RelativeConfig<'a> {
    /// Compiled rewriter (Default-subset or Strict full-set).
    pub rewriter: &'a RelativeRewriter,
    /// Reference instant for relative computations (typically `clock.now()`
    /// at startup; for snapshot tests, a `Fixed` clock pins this).
    pub reference: DateTime<Utc>,
}

/// Run the prefix-each-line pipeline. Returns `ExitCode` semantics: 0 on
/// clean stdin EOF, non-zero on IO error (excluding broken-pipe-on-stdout
/// which is treated as clean exit per HINT-004).
pub fn run_prefix<R: BufRead, W: Write>(
    mut reader: R,
    mut writer: W,
    cfg: &PrefixConfig<'_>,
) -> std::io::Result<()> {
    let program_start = cfg.clock.now();
    let mut previous_line_at = program_start;
    let mut line = Vec::with_capacity(256);

    loop {
        line.clear();
        match reader.read_until(b'\n', &mut line) {
            Ok(0) => return Ok(()), // clean EOF
            Ok(_) => {
                let now = cfg.clock.now();
                let prefix = match cfg.source {
                    PrefixSource::Absolute => format::format_with(cfg.format, now, cfg.tz),
                    PrefixSource::SincePreviousLine => {
                        let elapsed = (now - previous_line_at).to_std().unwrap_or_default();
                        previous_line_at = now;
                        render_elapsed(cfg.format, elapsed)
                    }
                    PrefixSource::SinceProgramStart => {
                        let elapsed = (now - program_start).to_std().unwrap_or_default();
                        render_elapsed(cfg.format, elapsed)
                    }
                };

                if let Err(err) = writer
                    .write_all(prefix.as_bytes())
                    .and_then(|_| writer.write_all(b" "))
                    .and_then(|_| writer.write_all(&line))
                    .and_then(|_| writer.flush())
                {
                    if err.kind() == ErrorKind::BrokenPipe {
                        return Ok(());
                    }
                    return Err(err);
                }
            }
            Err(err) => {
                if err.kind() == ErrorKind::BrokenPipe {
                    return Ok(());
                }
                return Err(err);
            }
        }
    }
}

/// Run the `-r` relative-mode pipeline. Each line is passed through the
/// rewriter; recognized timestamps become relative form, everything else
/// passes through unchanged.
pub fn run_relative<R: BufRead, W: Write>(
    mut reader: R,
    mut writer: W,
    cfg: &RelativeConfig<'_>,
) -> std::io::Result<()> {
    let mut line = Vec::with_capacity(256);
    loop {
        line.clear();
        match reader.read_until(b'\n', &mut line) {
            Ok(0) => return Ok(()),
            Ok(_) => {
                // Lossy UTF-8 conversion is acceptable here: timestamps in
                // `-r` mode are ASCII-only, and the surrounding payload is
                // passed through. Non-UTF-8 bytes in payload become U+FFFD
                // in the rewrite path; this is the documented compatibility
                // boundary for `-r` specifically (per FR-009 / FR-025).
                let text = String::from_utf8_lossy(&line);
                let rewritten = cfg.rewriter.rewrite(&text, cfg.reference);

                if let Err(err) = writer
                    .write_all(rewritten.as_bytes())
                    .and_then(|_| writer.flush())
                {
                    if err.kind() == ErrorKind::BrokenPipe {
                        return Ok(());
                    }
                    return Err(err);
                }
            }
            Err(err) => {
                if err.kind() == ErrorKind::BrokenPipe {
                    return Ok(());
                }
                return Err(err);
            }
        }
    }
}

/// Render an elapsed `Duration` through the format string. moreutils ts
/// treats the format string as time-of-day when rendering elapsed durations;
/// e.g., `ts -i '%H:%M:%S'` shows `00:00:03` for a 3-second elapsed window.
/// We do the same by constructing a `DateTime<Utc>` from epoch + elapsed and
/// rendering it.
fn render_elapsed(spec: &str, elapsed: std::time::Duration) -> String {
    let secs = elapsed.as_secs() as i64;
    let nsecs = elapsed.subsec_nanos();
    let synthetic = chrono::DateTime::<Utc>::from_timestamp(secs, nsecs).unwrap_or_else(|| {
        // 64-bit timestamp out of range — vanishingly unlikely for elapsed
        // duration. Fallback to zero.
        chrono::DateTime::<Utc>::from_timestamp(0, 0).expect("epoch is in range")
    });
    format::format_with(spec, synthetic, &TimezoneSource::Utc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::clock::Fixed;
    use chrono::TimeZone;
    use std::io::Cursor;

    fn fixed_clock() -> Fixed {
        Fixed::new(Utc.with_ymd_and_hms(2026, 5, 22, 14, 30, 45).unwrap())
    }

    #[test]
    fn absolute_default_format() {
        let clock = fixed_clock();
        let tz = TimezoneSource::Utc;
        let cfg = PrefixConfig {
            format: format::DEFAULT_FORMAT,
            tz: &tz,
            clock: &clock,
            source: PrefixSource::Absolute,
        };
        let mut out = Vec::new();
        run_prefix(Cursor::new("hello\nworld\n"), &mut out, &cfg).expect("ok");
        let s = String::from_utf8(out).expect("utf-8");
        assert_eq!(s, "May 22 14:30:45 hello\nMay 22 14:30:45 world\n");
    }

    #[test]
    fn since_program_start_renders_zero_on_first_line() {
        let clock = fixed_clock();
        let tz = TimezoneSource::Utc;
        let cfg = PrefixConfig {
            format: "%H:%M:%S",
            tz: &tz,
            clock: &clock,
            source: PrefixSource::SinceProgramStart,
        };
        let mut out = Vec::new();
        run_prefix(Cursor::new("a\n"), &mut out, &cfg).expect("ok");
        let s = String::from_utf8(out).expect("utf-8");
        assert!(
            s.starts_with("00:00:00 "),
            "expected elapsed-zero prefix; got {s:?}",
        );
    }

    #[test]
    fn empty_stdin_produces_no_output() {
        let clock = fixed_clock();
        let tz = TimezoneSource::Utc;
        let cfg = PrefixConfig {
            format: format::DEFAULT_FORMAT,
            tz: &tz,
            clock: &clock,
            source: PrefixSource::Absolute,
        };
        let mut out = Vec::new();
        run_prefix(Cursor::new(""), &mut out, &cfg).expect("ok");
        assert!(out.is_empty(), "expected no output; got {:?}", out);
    }

    #[test]
    fn partial_final_line_is_emitted_without_added_newline() {
        let clock = fixed_clock();
        let tz = TimezoneSource::Utc;
        let cfg = PrefixConfig {
            format: format::DEFAULT_FORMAT,
            tz: &tz,
            clock: &clock,
            source: PrefixSource::Absolute,
        };
        let mut out = Vec::new();
        run_prefix(Cursor::new("incomplete"), &mut out, &cfg).expect("ok");
        let s = String::from_utf8(out).expect("utf-8");
        assert_eq!(s, "May 22 14:30:45 incomplete");
        assert!(!s.ends_with('\n'));
    }

    #[test]
    fn binary_payload_passes_through() {
        // Two lines with a non-UTF-8 byte (0xFF) in the payload. The prefix
        // is locale-rendered ASCII; the payload bytes are emitted verbatim.
        let input: &[u8] = b"hello\xff\nworld\n";
        let clock = fixed_clock();
        let tz = TimezoneSource::Utc;
        let cfg = PrefixConfig {
            format: format::DEFAULT_FORMAT,
            tz: &tz,
            clock: &clock,
            source: PrefixSource::Absolute,
        };
        let mut out = Vec::new();
        run_prefix(Cursor::new(input), &mut out, &cfg).expect("ok");
        // The 0xFF byte must appear verbatim in the output between the
        // first prefix and the second one's newline.
        assert!(
            out.contains(&0xFF),
            "expected 0xFF byte to pass through; got {:?}",
            out,
        );
    }
}
