<script setup lang="ts">
import { ref } from "vue";
import { useAppStore } from "@/stores/appStore";
import { useI18n } from "@/i18n";
import { useToast } from "@/composables/useToast";
import { exportAsSubly, exportAsJson, exportAsCsv, exportExpensesCsv, importFromSubly, importFromJson, importFromCsv } from "@/services/export";
import { resetAppData } from "@/services/storage";
import { Download, Upload, RotateCcw, Archive, FileSpreadsheet } from "lucide-vue-next";

const store = useAppStore();
const { t } = useI18n();
const { toast } = useToast();

const isExporting = ref(false);
const showResetConfirm = ref(false);

// --- .subly archive ---
async function handleExportSubly() {
  isExporting.value = true;
  try {
    const ok = await exportAsSubly(store.getExportData());
    if (ok) toast(t("export_success"));
  } catch (e) {
    console.error(e);
    toast(t("error"), "error");
  } finally { isExporting.value = false; }
}

async function handleImportSubly() {
  try {
    const data = await importFromSubly();
    if (data) { store.importData(data); toast(t("import_success")); }
    else { toast(t("import_error"), "error"); }
  } catch (e) {
    console.error(e);
    toast(t("import_error"), "error");
  }
}

// --- Legacy JSON ---
async function handleExportJson() {
  isExporting.value = true;
  try {
    const ok = await exportAsJson(store.getExportData());
    if (ok) toast(t("export_success"));
  } catch (e) {
    console.error(e);
    toast(t("error"), "error");
  } finally { isExporting.value = false; }
}

async function handleExportCsv() {
  isExporting.value = true;
  try {
    const ok = await exportAsCsv(store.state.subscriptions, store.state.categories, store.state.currencies, store.state.paymentMethods, store.state.household);
    if (ok) toast(t("export_success"));
  } catch (e) {
    console.error(e);
    toast(t("error"), "error");
  } finally { isExporting.value = false; }
}

async function handleExportExpensesCsv() {
  isExporting.value = true;
  try {
    const ok = await exportExpensesCsv(store.state.expenses, store.state.categories, store.state.currencies, store.state.paymentMethods, store.state.household);
    if (ok) toast(t("export_success"));
  } catch (e) {
    console.error(e);
    toast(t("error"), "error");
  } finally { isExporting.value = false; }
}

async function handleImportJson() {
  try {
    const data = await importFromJson();
    if (data) { store.importData(data); toast(t("import_success")); }
    else { toast(t("import_error"), "error"); }
  } catch (e) {
    console.error(e);
    toast(t("import_error"), "error");
  }
}

async function handleImportCsv() {
  try {
    const subs = await importFromCsv({
      categories: store.state.categories,
      currencies: store.state.currencies,
      paymentMethods: store.state.paymentMethods,
      household: store.state.household,
      defaultCategoryId: store.state.settings.defaultCategoryId || "cat-1",
      defaultCurrencyId: store.state.settings.mainCurrencyId,
      defaultPaymentMethodId: store.state.settings.defaultPaymentMethodId || store.state.paymentMethods[0]?.id || "",
      defaultPayerUserId: store.state.household[0]?.id || "",
    });
    if (subs && subs.length > 0) {
      for (const sub of subs) {
        store.addSubscription(sub);
      }
      toast(t("csv_import_success").replace("{count}", String(subs.length)));
    } else {
      toast(t("import_error"), "error");
    }
  } catch (e) {
    console.error(e);
    toast(t("import_error"), "error");
  }
}

// --- Reset ---
async function handleResetData() {
  try {
    const defaultData = await resetAppData();
    store.importData(defaultData);
    showResetConfirm.value = false;
    toast(t("reset_success"));
  } catch (e) {
    console.error(e);
    toast(t("error"), "error");
  }
}
</script>

