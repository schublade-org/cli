"use strict";

const PLATFORMS = {
  "darwin-arm64": {
    rustTarget: "aarch64-apple-darwin",
    packageName: "@schublade/cli-darwin-arm64",
    npmOs: "darwin",
    npmCpu: "arm64",
  },
  "darwin-x64": {
    rustTarget: "x86_64-apple-darwin",
    packageName: "@schublade/cli-darwin-x64",
    npmOs: "darwin",
    npmCpu: "x64",
  },
  "linux-arm64": {
    rustTarget: "aarch64-unknown-linux-gnu",
    packageName: "@schublade/cli-linux-arm64",
    npmOs: "linux",
    npmCpu: "arm64",
    libc: "glibc",
  },
  "linux-x64": {
    rustTarget: "x86_64-unknown-linux-gnu",
    packageName: "@schublade/cli-linux-x64",
    npmOs: "linux",
    npmCpu: "x64",
    libc: "glibc",
  },
  "win32-arm64": {
    rustTarget: "aarch64-pc-windows-msvc",
    packageName: "@schublade/cli-win32-arm64",
    npmOs: "win32",
    npmCpu: "arm64",
  },
  "win32-x64": {
    rustTarget: "x86_64-pc-windows-msvc",
    packageName: "@schublade/cli-win32-x64",
    npmOs: "win32",
    npmCpu: "x64",
  },
};

function isMusl() {
  if (process.platform !== "linux") {
    return false;
  }
  try {
    const report = process.report && process.report.getReport();
    const glibc = report && report.header && report.header.glibcVersionRuntime;
    if (glibc) {
      return false;
    }
  } catch {
    // fall through
  }
  return true;
}

function platformKey(platform = process.platform, arch = process.arch) {
  return `${platform}-${arch}`;
}

function getPlatform(platform = process.platform, arch = process.arch) {
  const key = platformKey(platform, arch);
  const info = PLATFORMS[key];
  if (!info) {
    const supported = Object.keys(PLATFORMS).join(", ");
    throw new Error(
      `schublade has no prebuilt binary for ${key}. Supported platforms: ${supported}.`,
    );
  }
  if (platform === process.platform && platform === "linux" && isMusl()) {
    throw new Error(
      "schublade prebuilt binaries are glibc (GNU) only. Alpine/musl is not supported — use a glibc distro or build from a source checkout with Rust.",
    );
  }
  return info;
}

function rustTarget(platform = process.platform, arch = process.arch) {
  return getPlatform(platform, arch).rustTarget;
}

function binaryFileName(platform = process.platform) {
  return platform === "win32" ? "schublade.exe" : "schublade";
}

function platformBinarySpecifier(platform = process.platform, arch = process.arch) {
  const info = getPlatform(platform, arch);
  return `${info.packageName}/bin/${binaryFileName(info.npmOs)}`;
}

function findPlatformByRustTarget(target) {
  return Object.values(PLATFORMS).find((info) => info.rustTarget === target) || null;
}

function optionalDependencyMap(version) {
  const deps = {};
  for (const info of Object.values(PLATFORMS)) {
    deps[info.packageName] = version;
  }
  return deps;
}

function archiveName(version, target) {
  return `schublade-v${version}-${target}.tar.gz`;
}

module.exports = {
  PLATFORMS,
  archiveName,
  binaryFileName,
  findPlatformByRustTarget,
  getPlatform,
  isMusl,
  optionalDependencyMap,
  platformBinarySpecifier,
  platformKey,
  rustTarget,
};
