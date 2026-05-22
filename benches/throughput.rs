//! Criterion benches for `rusty-ts` per `plan.md` §Performance Budget & T120.
//!
//! Gated behind the `bench` cargo feature. Run with:
//!
//! ```sh
//! cargo bench --features bench --bench throughput
//! ```
//!
//! Targets per `plan.md`:
//! - Default-format throughput ≥ 10 MB/s on the reference Linux x86_64 runner
//! - `-r` Default-subset throughput ≥ 2 MB/s; Strict full set ≥ 1 MB/s
//! - Cold-start TZ-resolution overhead ≤ 5 ms one-time
//! - Per-line allocation profile ≤ 2 heap allocs steady state (covered by
//!   a separate allocation-counting test, not by criterion)

#![cfg(feature = "bench")]

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use rusty_ts::mode::CompatibilityMode;
use rusty_ts::relative::RelativeRewriter;
use rusty_ts::time::clock::{Clock, Wall};
use rusty_ts::time::format::{DEFAULT_FORMAT, format_default, format_with};
use rusty_ts::time::tz::TimezoneSource;

/// Synthesize an in-memory fixture: N lines of `width`-char ASCII content.
fn synth_lines(n: usize, width: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(n * (width + 1));
    for i in 0..n {
        let payload: String = (0..width)
            .map(|j| (b'a' + ((i + j) % 26) as u8) as char)
            .collect();
        out.extend_from_slice(payload.as_bytes());
        out.push(b'\n');
    }
    out
}

fn bench_default_format_render(c: &mut Criterion) {
    let clock = Wall;
    let tz = TimezoneSource::Utc;
    c.bench_function("format_default_one_call", |b| {
        b.iter(|| {
            let now = black_box(clock.now());
            let s = format_default(now, &tz);
            black_box(s);
        })
    });
}

fn bench_custom_format_render(c: &mut Criterion) {
    let clock = Wall;
    let tz = TimezoneSource::Utc;
    let spec = "%Y-%m-%d %H:%M:%S";
    c.bench_function("format_with_iso_one_call", |b| {
        b.iter(|| {
            let now = black_box(clock.now());
            let s = format_with(black_box(spec), now, &tz);
            black_box(s);
        })
    });
}

fn bench_fractional_format_render(c: &mut Criterion) {
    let clock = Wall;
    let tz = TimezoneSource::Utc;
    let spec = "%H:%M:%.S";
    c.bench_function("format_with_fractional_one_call", |b| {
        b.iter(|| {
            let now = black_box(clock.now());
            let s = format_with(black_box(spec), now, &tz);
            black_box(s);
        })
    });
}

fn bench_tz_resolution_startup_cost(c: &mut Criterion) {
    // SC-022 / FR-019: cached IANA lookup at startup. Bench the named()
    // call itself — this is the one-time cost paid before per-line render.
    c.bench_function("timezone_named_resolution", |b| {
        b.iter(|| {
            let tz = TimezoneSource::named(black_box("America/New_York")).expect("valid IANA");
            black_box(tz);
        })
    });
}

fn bench_relative_rewriter_default(c: &mut Criterion) {
    let rewriter = RelativeRewriter::for_mode(CompatibilityMode::Default);
    let reference = Wall.now();
    let line =
        "Event at 2026-05-22T14:30:42Z and epoch=1779798645 happened with no timestamp here.";
    c.bench_function("relative_default_subset_one_line", |b| {
        b.iter(|| {
            let out = rewriter.rewrite(black_box(line), black_box(reference));
            black_box(out);
        })
    });
}

fn bench_relative_rewriter_strict(c: &mut Criterion) {
    let rewriter = RelativeRewriter::for_mode(CompatibilityMode::Strict);
    let reference = Wall.now();
    let line = "Event at 2026-05-22 14:30:42 and 2026-05-22T14:30:42Z happened.";
    c.bench_function("relative_strict_full_set_one_line", |b| {
        b.iter(|| {
            let out = rewriter.rewrite(black_box(line), black_box(reference));
            black_box(out);
        })
    });
}

fn bench_throughput_default_format(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput_default_format");
    for (n, width) in [(100, 80), (1000, 80), (10_000, 80)].iter() {
        let input = synth_lines(*n, *width);
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{n}x{width}")),
            &input,
            |b, input| {
                b.iter(|| {
                    use rusty_ts::pipeline::{PrefixConfig, PrefixSource, run_prefix};
                    let clock = Wall;
                    let tz = TimezoneSource::Utc;
                    let cfg = PrefixConfig {
                        format: DEFAULT_FORMAT,
                        tz: &tz,
                        clock: &clock,
                        source: PrefixSource::Absolute,
                    };
                    let mut out = Vec::with_capacity(input.len() * 2);
                    run_prefix(std::io::Cursor::new(input.as_slice()), &mut out, &cfg).expect("ok");
                    black_box(out);
                })
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_default_format_render,
    bench_custom_format_render,
    bench_fractional_format_render,
    bench_tz_resolution_startup_cost,
    bench_relative_rewriter_default,
    bench_relative_rewriter_strict,
    bench_throughput_default_format,
);

criterion_main!(benches);
