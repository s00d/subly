import type { SyncProvider, SyncPayload, SyncMeta } from "./types";
import { SYNC_FILENAME } from "./types";

export class WebDAVProvider implements SyncProvider {
  readonly type = "webdav" as const;
  readonly name = "WebDAV";
  readonly icon = "/assets/webdav.svg";

  private serverUrl = "";
  private username = "";
  private password = "";

  setCredentials(serverUrl: string, username: string, password: string) {
    this.serverUrl = serverUrl.replace(/\/+$/, "");
    this.username = username;
    this.password = password;
  }

  private get filePath(): string {
    return `${this.serverUrl}/${SYNC_FILENAME}`;
  }

  private get authHeader(): string {
    return "Basic " + btoa(`${this.username}:${this.password}`);
  }

  async isAvailable(): Promise<boolean> {
    return !!this.serverUrl && !!this.username;
  }

  async isAuthenticated(): Promise<boolean> {
    if (!this.serverUrl || !this.username) return false;
    try {
      const resp = await fetch(this.serverUrl, {
        method: "PROPFIND",
        headers: {
          Authorization: this.authHeader,
          Depth: "0",
        },
      });
      return resp.status === 207 || resp.status === 200;
    } catch {
      return false;
    }
  }

  async authenticate(): Promise<boolean> {
    return this.isAuthenticated();
  }

  async disconnect(): Promise<void> {
    this.serverUrl = "";
    this.username = "";
    this.password = "";
  }

  async upload(payload: SyncPayload): Promise<void> {
    const json = JSON.stringify(payload);
    const resp = await fetch(this.filePath, {
      method: "PUT",
      headers: {
        Authorization: this.authHeader,
        "Content-Type": "application/json",
      },
      body: json,
    });
    if (!resp.ok && resp.status !== 201 && resp.status !== 204) {
      throw new Error(`WebDAV upload failed: ${resp.status}`);
    }
  }

  async download(): Promise<SyncPayload | null> {
    try {
      const resp = await fetch(this.filePath, {
        method: "GET",
        headers: { Authorization: this.authHeader },
      });
      if (!resp.ok) return null;
      return (await resp.json()) as SyncPayload;
    } catch {
      return null;
    }
  }

  async getRemoteMeta(): Promise<SyncMeta | null> {
    const payload = await this.download();
    return payload?.meta ?? null;
  }
}
