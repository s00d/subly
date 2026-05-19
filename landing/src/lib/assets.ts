/** Public asset URL with Vite base path (/subly/). */
export function assetUrl(relativePath: string): string {
  const base = import.meta.env.BASE_URL;
  const path = relativePath.replace(/^\//, "");
  return `${base}${path}`;
}

export function screenshotUrl(filename: string): string {
  return assetUrl(`screenshots/${filename}`);
}

export function providerIconUrl(filename: string): string {
  return assetUrl(`providers/${filename}`);
}
