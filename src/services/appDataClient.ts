import type { AppData } from "@/schemas/appData";
import { callCommand } from "@/services/commandClient";

// Keep only runtime-safe command in table-first mode.
export async function resetAppData(): Promise<AppData> {
  return callCommand("reset_app_data");
}
