<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { storeToRefs } from "pinia";
import {
  Sparkles,
  Upload,
  Loader2,
  CheckCircle2,
  FileText,
} from "@lucide/vue";

import Modal from "@/components/ui/Modal.vue";
import AiImportPreviewList from "@/components/ai/AiImportPreviewList.vue";
import { useToast } from "@/composables/useToast";
import { formatErrorForToast } from "@/utils/formatError";
import { useAppMetaStore } from "@/stores/appMetaStore";
import { upsertExpense } from "@/services/expensesClient";
import {
  aiImportStatementFile,
  type StatementDraftRow,
  type StatementImportProgress,
  type StatementImportResult,
} from "@/services/aiClient";

interface RowState {
  row: StatementDraftRow;
  selected: boolean;
}

const props = defineProps<{ show: boolean }>();
const emit = defineEmits<{
  close: [];
  imported: [count: number];
}>();

const { t, locale } = useI18n();
const { toast } = useToast();
const metaStore = useAppMetaStore();
const metaRefs = storeToRefs(metaStore);

const fileInput = ref<HTMLInputElement | null>(null);
const fileName = ref<string>("");
const result = ref<StatementImportResult | null>(null);
const rows = reactive<RowState[]>([]);
const isParsing = ref(false);
const isImporting = ref(false);
const progressText = ref<string>("");
const progressPercent = ref<number | null>(null);
const cancelRequested = ref(false);

watch(
  () => props.show,
  (val) => {
    if (!val) return;
    fileName.value = "";
    result.value = null;
    rows.splice(0, rows.length);
    isParsing.value = false;
    isImporting.value = false;
    progressText.value = "";
    progressPercent.value = null;
    cancelRequested.value = false;
  },
);

function handleProgress(ev: StatementImportProgress) {
  switch (ev.kind) {
    case "detected":
      progressText.value = t("ai_import_progress_detected", {
        format: ev.format.toUpperCase(),
      });
      progressPercent.value = 5;
      break;
    case "heuristic":
      progressText.value = t("ai_import_progress_heuristic", {
        resolved: ev.resolved,
        unresolved: ev.unresolved,
      });
      progressPercent.value = ev.unresolved === 0 ? 100 : 20;
      break;
    case "llmStart":
      progressText.value = t("ai_import_progress_llm_start", {
        chunks: ev.chunks,
      });
      progressPercent.value = 25;
      break;
    case "llmChunk":
      progressText.value = t("ai_import_progress_llm_chunk", {
        index: ev.index,
        total: ev.total,
      });
      // Stage A occupies 0-25%, Stage B covers 25-100% proportionally.
      progressPercent.value =
        ev.total > 0 ? 25 + Math.round((ev.index / ev.total) * 75) : 50;
      break;
    case "done":
      progressText.value = "";
      progressPercent.value = 100;
      break;
  }
}

function requestCancel() {
  cancelRequested.value = true;
  progressText.value = t("ai_import_cancel");
}

function triggerPicker() {
  fileInput.value?.click();
}

async function onPick(event: Event) {
  const target = event.target as HTMLInputElement;
  const file = target.files?.[0];
  if (!file) return;
  fileName.value = file.name;
  await parseFile(file);
  target.value = "";
}

async function parseFile(file: File) {
  isParsing.value = true;
  progressText.value = "";
  progressPercent.value = 0;
  cancelRequested.value = false;
  try {
    const buf = new Uint8Array(await file.arrayBuffer());
    const mime = file.type || guessMimeFromName(file.name);
    const res = await aiImportStatementFile(buf, mime, locale.value, handleProgress);
    result.value = res;
    rows.splice(0, rows.length);
    for (const row of res.rows) {
      rows.push({ row, selected: row.draft.amount > 0 && !!row.draft.name });
    }
    if (res.rows.length === 0) {
      toast(t("ai_import_no_rows"), "error");
    }
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  } finally {
    isParsing.value = false;
  }
}

function guessMimeFromName(name: string): string {
  const lower = name.toLowerCase();
  if (lower.endsWith(".csv")) return "text/csv";
  if (lower.endsWith(".xlsx") || lower.endsWith(".xls"))
    return "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";
  if (lower.endsWith(".pdf")) return "application/pdf";
  if (lower.endsWith(".json")) return "application/json";
  return "application/octet-stream";
}

function onRowUpdate(idx: number, patch: Partial<StatementDraftRow["draft"]>) {
  const state = rows[idx];
  if (!state) return;
  state.row.draft = { ...state.row.draft, ...patch };
}

function toggleRow(idx: number) {
  const state = rows[idx];
  if (state) state.selected = !state.selected;
}

function selectAll() {
  for (const r of rows) r.selected = true;
}

function deselectAll() {
  for (const r of rows) r.selected = false;
}

const selectedCount = computed(() => rows.filter((r) => r.selected).length);

