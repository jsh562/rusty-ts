# rusty-ts — Design Overview

This is the inaugural port in the [Rusty portfolio](https://github.com/jsh562/rustylib). The canonical spec, plan, research, and task list live in the umbrella repo under `specs/00001-inaugural-port-ts/`.

- [`spec.md`](../../rusty/specs/00001-inaugural-port-ts/spec.md) — product spec, 10 user stories, 30 functional requirements, 23 success criteria
- [`plan.md`](../../rusty/specs/00001-inaugural-port-ts/plan.md) — implementation plan, architecture decisions, Performance Budget, Testing Strategy
- [`research.md`](../../rusty/specs/00001-inaugural-port-ts/research.md) — gap survey, tech selections, snapshot capture protocol
- [`tasks.md`](../../rusty/specs/00001-inaugural-port-ts/tasks.md) — 130-task work breakdown

## Upstream Dependency Status

**T001 decision: pragmatic path.** The umbrella reusable workflow (project-plan epic E003) and the cargo-generate template (project-plan epic E004) do not yet exist at the time this port begins. Rather than block the inaugural port on those upstream epics, this repo scaffolds manually following [`plan.md §Project Structure`](../../rusty/specs/00001-inaugural-port-ts/plan.md) with inline CI workflows under `.github/workflows/`.

### Inline → reusable migration path

When E003 v1.0.0 ships:

1. The umbrella publishes `.github/workflows/port-ci.yml@v1.0.0` containing the gates currently inlined in this repo's `.github/workflows/ci.yml`: rustfmt, Clippy strict, cargo audit, full DDR-003 cross-compile matrix, MSRV, snapshot suite invocation, public-api drift, completions drift, compat-matrix drift, perf-gate.
2. The two CI files in this repo become thin callers:
   ```yaml
   jobs:
     port-ci:
       uses: jsh562/rustylib/.github/workflows/port-ci.yml@v1.0.0
       secrets: inherit
   ```
3. Any per-port-specific overrides (e.g., extra release assets like completion files) pass through as `workflow_call` inputs.
4. The inline workflows are removed from this repo in the same PR that adds the thin callers.

When E004 v1.0.0 ships:

- The cargo-generate template at `templates/port/` in the umbrella codifies this repo's structure. Future ports (`sponge`, `pv`, etc.) scaffold from the template; `rusty-ts` does not change but its structure becomes the de facto template baseline.

## Repository Status

**T002 decision: deferred to maintainer.** This local working tree is a fresh directory; the GitHub repository named `rusty-ts` has not been created yet. Before any external publish:

1. Create an empty GitHub repository at `https://github.com/jsh562/rustylib-ts`.
2. Update the `repository`, `homepage`, and `documentation` URLs in `Cargo.toml` (currently set to `jsh562`).
3. Update the umbrella references in this file (`docs/DESIGN.md`) and in `README.md`.
4. From this directory: `git init && git add . && git commit -m "initial scaffold" && git branch -M main && git remote add origin <repo-url> && git push -u origin main`.
5. Enable branch protection on `main`: require PRs, require status checks (`fmt`, `clippy`, `audit`, `msrv`, `test`, `library-no-default-features`, `publish-dry-run`).
6. Add repository secrets: `CARGO_REGISTRY_TOKEN` (publish-only scope from crates.io). 2FA must be enabled on the maintainer's GitHub and crates.io accounts.

## Crate Name Status

**T003 decision: `rusty-ts` claimed.** `cargo search rusty-ts` returned empty as of 2026-05-22 — the name is available on crates.io. No fallback (`rusty-ts-cli` was the documented fallback) is needed.

## Architecture (high-level)

See [plan.md §Architecture](../../rusty/specs/00001-inaugural-port-ts/plan.md) for the full C4 component view and the 10 architecture decisions (AD-001 through AD-010). In brief:

- **CLI Frontend** (`src/cli.rs`) — clap-derive `Cli` struct, single source of truth for flags; consumed by completions generator and Compatibility Matrix generator.
- **Mode Resolver** (`src/mode.rs`) — resolves Default vs Strict from flag / env var / argv[0] basename once at startup. Zero per-line cost.
- **Time Renderer** (`src/time/`) — chrono + chrono-tz, with a custom pre-tokenizer for the `%.S` / `%.s` fractional-second extensions moreutils ships.
- **Line Pipeline** (`src/pipeline.rs`) — stdin → format → stdout loop. Line-buffered. Non-UTF-8 byte passthrough on payload.
- **Relative Mode** (`src/relative.rs`) — regex-based recognition; Default subset (ISO-8601, RFC-3339, Unix epoch) and Strict full moreutils set.
- **Library API** (`src/lib.rs`) — `TimestamperBuilder` → `Timestamper::prefix_lines(impl BufRead) -> impl Iterator<Item = io::Result<Vec<u8>>>` byte-typed canonical surface, plus a `String`-typed convenience adapter.
- **Compatibility Matrix Generator** (`src/compat_matrix.rs`) — walks `Cli::command()` and emits `docs/COMPATIBILITY.md`. A `cargo test --test compat_matrix` integration test regenerates and asserts no drift, failing CI on mismatch.

## Snapshot capture protocol

Behavioral parity with moreutils is verified by snapshot tests. The capture protocol is **mandatory** (not advisory): pinned moreutils source commit, `TZ=UTC`, `LC_ALL=C.UTF-8`. Full protocol documented in [`fixtures/README.md`](../fixtures/README.md). The snapshot test harness (`tests/common/mod.rs`) asserts these env values are pinned before any snapshot comparison; mismatch fails the test before snapshot compare.
