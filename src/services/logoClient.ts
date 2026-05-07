import { callCommand } from "@/services/commandClient";

export interface LogoAsset {
  name: string;
  path: string;
  group: "service" | "payment";
}

let assetsCache: LogoAsset[] | null = null;

export async function getLogoAssets(): Promise<LogoAsset[]> {
  if (assetsCache) return assetsCache;
  assetsCache = await callCommand<LogoAsset[]>("logo_get_assets");
  return assetsCache;
}

export async function resolveFaviconFromInputUrl(input: string, size = 128): Promise<string | null> {
  return callCommand<string | null>("logo_resolve_favicon_from_input_url", {
    input,
    size,
  });
}

export function isImageIcon(icon: string): boolean {
  return icon.startsWith("/") || icon.startsWith("http") || icon.startsWith("data:");
}
