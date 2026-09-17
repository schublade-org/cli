"use strict";

const assert = require("assert");
const fs = require("fs");
const os = require("os");
const path = require("path");
const {
  packPlatform,
  parseArgs,
  renderPlatformPackageJson,
} = require("../scripts/pack-platform");
const { findPlatformByRustTarget } = require("../lib/platform");

const linux = findPlatformByRustTarget("x86_64-unknown-linux-gnu");
const darwin = findPlatformByRustTarget("aarch64-apple-darwin");
const win = findPlatformByRustTarget("x86_64-pc-windows-msvc");

const linuxPkg = renderPlatformPackageJson(linux, "0.1.0");
assert.strictEqual(linuxPkg.name, "@schublade/cli-linux-x64");
assert.deepStrictEqual(linuxPkg.os, ["linux"]);
assert.deepStrictEqual(linuxPkg.cpu, ["x64"]);
assert.deepStrictEqual(linuxPkg.libc, ["glibc"]);
assert.deepStrictEqual(linuxPkg.files, ["bin"]);
assert.strictEqual(
  linuxPkg.repository.url,
  "git+https://github.com/schublade-org/cli.git",
);

const darwinPkg = renderPlatformPackageJson(darwin, "0.1.0");
assert.strictEqual(darwinPkg.name, "@schublade/cli-darwin-arm64");
assert.strictEqual(darwinPkg.libc, undefined);

const winPkg = renderPlatformPackageJson(win, "0.1.0");
assert.strictEqual(winPkg.name, "@schublade/cli-win32-x64");
assert.deepStrictEqual(winPkg.os, ["win32"]);

const parsed = parseArgs([
  "--target",
  "x86_64-unknown-linux-gnu",
  "--bin",
  "/tmp/schublade",
  "--out",
  "/tmp/out",
  "--version",
  "1.2.3",
]);
assert.strictEqual(parsed.target, "x86_64-unknown-linux-gnu");
assert.strictEqual(parsed.version, "1.2.3");

const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "schublade-pack-"));
const fakeBin = path.join(tmp, "schublade");
fs.writeFileSync(fakeBin, "#!/bin/sh\necho ok\n", { mode: 0o755 });
const out = path.join(tmp, "pkg");
const packed = packPlatform({
  target: "x86_64-unknown-linux-gnu",
  bin: fakeBin,
  out,
  version: "0.1.0",
  repoRoot: path.resolve(__dirname, "..", ".."),
});
assert.strictEqual(packed.packageName, "@schublade/cli-linux-x64");
assert.ok(fs.existsSync(path.join(out, "bin", "schublade")));
const written = JSON.parse(fs.readFileSync(path.join(out, "package.json"), "utf8"));
assert.strictEqual(written.name, "@schublade/cli-linux-x64");
assert.strictEqual(written.version, "0.1.0");
assert.deepStrictEqual(written.libc, ["glibc"]);
fs.rmSync(tmp, { recursive: true, force: true });

console.log("pack-platform tests passed");
