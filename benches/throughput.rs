//! Criterion benches for `rusty-ts`. Stub at Phase 2 — populated by T120
//! during the Polish phase. Gated behind the `bench` cargo feature so it
//! does not affect default `cargo build`.

#[cfg(feature = "bench")]
fn main() {
    use criterion::Criterion;
    let mut c = Criterion::default().configure_from_args();
    c.bench_function("placeholder_pending_t120", |b| b.iter(|| 1 + 1));
    c.final_summary();
}

#[cfg(not(feature = "bench"))]
fn main() {
    eprintln!("rusty-ts: benches require `--features bench`");
}
