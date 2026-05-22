//! Time-rendering machinery: format strings, timezone resolution, clock
//! sources. See `plan.md` §Architecture (Components: Time Renderer, TZ
//! Resolver, Clock Source).

pub mod clock;
pub mod format;
pub mod tz;
