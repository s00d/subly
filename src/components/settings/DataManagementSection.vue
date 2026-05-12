<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import {
  exportAsSubly,
  importFromSublyBytes,
  getExportPathPresets,
} from "@/services/exportClient";
import { resetAppData } from "@/services/appDataClient";
import { Archive, RotateCcw } from "@lucide/vue";
import { ui } from "@/lib/tv";
import { formatErrorForToast } from "@/utils/formatError";

const { t } = useI18n();
const { toast } = useToast();

const isExporting = ref(false);
const showResetConfirm = ref(false);
const sublyInput = ref<HTMLInputElement | null>(null);

function isSublyArchive(file: File): boolean {
  return file.name.toLowerCase().endsWith(".subly");
}

async function resolveExportPathSubly(): Promise<string | null> {
  const presets = await getExportPathPresets();
  const selected = presets.downloads ?? presets.documents;
  if (!selected) {
    toast(t("export_path_unavailable"), "error");
    return null;
  }
  return selected.sublyBackup;
}

function toastImportStatus(code?: string | null, fallbackError = false) {
  switch (code) {
    case "cancelled":
      toast(t("cancel"));
      break;
    case "invalid_archive":
      toast(t("import_error"), "error");
      break;
    default:
      if (fallbackError) toast(t("import_error"), "error");
  }
}

async function handleExportSubly() {
  isExporting.value = true;
  try {
    const path = await resolveExportPathSubly();
    if (!path) return;
    const res = await exportAsSubly(path);
    if (res.ok) toast(t("export_success"));
  } catch (e) {
    console.error(e);
    toast(formatErrorForToast(e, t), "error");
  } finally {
    isExporting.value = false;
  }
}

async function handleImportSublyFile(file: File) {
  try {
    const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));
    const res = await importFromSublyBytes(bytes);
    if (!res.ok) {
      toastImportStatus(res.message, true);
      return;
    }
    toast(t("import_success"));
  } catch (e) {
    console.error(e);
    toast(formatErrorForToast(e, t), "error");
  }
}

function triggerImportSubly() {
  sublyInput.value?.click();
}

async function onPickSubly(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  if (!isSublyArchive(file)) {
    toast(t("import_error"), "error");
    input.value = "";
    return;
  }
  await handleImportSublyFile(file);
  input.value = "";
}

async function handleResetData() {
  try {
    await resetAppData();
    showResetConfirm.value = false;
    toast(t("reset_success"));
  } catch (e) {
    console.error(e);
    toast(formatErrorForToast(e, t), "error");
  }
}
</script>

<template>
  <section class="bg-surface rounded-xl border border-border p-4 sm:p-5">
    <h2 :class="[ui.sectionTitle(), 'mb-2']">{{ t('data_management') }}</h2>
    <p class="text-xs sm:text-sm text-text-muted mb-4">{{ t('export_import_info') }}</p>

    <div class="space-y-3 mb-6">
      <div class="rounded-lg border border-border p-3">
        <h3 :class="[ui.sectionTitle(), 'mb-2']">.subly</h3>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
          <button @click="handleExportSubly" :disabled="isExporting" class="flex items-center justify-center gap-1.5 sm:gap-2 px-3 sm:px-4 py-2 rounded-lg border border-border text-text-secondary text-xs sm:text-sm font-medium hover:bg-surface-hover disabled:opacity-50">
            <Archive :size="14" /> {{ t('export_subly') }}
          </button>
          <button @click="triggerImportSubly" class="flex items-center justify-center gap-1.5 sm:gap-2 px-3 sm:px-4 py-2 rounded-lg border border-primary text-primary text-xs sm:text-sm font-medium hover:bg-primary-light">
            <Archive :size="14" /> {{ t('import_subly') }}
          </button>
        </div>
      </div>
    </div>

    <div class="pt-4 border-t border-border">
      <div v-if="!showResetConfirm">
        <button @click="showResetConfirm = true" class="flex items-center gap-2 px-4 py-2 rounded-lg border border-red-300 text-red-600 text-sm font-medium hover:bg-red-50 dark:border-red-800 dark:text-red-400 dark:hover:bg-red-900/20">
          <RotateCcw :size="16" /> {{ t('reset_data') }}
        </button>
      </div>
      <div v-else class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
        <p class="text-sm text-red-700 dark:text-red-300 mb-3">{{ t('reset_data_confirm') }}</p>
        <div class="flex gap-2">
          <button @click="handleResetData" class="px-4 py-2 rounded-lg bg-red-600 text-white text-sm font-medium hover:bg-red-700">{{ t('confirm') }}</button>
          <button @click="showResetConfirm = false" class="px-4 py-2 rounded-lg border border-border text-text-secondary text-sm font-medium hover:bg-surface-hover">{{ t('cancel') }}</button>
        </div>
      </div>
    </div>
  </section>
  <input ref="sublyInput" type="file" accept=".subly" class="hidden" @change="onPickSubly" />
</template>
