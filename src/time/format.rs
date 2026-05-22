//! Strftime rendering with moreutils-`ts` fractional-second extensions.
//!
//! Per `spec.md` FR-003, FR-004, FR-008 and `plan.md` HINT-001:
//!
//! moreutils `ts` adds two strftime tokens chrono does not implement
//! natively: `%.S` (seconds-with-fractional-component) and `%.s` (Unix
//! epoch-with-fractional-component). We implement these as a one-pass
//! pre-tokenizer that splices the fractional digits into the chrono-rendered
//! output. Default precision is 6 digits (microsecond, matching moreutils'
//! `Time::HiRes`-backed default) per FR-008.
//!
//! The default format string is `"%b %d %H:%M:%S"` (FR-003).

use crate::time::tz::TimezoneSource;
use chrono::{DateTime, Utc};

/// The moreutils `ts` default format.
pub const DEFAULT_FORMAT: &str = "%b %d %H:%M:%S";

/// Default fractional precision (digits) for `%.S` / `%.s`. Microsecond
/// resolution per FR-008.
pub const DEFAULT_FRACTIONAL_DIGITS: usize = 6;

/// Render the moreutils default format via the supplied timezone.
pub fn format_default(now: DateTime<Utc>, tz: &TimezoneSource) -> String {
    format_with(DEFAULT_FORMAT, now, tz)
}

/// Render an arbitrary strftime format, expanding moreutils `%.S` and `%.s`
/// fractional tokens before delegating the rest to chrono.
pub fn format_with(spec: &str, now: DateTime<Utc>, tz: &TimezoneSource) -> String {
    if !spec.contains("%.S") && !spec.contains("%.s") {
        return tz.render(now, spec);
    }

    // One-pass pre-tokenizer: walk the format string, split on the two
    // fractional tokens, render the surrounding fragments via chrono, and
    // splice in the fractional component at each token site. Microsecond
    // precision; lower-precision systems get whatever the underlying clock
    // supplies (still ≤ 6 digits).
    let micros = now.timestamp_subsec_micros();
    let epoch = now.timestamp();
    let frac_seconds = format!("{micros:06}");
    let frac_epoch = format!("{epoch}.{micros:06}");

    let mut out = String::with_capacity(spec.len() + 16);
    let mut remaining = spec;

    loop {
        // Find the earliest of `%.S` and `%.s`.
        let pos_big = remaining.find("%.S");
        let pos_small = remaining.find("%.s");

        let (pos, is_big) = match (pos_big, pos_small) {
            (Some(a), Some(b)) if a < b => (a, true),
            (Some(_), Some(b)) => (b, false),
            (Some(a), None) => (a, true),
            (None, Some(b)) => (b, false),
            (None, None) => {
                // No more fractional tokens; render the rest via chrono.
                out.push_str(&tz.render(now, remaining));
                break;
            }
        };

        // Render the prefix (everything up to the fractional token) via chrono.
        if pos > 0 {
            out.push_str(&tz.render(now, &remaining[..pos]));
        }

        if is_big {
            // `%.S` = seconds with microsecond fraction. Render the integer
            // second component from the zone-converted instant, then append
            // ".{microseconds}".
            let seconds = tz.render(now, "%S");
            out.push_str(&seconds);
            out.push('.');
            out.push_str(&frac_seconds);
        } else {
            // `%.s` = Unix epoch with microsecond fraction. Independent of
            // timezone (epoch is UTC-anchored).
            out.push_str(&frac_epoch);
        }

        remaining = &remaining[pos + 3..];
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Timelike};

    fn fixture_instant() -> DateTime<Utc> {
        // 2026-05-22 14:30:45.123456 UTC — deterministic for snapshot-style
        // comparisons in unit tests.
        Utc.with_ymd_and_hms(2026, 5, 22, 14, 30, 45)
            .unwrap()
            .with_nanosecond(123_456_000)
            .unwrap()
    }

    #[test]
    fn default_format_matches_moreutils_default_string() {
        assert_eq!(DEFAULT_FORMAT, "%b %d %H:%M:%S");
    }

    #[test]
    fn default_format_under_utc_is_deterministic() {
        let rendered = format_default(fixture_instant(), &TimezoneSource::Utc);
        assert_eq!(rendered, "May 22 14:30:45");
    }

    #[test]
    fn custom_format_renders_tokens() {
        let rendered = format_with("%Y-%m-%d %H:%M:%S", fixture_instant(), &TimezoneSource::Utc);
        assert_eq!(rendered, "2026-05-22 14:30:45");
    }

    #[test]
    fn literal_brackets_are_preserved() {
        let rendered = format_with("[%H:%M:%S]", fixture_instant(), &TimezoneSource::Utc);
        assert_eq!(rendered, "[14:30:45]");
    }

    #[test]
    fn fractional_seconds_token_expands() {
        let rendered = format_with("%H:%M:%.S", fixture_instant(), &TimezoneSource::Utc);
        assert_eq!(rendered, "14:30:45.123456");
    }

    #[test]
    fn fractional_epoch_token_expands() {
        let rendered = format_with("%.s", fixture_instant(), &TimezoneSource::Utc);
        // 2026-05-22 14:30:45 UTC epoch = 1779798645
        let expected_epoch: i64 = fixture_instant().timestamp();
        assert_eq!(rendered, format!("{expected_epoch}.123456"));
    }

    #[test]
    fn both_fractional_tokens_in_one_string() {
        let rendered = format_with(
            "%H:%M:%.S epoch=%.s",
            fixture_instant(),
            &TimezoneSource::Utc,
        );
        let expected_epoch: i64 = fixture_instant().timestamp();
        assert_eq!(
            rendered,
            format!("14:30:45.123456 epoch={expected_epoch}.123456"),
        );
    }
}
