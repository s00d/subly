import { callCommand } from "@/services/commandClient";

export async function setSecureValue(name: string, value: string): Promise<void> {
  await callCommand("secure_storage_set", { key: name, value });
}

export async function getSecureValue(name: string): Promise<string | null> {
  return callCommand<string | null>("secure_storage_get", { key: name });
}

export async function deleteSecureValue(name: string): Promise<void> {
  await callCommand("secure_storage_delete", { key: name });
}
