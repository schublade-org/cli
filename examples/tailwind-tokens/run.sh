#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

# Production-like path (standalone mirrors: schublade-org/examples-<name>):
#   npx schublade serve --config ./schublade.toml
# Monorepo checkout: ../../Cargo.toml is the CLI crate — use cargo so local changes apply.
if [[ -f ../../Cargo.toml ]] && grep -q "^name = \"schublade\"" ../../Cargo.toml; then
  exec cargo run --manifest-path ../../Cargo.toml -- serve --config ./schublade.toml "$@"
fi

if ! command -v npx >/dev/null 2>&1; then
  echo "schublade: install Node.js 18+ and run: npx schublade serve --config ./schublade.toml" >&2
  exit 1
fi
exec npx --yes schublade serve --config ./schublade.toml "$@"
