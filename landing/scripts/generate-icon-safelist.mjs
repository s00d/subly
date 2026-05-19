#!/usr/bin/env node
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const landingRoot = path.resolve(__dirname, "..");
const iconsPath = path.join(landingRoot, "src/mdi-icons.ts");
const outPath = path.join(landingRoot, "icon-safelist.html");

const src = fs.readFileSync(iconsPath, "utf8");
const names = [...src.matchAll(/"([a-z0-9-]+)"/g)]
  .map((m) => m[1])
  .filter((n) => n !== "mdiIconNames" && !n.includes(" "));

const spans = names.map((n) => `<span class="i-mdi-${n}"></span>`).join("\n");
const html = `<!DOCTYPE html><html><body>${spans}</body></html>\n`;
fs.writeFileSync(outPath, html, "utf8");
console.log(`Wrote ${names.length} icon classes to icon-safelist.html`);
