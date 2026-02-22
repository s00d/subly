import { invoke } from "@tauri-apps/api/core";
import { platform } from "@tauri-apps/plugin-os";
import type { SyncProvider, SyncPayload, SyncMeta } from "./types";
import { SYNC_FILENAME } from "./types";

export class ICloudProvider implements SyncProvider {
  readonly type = "icloud" as const;
  readonly name = "iCloud";
  readonly icon = "/assets/icloud.svg";

  async isAvailable(): Promise<boolean> {
    const p = platform();
    if (p !== "macos" && p !== "ios") return false;
    try {
      const url = await invoke<string | null>("icloud_container_url");
      return url !== null;
    } catch {
      return false;
    }
  }

  async isAuthenticated(): Promise<boolean> {
    return this.isAvailable();
  }

  async authenticate(): Promise<boolean> {
    return this.isAvailable();
  }

  async disconnect(): Promise<void> {
    // iCloud is system-level, nothing to disconnect
  }

  async upload(payload: SyncPayload): Promise<void> {
    const json = JSON.stringify(payload);
    await invoke("icloud_write_file", { filename: SYNC_FILENAME, contents: json });
  }

  async download(): Promise<SyncPayload | null> {
    try {
      const raw = await invoke<string | null>("icloud_read_file", { filename: SYNC_FILENAME });
      if (!raw) return null;
      return JSON.parse(raw) as SyncPayload;
    } catch {
      return null;
    }
  }

  async getRemoteMeta(): Promise<SyncMeta | null> {
    const payload = await this.download();
    return payload?.meta ?? null;
  }
}
