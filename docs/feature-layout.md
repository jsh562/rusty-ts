# rusty-ts — v0.2.0 Feature Layout

**Status**: implementation draft for the v0.2.0 Cargo features convention
backfill (spec 00011, Phase 3 — rusty-ts).

**Authority**:
- `specs/adrs/0006-cargo-features-convention-for-portfolio-ports.md` (why)
- `project-instructions.md` §Cargo Feature Surface (what)
- This document — the per-port carving + WHY for each leaf, per HINT-003
  + HINT-009 of spec 00011.

**Reference port**: rusty-figlet v0.2.0 (commit b493d57) — see
`rusty-figlet/docs/feature-layout.md` (FROZEN reference port) for the format
anchor. rusty-ts conforms to the same shape with the minimum-convention
surface dictated by its single-capability scope.

## Single-capability port — spec 00011 §Scope Edge Cases

rusty-ts is the canonical **single-capability port** called out in spec
00011 §Scope Edge Cases: it has exactly **one** documented capability —
prefix each line of stdin with a timestamp. Spec 00011 dictates that
single-capability ports apply the **minimum convention**:

> ports with only one capability adopt the minimum convention:
> `full = ["cli"]` and `<port>-classic = ["cli"]` are the required
> umbrellas; ZERO leaves carved beyond those required umbrellas.

This document records the carving exercise and the explicit decision
to NOT split orthogonal sub-capabilities into leaves — every other
feature of `rusty-ts` (UTC, named IANA tz, elapsed modes, `-r` relative
rewriter, Strict mode, completions subcommand) is part of the single
core capability surface and removing any of them would break the
documented public CLI / library contract.

## Source-tree walk

`src/` modules (v0.1.0, post-Phase-1 baseline):

| Module                | Always-on? | CLI-only deps                          | Notes                                                              |
|-----------------------|-----------:|----------------------------------------|--------------------------------------------------------------------|
| `error.rs`            | yes        | (thiserror — always-on)                | `Error` enum; library + binary need it.                            |
| `mode.rs`             | yes        | none                                   | `CompatibilityMode` resolver. Library + binary need it.            |
| `pipeline.rs`         | yes        | none                                   | The `run_prefix` + `run_relative` line-processing core.            |
| `relative.rs`         | yes        | none                                   | `-r` rewriter (ISO-8601, RFC-3339, Unix epoch recognizers).        |
| `time/clock.rs`       | yes        | (chrono — always-on)                   | Wall + Monotonic + Fixed clock sources.                            |
| `time/format.rs`      | yes        | (chrono — always-on)                   | Strftime formatter + `%.S`/`%.s` fractional extensions.            |
| `time/tz.rs`          | yes        | (chrono, chrono-tz — always-on)        | `TimezoneSource` enum (Utc / Local / Named).                       |
| `lib.rs`              | yes        | none                                   | Public API (`TimestamperBuilder`, `Timestamper`, etc.).            |
| `cli.rs`              | no — `cli` | clap                                   | clap-derive `Cli` struct + `CliCommand::Completions` subcommand.   |
| `completions.rs`      | no — `cli` | clap_complete                          | Completion-script emitter for bash/zsh/fish/powershell.            |
| `compat_matrix.rs`    | no — `cli` | (clap — always-on for `cli`)           | Compatibility-matrix table for `--help` / drift-test (FR-049).     |
| `main.rs`             | no — `cli` | clap, clap_complete, anyhow            | Binary entry; gated by `required-features = ["cli"]`.              |
| `bin/ts.rs`           | no — `ts-alias` | clap, clap_complete, anyhow       | `ts` alias binary; gated by `required-features = ["ts-alias"]`.    |

## Leaf-carving criteria (HINT-009)

A capability becomes a leaf when ALL of the following hold:

1. It is **self-containable** — gated cleanly via `#[cfg(feature = "<leaf>")]`
   at the module or top-level item boundary (HINT-004).
2. Either (a) it has a **sole optional dependency** that no other leaf needs
   (HINT-005), OR (b) it is a pure-cfg-gate of an internal module worth
   exposing as a knob.
3. Disabling it does NOT break any always-on library/CLI surface.

A capability does NOT become a leaf when:

- It is foundational (timestamping pipeline, relative rewriter, clock,
  timezone resolver, format engine) — disabling it would break every
  other capability.
- It is part of the single documented capability surface (timestamping
  with optional UTC / IANA tz / elapsed modes / Strict mode / completions).
- It would create more than ~6 leaves (FR-007 + HINT-003 envelope).

## v0.2.0 Carved Leaves

**ZERO leaves carved**. Every capability inside rusty-ts is either:

1. Foundational always-on library code (timestamping pipeline, clock,
   timezone resolver, format engine, relative rewriter) — cannot be
   stripped without breaking the documented public surface.
2. Already gated by the required `cli` umbrella (clap-derived argument
   parsing, completions subcommand, anyhow error wrapping).
3. Already gated by an optional aliased-binary feature (`ts-alias`)
   that is itself the minimum-convention-required preset bundle.

### Leaves intentionally NOT carved

The following candidate leaves were considered + rejected:

- **`utc` / `timezone-named`**: `-u` / `--utc` and `--tz=<IANA>` are
  documented Default-mode flags that ship with the bare port. The
  underlying `chrono-tz` dep is always-on library code (the
  `TimezoneSource` enum is part of the public library API), so a
  leaf-gated version would either duplicate the surface or break
  library callers. Rejected per HINT-009 criterion 3.
- **`elapsed`**: `-i` / `-s` / `-m` (elapsed-time modes) ship in the
  bare port. The `ElapsedAnchor` enum is part of the public library
  surface. No carving signal.
