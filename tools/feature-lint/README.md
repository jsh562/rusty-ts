# feature-lint (vendored)

Portfolio-wide convention compliance lint for per-port Cargo features.

This is a **vendored copy** of the umbrella `tools/feature-lint/` script
suite. The umbrella governance repo (`jsh562/rustylib`) is private, so
cross-repo `actions/checkout` cannot reach it from per-port CI workflows
running in public per-port repos. Per the precedent set by rusty-figlet
v0.2.0 (E011 Phase 2 iteration 6), each public per-port repo carries a
synchronized copy of the lint script.

This vendored copy is byte-equal to the umbrella source of truth as of
the v0.2.0 freeze commit. Re-vendor on subsequent umbrella amendments
by `cp c:\claudecode\rusty\tools\feature-lint\{lint.sh,run.sh,.shellcheck}`
into this directory.

## v0.2.0 surface

rusty-ts is a **single-capability port** per spec 00011 §Scope Edge
Cases. The minimum convention applies — `full = ["cli"]`, `ts-classic
= ["cli"]`, `ts-minimal = ["cli"]` are the required umbrellas and
preset bundles. **Zero leaves** are carved beyond the required
umbrellas, so the lint's leaf-CI-matrix, phantom-leaf, and
README/CHANGELOG leaf-row checks are vacuously satisfied (nothing to
check, nothing to fail).

## Authority

- `specs/adrs/0006-cargo-features-convention-for-portfolio-ports.md`
- `project-instructions.md` §Cargo Feature Surface (v1.1.0+)

## Files

- `lint.sh` — POSIX bash script implementing all 5 lint sub-rules
  (T003..T007 of spec 00011).
- `run.sh` — top-level runner that invokes every sub-check in sequence,
  accumulates violations, and prints a final summary.
- `.shellcheck` — shellcheck configuration for both scripts.
- `README.md` — this file.

## Invocation Contract

| Variable | Required? | Meaning |
|---|---|---|
| `PORT_PATH` | Yes | Absolute path to the per-port repo root (the directory containing the port's `Cargo.toml`). |
| `UMBRELLA_PATH` | Yes | Absolute path to the umbrella governance repo root. For vendored invocations from inside the port repo, point this at the same dir as `PORT_PATH`. |
| `STRICT_MODE` | No (default: `1`) | When `1`, every violation is fatal. When `0`, violations are reported but the script exits 0. |

Example local invocation (from the port root, using the vendored copy):

```bash
UMBRELLA_PATH=. PORT_PATH=. bash tools/feature-lint/run.sh
```

CI invocation (see `.github/workflows/ci.yml` `lint-convention` job):

```yaml
- name: Run feature-lint
  env:
    UMBRELLA_PATH: ${{ github.workspace }}
    PORT_PATH: ${{ github.workspace }}
  run: bash tools/feature-lint/run.sh
```

## Exit Codes

| Exit code | Meaning |
|---|---|
| 0 | Compliance — all sub-checks passed. |
| 2 | At least one violation — the violated rule and offending file are named on stderr. |

## Sub-Checks (per FR-052 of spec 00011)

1. **Required umbrellas present** (T003) — `Cargo.toml` `[features]` MUST
   declare `default`, `full`, `cli`, and `<port>-classic`.
2. **Leaf has CI matrix entry** (T004) — every declared leaf MUST have a
   `check-leaf-<leaf>` job in `.github/workflows/ci.yml`. (Vacuous for
   rusty-ts — zero leaves.)
3. **No phantom leaves** (T005) — every declared leaf MUST be referenced by
   at least one `#[cfg(feature = "<leaf>")]` in the port's `src/` tree.
   (Vacuous for rusty-ts — zero leaves.)
4. **README feature-matrix sync** (T006) — the README's `## Cargo Features`
   matrix MUST list every leaf with the canonical column order.
5. **CHANGELOG migration-table exhaustiveness** (T007) — the CHANGELOG's
   `## [0.2.0]` `### BREAKING-CHANGE` migration table MUST list every
   v0.1.x feature name with the canonical column order.
