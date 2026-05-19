#!/usr/bin/env node
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const landingRoot = path.resolve(__dirname, "..");
const outDir = path.join(landingRoot, "src/generated");
const outPath = path.join(outDir, "releases.json");

const FALLBACK_URL = "https://github.com/s00d/subly/releases/latest";
const API_URL = "https://api.github.com/repos/s00d/subly/releases/latest";

function scoreAsset(name, osKey) {
  const n = name.toLowerCase();
  if (n.includes("android") || n.includes("apk")) return -1;

  if (osKey === "macos") {
    if (n.includes("subly_macos") && (n.endsWith(".dmg") || n.includes(".app.tar.gz") || n.endsWith(".zip")))
      return 100;
    if (n.endsWith(".dmg")) return 90;
    if (n.endsWith(".app.tar.gz")) return 80;
    return -1;
  }

  if (osKey === "windows") {
    if (n.includes("subly_windows") && (n.endsWith(".msi") || n.endsWith(".exe") || n.includes("nsis")))
      return 100;
    if (n.endsWith(".msi")) return 90;
    if (n.endsWith(".exe") && n.includes("setup")) return 85;
    if (n.endsWith(".exe")) return 70;
    return -1;
  }

  if (osKey === "linux") {
    if (n.includes("subly_linux") && (n.endsWith(".appimage") || n.endsWith(".deb"))) return 100;
    if (n.endsWith(".appimage")) return 90;
    if (n.endsWith(".deb")) return 80;
    if (n.endsWith(".rpm")) return 70;
    return -1;
  }

  return -1;
}

function pickAsset(assets, osKey) {
  const label = osKey === "macos" ? "macOS" : osKey === "windows" ? "Windows" : "Linux";
  let best = null;
  let bestScore = -1;

  for (const asset of assets) {
    const score = scoreAsset(asset.name, osKey);
    if (score > bestScore) {
      bestScore = score;
      best = asset;
    }
  }

  if (!best) return null;
  return {
    label,
    url: best.browser_download_url,
    fileName: best.name,
  };
}

async function main() {
  let payload = {
    tag: "",
    publishedAt: "",
    version: "",
    platforms: {
      macos: null,
      windows: null,
      linux: null,
    },
    fallbackUrl: FALLBACK_URL,
  };

  try {
    const res = await fetch(API_URL, {
      headers: { Accept: "application/vnd.github+json", "User-Agent": "subly-landing" },
    });
    if (!res.ok) throw new Error(`GitHub API ${res.status}`);
    const data = await res.json();
    const assets = data.assets ?? [];
    payload = {
      tag: data.tag_name ?? "",
      publishedAt: data.published_at ?? "",
      version: String(data.tag_name ?? "").replace(/^v/, ""),
      platforms: {
        macos: pickAsset(assets, "macos"),
        windows: pickAsset(assets, "windows"),
        linux: pickAsset(assets, "linux"),
      },
      fallbackUrl: data.html_url ?? FALLBACK_URL,
    };
    console.log(`Latest release: ${payload.tag}`);
    for (const [key, p] of Object.entries(payload.platforms)) {
      console.log(p ? `  ${key}: ${p.fileName}` : `  ${key}: (not found)`);
    }
  } catch (err) {
    console.warn("fetch-github-release:", err instanceof Error ? err.message : err);
    console.warn("Using fallback release URL only.");
  }

  fs.mkdirSync(outDir, { recursive: true });
  fs.writeFileSync(outPath, JSON.stringify(payload, null, 2), "utf8");
  console.log("Wrote", path.relative(landingRoot, outPath));
}

main();