- **`relative-rewrite`**: `-r` mode is part of the documented moreutils
  `ts` contract that the port reproduces. Stripping it would break the
  bare-port-replaces-moreutils-ts promise.
- **`strict-compat`**: rusty-ts's Strict mode is dispatched inline in
  `lib.rs::run` (~25 lines, no separate parser), not a hand-rolled
  parser module like rusty-figlet has. There is no compile-time savings
  worth a leaf carve.
- **`completions`**: Could be carved as `["dep:clap_complete"]`, but
  per spec 00011 §Scope Edge Cases minimum-convention single-capability
  ports declare ZERO leaves. `clap_complete` is bundled into the
  required `cli` umbrella unchanged from v0.1.0 to keep the keep-list
  workaround example trivial.
- **`bench`**: The v0.1.x `bench` feature is a tooling/benchmark
  scaffold (criterion benches), not a runtime capability leaf. It
  remains a dev-tooling feature outside the convention's purview and
  is retained verbatim from v0.1.0.

## Preset bundles (FR-007 — 2 required for single-capability ports)

Per spec 00011 §Scope Edge Cases + FR-007, even single-capability ports
declare 2 preset bundles to give the keep-list workaround documentation
something concrete to point at.

### `ts-classic` (REQUIRED — bare port, 1:1 with moreutils `ts`)

```toml
ts-classic = ["cli"]
```

- Includes `cli` so the binary exists.
- Single-capability port; the `cli` umbrella IS the bare-port surface.
- Use case: `cargo install rusty-ts --no-default-features --features ts-classic`
  for a moreutils-`ts` drop-in replacement (Strict mode is invoked via
  `--strict` flag, `RUSTY_TS_STRICT` env var, or `ts-alias` binary name
  per FR-021..023 — none of these require additional features).

### `ts-minimal`

```toml
ts-minimal = ["cli"]
```

- Same composition as `ts-classic` (single-capability port has no
  smaller subset to carve).
- Use case: explicit "minimal CLI install" alias for users who prefer
  the `-minimal` naming convention seen across other Rusty ports
  (figlet-minimal, pwgen-minimal). Documented as an intentional
  semantic alias rather than a distinct composition.

### `ts-alias` (v0.1.x feature retained, NOT a convention preset)

`ts-alias = ["cli"]` from v0.1.0 ships an additional `ts` binary
alongside `rusty-ts`. It is retained verbatim per the v0.2.0 SemVer
additive contract — but it is NOT one of the 2 required preset
bundles per FR-007 (those are `ts-classic` and `ts-minimal` above).
`ts-alias` is documented separately as an installation-time
convenience knob, not a capability subset.

## Cross-port glossary candidates

No leaves carved → no cross-port glossary contributions from rusty-ts
in this iteration. If a future minor release adds an orthogonal
capability (e.g., a `journald-export` feature), the leaf would be a
candidate for promotion to `docs/feature-vocabulary.md` per FR-053.

## CI matrix shape (FR-010..FR-014)

Per plan §Per-Port v0.2.0 CI Matrix, scaled to a zero-leaf port:

- **Tier 1 — `test-default`**: full DDR-003 cross-compile matrix
  (5 targets). Now equivalent to `--features full` (which is `cli`
  in this port).
- **Tier 2 — `test-no-default`**: Linux x86_64 only. `cargo test
  --no-default-features --lib` + dep-tree audit (SC-001 evidence).
- **Tier 3 — `test-<bundle>`**: one job per preset bundle. Linux only.
  - `test-ts-classic`
  - `test-ts-minimal`
- **Tier 4 — `check-leaf-<leaf>`**: SKIPPED. Zero leaves → no
  per-leaf compile-check jobs. A placeholder comment in `ci.yml`
  documents why this tier is empty.
- **Tier 5 — `lint-convention`**: single Linux job invoking the
  vendored `tools/feature-lint/run.sh` script.

Per FR-014, bundle/lint jobs are constrained to Linux x86_64.

## Vendored feature-lint

Per spec 00011 §Phase 2 iteration 6 precedent (rusty-figlet vendored
the lint script because the umbrella `jsh562/rustylib` is private and
cross-repo `actions/checkout` cannot reach it), rusty-ts vendors
`tools/feature-lint/{lint.sh,run.sh,README.md,.shellcheck}` from the
umbrella into the port repo. The vendored copy is byte-equal to the
umbrella source of truth as of the freeze commit.

## Why no leaves — explicit rationale

Spec 00011 §Scope Edge Cases anticipates this case verbatim:

> Some ports have only one orthogonal capability (e.g., rusty-ts
> only "prefix lines with timestamps"). Those ports adopt the minimum
> convention: `full = ["cli"]` and `<port>-classic = ["cli"]` as
> aliases; the convention SHAPE is consistent across the portfolio
> even when the per-port leaf carving yields zero leaves.

rusty-ts deliberately chooses the zero-leaf path because:

1. The cost of carving a speculative leaf (cfg-gate scaffolding,
   per-leaf CI matrix entry, README/CHANGELOG row, glossary
   candidacy) outweighs the value when no orthogonal capability
   exists to gate.
2. The portfolio-wide convention shape (umbrella set, README
   "Cargo Features" section, CHANGELOG migration table, lint
   compliance) is preserved verbatim — a downstream library
   consumer reading the README for rusty-ts gets the same one-glance
   feature matrix UX as one reading rusty-figlet.
3. Future minor releases can add leaves without breaking the v0.2.0
   contract: a hypothetical `journald-export` v0.3.0 feature would
   slot in as `journald-export = ["dep:journald"]` alongside the
   existing umbrellas with zero migration cost.
