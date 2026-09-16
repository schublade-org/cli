"use strict";

const SUPPORTED_TARGETS = {
  "darwin-arm64": "aarch64-apple-darwin",
  "darwin-x64": "x86_64-apple-darwin",
  "linux-arm64": "aarch64-unknown-linux-gnu",
  "linux-x64": "x86_64-unknown-linux-gnu",
  "win32-arm64": "aarch64-pc-windows-msvc",
  "win32-x64": "x86_64-pc-windows-msvc",
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

function rustTarget(platform = process.platform, arch = process.arch) {
  const key = `${platform}-${arch}`;
  const target = SUPPORTED_TARGETS[key];
  if (!target) {
    const supported = Object.keys(SUPPORTED_TARGETS).join(", ");
    throw new Error(
      `schublade has no prebuilt binary for ${key}. Supported platforms: ${supported}.`,
    );
  }
  if (platform === process.platform && platform === "linux" && isMusl()) {
    throw new Error(
      "schublade prebuilt binaries are glibc (GNU) only. Alpine/musl is not supported — use a glibc distro or build from a source checkout with Rust.",
    );
  }
  return target;
}

function binaryFileName(platform = process.platform) {
  return platform === "win32" ? "schublade.exe" : "schublade";
}

function archiveName(version, target) {
  return `schublade-v${version}-${target}.tar.gz`;
}

function releaseDownloadUrl(version, target, repo = "schublade-org/schublade") {
  const tag = `v${version}`;
  return `https://github.com/${repo}/releases/download/${tag}/${archiveName(version, target)}`;
}

module.exports = {
  SUPPORTED_TARGETS,
  archiveName,
  binaryFileName,
  isMusl,
  releaseDownloadUrl,
  rustTarget,
};
