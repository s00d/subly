import { load, type Store } from "@tauri-apps/plugin-store";
import type { AppData } from "@/schemas/appData";
import { validateAppData } from "@/schemas/appData";
import { getDefaultData } from "./seed";

let store: Store | null = null;

async function getStore(): Promise<Store> {
  if (!store) {
    store = await load("subly-data.json", { autoSave: true, defaults: {} });
  }
  return store;
}

export async function loadAppData(): Promise<AppData> {
  const s = await getStore();
  const raw = await s.get<unknown>("appData");

  if (!raw || (typeof raw === "object" && !(raw as Record<string, unknown>).initialized)) {
    const defaultData = getDefaultData();
    await s.set("appData", defaultData);
    await s.save();
    return defaultData;
  }

  // Validate and sanitize data with Zod — fills missing fields, strips invalid entries
  const validated = validateAppData(raw);
  if (!validated) {
    // Full recovery failed — reset to defaults
    const defaultData = getDefaultData();
    await s.set("appData", defaultData);
    await s.save();
    return defaultData;
  }
  return validated;
}

export async function saveAppData(data: AppData): Promise<void> {
  // Validate before saving to ensure data integrity
  const validated = validateAppData(data);
  const s = await getStore();
  await s.set("appData", validated ?? data);
  await s.save();
}

export async function resetAppData(): Promise<AppData> {
  const s = await getStore();
  const defaultData = getDefaultData();
  await s.set("appData", defaultData);
  await s.save();
  return defaultData;
}
