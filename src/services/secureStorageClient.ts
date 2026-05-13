import {
  deletePasswords,
  getPasswords,
  setPasswords,
} from "tauri-plugin-keyring-store-api";

/** Same prefix as legacy `secure_storage_*` IPC (`src-tauri` subscription / AI keys). */
const SECURE_PREFIX = "secure_storage.";

function storageAccount(rawKey: string): string {
  return `${SECURE_PREFIX}${rawKey.trim()}`;
}

export async function setSecureValue(name: string, value: string): Promise<void> {
  await setPasswords([{ account: storageAccount(name), secret: value }]);
}

export async function getSecureValue(name: string): Promise<string | null> {
  const out = await getPasswords([storageAccount(name)]);
  const first = out[0];
  return first ?? null;
}

export async function deleteSecureValue(name: string): Promise<void> {
  await deletePasswords([storageAccount(name)]);
}
