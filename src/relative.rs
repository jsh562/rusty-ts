//! Relative-mode (`-r`) timestamp rewriter.
//!
//! Per `spec.md` FR-009 and FR-025:
//!
//! - **Default mode**: recognizes ISO-8601, RFC-3339, and Unix epoch
//!   (integer or fractional) timestamps. Other timestamp formats pass
//!   through unchanged.
//! - **Strict mode** (FR-025): recognizes the full moreutils `ts -r` set,
//!   including human-readable formats moreutils ships regexes for.
//!
//! The recognized timestamps are rewritten in place to a human-relative
//! form ("3.2s ago", "1m12s ago", ...). Lines without recognizable
//! timestamps pass through unchanged.

use crate::mode::CompatibilityMode;
use chrono::{DateTime, Utc};
use regex::Regex;

/// Container for the precompiled regex set used by relative-mode rewriting.
/// Built once at startup; cloned cheaply via `Arc` internals.
#[derive(Debug)]
pub struct RelativeRewriter {
    patterns: Vec<RecognizedPattern>,
}

#[derive(Debug)]
struct RecognizedPattern {
    re: Regex,
    parser: ParserKind,
}

#[derive(Debug, Clone, Copy)]
enum ParserKind {
    Iso8601,
    Rfc3339,
    UnixEpoch,
    /// Strict-only: additional human-readable formats moreutils recognizes.
    /// For v0.1.0 we ship the "%Y-%m-%d %H:%M:%S" date-time pattern as the
    /// Strict superset baseline; future work expands this set.
    HumanDateTime,
}

impl RelativeRewriter {
    /// Build a rewriter configured for the given mode. Compiles each
    /// regex once — callers should hold the rewriter for the lifetime
    /// of the invocation.
    ///
    /// Pattern ordering matters: more-specific (longer match) patterns are
    /// listed first, so that when overlap resolution runs (in `rewrite`)
    /// the more-specific match wins.
    pub fn for_mode(mode: CompatibilityMode) -> Self {
        let mut patterns: Vec<RecognizedPattern> = Vec::new();

        if mode == CompatibilityMode::Strict {
            // Strict mode adds the moreutils human-date-time pattern.
            // Listed first so it wins over the shorter ISO-8601 date-only
            // pattern when both could match.
            patterns.push(RecognizedPattern {
                re: Regex::new(r"\b\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\b")
                    .expect("human date-time regex"),
                parser: ParserKind::HumanDateTime,
            });
        }

        // RFC-3339: 2026-05-22T14:30:45Z or 2026-05-22T14:30:45.123+02:00
        patterns.push(RecognizedPattern {
            re: Regex::new(
                r"\b\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:?\d{2})\b",
            )
            .expect("RFC-3339 regex"),
            parser: ParserKind::Rfc3339,
        });
        // ISO-8601 date-only: 2026-05-22 (less specific; placed after RFC-3339)
        patterns.push(RecognizedPattern {
            re: Regex::new(r"\b\d{4}-\d{2}-\d{2}\b").expect("ISO-8601 date regex"),
            parser: ParserKind::Iso8601,
        });
        // Unix epoch (integer + optional fractional): 1779798645 or 1779798645.123456
        // Conservative bound: 10-digit epoch (current era) with optional fractional part.
        patterns.push(RecognizedPattern {
            re: Regex::new(r"\b1\d{9}(?:\.\d+)?\b").expect("Unix epoch regex"),
            parser: ParserKind::UnixEpoch,
        });

        Self { patterns }
    }

    /// Rewrite a single line, replacing each recognized timestamp with its
    /// relative form against the supplied reference instant. Lines with no
    /// recognizable timestamp pass through unchanged.
    pub fn rewrite(&self, line: &str, reference: DateTime<Utc>) -> String {
        let mut output = String::with_capacity(line.len());
        let mut cursor = 0usize;

        // Walk all patterns and collect non-overlapping matches sorted by
        // start position.
        let mut matches: Vec<(usize, usize, ParserKind)> = Vec::new();
        for pat in &self.patterns {
            for m in pat.re.find_iter(line) {
                matches.push((m.start(), m.end(), pat.parser));
            }
        }
        matches.sort_by_key(|m| m.0);

        // Resolve overlaps by preferring earlier (and longer-tied) matches.
        let mut filtered: Vec<(usize, usize, ParserKind)> = Vec::new();
        for m in matches {
            if let Some(prev) = filtered.last() {
                if m.0 < prev.1 {
                    continue;
                }
            }
            filtered.push(m);
        }

        for (start, end, parser) in filtered {
            output.push_str(&line[cursor..start]);
            let token = &line[start..end];
            match parse_token(token, parser) {
                Some(parsed) => output.push_str(&relative_form(reference, parsed)),
                None => output.push_str(token), // unparseable — leave alone
            }
            cursor = end;
        }
        output.push_str(&line[cursor..]);
        output
    }
}

