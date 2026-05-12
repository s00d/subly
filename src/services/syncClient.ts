import { reactive } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { callCommand } from "@/services/commandClient";

export type SyncProviderType = "icloud" | "gdrive" | "dropbox" | "onedrive" | "webdav";

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

export interface SyncProviderFieldValidation {
  minLength?: number;
  pattern?: string;
}

export interface SyncProviderFieldSchema {
  key: string;
  label: string;
  required: boolean;
  secret: boolean;
  placeholder?: string;
  inputType?: string;
  helpText?: string;
  validation?: SyncProviderFieldValidation;
  /**
   * `true` ⇔ a value for this secret field is already stored in the OS
   * keyring (backend has probed it). Only meaningful when `secret = true`.
   */
  hasSavedValue?: boolean;
}

export interface SyncProviderSchema {
  type: SyncProviderType;
  name: string;
  icon: string;
  fields: SyncProviderFieldSchema[];
}

export interface SyncSettings {
  provider: SyncProviderType | null;
  enabled: boolean;
  lastSynced: number;
  localUpdatedAt: number;
  deviceId: string;
  dropboxAppKey: string;
  onedriveClientId: string;
  webdavUrl: string;
  webdavUsername: string;
  remoteRevision: string;
}

interface SyncOkResponse {
  ok: boolean;
  pendingUpdate?: boolean;
  /** Backend i18n key when ok is false */
  messageKey?: string;
}

interface SyncUiSchemaResponse {
  status: SyncStatus;
  config: SyncSettings;
  providers: SyncProviderSchema[];
}

const defaultSettings: SyncSettings = {
  provider: null,
  enabled: false,
  lastSynced: 0,
  localUpdatedAt: 0,
  deviceId: "",
  dropboxAppKey: "",
  onedriveClientId: "",
  webdavUrl: "",
  webdavUsername: "",
  remoteRevision: "",
};

export const syncStatus = reactive<SyncStatus>({
  provider: null,
  enabled: false,
  lastSynced: 0,
  syncing: false,
  error: null,
  remoteUpdatedAt: 0,
  localUpdatedAt: 0,
  pendingUpdate: false,
});

let syncSettings: SyncSettings = { ...defaultSettings };
let providersSchema: SyncProviderSchema[] = [];

function applyStatus(next: SyncStatus) {
  syncStatus.provider = next.provider;
  syncStatus.enabled = next.enabled;
  syncStatus.lastSynced = next.lastSynced;
  syncStatus.syncing = next.syncing;
  syncStatus.error = next.error;
  syncStatus.remoteUpdatedAt = next.remoteUpdatedAt;
  syncStatus.localUpdatedAt = next.localUpdatedAt;
  syncStatus.pendingUpdate = next.pendingUpdate;
}

async function refreshStatus() {
  const status = await callCommand<SyncStatus>("sync_get_status");
  applyStatus(status);
}

export async function initSync(): Promise<void> {
  const schema = await callCommand<SyncUiSchemaResponse>("sync_get_ui_schema");
  applyStatus(schema.status);
  syncSettings = { ...defaultSettings, ...schema.config };
  providersSchema = schema.providers ?? [];
}

/**
 * Re-fetch the providers UI schema. Use after credential mutations so the
 * `hasSavedValue` flags reflect the new keyring state without forcing a
 * full app reload.
 */
export async function refreshProvidersSchema(): Promise<SyncProviderSchema[]> {
  const schema = await callCommand<SyncUiSchemaResponse>("sync_get_ui_schema");
  providersSchema = schema.providers ?? [];
  return providersSchema;
}

export function getProviders(): SyncProviderSchema[] {
  return providersSchema;
}

export function getSyncSettings(): SyncSettings {
  return { ...syncSettings };
}

export async function saveProviderSettings(provider: SyncProviderType, credentials: Record<string, string>): Promise<void> {
  const settings = await callCommand<SyncSettings>("sync_save_settings", { provider, credentials });
  syncSettings = { ...defaultSettings, ...settings };
  // Keep `hasSavedValue` indicators in sync with the keyring without
  // forcing the caller to reload the whole UI.
  await refreshProvidersSchema();
}

export type EnableProviderResult = {
  ok: boolean;
  /** Backend i18n key when `ok` is false */
  messageKey?: string;
};

export async function enableProvider(provider: SyncProviderType): Promise<EnableProviderResult> {
  const res = await callCommand<{
    ok: boolean;
    awaitingOAuth?: boolean;
    authUrl?: string;
    messageKey?: string;
  }>("sync_enable_provider", { provider });
  if (res.awaitingOAuth && res.authUrl) {
    await openUrl(res.authUrl);
  }
  await refreshStatus();
  return { ok: !!res.ok, messageKey: res.messageKey };
}

export async function disableProvider(): Promise<void> {
  await callCommand("sync_disable_provider");
  await refreshStatus();
}

export async function finishOAuth(code: string, provider?: SyncProviderType): Promise<boolean> {
  const res = await callCommand<{ ok: boolean }>("sync_oauth_finish", { code, provider });
  await refreshStatus();
  return !!res.ok;
}

export async function checkRemote(): Promise<boolean> {
  const res = await callCommand<{ hasUpdate: boolean }>("sync_check_remote");
  await refreshStatus();
  return !!res.hasUpdate;
}

export async function pullRemote(): Promise<{ ok: boolean; messageKey?: string }> {
  syncStatus.syncing = true;
  try {
    const res = await callCommand<SyncOkResponse>("sync_pull_remote");
    await refreshStatus();
    return { ok: !!res.ok, messageKey: res.messageKey };
  } finally {
    syncStatus.syncing = false;
  }
}

export async function pushLocal(): Promise<{ ok: boolean; messageKey?: string }> {
  syncStatus.syncing = true;
  try {
    const res = await callCommand<SyncOkResponse>("sync_push_local");
    await refreshStatus();
    return { ok: !!res.ok, messageKey: res.messageKey };
  } finally {
    syncStatus.syncing = false;
  }
}

/** Upload local snapshot without revision check (overwrites remote). Use after explicit user confirmation. */
export async function pushLocalForce(): Promise<{ ok: boolean; messageKey?: string }> {
  syncStatus.syncing = true;
  try {
    const res = await callCommand<SyncOkResponse>("sync_force_push_local");
    await refreshStatus();
    return { ok: !!res.ok, messageKey: res.messageKey };
  } finally {
    syncStatus.syncing = false;
  }
}

export async function flushSyncBeforeExit(timeoutMs = 2500): Promise<boolean> {
  const res = await callCommand<SyncOkResponse>("sync_flush_before_exit", { timeoutMs });
  return !!res.ok;
}

export function dismissPendingUpdate(): void {
  syncStatus.pendingUpdate = false;
  void callCommand("sync_dismiss_pending_update");
}
