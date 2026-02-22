import type { AppData } from "@/schemas/appData";

export type SyncProviderType = "icloud" | "gdrive" | "dropbox" | "onedrive" | "webdav";

export interface SyncMeta {
  lastSyncedAt: number;
  updatedAt: number;
  deviceId: string;
}

export interface SyncPayload {
  data: AppData;
  meta: SyncMeta;
}

export interface SyncProvider {
  readonly type: SyncProviderType;
  readonly name: string;
  readonly icon: string;

  isAvailable(): Promise<boolean>;
  isAuthenticated(): Promise<boolean>;
  authenticate(): Promise<boolean>;
  disconnect(): Promise<void>;

  upload(payload: SyncPayload): Promise<void>;
  download(): Promise<SyncPayload | null>;
  getRemoteMeta(): Promise<SyncMeta | null>;
}

export interface SyncStatus {
  provider: SyncProviderType | null;
  enabled: boolean;
  lastSynced: number;
  syncing: boolean;
  error: string | null;
  remoteUpdatedAt: number;
  localUpdatedAt: number;
  pendingUpdate: boolean;
}

export const SYNC_FILENAME = "subly-sync.json";
