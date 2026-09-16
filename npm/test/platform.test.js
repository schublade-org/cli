"use strict";

const assert = require("assert");
const fs = require("fs");
const path = require("path");
const {
  PLATFORMS,
  archiveName,
  binaryFileName,
  findPlatformByRustTarget,
  getPlatform,
  optionalDependencyMap,
  platformBinarySpecifier,
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
assert.strictEqual(
  archiveName("0.1.0", "x86_64-unknown-linux-gnu"),
  "schublade-v0.1.0-x86_64-unknown-linux-gnu.tar.gz",
);
assert.strictEqual(
  platformBinarySpecifier("darwin", "arm64"),
  "@schublade/cli-darwin-arm64/bin/schublade",
);
assert.strictEqual(
  platformBinarySpecifier("win32", "x64"),
  "@schublade/cli-win32-x64/bin/schublade.exe",
);
assert.strictEqual(getPlatform("linux", "x64").libc, "glibc");
assert.strictEqual(getPlatform("darwin", "arm64").libc, undefined);
assert.strictEqual(
  findPlatformByRustTarget("aarch64-apple-darwin").packageName,
  "@schublade/cli-darwin-arm64",
);
assert.strictEqual(findPlatformByRustTarget("wasm32-unknown-unknown"), null);

const rootPkg = JSON.parse(
  fs.readFileSync(path.resolve(__dirname, "..", "..", "package.json"), "utf8"),
);
assert.deepStrictEqual(rootPkg.optionalDependencies, optionalDependencyMap(rootPkg.version));
assert.ok(!rootPkg.scripts.postinstall, "root package must not download binaries in postinstall");
assert.ok(!rootPkg.scripts.preinstall);
assert.ok(!rootPkg.files.includes("npm/install.js"));

let threw = false;
try {
  rustTarget("freebsd", "x64");
} catch (error) {
  threw = true;
  assert.match(error.message, /freebsd-x64/);
}
assert.ok(threw, "unsupported platform should throw");

const names = Object.values(PLATFORMS).map((info) => info.packageName);
assert.deepStrictEqual(
  names,
  [
    "@schublade/cli-darwin-arm64",
    "@schublade/cli-darwin-x64",
    "@schublade/cli-linux-arm64",
    "@schublade/cli-linux-x64",
    "@schublade/cli-win32-arm64",
    "@schublade/cli-win32-x64",
  ],
);

console.log("platform tests passed");
