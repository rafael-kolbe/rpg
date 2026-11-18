#!/usr/bin/env bash
# Fails when web/src/protocol/generated is out of date with crates/protocol.
# Regenerates the bindings into a temp dir and compares; never touches the repo.
set -euo pipefail
cd "$(dirname "$0")/.."

fresh="$(mktemp -d)"
trap 'rm -rf "$fresh"' EXIT

TS_RS_EXPORT_DIR="$fresh" cargo test --quiet -p protocol >/dev/null

if ! diff -r web/src/protocol/generated "$fresh"; then
  echo "Protocol types are stale. Run: make protocol-types" >&2
  exit 1
fi
echo "Protocol types up to date."