<template>
  <section class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-4 sm:p-5">
    <h2 class="text-base sm:text-lg font-semibold text-[var(--color-text-primary)] mb-2">{{ t('data_management') }}</h2>
    <p class="text-xs sm:text-sm text-[var(--color-text-muted)] mb-4">{{ t('export_import_info') }}</p>

    <!-- Primary: .subly archive -->
    <div class="grid grid-cols-2 gap-2 sm:flex sm:flex-wrap sm:gap-3 mb-4">
      <button @click="handleExportSubly" :disabled="isExporting" class="flex items-center justify-center gap-1.5 sm:gap-2 px-3 sm:px-4 py-2 rounded-lg bg-[var(--color-primary)] text-white text-xs sm:text-sm font-medium hover:bg-[var(--color-primary-hover)] disabled:opacity-50">
        <Archive :size="14" /> {{ t('export_subly') }}
      </button>
      <button @click="handleImportSubly" class="flex items-center justify-center gap-1.5 sm:gap-2 px-3 sm:px-4 py-2 rounded-lg border border-[var(--color-primary)] text-[var(--color-primary)] text-xs sm:text-sm font-medium hover:bg-[var(--color-primary-light)]">
        <Archive :size="14" /> {{ t('import_subly') }}
      </button>
    </div>

    <!-- Secondary: JSON / CSV -->
    <div class="grid grid-cols-2 gap-2 sm:flex sm:flex-wrap sm:gap-3 mb-6">
      <button @click="handleExportJson" :disabled="isExporting" class="flex items-center justify-center gap-1.5 sm:gap-2 px-3 sm:px-4 py-2 rounded-lg border border-[var(--color-border)] text-[var(--color-text-secondary)] text-xs sm:text-sm font-medium hover:bg-[var(--color-surface-hover)] disabled:opacity-50">
        <Download :size="14" /> {{ t('export_as_json') }}
      </button>
      <button @click="handleExportCsv" :disabled="isExporting" class="flex items-center justify-center gap-1.5 sm:gap-2 px-3 sm:px-4 py-2 rounded-lg border border-[var(--color-border)] text-[var(--color-text-secondary)] text-xs sm:text-sm font-medium hover:bg-[var(--color-surface-hover)] disabled:opacity-50">
        <Download :size="14" /> {{ t('export_as_csv') }}
      </button>
      <button @click="handleExportExpensesCsv" :disabled="isExporting" class="flex items-center justify-center gap-1.5 sm:gap-2 px-3 sm:px-4 py-2 rounded-lg border border-[var(--color-border)] text-[var(--color-text-secondary)] text-xs sm:text-sm font-medium hover:bg-[var(--color-surface-hover)] disabled:opacity-50">
        <FileSpreadsheet :size="14" /> {{ t('expenses') }} CSV
      </button>
      <button @click="handleImportJson" class="flex items-center justify-center gap-1.5 sm:gap-2 px-3 sm:px-4 py-2 rounded-lg border border-[var(--color-border)] text-[var(--color-text-secondary)] text-xs sm:text-sm font-medium hover:bg-[var(--color-surface-hover)]">
        <Upload :size="14" /> {{ t('import_from_json') }}
      </button>
      <button @click="handleImportCsv" class="flex items-center justify-center gap-1.5 sm:gap-2 px-3 sm:px-4 py-2 rounded-lg border border-[var(--color-border)] text-[var(--color-text-secondary)] text-xs sm:text-sm font-medium hover:bg-[var(--color-surface-hover)]">
        <FileSpreadsheet :size="14" /> {{ t('import_from_csv') }}
      </button>
    </div>

    <div class="pt-4 border-t border-[var(--color-border)]">
      <div v-if="!showResetConfirm">
        <button @click="showResetConfirm = true" class="flex items-center gap-2 px-4 py-2 rounded-lg border border-red-300 text-red-600 text-sm font-medium hover:bg-red-50 dark:border-red-800 dark:text-red-400 dark:hover:bg-red-900/20">
          <RotateCcw :size="16" /> {{ t('reset_data') }}
        </button>
      </div>
      <div v-else class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
        <p class="text-sm text-red-700 dark:text-red-300 mb-3">{{ t('reset_data_confirm') }}</p>
        <div class="flex gap-2">
          <button @click="handleResetData" class="px-4 py-2 rounded-lg bg-red-600 text-white text-sm font-medium hover:bg-red-700">{{ t('confirm') }}</button>
          <button @click="showResetConfirm = false" class="px-4 py-2 rounded-lg border border-[var(--color-border)] text-[var(--color-text-secondary)] text-sm font-medium hover:bg-[var(--color-surface-hover)]">{{ t('cancel') }}</button>
        </div>
      </div>
    </div>
  </section>
</template>
