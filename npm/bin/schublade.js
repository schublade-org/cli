#!/usr/bin/env node
"use strict";

const { spawn } = require("child_process");
const { resolveBinary } = require("../lib/resolve");
const { ensureBinary } = require("../lib/download");

async function main() {
  let binary = resolveBinary();
  if (!binary) {
    binary = await ensureBinary();
  }
  const child = spawn(binary, process.argv.slice(2), { stdio: "inherit" });
  child.on("exit", (code, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
      return;
    }
    process.exit(code ?? 1);
  });
  child.on("error", (error) => {
    console.error(`schublade: failed to start ${binary}: ${error.message}`);
    process.exit(1);
  });
}

main().catch((error) => {
  console.error(`schublade: ${error.message}`);
  process.exit(1);
});
