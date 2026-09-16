"use strict";

const fs = require("fs");
const os = require("os");
const path = require("path");
const { execFileSync } = require("child_process");
const { archiveName, binaryFileName, releaseDownloadUrl, rustTarget } = require("./platform");
const { PACKAGE_ROOT, vendorBinaryPath, vendorDir } = require("./resolve");

function packageVersion() {
  const pkg = JSON.parse(fs.readFileSync(path.join(PACKAGE_ROOT, "package.json"), "utf8"));
  return pkg.version;
}

function cargoTomlPath() {
  return path.join(PACKAGE_ROOT, "Cargo.toml");
}

async function downloadToFile(url, dest) {
  const response = await fetch(url, {
    headers: { "User-Agent": "schublade-npm-installer" },
    redirect: "follow",
  });
  if (!response.ok) {
    throw new Error(`download failed (${response.status} ${response.statusText}): ${url}`);
  }
  const buffer = Buffer.from(await response.arrayBuffer());
  fs.writeFileSync(dest, buffer);
}

function extractArchive(archivePath, destDir) {
  fs.mkdirSync(destDir, { recursive: true });
  execFileSync("tar", ["-xzf", archivePath, "-C", destDir], { stdio: "pipe" });
}

function findExtractedBinary(dir, fileName) {
  const direct = path.join(dir, fileName);
  if (fs.existsSync(direct)) {
    return direct;
  }
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  for (const entry of entries) {
    const full = path.join(dir, entry.name);
    if (entry.isFile() && entry.name === fileName) {
      return full;
    }
    if (entry.isDirectory()) {
      const nested = findExtractedBinary(full, fileName);
      if (nested) {
        return nested;
      }
    }
  }
  return null;
}

function installFromRelease(version = packageVersion()) {
  const target = rustTarget();
  const url = releaseDownloadUrl(version, target);
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "schublade-"));
  const archivePath = path.join(tmp, archiveName(version, target));
  return downloadToFile(url, archivePath).then(() => {
    extractArchive(archivePath, tmp);
    const extracted = findExtractedBinary(tmp, binaryFileName());
    if (!extracted) {
      throw new Error(`archive from ${url} did not contain ${binaryFileName()}`);
    }
    const destDir = vendorDir();
    fs.mkdirSync(destDir, { recursive: true });
    const dest = vendorBinaryPath();
    fs.copyFileSync(extracted, dest);
    fs.chmodSync(dest, 0o755);
    fs.rmSync(tmp, { recursive: true, force: true });
    return dest;
  });
}

function cargoBinaryCandidates() {
  const name = binaryFileName();
  return [
    path.join(PACKAGE_ROOT, "target", "release", name),
    path.join(PACKAGE_ROOT, "target", "debug", name),
  ];
}

function copyExistingCargoBinary() {
  for (const candidate of cargoBinaryCandidates()) {
    if (fs.existsSync(candidate)) {
      fs.mkdirSync(vendorDir(), { recursive: true });
      const dest = vendorBinaryPath();
      fs.copyFileSync(candidate, dest);
      fs.chmodSync(dest, 0o755);
      return dest;
    }
  }
  return null;
}

function buildFromSource() {
  if (!fs.existsSync(cargoTomlPath())) {
    return null;
  }
  const existing = copyExistingCargoBinary();
  if (existing) {
    return existing;
  }
  execFileSync("cargo", ["build", "--release", "--locked", "--manifest-path", cargoTomlPath()], {
    stdio: "inherit",
    cwd: PACKAGE_ROOT,
  });
  const built = copyExistingCargoBinary();
  if (!built) {
    throw new Error("cargo build --release succeeded but the schublade binary was not found under target/release");
  }
  return built;
}

async function ensureBinary() {
  const fromEnv = process.env.SCHUBLADE_BINARY;
  if (fromEnv) {
    if (!fs.existsSync(fromEnv)) {
      throw new Error(`SCHUBLADE_BINARY=${fromEnv} does not exist`);
    }
    return fromEnv;
  }

  const vendor = vendorBinaryPath();
  if (fs.existsSync(vendor)) {
    return vendor;
  }

  const skipDownload = process.env.SCHUBLADE_SKIP_DOWNLOAD === "1";
  if (!skipDownload) {
    try {
      return await installFromRelease();
    } catch (error) {
      if (!fs.existsSync(cargoTomlPath())) {
        throw new Error(
          `Could not download a prebuilt schublade binary.\n${error.message}\n\n` +
            `Publish a GitHub Release tagged v${packageVersion()} with platform archives, or install from a source checkout that has Rust.\n` +
            `See https://github.com/schublade-org/schublade#install`,
        );
      }
    }
  }

  const fromSource = buildFromSource();
  if (fromSource) {
    return fromSource;
  }

  throw new Error(
    `No schublade binary for this install. Tag v${packageVersion()} to cut GitHub Release assets, or run npm install from the schublade-org/schublade checkout with Rust installed.`,
  );
}

module.exports = {
  buildFromSource,
  cargoTomlPath,
  ensureBinary,
  installFromRelease,
  packageVersion,
};
