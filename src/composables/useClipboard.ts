import {
  readText as readClipboardPlugin,
  writeText,
} from "@tauri-apps/plugin-clipboard-manager";

/**
 * Read plain text from the clipboard: Tauri plugin first (consistent across
 * desktop/mobile WebViews), then `navigator.clipboard` as a dev/browser fallback.
 */
export async function readClipboardText(): Promise<string | null> {
  try {
    const t = await readClipboardPlugin();
    const trimmed = typeof t === "string" ? t.trim() : "";
    return trimmed.length > 0 ? trimmed : null;
  } catch (e) {
    console.warn("[clipboard] Plugin read failed, trying navigator.clipboard", e);
  }

  try {
    if (typeof navigator !== "undefined" && navigator.clipboard?.readText) {
      const t = await navigator.clipboard.readText();
      const trimmed = typeof t === "string" ? t.trim() : "";
      return trimmed.length > 0 ? trimmed : null;
    }
  } catch (e) {
    console.warn("[clipboard] navigator.clipboard.readText failed", e);
  }

  return null;
}

export function useClipboard() {
  async function copyToClipboard(value: string): Promise<boolean> {
    const text = String(value ?? "");
    if (!text) {
      console.warn("[clipboard] Empty text, skipping copy");
      return false;
    }

    console.debug("[clipboard] Copy started", { length: text.length });

    try {
      await writeText(text);
      console.debug("[clipboard] Copied via plugin-clipboard-manager");
      return true;
    } catch (error) {
      console.error("[clipboard] Plugin copy failed", error);
    }

    console.error("[clipboard] Copy failed");
    return false;
  }

  return { copyToClipboard };
}
