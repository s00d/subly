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
import { ui, button } from "@/lib/tv";
import UiCard from "@/components/ui/UiCard.vue";
import ConfirmDialog from "@/components/ui/ConfirmDialog.vue";
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
  showResetConfirm.value = false;
  try {
    await resetAppData();
    toast(t("reset_success"));
  } catch (e) {
    console.error(e);
    toast(formatErrorForToast(e, t), "error");
  }
}
</script>

<template>
  <UiCard :title="t('data_management')">
    <p class="text-xs sm:text-sm text-text-muted mb-4">{{ t('export_import_info_subly') }}</p>

    <div class="rounded-lg border border-border p-3 mb-6">
      <h3 :class="[ui.sectionTitle(), 'mb-2']">.subly</h3>
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
        <button type="button" @click="handleExportSubly" :disabled="isExporting" :class="button({ intent: 'secondary' })">
          <Archive :size="14" /> {{ t('export_subly') }}
        </button>
        <button type="button" @click="triggerImportSubly" :class="button({ intent: 'secondary' })">
          <Archive :size="14" /> {{ t('import_subly') }}
        </button>
      </div>
    </div>

    <div class="pt-4 border-t border-border">
      <button
        type="button"
        @click="showResetConfirm = true"
        :class="button({ intent: 'ghost', class: 'text-danger hover:bg-danger/12 hover:text-danger' })"
      >
        <RotateCcw :size="14" /> {{ t('reset_data') }}
      </button>
    </div>
  </UiCard>
  <ConfirmDialog
    :show="showResetConfirm"
    :title="t('reset_data')"
    :message="t('reset_data_confirm')"
    :confirm-label="t('confirm')"
    @confirm="handleResetData"
    @cancel="showResetConfirm = false"
  />
  <input ref="sublyInput" type="file" accept=".subly" class="hidden" @change="onPickSubly" />
</template>
