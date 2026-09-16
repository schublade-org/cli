"use strict";

const assert = require("assert");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { envBinaryPath, resolveBinary, vendorBinaryPath } = require("../lib/resolve");

assert.ok(vendorBinaryPath().endsWith("schublade") || vendorBinaryPath().endsWith("schublade.exe"));

const previous = process.env.SCHUBLADE_BINARY;
try {
  delete process.env.SCHUBLADE_BINARY;
  assert.strictEqual(envBinaryPath(), null);

  const tmp = path.join(os.tmpdir(), `schublade-resolve-test-${process.pid}`);
  fs.writeFileSync(tmp, "ok");
  process.env.SCHUBLADE_BINARY = tmp;
  assert.strictEqual(envBinaryPath(), tmp);
  assert.strictEqual(resolveBinary(), tmp);
  fs.unlinkSync(tmp);

  process.env.SCHUBLADE_BINARY = path.join(os.tmpdir(), "schublade-missing-binary");
  assert.strictEqual(envBinaryPath(), null);
} finally {
  if (previous === undefined) {
    delete process.env.SCHUBLADE_BINARY;
  } else {
    process.env.SCHUBLADE_BINARY = previous;
  }
}

console.log("resolve tests passed");
