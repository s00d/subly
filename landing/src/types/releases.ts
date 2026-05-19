export type PlatformKey = "macos" | "windows" | "linux";

export interface PlatformAsset {
  label: string;
  url: string;
  fileName: string;
}

export interface ReleasesManifest {
  tag: string;
  publishedAt: string;
  version: string;
  platforms: Record<PlatformKey, PlatformAsset | null>;
  fallbackUrl: string;
}
