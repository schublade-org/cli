#!/usr/bin/env node
"use strict";

const { spawnSync } = require("child_process");
const { resolveBinary } = require("../lib/resolve");
const { ensureCargoBinary } = require("../lib/cargo");

function fail(message) {
  console.error(`schublade: ${message}`);
  process.exit(1);
}

function main() {
  let binary;
  try {
    binary = resolveBinary();
    if (!binary) {
      binary = ensureCargoBinary();
    }
  } catch (error) {
    fail(error.message);
  }

  if (!binary) {
    fail(
      "Couldn't find a platform package (@schublade/cli-<os>-<arch>) or a Cargo checkout to build from. " +
        "Install via npm so the matching optional dependency is present, or run from the schublade source repo with Rust installed.",
    );
  }

  const result = spawnSync(binary, process.argv.slice(2), { stdio: "inherit" });
  if (result.error) {
    fail(`failed to start ${binary}: ${result.error.message}`);
  }
  if (result.signal) {
    process.kill(process.pid, result.signal);
    return;
  }
  process.exit(result.status ?? 1);
}

main();
