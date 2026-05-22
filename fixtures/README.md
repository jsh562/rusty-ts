# Snapshot Fixture Capture Protocol

Behavioral parity with moreutils `ts` is verified by snapshot tests that compare the port's output to **captured moreutils-`ts` output** for the same inputs. To make those captures reproducible across machines and CI runners, every fixture is captured under a **pinned environment**. This is a hard requirement, not a recommendation.

## Pinned environment

| Variable | Value | Rationale |
|---|---|---|
| moreutils source | commit `TBD-RECORDED-AT-FIRST-CAPTURE` | Pinned so upstream changes are deliberate review events, not silent fixture drift |
| `TZ` | `UTC` | Predictable, no DST, supported byte-identically by chrono-tz on every DDR-003 target |
| `LC_ALL` | `C.UTF-8` | Stops non-ASCII strftime tokens (e.g., `%b` month abbreviation) from drifting across runner OSes |

When `C.UTF-8` is unavailable (some Windows runners), the harness falls back to `LANG=C` / `LC_ALL=C` (ASCII-only behavior) and the matrix asserts no fixture in the matrix exercises a non-ASCII format token.

## Capture procedure

1. Build moreutils at the pinned commit on a reference Linux distro:
   ```sh
   git clone https://git.joeyh.name/git/moreutils.git
   cd moreutils
   git checkout <PINNED-COMMIT>
   make ts
   ```

2. Run the capture scripts under `fixtures/scripts/`:
   ```sh
   TZ=UTC LC_ALL=C.UTF-8 ./fixtures/scripts/capture-default.sh
   TZ=UTC LC_ALL=C.UTF-8 ./fixtures/scripts/capture-custom-format.sh
   TZ=UTC LC_ALL=C.UTF-8 ./fixtures/scripts/capture-elapsed.sh
   TZ=UTC LC_ALL=C.UTF-8 ./fixtures/scripts/capture-tz.sh
   TZ=UTC LC_ALL=C.UTF-8 ./fixtures/scripts/capture-fractional.sh
   TZ=UTC LC_ALL=C.UTF-8 ./fixtures/scripts/capture-relative.sh
   TZ=UTC LC_ALL=C.UTF-8 ./fixtures/scripts/capture-strict-errors.sh
   ```

3. Review the resulting binary diffs in `fixtures/moreutils_outputs/` as part of the PR adding or updating fixtures.

4. Capture-script invocations assert env-match at start (TZ, LC_ALL) and refuse to run otherwise. The same assertion is made by the test harness at replay time — see `tests/common/mod.rs::assert_pinned_env`.

## Refresh protocol (when moreutils upstream changes)

1. Update the pinned commit in this file.
2. Re-run all capture scripts under the pinned env on the reference distro.
3. Review the resulting binary diff in PR. The diff IS the upstream-change review.
4. Update any divergence notes in the README compatibility statement.
5. Refreshes are reviewed as a discrete commit, never bundled with unrelated source changes.

## Directory layout

```
fixtures/
├── README.md                         # this file
├── scripts/                          # capture scripts (assert pinned env)
├── inputs/                           # captured input streams (text + binary)
│   ├── default/
│   ├── custom_format/
│   ├── elapsed/
│   ├── tz_utc/  tz_tokyo/  tz_dst/
│   ├── fractional/
│   ├── relative_default/  relative_strict/
│   └── binary_passthrough.bin
├── moreutils_outputs/                # captured moreutils ts output (byte-diff target)
│   └── (mirrors inputs/)
└── perf/                             # criterion bench fixtures (separate regen process)
```

## Status

This file is the protocol definition; the capture scripts (`fixtures/scripts/`) and the fixture corpus (`fixtures/inputs/`, `fixtures/moreutils_outputs/`) are populated during T024 / T025 / T032 / T041 / T060 / T075 / T082 / T087 etc. per `specs/00001-inaugural-port-ts/tasks.md`. At the time this file is committed, fixtures may still be empty pending the per-story capture tasks; snapshot tests will skip with a clear "fixture not yet captured" message until populated.
