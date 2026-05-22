//! Binary entry point for the `rusty-ts` CLI. Thin delegation into the
//! library's `run()` helper so the `ts-alias` feature can share the same
//! entry point via `src/bin/ts.rs`.

fn main() -> std::process::ExitCode {
    rusty_ts::run()
}
