import { openUrl } from "@tauri-apps/plugin-opener";
import { getConfigValue, setConfigValue, deleteConfigValue } from "@/services/database";
import type { SyncProvider, SyncPayload, SyncMeta } from "./types";
import { SYNC_FILENAME } from "./types";

const DROPBOX_TOKEN_KEY = "dropbox_tokens";

interface DropboxTokens {
  access_token: string;
  refresh_token: string;
  expires_at: number;
}

export class DropboxProvider implements SyncProvider {
  readonly type = "dropbox" as const;
  readonly name = "Dropbox";
  readonly icon = "/assets/dropbox.svg";

  private appKey = "";
  private appSecret = "";
  private redirectUri = "http://localhost:19284/callback";
  private tokens: DropboxTokens | null = null;

  constructor(appKey?: string, appSecret?: string) {
    this.appKey = appKey || "";
    this.appSecret = appSecret || "";
  }

  setCredentials(appKey: string, appSecret: string) {
    this.appKey = appKey;
    this.appSecret = appSecret;
  }

  async isAvailable(): Promise<boolean> {
    return !!this.appKey;
  }

  async isAuthenticated(): Promise<boolean> {
    if (this.tokens && this.tokens.expires_at > Date.now()) return true;
    await this.loadTokens();
    if (!this.tokens) return false;
    if (this.tokens.expires_at < Date.now()) return this.refreshAccessToken();
    return true;
  }

  async authenticate(): Promise<boolean> {
    if (!this.appKey) return false;

    const authUrl = `https://www.dropbox.com/oauth2/authorize?client_id=${encodeURIComponent(this.appKey)}&redirect_uri=${encodeURIComponent(this.redirectUri)}&response_type=code&token_access_type=offline`;

    try {
      await openUrl(authUrl);
      return true;
    } catch {
      return false;
    }
  }

  async handleAuthCode(code: string): Promise<boolean> {
    try {
      const resp = await fetch("https://api.dropbox.com/oauth2/token", {
        method: "POST",
        headers: { "Content-Type": "application/x-www-form-urlencoded" },
        body: new URLSearchParams({
          code,
          grant_type: "authorization_code",
          client_id: this.appKey,
          client_secret: this.appSecret,
          redirect_uri: this.redirectUri,
        }),
      });
      if (!resp.ok) return false;
      const data = await resp.json();
      this.tokens = {
        access_token: data.access_token,
        refresh_token: data.refresh_token,
        expires_at: Date.now() + data.expires_in * 1000,
      };
      await this.saveTokens();
      return true;
    } catch {
      return false;
    }
  }

  async disconnect(): Promise<void> {
    if (this.tokens?.access_token) {
      try {
        await fetch("https://api.dropboxapi.com/2/auth/token/revoke", {
          method: "POST",
          headers: { Authorization: `Bearer ${this.tokens.access_token}` },
        });
      } catch {
        // best-effort revoke
      }
    }
    this.tokens = null;
    await deleteConfigValue(DROPBOX_TOKEN_KEY);
  }

  async upload(payload: SyncPayload): Promise<void> {
    if (!(await this.ensureAuth())) throw new Error("Not authenticated");

    const json = JSON.stringify(payload);
    const resp = await fetch("https://content.dropboxapi.com/2/files/upload", {
      method: "POST",
      headers: {
        Authorization: `Bearer ${this.tokens!.access_token}`,
        "Content-Type": "application/octet-stream",
        "Dropbox-API-Arg": JSON.stringify({
          path: `/Apps/Subly/${SYNC_FILENAME}`,
          mode: "overwrite",
          autorename: false,
          mute: true,
        }),
      },
      body: json,
    });
    if (!resp.ok) throw new Error(`Dropbox upload failed: ${resp.status}`);
  }

  async download(): Promise<SyncPayload | null> {
    if (!(await this.ensureAuth())) return null;

    try {
      const resp = await fetch("https://content.dropboxapi.com/2/files/download", {
        method: "POST",
        headers: {
          Authorization: `Bearer ${this.tokens!.access_token}`,
          "Dropbox-API-Arg": JSON.stringify({ path: `/Apps/Subly/${SYNC_FILENAME}` }),
        },
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

  private async refreshAccessToken(): Promise<boolean> {
    if (!this.tokens?.refresh_token) return false;
    try {
      const resp = await fetch("https://api.dropbox.com/oauth2/token", {
        method: "POST",
        headers: { "Content-Type": "application/x-www-form-urlencoded" },
        body: new URLSearchParams({
          refresh_token: this.tokens.refresh_token,
          client_id: this.appKey,
          client_secret: this.appSecret,
          grant_type: "refresh_token",
        }),
      });
      if (!resp.ok) return false;
      const data = await resp.json();
      this.tokens = {
        ...this.tokens,
        access_token: data.access_token,
        expires_at: Date.now() + data.expires_in * 1000,
      };
      await this.saveTokens();
      return true;
    } catch {
      return false;
    }
  }

  private async ensureAuth(): Promise<boolean> {
    if (this.tokens && this.tokens.expires_at > Date.now()) return true;
    await this.loadTokens();
    if (!this.tokens) return false;
    if (this.tokens.expires_at < Date.now()) return this.refreshAccessToken();
    return true;
  }

  private async loadTokens() {
    try {
      this.tokens = await getConfigValue<DropboxTokens>(DROPBOX_TOKEN_KEY);
    } catch {
      this.tokens = null;
    }
  }

  private async saveTokens() {
    if (!this.tokens) return;
    await setConfigValue(DROPBOX_TOKEN_KEY, this.tokens);
  }
}
