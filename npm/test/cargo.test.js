"use strict";

const assert = require("assert");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { binaryFileName } = require("../lib/platform");
const {
  cargoBinaryCandidates,
  cargoTomlPath,
  existingCargoBinary,
  isSourceCheckout,
} = require("../lib/cargo");

assert.ok(isSourceCheckout());
assert.ok(fs.existsSync(cargoTomlPath()));

const name = binaryFileName();
const candidates = cargoBinaryCandidates();
assert.ok(candidates.some((candidate) => candidate.endsWith(`${path.sep}release${path.sep}${name}`)));
assert.ok(candidates.some((candidate) => candidate.endsWith(`${path.sep}debug${path.sep}${name}`)));

const emptyRoot = fs.mkdtempSync(path.join(os.tmpdir(), "schublade-cargo-"));
assert.strictEqual(isSourceCheckout(emptyRoot), false);
assert.strictEqual(existingCargoBinary(emptyRoot), null);
fs.writeFileSync(path.join(emptyRoot, "Cargo.toml"), "[package]\nname = \"other\"\n");
assert.strictEqual(isSourceCheckout(emptyRoot), false);
fs.rmSync(emptyRoot, { recursive: true, force: true });

console.log("cargo tests passed");
