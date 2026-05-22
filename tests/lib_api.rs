//! Library-API integration tests.
//!
//! Per US7 (Reusable Library API) and `plan.md` AD-008: tests that exercise
//! the public surface (`Timestamper`, `TimestamperBuilder`, `Format`,
//! `TimezoneSource`, `CompatibilityMode`, `ElapsedAnchor`, `Error`) without
//! going through the binary.
//!
//! Covers:
//! - FR-012 library API shape and byte-typed canonical surface
//! - FR-011 non-UTF-8 byte passthrough preserved through the library
//! - FR-020 library-layer mirror via `Error::InvalidUtcWithNamedTz`
//! - SC-007 library can be consumed from an external crate without
//!   enabling the binary feature (verified by `cargo build
//!   --no-default-features` in CI rather than by this file)
//! - Library/binary output parity (T101)

use rusty_ts::{
    CompatibilityMode, ElapsedAnchor, Error, Format, TimestamperBuilder, TimezoneSource,
};
use std::io::Cursor;

// ─────────── FR-020 library-layer mirror (T100) ───────────

#[test]
fn builder_rejects_utc_plus_tz_name() {
    let result = TimestamperBuilder::new()
        .utc(true)
        .tz_name("Asia/Tokyo")
        .build();

    match result {
        Err(Error::InvalidUtcWithNamedTz { tz }) => {
            assert_eq!(tz, "Asia/Tokyo");
        }
        other => panic!("expected InvalidUtcWithNamedTz, got {other:?}"),
    }
}

#[test]
fn builder_rejects_unknown_iana_name() {
    let result = TimestamperBuilder::new()
        .tz_name("Atlantis/Atlantica")
        .build();

    match result {
        Err(Error::InvalidIanaName(name)) => {
            assert_eq!(name, "Atlantis/Atlantica");
        }
        other => panic!("expected InvalidIanaName, got {other:?}"),
    }
}

#[test]
fn builder_utc_alone_succeeds() {
    let ts = TimestamperBuilder::new().utc(true).build().expect("builds");
    assert!(matches!(ts.timezone(), TimezoneSource::Utc));
}

#[test]
fn builder_tz_name_alone_succeeds() {
    let ts = TimestamperBuilder::new()
        .tz_name("Asia/Tokyo")
        .build()
        .expect("builds");
    assert!(matches!(ts.timezone(), TimezoneSource::Named(_)));
}

#[test]
fn builder_local_is_default() {
    let ts = TimestamperBuilder::new().build().expect("builds");
    assert!(matches!(ts.timezone(), TimezoneSource::Local));
}

#[test]
fn builder_timezone_override_takes_precedence() {
    // utc(true) + low-level timezone() override → the override wins.
    let ts = TimestamperBuilder::new()
        .utc(true)
        .timezone(TimezoneSource::Local)
        .build()
        .expect("builds");
    assert!(matches!(ts.timezone(), TimezoneSource::Local));
}

// ─────────── FR-011 non-UTF-8 passthrough via library (T097) ───────────

#[test]
fn byte_typed_iterator_preserves_non_utf8_payload() {
    let ts = TimestamperBuilder::new()
        .format(Format::Strftime("[%H:%M:%S]".into()))
        .utc(true)
        .build()
        .expect("builds");

    let input: &[u8] = b"hello\xff\nworld\n";
    let chunks: Vec<Vec<u8>> = ts
        .prefix_lines(Cursor::new(input.to_vec()))
        .collect::<Result<Vec<_>, _>>()
        .expect("io ok");

    assert_eq!(chunks.len(), 2, "expected two output chunks");
    assert!(
        chunks[0].contains(&0xFF),
        "first chunk should contain raw 0xFF byte; got {:?}",
        chunks[0],
    );
    assert!(
        chunks[1].ends_with(b"  world\n"),
        "second chunk should end with '  world\\n'; got {:?}",
        chunks[1],
    );
}

// ─────────── Compile-time Send / Sync assertions (T099) ───────────

#[test]
fn timestamper_is_send() {
    use rusty_ts::Timestamper;
    static_assertions::assert_impl_all!(Timestamper: Send);
}

#[test]
fn builder_is_send_sync() {
    static_assertions::assert_impl_all!(TimestamperBuilder: Send, Sync);
}

#[test]
fn error_is_send_sync() {
    static_assertions::assert_impl_all!(Error: Send, Sync);
}

// ─────────── #[must_use] coverage (T098) ───────────
//
// A compile-fail test for `#[must_use]` would require a separate trybuild
// crate. We approximate by asserting that every chain method's return type
// is Self (which combined with the `#[must_use]` attribute means dropping
// the return value triggers an unused_must_use warning under
// `#[deny(unused_must_use)]`).
#[test]
fn builder_chain_methods_return_self() {
    // Construct via every chain method to confirm each one yields a usable
    // `TimestamperBuilder`. If any method ever changed to return `()` or
    // `&mut Self`, this test would fail to compile.
    let _ = TimestamperBuilder::new()
        .format(Format::Default)
        .utc(false)
        .tz_name("UTC") // valid IANA
        .timezone(TimezoneSource::Utc)
        .compat(CompatibilityMode::Default)
        .elapsed(ElapsedAnchor::Absolute)
        .build()
        .expect("builds");
}

