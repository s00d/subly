import { reactive } from "vue";
import type { AppData } from "@/schemas/appData";
import { validateAppData } from "@/schemas/appData";
import { getConfigValue, setConfigValue } from "@/services/database";
import type { SyncProvider, SyncPayload, SyncStatus, SyncProviderType } from "./types";
import { ICloudProvider } from "./icloudProvider";
import { GDriveProvider } from "./gdriveProvider";
import { DropboxProvider } from "./dropboxProvider";
import { OneDriveProvider } from "./onedriveProvider";
import { WebDAVProvider } from "./webdavProvider";

const SYNC_CONFIG_KEY = "sync_config";
const CHECK_INTERVAL_MS = 2 * 60_000;

interface SyncConfig {
  provider: SyncProviderType | null;
  enabled: boolean;
  lastSynced: number;
  localUpdatedAt: number;
  deviceId: string;
  gdriveClientId: string;
  gdriveClientSecret: string;
  dropboxAppKey: string;
  dropboxAppSecret: string;
  onedriveClientId: string;
  webdavUrl: string;
  webdavUsername: string;
  webdavPassword: string;
}

const defaultConfig: SyncConfig = {
  provider: null,
  enabled: false,
  lastSynced: 0,
  localUpdatedAt: 0,
  deviceId: "",
  gdriveClientId: "",
  gdriveClientSecret: "",
  dropboxAppKey: "",
  dropboxAppSecret: "",
  onedriveClientId: "",
  webdavUrl: "",
  webdavUsername: "",
  webdavPassword: "",
};

let bgInterval: ReturnType<typeof setInterval> | null = null;

function generateDeviceId(): string {
  return "dev_" + crypto.randomUUID().slice(0, 8);
}

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

const providers: Record<SyncProviderType, SyncProvider> = {
  icloud: new ICloudProvider(),
  gdrive: new GDriveProvider(),
  dropbox: new DropboxProvider(),
  onedrive: new OneDriveProvider(),
  webdav: new WebDAVProvider(),
};

let config: SyncConfig = { ...defaultConfig };
let onDataReceived: ((data: AppData) => void) | null = null;
let getLocalData: (() => AppData) | null = null;

export function setSyncCallbacks(
  onReceive: (data: AppData) => void,
  getData: () => AppData,
) {
  onDataReceived = onReceive;
  getLocalData = getData;
}

export function getLocalUpdatedAt(): number {
  return config.localUpdatedAt;
}

export function setLocalUpdatedAt(ts: number) {
  config.localUpdatedAt = ts;
  syncStatus.localUpdatedAt = ts;
}

export async function initSync() {
  const saved = await getConfigValue<SyncConfig>(SYNC_CONFIG_KEY);
  if (saved) {
    config = { ...defaultConfig, ...saved };
  }
  if (!config.deviceId) {
    config.deviceId = generateDeviceId();
    await saveConfig();
  }

  if (config.gdriveClientId) {
    (providers.gdrive as GDriveProvider).setCredentials(config.gdriveClientId, config.gdriveClientSecret);
  }
  if (config.dropboxAppKey) {
    (providers.dropbox as DropboxProvider).setCredentials(config.dropboxAppKey, config.dropboxAppSecret);
  }
  if (config.onedriveClientId) {
    (providers.onedrive as OneDriveProvider).setCredentials(config.onedriveClientId);
  }
  if (config.webdavUrl) {
    (providers.webdav as WebDAVProvider).setCredentials(config.webdavUrl, config.webdavUsername, config.webdavPassword);
  }

  syncStatus.provider = config.provider;
  syncStatus.enabled = config.enabled;
  syncStatus.lastSynced = config.lastSynced;
  syncStatus.localUpdatedAt = config.localUpdatedAt;

  if (config.enabled && config.provider) {
    await checkRemote();
    startBackgroundCheck();
  }
}

export function getProviders(): { type: SyncProviderType; name: string; icon: string }[] {
  return Object.values(providers).map((p) => ({
    type: p.type,
    name: p.name,
    icon: p.icon,
  }));
}

export function getActiveProvider(): SyncProvider | null {
  return config.provider ? providers[config.provider] : null;
}

export function getSyncConfig(): SyncConfig {
  return { ...config };
}

export async function setProviderCredentials(
  type: SyncProviderType,
  credentials: Record<string, string>,
) {
  if (type === "gdrive") {
    config.gdriveClientId = credentials.clientId || "";
    config.gdriveClientSecret = credentials.clientSecret || "";
    (providers.gdrive as GDriveProvider).setCredentials(config.gdriveClientId, config.gdriveClientSecret);
  } else if (type === "dropbox") {
    config.dropboxAppKey = credentials.appKey || "";
    config.dropboxAppSecret = credentials.appSecret || "";
    (providers.dropbox as DropboxProvider).setCredentials(config.dropboxAppKey, config.dropboxAppSecret);
  } else if (type === "onedrive") {
    config.onedriveClientId = credentials.clientId || "";
    (providers.onedrive as OneDriveProvider).setCredentials(config.onedriveClientId);
  } else if (type === "webdav") {
    config.webdavUrl = credentials.serverUrl || "";
    config.webdavUsername = credentials.username || "";
    config.webdavPassword = credentials.password || "";
    (providers.webdav as WebDAVProvider).setCredentials(config.webdavUrl, config.webdavUsername, config.webdavPassword);
  }
  await saveConfig();
}

