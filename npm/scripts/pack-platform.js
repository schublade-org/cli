#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const { binaryFileName, findPlatformByRustTarget, getPlatform } = require("../lib/platform");

function parseArgs(argv) {
  const args = { repoRoot: path.resolve(__dirname, "..", "..") };
  for (let i = 0; i < argv.length; i += 1) {
    const key = argv[i];
    const value = argv[i + 1];
    switch (key) {
      case "--target":
        args.target = value;
        i += 1;
        break;
      case "--bin":
        args.bin = value;
        i += 1;
        break;
      case "--out":
        args.out = value;
        i += 1;
        break;
      case "--version":
        args.version = value;
        i += 1;
        break;
      case "--repo-root":
        args.repoRoot = path.resolve(value);
        i += 1;
        break;
      default:
        throw new Error(`unknown argument: ${key}`);
    }
  }
  return args;
}

function packageVersion(repoRoot) {
  return JSON.parse(fs.readFileSync(path.join(repoRoot, "package.json"), "utf8")).version;
}

function renderPlatformPackageJson(platform, version) {
  const pkg = {
    name: platform.packageName,
    version,
    description: `Prebuilt schublade CLI for ${platform.npmOs}-${platform.npmCpu}`,
    license: "MIT",
    os: [platform.npmOs],
    cpu: [platform.npmCpu],
    engines: { node: ">=18" },
    files: ["bin"],
    repository: {
      type: "git",
      url: "git+https://github.com/schublade-org/cli.git",
    },
    homepage: "https://github.com/schublade-org/cli#readme",
    publishConfig: { access: "public" },
  };
  if (platform.libc) {
    pkg.libc = [platform.libc];
  }
  return pkg;
}

function packPlatform({ target, bin, out, version, repoRoot }) {
  if (!bin) {
    throw new Error("--bin is required");
  }
  if (!out) {
    throw new Error("--out is required");
  }
  if (!fs.existsSync(bin)) {
    throw new Error(`binary not found: ${bin}`);
  }

  const platform = target ? findPlatformByRustTarget(target) : getPlatform();
  if (!platform) {
    throw new Error(`unknown rust target: ${target}`);
  }

  const resolvedVersion = version || packageVersion(repoRoot);
  const destDir = path.resolve(out);
  const binDir = path.join(destDir, "bin");
  fs.mkdirSync(binDir, { recursive: true });
  const destBin = path.join(binDir, binaryFileName(platform.npmOs));
  fs.copyFileSync(bin, destBin);
  fs.chmodSync(destBin, 0o755);
  fs.writeFileSync(
    path.join(destDir, "package.json"),
    `${JSON.stringify(renderPlatformPackageJson(platform, resolvedVersion), null, 2)}\n`,
  );
  return { destDir, packageName: platform.packageName, destBin, version: resolvedVersion };
}

function main() {
  const packed = packPlatform(parseArgs(process.argv.slice(2)));
  console.log(`packed ${packed.packageName}@${packed.version} -> ${packed.destDir}`);
}

if (require.main === module) {
  try {
    main();
  } catch (error) {
    console.error(`pack-platform: ${error.message}`);
    process.exit(1);
  }
}

module.exports = {
  packPlatform,
  parseArgs,
  renderPlatformPackageJson,
};
