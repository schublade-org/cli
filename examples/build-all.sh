#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
out="${1:-$root/dist/examples}"
cd "$root"

for config in "$root"/examples/*/schublade.toml; do
  name="$(basename "$(dirname "$config")")"
  dest="$out/$name"
  echo "Building $name → $dest"
  cargo run --quiet --manifest-path "$root/Cargo.toml" -- build --config "$config" --out "$dest"
done

echo "Static workshops: $out"
