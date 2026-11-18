#!/usr/bin/env bash
# Every public function in crates/utils must be documented with an
# `# Examples` section (run as a doctest by `cargo test`). See CLAUDE.md, "Utils".
set -euo pipefail
cd "$(dirname "$0")/.."

missing="$(awk '
  /^[[:space:]]*\/\/\// { doc = doc $0 "\n"; next }
  /^[[:space:]]*#\[/    { next }
  /^[[:space:]]*pub (const )?fn / {
    if (doc !~ /# Examples/) { print FILENAME ":" FNR ": " $0 }
    doc = ""; next
  }
  { doc = "" }
' crates/utils/src/*.rs)"

if [ -n "$missing" ]; then
  echo "Utils functions without an \`# Examples\` doc section:" >&2
  echo "$missing" >&2
  exit 1
fi
echo "Rust utils documented."