// ─────────── Format selection (T040 / T042 shape) ───────────

#[test]
fn default_format_under_utc_is_deterministic_shape() {
    let ts = TimestamperBuilder::new().utc(true).build().expect("builds");
    let input = Cursor::new(b"abc\n".to_vec());
    let chunks: Vec<Vec<u8>> = ts
        .prefix_lines(input)
        .collect::<Result<Vec<_>, _>>()
        .expect("io ok");
    let s = std::str::from_utf8(&chunks[0]).expect("utf-8");
    let re = regex::Regex::new(r"^[A-Z][a-z]{2} [ 0-9]\d \d{2}:\d{2}:\d{2}  abc\n$").unwrap();
    assert!(re.is_match(s), "default format shape failed: {s:?}");
}

#[test]
fn custom_format_string_is_honored() {
    let ts = TimestamperBuilder::new()
        .utc(true)
        .format(Format::Strftime("[%H:%M]".into()))
        .build()
        .expect("builds");
    let input = Cursor::new(b"abc\n".to_vec());
    let chunks: Vec<Vec<u8>> = ts
        .prefix_lines(input)
        .collect::<Result<Vec<_>, _>>()
        .expect("io ok");
    let s = std::str::from_utf8(&chunks[0]).expect("utf-8");
    let re = regex::Regex::new(r"^\[\d{2}:\d{2}\]  abc\n$").unwrap();
    assert!(re.is_match(s), "custom format failed: {s:?}");
}

// ─────────── String convenience adapter (T094) ───────────

#[test]
fn string_lines_adapter_yields_prefixed_strings() {
    let ts = TimestamperBuilder::new()
        .utc(true)
        .format(Format::Strftime("[%H:%M]".into()))
        .build()
        .expect("builds");
    let lines = vec!["alpha\n".to_string(), "beta\n".to_string()];
    let out: Vec<String> = ts.prefix_string_lines(lines).collect();
    assert_eq!(out.len(), 2);
    assert!(out[0].ends_with("  alpha\n"), "got {:?}", out[0]);
    assert!(out[1].ends_with("  beta\n"), "got {:?}", out[1]);
}

// ─────────── Library / binary parity (T101) ───────────
//
// We can't byte-match library output to binary output because the binary
// uses Wall clock and the library does too — timestamps will differ by ms.
// Instead we verify *structural* parity: same line count, same payload
// preservation, same format shape.
#[test]
fn library_and_binary_produce_structurally_identical_output() {
    use assert_cmd::Command;

    let input = "hello\nworld\n";
    let format_spec = "[%H:%M]";

    // Library path.
    let ts = TimestamperBuilder::new()
        .utc(true)
        .format(Format::Strftime(format_spec.into()))
        .build()
        .expect("builds");
    let lib_chunks: Vec<Vec<u8>> = ts
        .prefix_lines(Cursor::new(input.as_bytes().to_vec()))
        .collect::<Result<Vec<_>, _>>()
        .expect("io ok");
    let lib_lines: Vec<String> = lib_chunks
        .iter()
        .map(|b| String::from_utf8(b.clone()).unwrap())
        .collect();

    // Binary path.
    let mut cmd = Command::cargo_bin("rusty-ts").unwrap();
    cmd.env("TZ", "UTC")
        .env("LC_ALL", "C.UTF-8")
        .env_remove("RUSTY_TS_FORMAT")
        .env_remove("RUSTY_TS_STRICT")
        .args(["-u", format_spec])
        .write_stdin(input);
    let assertion = cmd.assert().success();
    let bin_stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    let bin_lines: Vec<&str> = bin_stdout.lines().collect();

    // Parity checks: same line count, same shape, same payload suffix.
    assert_eq!(
        lib_lines.len(),
        bin_lines.len(),
        "library and binary disagree on line count: lib={lib_lines:?} bin={bin_lines:?}",
    );

    let shape = regex::Regex::new(r"^\[\d{2}:\d{2}\]  (hello|world)\n?$").unwrap();
    for (i, (lib, bin)) in lib_lines.iter().zip(bin_lines.iter()).enumerate() {
        assert!(shape.is_match(lib), "lib line {i} shape mismatch: {lib:?}");
        assert!(shape.is_match(bin), "bin line {i} shape mismatch: {bin:?}");
    }
    assert!(lib_lines[0].ends_with("  hello\n"));
    assert!(lib_lines[1].ends_with("  world\n"));
    assert!(bin_lines[0].ends_with("  hello"));
    assert!(bin_lines[1].ends_with("  world"));
}
