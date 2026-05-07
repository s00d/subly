import { callCommand } from "@/services/commandClient";

export async function getConfigValue<T>(key: string): Promise<T | null> {
  return callCommand<T | null>("config_get", { key });
}

export async function setConfigValue(key: string, value: unknown): Promise<void> {
  return callCommand("config_set", { key, value });
}

export async function deleteConfigValue(key: string): Promise<void> {
  return callCommand("config_delete", { key });
}

