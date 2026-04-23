import { writeText } from "@tauri-apps/plugin-clipboard-manager";

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
