"use strict";

const fs = require("fs");
const path = require("path");
const { execFileSync } = require("child_process");
const { binaryFileName } = require("./platform");

const PACKAGE_ROOT = path.resolve(__dirname, "..", "..");

function cargoTomlPath(root = PACKAGE_ROOT) {
  return path.join(root, "Cargo.toml");
}

function isSourceCheckout(root = PACKAGE_ROOT) {
  const manifest = cargoTomlPath(root);
  if (!fs.existsSync(manifest)) {
    return false;
  }
  return /name\s*=\s*"schublade"/.test(fs.readFileSync(manifest, "utf8"));
}

function cargoBinaryCandidates(root = PACKAGE_ROOT) {
  const name = binaryFileName();
  return [
    path.join(root, "target", "release", name),
    path.join(root, "target", "debug", name),
  ];
}

function existingCargoBinary(root = PACKAGE_ROOT) {
  for (const candidate of cargoBinaryCandidates(root)) {
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }
  return null;
}

function ensureCargoBinary(root = PACKAGE_ROOT) {
  if (!isSourceCheckout(root)) {
    return null;
  }
  const existing = existingCargoBinary(root);
  if (existing) {
    return existing;
  }
  execFileSync("cargo", ["build", "--release", "--locked", "--manifest-path", cargoTomlPath(root)], {
    stdio: "inherit",
    cwd: root,
  });
  const built = existingCargoBinary(root);
  if (!built) {
    throw new Error(
      "cargo build --release succeeded but the schublade binary was not found under target/release",
    );
  }
  return built;
}

module.exports = {
  PACKAGE_ROOT,
  cargoBinaryCandidates,
  cargoTomlPath,
  ensureCargoBinary,
  existingCargoBinary,
  isSourceCheckout,
};
