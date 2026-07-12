#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

RUST_OUT="coverage/rust/test-output.txt"
RUST_SUMMARY="coverage/rust/summary.txt"
WEB_OUT="coverage/web/test-output.txt"
REPORT="coverage/REPORT.md"
GENERATED_AT="$(date '+%Y-%m-%d %H:%M %Z')"

if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
  echo "error: cargo-llvm-cov is required. Install with: cargo install cargo-llvm-cov" >&2
  exit 1
fi

mkdir -p coverage/rust coverage/web

echo "==> Rust tests (cargo llvm-cov)"
cargo llvm-cov --workspace --html --output-dir coverage/rust 2>&1 | tee "$RUST_OUT"
cargo llvm-cov --workspace --summary-only 2>&1 | tee "$RUST_SUMMARY"
cargo llvm-cov --workspace --lcov --output-path coverage/rust/lcov.info >/dev/null

echo "==> Web tests (bun test --coverage)"
(
  cd apps/web
  bun test --coverage --coverage-reporter=text --coverage-reporter=lcov --preload ./src/test/setup.ts 2>&1
) | tee "$WEB_OUT"
cp apps/web/coverage/lcov.info coverage/web/lcov.info

rust_totals="$(awk '/^TOTAL / { print $0 }' "$RUST_SUMMARY" | tail -1)"
web_totals="$(awk '/^All files/ { print $0 }' "$WEB_OUT" | tail -1)"

rust_line_cov="$(echo "$rust_totals" | awk '{ print $10 }')"
rust_func_cov="$(echo "$rust_totals" | awk '{ print $7 }')"
web_func_cov="$(echo "$web_totals" | awk -F'|' '{ gsub(/^ +| +$/, "", $2); print $2 }')%"
web_line_cov="$(echo "$web_totals" | awk -F'|' '{ gsub(/^ +| +$/, "", $3); print $3 }')%"

rust_passed="$(grep -E '^test result: ok\.' "$RUST_OUT" | awk '{ s+=$4 } END { print s+0 }')"
web_passed="$(awk '/^ [0-9]+ pass$/ { print $1 }' "$WEB_OUT" | tail -1)"
web_failed="$(awk '/^ [0-9]+ fail$/ { print $1 }' "$WEB_OUT" | tail -1)"
web_failed="${web_failed:-0}"

cat >"$REPORT" <<EOF
# Ozzy Test Coverage Report

**Generated:** ${GENERATED_AT}

## Executive Summary

| Suite | Passed | Failed | Line Coverage | Function Coverage |
|-------|-------:|-------:|--------------:|------------------:|
| Rust workspace | ${rust_passed} | 0 | **${rust_line_cov}** | **${rust_func_cov}** |
| Web client | ${web_passed} | ${web_failed} | **${web_line_cov}** | **${web_func_cov}** |
| Node SDK | — | — | No tests | — |

## Artifacts

| Artifact | Path |
|----------|------|
| This report | \`coverage/REPORT.md\` |
| Rust HTML report | \`coverage/rust/html/index.html\` |
| Rust LCOV | \`coverage/rust/lcov.info\` |
| Web LCOV | \`coverage/web/lcov.info\` |
| Rust raw output | \`coverage/rust/test-output.txt\` |
| Web raw output | \`coverage/web/test-output.txt\` |

Open the interactive Rust report:

\`\`\`bash
open coverage/rust/html/index.html
\`\`\`

## Rust Coverage by File

\`\`\`
$(grep -E '^(ozzy-|TOTAL)' "$RUST_SUMMARY" || true)
\`\`\`

## Web Coverage by File

\`\`\`
$(awk '/^File |^All files|^ src\// { print }' "$WEB_OUT" || true)
\`\`\`

## Reproduce

\`\`\`bash
bun run test:coverage
\`\`\`
EOF

echo ""
echo "Coverage report written to $REPORT"
echo "Rust HTML report: coverage/rust/html/index.html"

if [[ "$web_failed" != "0" ]]; then
  exit 1
fi
