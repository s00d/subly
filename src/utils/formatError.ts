import { CommandError } from "@/services/commandClient";

type TFn = (key: string, values?: Record<string, string | number>) => string;

/**
 * User-visible error string for toasts. Avoids a bare "Error" when the cause has no message.
 */
export function formatErrorForToast(err: unknown, t: TFn): string {
  const mapKnown = (m: string): string => {
    if (
      m.includes("startDownloadingUbiquitousItem failed") ||
      m.includes("iCloud ubiquity container is unavailable") ||
      m.includes("icloud file is not materialized yet")
    ) {
      return t("sync_pull_no_remote_file");
    }
    return m;
  };
  if (err instanceof CommandError) {
    const m = err.message?.trim();
    if (m) return mapKnown(m);
    return t("error_command_failed", { command: err.command });
  }
  if (err instanceof Error) {
    const m = err.message?.trim();
    if (m) return mapKnown(m);
  }
  if (typeof err === "string") {
    const m = err.trim();
    if (m) return mapKnown(m);
  }
  return t("error_unexpected");
}
