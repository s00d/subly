#!/usr/bin/env node
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const distDir = path.resolve(__dirname, "../dist");
const indexPath = path.join(distDir, "index.html");

if (!fs.existsSync(indexPath)) {
  console.error("dist/index.html not found — run vite build first");
  process.exit(1);
}

const indexHtml = fs.readFileSync(indexPath, "utf8");

function writeCopy(destRelative) {
  const dest = path.join(distDir, destRelative);
  fs.mkdirSync(path.dirname(dest), { recursive: true });
  fs.writeFileSync(dest, indexHtml, "utf8");
  console.log(`Wrote ${destRelative}`);
}

writeCopy("404.html");

const spaRoutes = ["ru/index.html", "privacy/index.html", "changelog/index.html"];

for (const rel of spaRoutes) {
  writeCopy(rel);
}

console.log("SPA fallbacks ready for GitHub Pages");
