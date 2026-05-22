//! Clock sources for absolute and elapsed-mode timestamping.
//!
//! Per `plan.md` AD-008 and HINT-003: the `Clock` trait is injectable so
//! snapshot tests can pin time to a deterministic fixed instant. The three
//! impls cover:
//!
//! - `Wall` — system local wall clock (default for absolute timestamps).
//! - `Monotonic` — std::time::Instant-backed monotonic source for the
//!   `-m` flag; unaffected by NTP / manual clock adjustments.
//! - `Fixed` — test-only, returns a constant `DateTime<Utc>` so byte-level
//!   snapshot tests are deterministic regardless of when they run.

use chrono::{DateTime, Utc};
use std::time::Instant;

/// Abstract clock source. Implementors must produce a `DateTime<Utc>` on
/// every call to `now()`. Implementors that track elapsed time (Monotonic,
/// Fixed) maintain their own internal anchor and must not require external
/// synchronisation.
pub trait Clock {
    /// Current instant as a UTC datetime. The timezone-resolution layer
    /// (`crate::time::tz`) is responsible for converting to the rendered
    /// zone.
    fn now(&self) -> DateTime<Utc>;
}

/// System wall clock via `chrono::Utc::now()`. The default for absolute
/// timestamps in Default mode. Reflects any NTP / manual adjustments.
#[derive(Debug, Default, Clone, Copy)]
pub struct Wall;

impl Clock for Wall {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Monotonic clock anchored at construction time. Reports a `DateTime<Utc>`
/// derived as `program_start_wall + elapsed_since_start`. The wall component
/// is fixed at construction; only the elapsed component advances. Used by
/// `-m` to make `-i` and `-s` elapsed measurements robust against clock
/// adjustments.
#[derive(Debug)]
pub struct Monotonic {
    /// Wall-clock anchor captured at construction.
    anchor_wall: DateTime<Utc>,
    /// Monotonic anchor captured at construction.
    anchor_mono: Instant,
}

impl Monotonic {
    /// Capture both wall and monotonic anchors at the moment of construction.
    pub fn new() -> Self {
        Self {
            anchor_wall: Utc::now(),
            anchor_mono: Instant::now(),
        }
    }
}

impl Default for Monotonic {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for Monotonic {
    fn now(&self) -> DateTime<Utc> {
        let elapsed = self.anchor_mono.elapsed();
        self.anchor_wall + chrono::Duration::from_std(elapsed).unwrap_or(chrono::Duration::zero())
    }
}

/// Fixed clock for snapshot determinism. Always returns the same instant.
/// Test-only — gated to `cfg(test)` callers and the dev-only test harness.
#[derive(Debug, Clone, Copy)]
pub struct Fixed {
    instant: DateTime<Utc>,
}

impl Fixed {
    /// Pin the clock at a specific UTC datetime.
    pub fn new(instant: DateTime<Utc>) -> Self {
        Self { instant }
    }
}

impl Clock for Fixed {
    fn now(&self) -> DateTime<Utc> {
        self.instant
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn fixed_clock_is_deterministic() {
        let target = Utc.with_ymd_and_hms(2026, 5, 22, 14, 30, 45).unwrap();
        let clock = Fixed::new(target);
        assert_eq!(clock.now(), target);
        assert_eq!(clock.now(), target);
    }

    #[test]
    fn wall_clock_is_monotonic_ish() {
        let clock = Wall;
        let a = clock.now();
        let b = clock.now();
        assert!(b >= a, "wall clock went backwards: {a} -> {b}");
    }

    #[test]
    fn monotonic_clock_advances() {
        let clock = Monotonic::new();
        let a = clock.now();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let b = clock.now();
        assert!(b > a, "monotonic clock did not advance: {a} -> {b}");
    }
}
