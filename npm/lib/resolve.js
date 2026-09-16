"use strict";

const fs = require("fs");
const path = require("path");
const { binaryFileName } = require("./platform");

const PACKAGE_ROOT = path.resolve(__dirname, "..", "..");

function vendorDir() {
  return path.join(PACKAGE_ROOT, "npm", "vendor");
}

function vendorBinaryPath(platform = process.platform) {
  return path.join(vendorDir(), binaryFileName(platform));
}

function envBinaryPath() {
  const fromEnv = process.env.SCHUBLADE_BINARY;
  if (fromEnv && fs.existsSync(fromEnv)) {
    return fromEnv;
  }
  return null;
}

function resolveBinary() {
  const override = envBinaryPath();
  if (override) {
    return override;
  }
  const vendor = vendorBinaryPath();
  if (fs.existsSync(vendor)) {
    return vendor;
  }
  return null;
}

module.exports = {
  PACKAGE_ROOT,
  envBinaryPath,
  resolveBinary,
  vendorBinaryPath,
  vendorDir,
};
