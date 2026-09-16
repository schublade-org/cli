#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

if [[ -f ../../Cargo.toml ]] && grep -q "^name = \"schublade\"" ../../Cargo.toml; then
  exec cargo run --manifest-path ../../Cargo.toml -- build --config ./schublade.toml --out ./dist "$@"
fi

if ! command -v npx >/dev/null 2>&1; then
  echo "schublade: install Node.js 18+ and run: npx schublade build --config ./schublade.toml --out ./dist" >&2
  exit 1
fi
exec npx --yes schublade build --config ./schublade.toml --out ./dist "$@"
