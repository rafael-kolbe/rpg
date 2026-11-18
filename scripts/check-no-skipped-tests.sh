#!/usr/bin/env bash
# A test exists for current behavior; a skipped test is either dead or hiding a
# failure. Fails on #[ignore] in Rust and .skip/.only/.todo in TypeScript.
set -euo pipefail
cd "$(dirname "$0")/.."

found=0
if grep -rn --include='*.rs' -E '#\[ignore' crates; then found=1; fi
if grep -rn --include='*.ts' --include='*.tsx' -E '\b(it|test|describe)\.(skip|only|todo)\(' web/src web/scripts; then
  found=1
fi

if [ "$found" -ne 0 ]; then
  echo "Skipped or focused tests found (above). Fix or delete them." >&2
  exit 1
fi
echo "No skipped tests."
