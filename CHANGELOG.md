# Changelog

All notable changes to `rusty-ts` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial scaffold for the inaugural Rusty portfolio port.
- Public library API (`Timestamper`, `TimestamperBuilder`) — byte-typed canonical
  surface plus String convenience adapter.
- CLI binary `rusty-ts` with default-mode timestamp prefixing.
- Optional `ts` binary alias gated behind the `ts-alias` cargo feature.
- Strict moreutils-compat mode via `--strict` / `RUSTY_TS_STRICT` / argv[0]=`ts`.
- `-u` / `--utc` and `--tz=<IANA>` timezone control flags.
- `RUSTY_TS_FORMAT` env var for implicit default format.
- `completions <shell>` subcommand for bash/zsh/fish/PowerShell.
- README Compatibility Matrix at `docs/COMPATIBILITY.md`, generated from CLI
  definition and drift-tested in CI.
- Behavioral-parity snapshot test suite against captured moreutils-`ts`
  fixtures under `TZ=UTC` and `LC_ALL=C.UTF-8`.
