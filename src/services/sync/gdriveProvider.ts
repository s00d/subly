import { openUrl } from "@tauri-apps/plugin-opener";
import { getConfigValue, setConfigValue, deleteConfigValue } from "@/services/database";
import type { SyncProvider, SyncPayload, SyncMeta } from "./types";
import { SYNC_FILENAME } from "./types";

const GDRIVE_SCOPES = "https://www.googleapis.com/auth/drive.appdata";
const GDRIVE_TOKEN_KEY = "gdrive_tokens";

interface GDriveTokens {
  access_token: string;
  refresh_token: string;
  expires_at: number;
}

export class GDriveProvider implements SyncProvider {
  readonly type = "gdrive" as const;
  readonly name = "Google Drive";
  readonly icon = "/assets/google-drive.svg";

  private clientId = "";
  private clientSecret = "";
  private redirectUri = "http://localhost:19284/callback";
  private tokens: GDriveTokens | null = null;

  constructor(clientId?: string, clientSecret?: string) {
    this.clientId = clientId || "";
    this.clientSecret = clientSecret || "";
  }

  setCredentials(clientId: string, clientSecret: string) {
    this.clientId = clientId;
    this.clientSecret = clientSecret;
  }

  async isAvailable(): Promise<boolean> {
    return !!this.clientId;
  }

  async isAuthenticated(): Promise<boolean> {
    if (this.tokens && this.tokens.expires_at > Date.now()) return true;
    await this.loadTokens();
    if (!this.tokens) return false;
    if (this.tokens.expires_at < Date.now()) {
      return this.refreshAccessToken();
    }
    return true;
  }

  async authenticate(): Promise<boolean> {
    if (!this.clientId) return false;

    const authUrl = `https://accounts.google.com/o/oauth2/v2/auth?client_id=${encodeURIComponent(this.clientId)}&redirect_uri=${encodeURIComponent(this.redirectUri)}&response_type=code&scope=${encodeURIComponent(GDRIVE_SCOPES)}&access_type=offline&prompt=consent`;

    try {
      await openUrl(authUrl);
      return true;
    } catch {
      return false;
    }
  }

  async handleAuthCode(code: string): Promise<boolean> {
    try {
      const resp = await fetch("https://oauth2.googleapis.com/token", {
        method: "POST",
        headers: { "Content-Type": "application/x-www-form-urlencoded" },
        body: new URLSearchParams({
          code,
          client_id: this.clientId,
          client_secret: this.clientSecret,
          redirect_uri: this.redirectUri,
          grant_type: "authorization_code",
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
    this.tokens = null;
    await deleteConfigValue(GDRIVE_TOKEN_KEY);
  }

  async upload(payload: SyncPayload): Promise<void> {
    if (!(await this.ensureAuth())) throw new Error("Not authenticated");

    const existingId = await this.findFileId();
    const json = JSON.stringify(payload);
    const boundary = "subly_boundary_" + Date.now();
    const metadata = JSON.stringify({
      name: SYNC_FILENAME,
      ...(existingId ? {} : { parents: ["appDataFolder"] }),
    });

    const body =
      `--${boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n${metadata}\r\n` +
      `--${boundary}\r\nContent-Type: application/json\r\n\r\n${json}\r\n` +
      `--${boundary}--`;

    const url = existingId
      ? `https://www.googleapis.com/upload/drive/v3/files/${existingId}?uploadType=multipart`
      : "https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart";

    const resp = await fetch(url, {
      method: existingId ? "PATCH" : "POST",
      headers: {
        Authorization: `Bearer ${this.tokens!.access_token}`,
        "Content-Type": `multipart/related; boundary=${boundary}`,
      },
      body,
    });

    if (!resp.ok) throw new Error(`GDrive upload failed: ${resp.status}`);
  }

  async download(): Promise<SyncPayload | null> {
    if (!(await this.ensureAuth())) return null;

    const fileId = await this.findFileId();
    if (!fileId) return null;

    const resp = await fetch(
      `https://www.googleapis.com/drive/v3/files/${fileId}?alt=media`,
      { headers: { Authorization: `Bearer ${this.tokens!.access_token}` } },
    );
    if (!resp.ok) return null;
    return (await resp.json()) as SyncPayload;
  }

  async getRemoteMeta(): Promise<SyncMeta | null> {
    const payload = await this.download();
    return payload?.meta ?? null;
  }

  private async findFileId(): Promise<string | null> {
    const resp = await fetch(
      `https://www.googleapis.com/drive/v3/files?spaces=appDataFolder&q=name='${SYNC_FILENAME}'&fields=files(id)`,
      { headers: { Authorization: `Bearer ${this.tokens!.access_token}` } },
    );
    if (!resp.ok) return null;
    const data = await resp.json();
    return data.files?.[0]?.id ?? null;
  }

  private async refreshAccessToken(): Promise<boolean> {
    if (!this.tokens?.refresh_token) return false;
    try {
      const resp = await fetch("https://oauth2.googleapis.com/token", {
        method: "POST",
        headers: { "Content-Type": "application/x-www-form-urlencoded" },
        body: new URLSearchParams({
          refresh_token: this.tokens.refresh_token,
          client_id: this.clientId,
          client_secret: this.clientSecret,
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
      this.tokens = await getConfigValue<GDriveTokens>(GDRIVE_TOKEN_KEY);
    } catch {
      this.tokens = null;
    }
  }

  private async saveTokens() {
    if (!this.tokens) return;
    await setConfigValue(GDRIVE_TOKEN_KEY, this.tokens);
  }
}
