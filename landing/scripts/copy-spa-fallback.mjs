#!/usr/bin/env node
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const distDir = path.resolve(__dirname, "../dist");
const indexPath = path.join(distDir, "index.html");
const fallbackPath = path.join(distDir, "404.html");

if (!fs.existsSync(indexPath)) {
  console.error("dist/index.html not found — run vite build first");
  process.exit(1);
}

fs.copyFileSync(indexPath, fallbackPath);
console.log("Copied index.html → 404.html for GitHub Pages SPA routing");
