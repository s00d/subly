import { callCommand } from "@/services/commandClient";

export interface ExportResult {
  ok: boolean;
  message?: string | null;
  importedCount?: number | null;
}

export interface ExportPathSet {
  sublyBackup: string;
}

export interface ExportPathPresets {
  documents?: ExportPathSet | null;
  downloads?: ExportPathSet | null;
}

interface ExportPathArgs {
  path?: string;
}

export async function exportAsSubly(path?: string): Promise<ExportResult> {
  return callCommand("export_subly_backup", {
    args: path ? ({ path } satisfies ExportPathArgs) : undefined,
  });
}

export async function importFromSubly(path?: string): Promise<ExportResult> {
  return callCommand("import_subly_backup", {
    args: path ? ({ path } satisfies ExportPathArgs) : undefined,
  });
}

export async function importFromSublyBytes(bytes: number[]): Promise<ExportResult> {
  return callCommand("import_subly_backup_bytes", { args: { bytes } });
}

export async function getExportPathPresets(): Promise<ExportPathPresets> {
  return callCommand("export_get_path_presets");
}
