#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const rootDir = resolve(scriptDir, "..");
const args = process.argv.slice(2);

function isPrereleaseVersion() {
  const configPath = resolve(rootDir, "src-tauri", "tauri.conf.json");
  const config = JSON.parse(readFileSync(configPath, "utf8"));
  return typeof config.version === "string" && config.version.includes("-");
}

function hasBundleOverride() {
  return args.some(
    (arg) => arg === "--bundles" || arg.startsWith("--bundles=") || arg === "-b",
  );
}

if (args[0] === "build" && isPrereleaseVersion() && !hasBundleOverride()) {
  args.push("--bundles", "nsis");
}

const tauriBin = process.platform === "win32" ? "tauri.cmd" : "tauri";
const result = spawnSync(tauriBin, args, {
  cwd: rootDir,
  stdio: "inherit",
  shell: process.platform === "win32",
});

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

if (result.signal) {
  process.kill(process.pid, result.signal);
}

process.exit(result.status ?? 0);
