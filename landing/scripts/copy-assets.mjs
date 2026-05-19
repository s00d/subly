#!/usr/bin/env node
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const landingRoot = path.resolve(__dirname, "..");
const repoRoot = path.resolve(landingRoot, "..");
const publicDir = path.join(landingRoot, "public");

function ensureDir(dir) {
  fs.mkdirSync(dir, { recursive: true });
}

function copyFile(src, dest) {
  if (!fs.existsSync(src)) {
    console.warn(`skip (missing): ${path.relative(repoRoot, src)}`);
    return;
  }
  ensureDir(path.dirname(dest));
  fs.copyFileSync(src, dest);
  console.log(`copied ${path.basename(dest)}`);
}

function copyDir(srcDir, destDir) {
  if (!fs.existsSync(srcDir)) {
    console.warn(`skip dir (missing): ${path.relative(repoRoot, srcDir)}`);
    return;
  }
  ensureDir(destDir);
  for (const name of fs.readdirSync(srcDir)) {
    if (name.endsWith(".png")) {
      copyFile(path.join(srcDir, name), path.join(destDir, name));
    }
  }
}

ensureDir(publicDir);
ensureDir(path.join(publicDir, "screenshots"));
ensureDir(path.join(publicDir, "providers"));

const macShots = path.join(repoRoot, "store-assets/mac_app_store/mac_1280x800");
const shotNames = [
  "01_dashboard_overview.png",
  "02_subscriptions_list.png",
  "03_expenses_list.png",
  "04_calendar_month.png",
  "05_rates_screen.png",
  "06_settings_overview.png",
];

for (const name of shotNames) {
  copyFile(path.join(macShots, name), path.join(publicDir, "screenshots", name));
}

copyFile(path.join(repoRoot, "public/brand-logo.svg"), path.join(publicDir, "brand-logo.svg"));
copyFile(path.join(repoRoot, "app-icon.svg"), path.join(publicDir, "app-icon.svg"));

const providerSvgs = ["google-drive.svg", "dropbox.svg", "onedrive.svg"];
for (const name of providerSvgs) {
  copyFile(
    path.join(repoRoot, "public/assets", name),
    path.join(publicDir, "providers", name),
  );
}

copyDir(
  path.join(repoRoot, "store-assets/google_play/phone_1080x1920"),
  path.join(publicDir, "screenshots/phone"),
);
copyDir(
  path.join(repoRoot, "store-assets/google_play/tablet_10in"),
  path.join(publicDir, "screenshots/tablet"),
);

console.log("Assets ready in landing/public/");
