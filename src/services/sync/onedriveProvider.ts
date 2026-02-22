import { openUrl } from "@tauri-apps/plugin-opener";
import { getConfigValue, setConfigValue, deleteConfigValue } from "@/services/database";
import type { SyncProvider, SyncPayload, SyncMeta } from "./types";
import { SYNC_FILENAME } from "./types";

const ONEDRIVE_TOKEN_KEY = "onedrive_tokens";
const ONEDRIVE_SCOPES = "Files.ReadWrite.AppFolder offline_access";

interface OneDriveTokens {
  access_token: string;
  refresh_token: string;
  expires_at: number;
}

export class OneDriveProvider implements SyncProvider {
  readonly type = "onedrive" as const;
  readonly name = "OneDrive";
  readonly icon = "/assets/onedrive.svg";

  private clientId = "";
  private redirectUri = "http://localhost:19284/callback";
  private tokens: OneDriveTokens | null = null;

  setCredentials(clientId: string) {
    this.clientId = clientId;
  }

  async isAvailable(): Promise<boolean> {
    return !!this.clientId;
  }

  async isAuthenticated(): Promise<boolean> {
    if (this.tokens && this.tokens.expires_at > Date.now()) return true;
    await this.loadTokens();
    if (!this.tokens) return false;
    if (this.tokens.expires_at < Date.now()) return this.refreshAccessToken();
    return true;
  }

  async authenticate(): Promise<boolean> {
    if (!this.clientId) return false;

    const authUrl =
      `https://login.microsoftonline.com/common/oauth2/v2.0/authorize` +
      `?client_id=${encodeURIComponent(this.clientId)}` +
      `&redirect_uri=${encodeURIComponent(this.redirectUri)}` +
      `&response_type=code` +
      `&scope=${encodeURIComponent(ONEDRIVE_SCOPES)}`;

    try {
      await openUrl(authUrl);
      return true;
    } catch {
      return false;
    }
  }

  async handleAuthCode(code: string): Promise<boolean> {
    try {
      const resp = await fetch(
        "https://login.microsoftonline.com/common/oauth2/v2.0/token",
        {
          method: "POST",
          headers: { "Content-Type": "application/x-www-form-urlencoded" },
          body: new URLSearchParams({
            code,
            client_id: this.clientId,
            redirect_uri: this.redirectUri,
            grant_type: "authorization_code",
            scope: ONEDRIVE_SCOPES,
          }),
        },
      );
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
    await deleteConfigValue(ONEDRIVE_TOKEN_KEY);
  }

  async upload(payload: SyncPayload): Promise<void> {
    if (!(await this.ensureAuth())) throw new Error("Not authenticated");

    const json = JSON.stringify(payload);
    const resp = await fetch(
      `https://graph.microsoft.com/v1.0/me/drive/special/approot:/${SYNC_FILENAME}:/content`,
      {
        method: "PUT",
        headers: {
          Authorization: `Bearer ${this.tokens!.access_token}`,
          "Content-Type": "application/json",
        },
        body: json,
      },
    );
    if (!resp.ok) throw new Error(`OneDrive upload failed: ${resp.status}`);
  }

  async download(): Promise<SyncPayload | null> {
    if (!(await this.ensureAuth())) return null;

    try {
      const resp = await fetch(
        `https://graph.microsoft.com/v1.0/me/drive/special/approot:/${SYNC_FILENAME}:/content`,
        {
          headers: { Authorization: `Bearer ${this.tokens!.access_token}` },
        },
      );
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
      const resp = await fetch(
        "https://login.microsoftonline.com/common/oauth2/v2.0/token",
        {
          method: "POST",
          headers: { "Content-Type": "application/x-www-form-urlencoded" },
          body: new URLSearchParams({
            refresh_token: this.tokens.refresh_token,
            client_id: this.clientId,
            grant_type: "refresh_token",
            scope: ONEDRIVE_SCOPES,
          }),
        },
      );
      if (!resp.ok) return false;
      const data = await resp.json();
      this.tokens = {
        ...this.tokens,
        access_token: data.access_token,
        expires_at: Date.now() + data.expires_in * 1000,
      };
      if (data.refresh_token) this.tokens.refresh_token = data.refresh_token;
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
      this.tokens = await getConfigValue<OneDriveTokens>(ONEDRIVE_TOKEN_KEY);
    } catch {
      this.tokens = null;
    }
  }

  private async saveTokens() {
    if (!this.tokens) return;
    await setConfigValue(ONEDRIVE_TOKEN_KEY, this.tokens);
  }
}
