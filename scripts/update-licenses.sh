#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXPECTED_CARGO_ABOUT_VERSION="cargo-about 0.9.2"

if ! command -v cargo-about >/dev/null 2>&1; then
  echo "cargo-about 0.9.2 is required. Install it with:" >&2
  echo "  cargo install cargo-about --version 0.9.2 --locked --features cli" >&2
  exit 1
fi

actual_version="$(cargo about --version)"
if [[ "$actual_version" != "$EXPECTED_CARGO_ABOUT_VERSION" ]]; then
  echo "Expected $EXPECTED_CARGO_ABOUT_VERSION, found $actual_version." >&2
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

cargo about generate \
  --format json \
  --manifest-path "$ROOT/Cargo.toml" \
  --config "$ROOT/about.toml" \
  --workspace \
  --all-features \
  --locked \
  --fail \
  > "$tmp_dir/licenses.json"

cargo run --quiet --manifest-path "$ROOT/Cargo.toml" --bin render_licenses -- \
  --input "$tmp_dir/licenses.json" \
  --markdown "$tmp_dir/THIRD_PARTY_LICENSES.md" \
  --html "$tmp_dir/third_party_licenses.html"

if [[ "${1:-}" == "--check" ]]; then
  diff -u "$ROOT/THIRD_PARTY_LICENSES.md" "$tmp_dir/THIRD_PARTY_LICENSES.md"
  diff -u "$ROOT/frontend/third_party_licenses.html" "$tmp_dir/third_party_licenses.html"
else
  cp "$tmp_dir/THIRD_PARTY_LICENSES.md" "$ROOT/THIRD_PARTY_LICENSES.md"
  cp "$tmp_dir/third_party_licenses.html" "$ROOT/frontend/third_party_licenses.html"
fi

