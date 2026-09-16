"use strict";

const assert = require("assert");
const {
  archiveName,
  binaryFileName,
  releaseDownloadUrl,
  rustTarget,
} = require("../lib/platform");

assert.strictEqual(rustTarget("darwin", "arm64"), "aarch64-apple-darwin");
assert.strictEqual(rustTarget("darwin", "x64"), "x86_64-apple-darwin");
assert.strictEqual(rustTarget("linux", "x64"), "x86_64-unknown-linux-gnu");
assert.strictEqual(rustTarget("linux", "arm64"), "aarch64-unknown-linux-gnu");
assert.strictEqual(rustTarget("win32", "x64"), "x86_64-pc-windows-msvc");
assert.strictEqual(rustTarget("win32", "arm64"), "aarch64-pc-windows-msvc");
assert.strictEqual(binaryFileName("win32"), "schublade.exe");
assert.strictEqual(binaryFileName("linux"), "schublade");
assert.strictEqual(archiveName("0.1.0", "x86_64-unknown-linux-gnu"), "schublade-v0.1.0-x86_64-unknown-linux-gnu.tar.gz");
assert.strictEqual(
  releaseDownloadUrl("0.1.0", "x86_64-unknown-linux-gnu"),
  "https://github.com/schublade-org/schublade/releases/download/v0.1.0/schublade-v0.1.0-x86_64-unknown-linux-gnu.tar.gz",
);

let threw = false;
try {
  rustTarget("freebsd", "x64");
} catch (error) {
  threw = true;
  assert.match(error.message, /freebsd-x64/);
}
assert.ok(threw, "unsupported platform should throw");

console.log("platform tests passed");
