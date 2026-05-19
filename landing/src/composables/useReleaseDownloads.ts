import releases from "@/generated/releases.json";
import type { PlatformKey, ReleasesManifest } from "@/types/releases";

const manifest = releases as ReleasesManifest;

export function platformDownloadUrl(key: PlatformKey): string {
  return manifest.platforms[key]?.url ?? manifest.fallbackUrl;
}

export function preferredDownloadUrl(): string {
  if (typeof navigator === "undefined") {
    return manifest.platforms.macos?.url ?? manifest.fallbackUrl;
  }
  const p = navigator.platform.toLowerCase();
  if (p.includes("mac")) return platformDownloadUrl("macos");
  if (p.includes("win")) return platformDownloadUrl("windows");
  if (p.includes("linux")) return platformDownloadUrl("linux");
  return platformDownloadUrl("macos");
}

export function useReleaseDownloads() {
  return { manifest, platformDownloadUrl, preferredDownloadUrl };
}