export async function enableSync(type: SyncProviderType): Promise<boolean> {
  const provider = providers[type];
  if (!(await provider.isAvailable())) return false;

  const authed = await provider.isAuthenticated();
  if (!authed) {
    const ok = await provider.authenticate();
    if (!ok) return false;
  }

  config.provider = type;
  config.enabled = true;
  syncStatus.provider = type;
  syncStatus.enabled = true;
  syncStatus.error = null;
  await saveConfig();

  startBackgroundCheck();
  await checkRemote();
  return true;
}

export async function disableSync() {
  stopBackgroundCheck();
  if (config.provider) {
    try {
      await providers[config.provider].disconnect();
    } catch {
      // best-effort
    }
  }
  config.provider = null;
  config.enabled = false;
  syncStatus.provider = null;
  syncStatus.enabled = false;
  syncStatus.error = null;
  syncStatus.pendingUpdate = false;
  syncStatus.remoteUpdatedAt = 0;
  await saveConfig();
}

export async function handleOAuthCode(code: string): Promise<boolean> {
  if (!config.provider) return false;
  const provider = providers[config.provider];
  if ("handleAuthCode" in provider) {
    return (provider as GDriveProvider | DropboxProvider | OneDriveProvider).handleAuthCode(code);
  }
  return false;
}

/**
 * Lightweight check: fetch remote meta only, compare updatedAt.
 * Sets pendingUpdate flag if remote is newer.
 */
export async function checkRemote(): Promise<boolean> {
  if (!config.enabled || !config.provider) return false;
  const provider = providers[config.provider];

  try {
    if (!(await provider.isAuthenticated())) return false;

    const remoteMeta = await provider.getRemoteMeta();
    if (!remoteMeta) {
      syncStatus.pendingUpdate = false;
      return false;
    }

    const remoteTs = remoteMeta.updatedAt || remoteMeta.lastSyncedAt || 0;
    syncStatus.remoteUpdatedAt = remoteTs;

    if (remoteTs > config.localUpdatedAt && remoteMeta.deviceId !== config.deviceId) {
      syncStatus.pendingUpdate = true;
      return true;
    }

    syncStatus.pendingUpdate = false;
    return false;
  } catch (e) {
    console.warn("checkRemote failed:", e);
    return false;
  }
}

/**
 * Pull remote data into app (user confirmed).
 */
export async function pullRemote(): Promise<boolean> {
  if (!config.enabled || !config.provider || syncStatus.syncing) return false;
  const provider = providers[config.provider];

  syncStatus.syncing = true;
  syncStatus.error = null;

  try {
    if (!(await provider.isAuthenticated())) {
      syncStatus.error = "Not authenticated";
      return false;
    }

    const remote = await provider.download();
    if (!remote?.data) return false;

    const validated = validateAppData(remote.data);
    if (!validated) return false;

    const remoteTs = remote.meta.updatedAt || remote.meta.lastSyncedAt;

    onDataReceived?.(validated);

    config.lastSynced = Date.now();
    config.localUpdatedAt = remoteTs;
    syncStatus.lastSynced = config.lastSynced;
    syncStatus.localUpdatedAt = config.localUpdatedAt;
    syncStatus.pendingUpdate = false;
    syncStatus.remoteUpdatedAt = remoteTs;
    await saveConfig();
    return true;
  } catch (e) {
    syncStatus.error = e instanceof Error ? e.message : String(e);
    console.warn("pullRemote failed:", e);
    return false;
  } finally {
    syncStatus.syncing = false;
  }
}

/**
 * Push local data to cloud.
 */
export async function pushLocal(): Promise<boolean> {
  if (!config.enabled || !config.provider || syncStatus.syncing) return false;
  const provider = providers[config.provider];

  syncStatus.syncing = true;
  syncStatus.error = null;

  try {
    if (!(await provider.isAuthenticated())) {
      syncStatus.error = "Not authenticated";
      return false;
    }

    const localData = getLocalData?.();
    if (!localData) return false;

    const now = Date.now();
    const payload: SyncPayload = {
      data: localData,
      meta: {
        lastSyncedAt: now,
        updatedAt: config.localUpdatedAt || now,
        deviceId: config.deviceId,
      },
    };
    await provider.upload(payload);

    config.lastSynced = now;
    syncStatus.lastSynced = now;
    syncStatus.remoteUpdatedAt = payload.meta.updatedAt;
    syncStatus.pendingUpdate = false;
    await saveConfig();
    return true;
  } catch (e) {
    syncStatus.error = e instanceof Error ? e.message : String(e);
    console.warn("pushLocal failed:", e);
    return false;
  } finally {
    syncStatus.syncing = false;
  }
}

/**
 * Full sync: check remote, if newer — returns true (caller should ask user).
 * Otherwise push local data up.
 */
export async function syncNow(): Promise<boolean> {
  if (!config.enabled || !config.provider || syncStatus.syncing) return false;

  const hasRemoteUpdate = await checkRemote();
  if (hasRemoteUpdate) {
    return false;
  }

  return pushLocal();
}

/**
 * Upload-only alias used by storage after save.
 */
export async function uploadNow(): Promise<boolean> {
  return pushLocal();
}

/** Dismiss the pending update notification without pulling. */
export function dismissPendingUpdate() {
  syncStatus.pendingUpdate = false;
}

function startBackgroundCheck() {
  stopBackgroundCheck();
  bgInterval = setInterval(() => {
    checkRemote().catch(console.warn);
  }, CHECK_INTERVAL_MS);
}

function stopBackgroundCheck() {
  if (bgInterval) {
    clearInterval(bgInterval);
    bgInterval = null;
  }
}

async function saveConfig() {
  await setConfigValue(SYNC_CONFIG_KEY, config);
}
