#!/usr/bin/env node
/**
 * Bump marketing semver (package.json + Apple metadata) and auto-increment CFBundleVersion
 * by +1 from the current value in src-tauri/tauri.conf.json.
 *
 * Usage: node scripts/bump-app-version.mjs <new-semver>
 * Example: pnpm bump:version 1.0.3
 */

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.join(__dirname, "..");

const newSemver = process.argv[2];
if (!newSemver || !/^\d+\.\d+\.\d+([.-][0-9A-Za-z.-]+)?$/.test(newSemver)) {
  console.error(
    "Usage: node scripts/bump-app-version.mjs <semver>\nExample: pnpm bump:version 1.0.3",
  );
  process.exit(1);
}

const pkgPath = path.join(root, "package.json");
const tauriPath = path.join(root, "src-tauri", "tauri.conf.json");
const projectYml = path.join(
  root,
  "src-tauri",
  "gen",
  "apple",
  "project.yml",
);
const iosInfoPlist = path.join(
  root,
  "src-tauri",
  "gen",
  "apple",
  "subly_iOS",
  "Info.plist",
);
const widgetInfoPlist = path.join(
  root,
  "src-tauri",
  "ios-apple",
  "SublyWidget",
  "Info.plist",
);
const pbxprojPath = path.join(
  root,
  "src-tauri",
  "gen",
  "apple",
  "subly.xcodeproj",
  "project.pbxproj",
);

function readJson(p) {
  return JSON.parse(fs.readFileSync(p, "utf8"));
}

const pkg = readJson(pkgPath);
const oldSemver = pkg.version;

const tauri = readJson(tauriPath);
const iosBv = String(tauri.bundle?.iOS?.bundleVersion ?? "").trim();
const macBv = String(tauri.bundle?.macOS?.bundleVersion ?? "").trim();
if (!/^\d+$/.test(iosBv) || !/^\d+$/.test(macBv)) {
  console.error(
    "Could not read numeric bundleVersion from tauri.conf.json (bundle.iOS / bundle.macOS).",
  );
  process.exit(1);
}
if (iosBv !== macBv) {
  console.warn(
    `Warning: iOS bundleVersion (${iosBv}) !== macOS (${macBv}). Using max+1.`,
  );
}
const oldBuild = String(Math.max(parseInt(iosBv, 10), parseInt(macBv, 10)));
const newBuild = String(parseInt(oldBuild, 10) + 1);

console.log(
  `Marketing: ${oldSemver} → ${newSemver}\nBuild (CFBundleVersion): ${oldBuild} → ${newBuild}`,
);

/** @param {string} s */
function escapeRegExp(s) {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

// 1) package.json
pkg.version = newSemver;
fs.writeFileSync(pkgPath, `${JSON.stringify(pkg, null, 2)}\n`, "utf8");

// 2) tauri.conf.json — both bundleVersion fields
let tauriText = fs.readFileSync(tauriPath, "utf8");
tauriText = tauriText.replace(
  /("bundleVersion":\s*")\d+(")/g,
  `$1${newBuild}$2`,
);
fs.writeFileSync(tauriPath, tauriText, "utf8");

// 3) project.yml
let yml = fs.readFileSync(projectYml, "utf8");
yml = yml.replace(
  new RegExp(
    `CFBundleShortVersionString:\\s*${escapeRegExp(oldSemver)}\\b`,
    "g",
  ),
  `CFBundleShortVersionString: ${newSemver}`,
);
yml = yml.replace(
  new RegExp(`CFBundleVersion:\\s*"${escapeRegExp(oldBuild)}"`, "g"),
  `CFBundleVersion: "${newBuild}"`,
);
yml = yml.replace(
  new RegExp(`MARKETING_VERSION:\\s*${escapeRegExp(oldSemver)}\\b`, "g"),
  `MARKETING_VERSION: ${newSemver}`,
);
yml = yml.replace(
  new RegExp(`CURRENT_PROJECT_VERSION:\\s*"${escapeRegExp(oldBuild)}"`, "g"),
  `CURRENT_PROJECT_VERSION: "${newBuild}"`,
);
fs.writeFileSync(projectYml, yml, "utf8");

// 4) iOS app Info.plist
let iosPlist = fs.readFileSync(iosInfoPlist, "utf8");
iosPlist = iosPlist.replace(
  /(<key>CFBundleShortVersionString<\/key>\s*<string>)[^<]*(<\/string>)/,
  `$1${newSemver}$2`,
);
iosPlist = iosPlist.replace(
  /(<key>CFBundleVersion<\/key>\s*<string>)[^<]*(<\/string>)/,
  `$1${newBuild}$2`,
);
fs.writeFileSync(iosInfoPlist, iosPlist, "utf8");

// 5) Widget Info.plist (CFBundleVersion literal; short uses $(MARKETING_VERSION))
let widgetPlist = fs.readFileSync(widgetInfoPlist, "utf8");
widgetPlist = widgetPlist.replace(
  /(<key>CFBundleVersion<\/key>\s*<string>)[^<]*(<\/string>)/,
  `$1${newBuild}$2`,
);
fs.writeFileSync(widgetInfoPlist, widgetPlist, "utf8");

// 6) Xcode project — patch in place. Do NOT run `xcodegen generate` here: XcodeGen can
//    rewrite entitlements plist files (ios.plist / SublyWidget.entitlements) to empty dicts.
let pbx = fs.readFileSync(pbxprojPath, "utf8");
pbx = pbx.replace(
  new RegExp(
    `CURRENT_PROJECT_VERSION = ${escapeRegExp(oldBuild)};`,
    "g",
  ),
  `CURRENT_PROJECT_VERSION = ${newBuild};`,
);
pbx = pbx.replace(
  new RegExp(`MARKETING_VERSION = ${escapeRegExp(oldSemver)};`, "g"),
  `MARKETING_VERSION = ${newSemver};`,
);
fs.writeFileSync(pbxprojPath, pbx, "utf8");

console.log("Done.");
