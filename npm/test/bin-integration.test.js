"use strict";

const assert = require("assert");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { spawnSync } = require("child_process");
const { getPlatform } = require("../lib/platform");
const { packPlatform } = require("../scripts/pack-platform");

const root = path.resolve(__dirname, "..", "..");
const platform = getPlatform();
const pkgDir = path.join(root, "node_modules", ...platform.packageName.split("/"));
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "schublade-bin-"));
const fakeSrc = path.join(tmp, "fake-schublade");
fs.writeFileSync(fakeSrc, "#!/bin/sh\nprintf '%s\\n' \"$*\"\n", { mode: 0o755 });

const previous = process.env.SCHUBLADE_BINARY;
delete process.env.SCHUBLADE_BINARY;

try {
  packPlatform({
    target: platform.rustTarget,
    bin: fakeSrc,
    out: pkgDir,
    version: "0.1.0",
    repoRoot: root,
  });

  const result = spawnSync(
    process.execPath,
    [path.join(root, "npm", "bin", "schublade.js"), "serve", "--port", "47291"],
    { encoding: "utf8", cwd: root },
  );
  assert.strictEqual(result.status, 0, result.stderr);
  assert.strictEqual(result.stdout.trim(), "serve --port 47291");
} finally {
  if (previous === undefined) {
    delete process.env.SCHUBLADE_BINARY;
  } else {
    process.env.SCHUBLADE_BINARY = previous;
  }
  fs.rmSync(pkgDir, { recursive: true, force: true });
  fs.rmSync(tmp, { recursive: true, force: true });
  const scopeDir = path.join(root, "node_modules", "@schublade");
  if (fs.existsSync(scopeDir) && fs.readdirSync(scopeDir).length === 0) {
    fs.rmSync(scopeDir, { recursive: true, force: true });
  }
  const modulesDir = path.join(root, "node_modules");
  if (fs.existsSync(modulesDir) && fs.readdirSync(modulesDir).length === 0) {
    fs.rmSync(modulesDir, { recursive: true, force: true });
  }
}

console.log("bin-integration tests passed");
