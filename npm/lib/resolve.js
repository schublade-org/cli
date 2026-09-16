"use strict";

const fs = require("fs");
const { platformBinarySpecifier } = require("./platform");

function envBinaryPath() {
  const fromEnv = process.env.SCHUBLADE_BINARY;
  if (fromEnv && fs.existsSync(fromEnv)) {
    return fromEnv;
  }
  return null;
}

function resolvePlatformPackageBinary(resolver = require.resolve) {
  try {
    return resolver(platformBinarySpecifier());
  } catch {
    return null;
  }
}

function resolveBinary(resolver = require.resolve) {
  const override = envBinaryPath();
  if (override) {
    return override;
  }
  return resolvePlatformPackageBinary(resolver);
}

module.exports = {
  envBinaryPath,
  resolveBinary,
  resolvePlatformPackageBinary,
};