fn parse_token(token: &str, parser: ParserKind) -> Option<DateTime<Utc>> {
    match parser {
        ParserKind::Rfc3339 => DateTime::parse_from_rfc3339(token)
            .ok()
            .map(|dt| dt.with_timezone(&Utc)),
        ParserKind::Iso8601 => {
            // Date-only → interpret as midnight UTC of that date.
            chrono::NaiveDate::parse_from_str(token, "%Y-%m-%d")
                .ok()
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .map(|naive| naive.and_utc())
        }
        ParserKind::UnixEpoch => {
            if let Some(dot_pos) = token.find('.') {
                let secs: i64 = token[..dot_pos].parse().ok()?;
                let frac: f64 = format!("0.{}", &token[dot_pos + 1..]).parse().ok()?;
                let nsecs = (frac * 1_000_000_000.0) as u32;
                DateTime::<Utc>::from_timestamp(secs, nsecs)
            } else {
                let secs: i64 = token.parse().ok()?;
                DateTime::<Utc>::from_timestamp(secs, 0)
            }
        }
        ParserKind::HumanDateTime => {
            chrono::NaiveDateTime::parse_from_str(token, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|naive| naive.and_utc())
        }
    }
}

/// Render a relative-duration string in the moreutils style. Outputs forms
/// like "5s ago", "1m23s ago", "2h ago", "1d2h ago", or "now". Future at the
/// reference instant produces "in 5s" / etc.
fn relative_form(reference: DateTime<Utc>, target: DateTime<Utc>) -> String {
    use chrono::Duration;
    let delta: Duration = reference.signed_duration_since(target);
    let (sign, ago_or_in) = if delta.num_seconds() >= 0 {
        ("", "ago")
    } else {
        ("", "in")
    };
    let total = delta.num_seconds().unsigned_abs();
    if total == 0 {
        return "now".into();
    }
    let days = total / 86_400;
    let hours = (total % 86_400) / 3_600;
    let mins = (total % 3_600) / 60;
    let secs = total % 60;

    let body = if days > 0 {
        format!("{days}d{hours}h")
    } else if hours > 0 {
        format!("{hours}h{mins}m")
    } else if mins > 0 {
        format!("{mins}m{secs}s")
    } else {
        format!("{secs}s")
    };

    if ago_or_in == "ago" {
        format!("{sign}{body} ago")
    } else {
        format!("in {body}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn reference() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 5, 22, 14, 30, 45).unwrap()
    }

    #[test]
    fn default_subset_recognizes_rfc3339() {
        let rewriter = RelativeRewriter::for_mode(CompatibilityMode::Default);
        let line = "Event at 2026-05-22T14:30:42Z happened.";
        let out = rewriter.rewrite(line, reference());
        assert!(
            out.contains("3s ago"),
            "expected '3s ago' replacement; got {out:?}",
        );
    }

    #[test]
    fn default_subset_recognizes_unix_epoch() {
        let rewriter = RelativeRewriter::for_mode(CompatibilityMode::Default);
        let epoch = reference().timestamp() - 65;
        let line = format!("epoch={epoch} now=...");
        let out = rewriter.rewrite(&line, reference());
        assert!(
            out.contains("1m5s ago"),
            "expected '1m5s ago' replacement; got {out:?}",
        );
    }

    #[test]
    fn line_without_timestamp_passes_through() {
        let rewriter = RelativeRewriter::for_mode(CompatibilityMode::Default);
        let line = "plain text no timestamp here";
        let out = rewriter.rewrite(line, reference());
        assert_eq!(out, line);
    }

    #[test]
    fn default_mode_does_not_match_human_date_time() {
        let rewriter = RelativeRewriter::for_mode(CompatibilityMode::Default);
        let line = "Event at 2026-05-22 14:30:42 happened.";
        let out = rewriter.rewrite(line, reference());
        // Default mode recognizes the date-only `2026-05-22` portion via
        // ISO-8601 pattern but leaves the time portion alone.
        assert!(
            out.contains("14:30:42"),
            "time component should pass through in Default mode; got {out:?}",
        );
    }

    #[test]
    fn strict_mode_recognizes_human_date_time() {
        let rewriter = RelativeRewriter::for_mode(CompatibilityMode::Strict);
        let line = "Event at 2026-05-22 14:30:42 happened.";
        let out = rewriter.rewrite(line, reference());
        assert!(
            out.contains("3s ago"),
            "Strict mode should rewrite human date-time; got {out:?}",
        );
    }

    #[test]
    fn relative_form_zero_delta_is_now() {
        assert_eq!(relative_form(reference(), reference()), "now");
    }

    #[test]
    fn relative_form_future_uses_in_prefix() {
        let future = reference() + chrono::Duration::seconds(45);
        let out = relative_form(reference(), future);
        assert!(out.starts_with("in "), "got {out:?}");
    }
}
