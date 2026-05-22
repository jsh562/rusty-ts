//! `ts` binary alias. Identical body to `src/main.rs`. Installed only when
//! the `ts-alias` cargo feature is enabled. The argv[0] basename auto-detect
//! in `mode::resolve` routes invocations under this name into Strict mode
//! (FR-023).

fn main() -> std::process::ExitCode {
    rusty_ts::run()
}
