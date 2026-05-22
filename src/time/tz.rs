//! Timezone-source resolution.
//!
//! Per `spec.md` FR-017, FR-018, FR-019 and `plan.md` AD-001:
//!
//! - Default: system local time, honoring the `TZ` env var (handled by
//!   `chrono::Local` via the OS).
//! - `-u` / `--utc`: rendering in UTC.
//! - `--tz=<IANA>`: a named IANA zone resolved via `chrono-tz`. The lookup
//!   is paid once at startup; per-line render is a fixed-offset conversion.

use crate::error::Error;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;

/// Resolved timezone source. Built once at startup; used for every per-line
/// render. `#[non_exhaustive]` so a future variant (e.g., explicit FixedOffset)
/// can be added in minor releases.
///
/// # Example
///
/// ```
/// use rusty_ts::TimezoneSource;
///
/// // Three ways to construct a timezone source:
/// let local = TimezoneSource::Local;
/// let utc = TimezoneSource::Utc;
/// let tokyo = TimezoneSource::named("Asia/Tokyo").expect("valid IANA");
///
/// // Unknown IANA names return Error::InvalidIanaName.
/// assert!(TimezoneSource::named("Atlantis/Atlantica").is_err());
/// # let _ = (local, utc, tokyo);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum TimezoneSource {
    /// System local time as adjusted by the `TZ` env var if set.
    Local,
    /// UTC (no offset, no DST).
    Utc,
    /// A named IANA zone.
    Named(Tz),
}

impl TimezoneSource {
    /// Build a `TimezoneSource::Local`.
    pub fn local() -> Self {
        Self::Local
    }

    /// Build a `TimezoneSource::Utc`.
    pub fn utc() -> Self {
        Self::Utc
    }

    /// Resolve an IANA name (e.g., `"America/New_York"`) via `chrono-tz`.
    /// Returns `Error::InvalidIanaName` if the name is not recognized.
    pub fn named(iana: &str) -> Result<Self, Error> {
        iana.parse::<Tz>()
            .map(Self::Named)
            .map_err(|_| Error::InvalidIanaName(iana.to_owned()))
    }

    /// Format a UTC instant as the zone-local wall-clock string using the
    /// provided strftime format. The rendering cost is uniform across the
    /// three variants — a single offset conversion per call.
    pub fn render(&self, instant: DateTime<Utc>, fmt: &str) -> String {
        match self {
            Self::Local => instant
                .with_timezone(&chrono::Local)
                .format(fmt)
                .to_string(),
            Self::Utc => instant.format(fmt).to_string(),
            Self::Named(tz) => instant.with_timezone(tz).format(fmt).to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn utc_renders_hours_as_zero_offset() {
        let instant = Utc.with_ymd_and_hms(2026, 5, 22, 14, 30, 45).unwrap();
        let rendered = TimezoneSource::Utc.render(instant, "%H:%M:%S");
        assert_eq!(rendered, "14:30:45");
    }

    #[test]
    fn named_resolves_known_iana() {
        let tz = TimezoneSource::named("America/New_York").expect("known zone");
        let instant = Utc.with_ymd_and_hms(2026, 5, 22, 14, 30, 45).unwrap();
        let rendered = tz.render(instant, "%H:%M");
        // New York in May 2026 is EDT (UTC-4); 14:30 UTC -> 10:30 EDT
        assert_eq!(rendered, "10:30");
    }

    #[test]
    fn named_rejects_unknown_iana() {
        let result = TimezoneSource::named("Atlantis/Atlantica");
        match result {
            Err(Error::InvalidIanaName(name)) => assert_eq!(name, "Atlantis/Atlantica"),
            other => panic!("expected InvalidIanaName, got {other:?}"),
        }
    }
}
