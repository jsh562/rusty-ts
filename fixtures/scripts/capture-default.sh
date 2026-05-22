#!/usr/bin/env bash
# Capture moreutils `ts` default-format output for the rusty-ts byte-equal
# snapshot suite. Per T025 / fixtures/README.md capture protocol.
#
# Requirements:
#   - moreutils `ts` script in $MOREUTILS_TS (defaults to ./ts).
#     Fetch from: https://raw.githubusercontent.com/madx/moreutils/master/ts
#   - bash, sed, head, date
#
# Usage:
#   MOREUTILS_TS=/path/to/ts ./capture-default.sh [output-dir]
#
# Default output dir: ../moreutils_outputs/default/
#
# This script ASSERTS the environment is pinned per the capture protocol:
#   TZ=UTC, LC_ALL=C.UTF-8
# If those are not set, the script aborts before touching any fixture file.

set -euo pipefail

if [[ "${TZ:-}" != "UTC" ]]; then
    echo "fatal: TZ must be set to UTC (got ${TZ:-unset})" >&2
    exit 2
fi
if [[ "${LC_ALL:-}" != "C.UTF-8" && "${LC_ALL:-}" != "C" ]]; then
    echo "fatal: LC_ALL must be set to C.UTF-8 (or C) (got ${LC_ALL:-unset})" >&2
    exit 2
fi

MOREUTILS_TS="${MOREUTILS_TS:-./ts}"
if [[ ! -x "$MOREUTILS_TS" ]]; then
    echo "fatal: moreutils ts not executable at $MOREUTILS_TS" >&2
    exit 2
fi

OUTPUT_DIR="${1:-../moreutils_outputs/default}"
INPUT_DIR="${2:-../inputs/default}"
mkdir -p "$OUTPUT_DIR" "$INPUT_DIR"

capture_one() {
    local name="$1"
    local payload="$2"
    printf '%b' "$payload" > "$INPUT_DIR/${name}.txt"
    "$MOREUTILS_TS" < "$INPUT_DIR/${name}.txt" > "$OUTPUT_DIR/${name}.txt"
    # First 15 chars of first line = "%b %d %H:%M:%S" output (15 chars).
    local stamp
    stamp=$(head -c 15 "$OUTPUT_DIR/${name}.txt" || true)
    local iso
    iso=$(date -u -d "${stamp} $(date -u +%Y)" +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || echo "$(date -u +%Y-%m-%dT%H:%M:%SZ)")
    echo "captured $name: moreutils_ts_prefix=${stamp} iso=${iso}"
}

capture_one three_lines 'first\nsecond\nthird\n'
capture_one single_line 'hello\n'
# Empty input: capture without timestamp metadata.
: > "$INPUT_DIR/empty.txt"
"$MOREUTILS_TS" < "$INPUT_DIR/empty.txt" > "$OUTPUT_DIR/empty.txt"
echo "captured empty: 0-byte output"

echo "---"
echo "Update fixtures/moreutils_outputs/default/CAPTURE.json with the iso values printed above."
