#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
ROOT="$(cd ../.. && pwd)"
exec cargo run --manifest-path "$ROOT/Cargo.toml" -- serve --config ./schublade.toml "$@"