async function confirmImport() {
  if (selectedCount.value === 0) {
    toast(t("ai_import_select_at_least_one"), "error");
    return;
  }
  isImporting.value = true;
  let saved = 0;
  let failed = 0;
  try {
    for (const state of rows) {
      if (!state.selected) continue;
      const draft = state.row.draft;
      if (!draft.name || draft.amount <= 0 || !draft.date || !draft.currencyId) {
        failed += 1;
        continue;
      }
      try {
        await upsertExpense({
          id: crypto.randomUUID(),
          name: draft.name,
          amount: draft.amount,
          currencyId: draft.currencyId,
          createdAt: new Date(draft.date).toISOString(),
          categoryId: draft.categoryId || "cat-1",
          paymentMethodId: draft.paymentMethodId || "",
          tags: draft.tags ?? [],
          notes: draft.notes ?? "",
          url: draft.url ?? "",
        });
        saved += 1;
      } catch (e) {
        console.warn("[AiImportDialog] upsert failed", e, draft);
        failed += 1;
      }
    }
    if (saved > 0) {
      toast(t("ai_import_saved", { count: saved }));
      emit("imported", saved);
    }
    if (failed > 0) {
      toast(t("ai_import_failed", { count: failed }), "error");
    }
    if (saved > 0 && failed === 0) {
      emit("close");
    }
  } finally {
    isImporting.value = false;
  }
}
</script>

<template>
  <Modal
    :show="show"
    :title="t('ai_import_statement')"
    @close="emit('close')"
    maxWidth="56rem"
  >
    <div class="space-y-4">
      <div
        class="flex items-start gap-2.5 p-3 rounded-lg bg-primary-light text-primary"
      >
        <Sparkles :size="16" class="shrink-0 mt-0.5" />
        <p class="text-xs leading-relaxed">{{ t("ai_import_statement_hint") }}</p>
      </div>

      <div v-if="!result" class="flex flex-col items-center gap-3 py-6">
        <button
          type="button"
          @click="triggerPicker"
          :disabled="isParsing"
          class="inline-flex items-center gap-2 px-4 py-2 rounded-lg border border-primary text-primary text-sm font-medium hover:bg-primary-light disabled:opacity-50"
        >
          <Loader2 v-if="isParsing" :size="14" class="animate-spin" />
          <Upload v-else :size="14" />
          {{ t("ai_import_pick_file") }}
        </button>
        <p v-if="!isParsing" class="text-[11px] text-text-muted">
          CSV, XLSX, JSON, PDF
        </p>
        <div v-if="isParsing && progressPercent !== null" class="w-full max-w-md">
          <div class="h-1.5 w-full bg-surface-secondary rounded-full overflow-hidden">
            <div
              class="h-full bg-primary transition-all duration-200"
              :style="{ width: `${progressPercent}%` }"
            />
          </div>
          <div class="flex items-center justify-between mt-1.5 gap-2">
            <p class="text-[11px] text-text-secondary truncate">
              {{ progressText || t("ai_import_progress_label", { percent: progressPercent }) }}
            </p>
            <button
              v-if="!cancelRequested"
              type="button"
              @click="requestCancel"
              class="text-[10px] text-text-muted hover:text-red-500"
            >
              {{ t("ai_import_cancel") }}
            </button>
          </div>
        </div>
      </div>

      <div v-else class="space-y-3">
        <div
          class="flex flex-wrap items-center gap-2 p-2.5 rounded-lg border border-border bg-surface-secondary"
        >
          <FileText :size="14" class="text-text-muted" />
          <span class="text-xs font-medium text-text-primary truncate">
            {{ fileName }}
          </span>
          <span
            class="inline-flex items-center gap-1 px-1.5 py-0 rounded text-[9px] font-medium bg-surface-hover text-text-muted border border-border"
          >
            {{ result.format.toUpperCase() }}
          </span>
          <div class="flex-1" />
          <span class="text-[11px] text-text-muted">
            <span class="text-emerald-600 dark:text-emerald-400">
              {{ result.stats.matchedByHeuristic }}
            </span>
            +
            <span class="text-blue-600 dark:text-blue-400">
              {{ result.stats.matchedByLlm }}
            </span>
            ·
            <span v-if="result.stats.failed > 0" class="text-amber-600 dark:text-amber-400">
              {{ result.stats.failed }} {{ t("ai_import_failed_short") }}
            </span>
          </span>
        </div>

        <AiImportPreviewList
          :rows="rows"
          :categories="metaRefs.categories.value ?? []"
          :currencies="metaRefs.currencies.value ?? []"
          :payment-methods="metaRefs.paymentMethods.value ?? []"
          @update:row="onRowUpdate"
          @toggle-selection="toggleRow"
          @select-all="selectAll"
          @deselect-all="deselectAll"
        />
      </div>
    </div>

    <template #footer>
      <button
        type="button"
        @click="emit('close')"
        :disabled="isParsing || isImporting"
        class="px-3 sm:px-4 py-2 rounded-lg border border-border text-xs sm:text-sm font-medium text-text-secondary hover:bg-surface-hover disabled:opacity-50"
      >
        {{ t("cancel") }}
      </button>
      <button
        v-if="result"
        type="button"
        @click="confirmImport"
        :disabled="isImporting || selectedCount === 0"
        class="flex items-center gap-1.5 px-3 sm:px-4 py-2 rounded-lg bg-primary text-white text-xs sm:text-sm font-medium hover:bg-primary-hover disabled:opacity-50"
      >
        <Loader2 v-if="isImporting" :size="14" class="animate-spin" />
        <CheckCircle2 v-else :size="14" />
        {{ t("ai_import_confirm", { count: selectedCount }) }}
      </button>
    </template>

    <input
      ref="fileInput"
      type="file"
      accept=".csv,.xlsx,.xls,.json,.pdf,text/csv,application/json,application/pdf"
      class="hidden"
      @change="onPick"
    />
  </Modal>
</template>
