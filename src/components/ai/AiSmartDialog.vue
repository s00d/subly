<script setup lang="ts">
import { computed, onBeforeUnmount, reactive, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { storeToRefs } from "pinia";
import {
  Sparkles,
  UploadCloud,
  Loader2,
  CheckCircle2,
  FileText,
  Image as ImageIcon,
  Receipt as ReceiptIcon,
  ListChecks,
  CreditCard,
  X,
  Send,
  ClipboardPaste,
} from "@lucide/vue";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { platform } from "@tauri-apps/plugin-os";

import Modal from "@/components/ui/Modal.vue";
import AiImportPreviewList from "@/components/ai/AiImportPreviewList.vue";
import AiSubscriptionImportPreviewList from "@/components/ai/AiSubscriptionImportPreviewList.vue";
import { useToast } from "@/composables/useToast";
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import { formatErrorForToast } from "@/utils/formatError";
import { useAppMetaStore } from "@/stores/appMetaStore";
import { upsertExpense } from "@/services/expensesClient";
import { upsertSubscription } from "@/services/subscriptionsClient";
import {
  aiSmartInput,
  type AiImportProgress,
  type AiSmartResult,
  type AiSmartSurface,
  type AiSubscriptionDraftRow,
  type StatementDraftRow,
} from "@/services/aiClient";
import { fsReadDroppedFile } from "@/services/fsClient";

interface ExpenseRowState {
  row: StatementDraftRow;
  selected: boolean;
}
interface SubscriptionRowState {
  row: AiSubscriptionDraftRow;
  selected: boolean;
}

const props = defineProps<{ show: boolean; surface: AiSmartSurface }>();
const emit = defineEmits<{
  close: [];
  imported: [count: number];
}>();

const { t, locale } = useI18n();
const { toast } = useToast();
const { fmtCurrency } = useLocaleFormat();
const metaStore = useAppMetaStore();
const metaRefs = storeToRefs(metaStore);

/**
 * Cached file: either a real `File` from the OS picker / clipboard, or a
 * synthetic envelope we build for files dropped through Tauri's native
 * drag-and-drop (the WebView's HTML5 `drop` event never fires in Tauri 2,
 * so we receive a path and read bytes ourselves).
 */
interface AttachedFile {
  name: string;
  mime: string;
  size: number;
  /** Eagerly read for both code paths so submit() is just a memcpy. */
  bytes: Uint8Array;
}

const fileInput = ref<HTMLInputElement | null>(null);
const fileName = ref<string>("");
const attached = ref<AttachedFile | null>(null);
const previewUrl = ref<string>("");
const isDragOver = ref(false);
const textInput = ref<string>("");
const result = ref<AiSmartResult | null>(null);
const expenseRows = reactive<ExpenseRowState[]>([]);
const subscriptionRows = reactive<SubscriptionRowState[]>([]);
const isParsing = ref(false);
const isImporting = ref(false);
const progressText = ref<string>("");
const progressPercent = ref<number | null>(null);
const cancelRequested = ref(false);

let unlistenDragDrop: UnlistenFn | null = null;

/**
 * Finder/Explorer → WebView drag-drop is only meaningful on desktop OS builds.
 * iOS/Android builds use the photo/file picker instead — same rule as
 * `desktopAutostartAvailable()` in `BudgetNotificationsSection.vue`.
 */
function filesystemDragDropSupported(): boolean {
  try {
    const p = platform();
    return p !== "ios" && p !== "android";
  } catch {
    return false;
  }
}

/** Fixed for this WebView for the lifetime of the component (OS does not change). */
const dragDropFromFilesystem = filesystemDragDropSupported();

function revokePreview() {
  if (previewUrl.value) {
    URL.revokeObjectURL(previewUrl.value);
    previewUrl.value = "";
  }
}

function isImageName(name: string, mime: string): boolean {
  if (mime.startsWith("image/")) return true;
  return /\.(png|jpe?g|webp|gif|heic|heif)$/i.test(name);
}

function attachFromBytes(bytes: Uint8Array, name: string, mime: string) {
  revokePreview();
  attached.value = { bytes, name, mime, size: bytes.byteLength };
  fileName.value = name;
  // HEIC can't render in WebViews; the backend transcodes on submit anyway.
  if (isImageName(name, mime) && !/\.(heic|heif)$/i.test(name)) {
    const blob = new Blob([new Uint8Array(bytes)], {
      type: mime || guessMimeFromName(name),
    });
    previewUrl.value = URL.createObjectURL(blob);
  }
}

async function attachFromFile(file: File) {
  const buf = new Uint8Array(await file.arrayBuffer());
  attachFromBytes(
    buf,
    file.name,
    file.type || guessMimeFromName(file.name),
  );
}

function resetForm() {
  revokePreview();
  fileName.value = "";
  attached.value = null;
  textInput.value = "";
  result.value = null;
  expenseRows.splice(0, expenseRows.length);
  subscriptionRows.splice(0, subscriptionRows.length);
  isParsing.value = false;
  isImporting.value = false;
  progressText.value = "";
  progressPercent.value = null;
  cancelRequested.value = false;
  isDragOver.value = false;
}

watch(
  () => props.show,
  async (val) => {
    if (val) {
      resetForm();
      await registerDragDropListener();
    } else {
      revokePreview();
      teardownDragDropListener();
    }
  },
);

onBeforeUnmount(() => {
  revokePreview();
  teardownDragDropListener();
});

async function registerDragDropListener() {
  if (!dragDropFromFilesystem) return;
  if (unlistenDragDrop) return;
  try {
    unlistenDragDrop = await getCurrentWebview().onDragDropEvent(
      async (event) => {
        if (!props.show || isParsing.value) return;
        const payload = event.payload;
        if (payload.type === "over") {
          isDragOver.value = true;
        } else if (payload.type === "leave") {
          isDragOver.value = false;
        } else if (payload.type === "drop") {
          isDragOver.value = false;
          const paths = payload.paths ?? [];
          if (paths.length === 0) return;
          await handleDroppedPath(paths[0]);
        }
      },
    );
  } catch (e) {
    // The webview API throws in non-Tauri environments (Storybook, unit
    // tests). Drag-and-drop just won't work there, which is fine.
    console.warn("[AiSmartDialog] onDragDropEvent unavailable", e);
  }
}

function teardownDragDropListener() {
  if (unlistenDragDrop) {
    try {
      unlistenDragDrop();
    } catch (e) {
      console.warn("[AiSmartDialog] failed to detach drag-drop listener", e);
    }
    unlistenDragDrop = null;
  }
  isDragOver.value = false;
}

async function handleDroppedPath(path: string) {
  try {
    const file = await fsReadDroppedFile(path);
    attachFromBytes(file.bytes, file.name, file.mime);
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  }
}

const acceptedFileTypes =
  ".csv,.xlsx,.xls,.json,.pdf,.txt,.png,.jpg,.jpeg,.webp,.gif,.heic,.heif," +
  "text/csv,application/json,application/pdf,text/plain,image/*";

const dialogTitle = computed(() =>
  props.surface === "expense"
    ? t("ai_smart_expense_title")
    : t("ai_smart_subscription_title"),
);

const hintText = computed(() =>
  props.surface === "expense"
    ? t("ai_smart_expense_hint")
    : t("ai_smart_subscription_hint"),
);

const placeholderText = computed(() =>
  props.surface === "expense"
    ? t("ai_smart_expense_text_placeholder")
    : t("ai_smart_subscription_text_placeholder"),
);

const fileAttachHeadline = computed(() => {
  if (!dragDropFromFilesystem) return t("ai_smart_pick_file");
  return isDragOver.value
    ? t("ai_smart_drop_active")
    : t("ai_smart_drop_here");
});

const canSubmit = computed(
  () =>
    (textInput.value.trim().length > 0 || attached.value !== null) &&
    !isParsing.value,
);

function handleProgress(ev: AiImportProgress) {
  switch (ev.kind) {
    case "detected":
      progressText.value = t("ai_import_progress_detected", {
        format: ev.format.toUpperCase(),
      });
      progressPercent.value = 10;
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
      progressPercent.value =
        ev.total > 0 ? 25 + Math.round((ev.index / ev.total) * 75) : 75;
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

async function onPickFile(event: Event) {
  const target = event.target as HTMLInputElement;
  const file = target.files?.[0];
  if (!file) return;
  await attachFromFile(file);
  target.value = "";
}

function clearFile() {
  revokePreview();
  attached.value = null;
  fileName.value = "";
}

async function onTextareaPaste(ev: ClipboardEvent) {
  const items = ev.clipboardData?.items;
  if (!items) return;
  for (const item of items) {
    if (item.kind === "file") {
      const file = item.getAsFile();
      if (file) {
        ev.preventDefault();
        await attachFromFile(file);
        return;
      }
    }
  }
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function guessMimeFromName(name: string): string {
  const lower = name.toLowerCase();
  if (lower.endsWith(".csv")) return "text/csv";
  if (lower.endsWith(".xlsx") || lower.endsWith(".xls"))
    return "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";
  if (lower.endsWith(".pdf")) return "application/pdf";
  if (lower.endsWith(".json")) return "application/json";
  if (lower.endsWith(".txt")) return "text/plain";
  if (lower.endsWith(".png")) return "image/png";
  if (lower.endsWith(".jpg") || lower.endsWith(".jpeg")) return "image/jpeg";
  if (lower.endsWith(".webp")) return "image/webp";
  if (lower.endsWith(".gif")) return "image/gif";
  if (lower.endsWith(".heic") || lower.endsWith(".heif")) return "image/heic";
  return "application/octet-stream";
}

async function submit() {
  if (!canSubmit.value) return;
  isParsing.value = true;
  progressText.value = "";
  progressPercent.value = 0;
  cancelRequested.value = false;
  try {
    let res: AiSmartResult;
    if (attached.value) {
      const file = attached.value;
      const mime = file.mime || guessMimeFromName(file.name);
      res = await aiSmartInput(
        {
          surface: props.surface,
          bytes: file.bytes,
          mime,
          locale: locale.value,
        },
        handleProgress,
      );
    } else {
      res = await aiSmartInput(
        {
          surface: props.surface,
          text: textInput.value.trim(),
          locale: locale.value,
        },
        handleProgress,
      );
    }
    result.value = res;
    if (res.surface === "expense") {
      expenseRows.splice(0, expenseRows.length);
      for (const row of res.rows) {
        expenseRows.push({
          row,
          selected: row.draft.amount > 0 && !!row.draft.name,
        });
      }
    } else {
      subscriptionRows.splice(0, subscriptionRows.length);
      for (const row of res.rows) {
        subscriptionRows.push({
          row,
          selected: !!row.draft.name && row.draft.price > 0,
        });
      }
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

function onExpenseRowUpdate(
  idx: number,
  patch: Partial<StatementDraftRow["draft"]>,
) {
  const state = expenseRows[idx];
  if (!state) return;
  state.row.draft = { ...state.row.draft, ...patch };
}

function onSubscriptionRowUpdate(
  idx: number,
  patch: Partial<AiSubscriptionDraftRow["draft"]>,
) {
  const state = subscriptionRows[idx];
  if (!state) return;
  state.row.draft = { ...state.row.draft, ...patch };
}

function toggleExpenseRow(idx: number) {
  const s = expenseRows[idx];
  if (s) s.selected = !s.selected;
}
function toggleSubscriptionRow(idx: number) {
  const s = subscriptionRows[idx];
  if (s) s.selected = !s.selected;
}

function removeExpenseRow(idx: number) {
  expenseRows.splice(idx, 1);
}
function removeSubscriptionRow(idx: number) {
  subscriptionRows.splice(idx, 1);
}

function selectAll() {
  if (result.value?.surface === "expense") {
    for (const r of expenseRows) r.selected = true;
  } else {
    for (const r of subscriptionRows) r.selected = true;
  }
}
function deselectAll() {
  if (result.value?.surface === "expense") {
    for (const r of expenseRows) r.selected = false;
  } else {
    for (const r of subscriptionRows) r.selected = false;
  }
}

const selectedCount = computed(() => {
  if (result.value?.surface === "expense") {
    return expenseRows.filter((r) => r.selected).length;
  }
  return subscriptionRows.filter((r) => r.selected).length;
});

const kindLabel = computed(() => {
  if (!result.value) return "";
  if (result.value.surface === "expense") {
    return result.value.kind === "receipt"
      ? t("ai_import_kind_receipt")
      : t("ai_import_kind_statement");
  }
  return result.value.kind === "single"
    ? t("ai_import_kind_subscription_single")
    : t("ai_import_kind_subscription_list");
});

const isReceiptKind = computed(
  () => result.value?.surface === "expense" && result.value.kind === "receipt",
);
const isSingleSubscriptionKind = computed(
  () => result.value?.surface === "subscription" && result.value.kind === "single",
);

const merchantCurrencyCode = computed(() => {
  if (result.value?.surface !== "expense") return "";
  const first = expenseRows[0]?.row.draft;
  if (!first) return "";
  const cur = metaRefs.currencies.value?.find((c) => c.id === first.currencyId);
  return cur?.code ?? "";
});

const sourceLabel = computed(() => {
  if (!result.value) return "";
  if (result.value.format === "text") return t("ai_smart_source_text");
  if (result.value.format === "image") return t("ai_smart_source_image");
  return result.value.format.toUpperCase();
});

async function confirmImport() {
  if (selectedCount.value === 0) {
    toast(t("ai_import_select_at_least_one"), "error");
    return;
  }
  isImporting.value = true;
  let saved = 0;
  let failed = 0;
  try {
    if (result.value?.surface === "expense") {
      for (const state of expenseRows) {
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
          console.warn("[AiSmartDialog] expense upsert failed", e, draft);
          failed += 1;
        }
      }
    } else if (result.value?.surface === "subscription") {
      const today = new Date().toISOString().split("T")[0];
      const householdId = metaRefs.household.value?.[0]?.id ?? "";
      for (const state of subscriptionRows) {
        if (!state.selected) continue;
        const draft = state.row.draft;
        if (!draft.name || draft.price <= 0 || !draft.currencyId) {
          failed += 1;
          continue;
        }
        try {
          await upsertSubscription({
            id: crypto.randomUUID(),
            name: draft.name,
            logo: "",
            price: Number(draft.price) || 0,
            currencyId: draft.currencyId,
            nextPayment: draft.nextPayment || today,
            startDate: draft.startDate || today,
            cycle: Number(draft.cycle) || 3,
            frequency: Number(draft.frequency) || 1,
            notes: draft.notes ?? "",
            paymentMethodId: draft.paymentMethodId || "",
            payerUserId: householdId,
            categoryId: draft.categoryId || "cat-1",
            notify: true,
            notifyDaysBefore: -1,
            lastNotifiedDate: "",
            inactive: false,
            autoRenew: true,
            url: draft.url ?? "",
            cancellationDate: null,
            replacementSubscriptionId: null,
            createdAt: new Date().toISOString(),
            tags: Array.isArray(draft.tags) ? [...draft.tags] : [],
            favorite: false,
          });
          saved += 1;
        } catch (e) {
          console.warn("[AiSmartDialog] subscription upsert failed", e, draft);
          failed += 1;
        }
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

function onTextareaKeydown(ev: KeyboardEvent) {
  if (ev.key === "Enter" && (ev.metaKey || ev.ctrlKey)) {
    ev.preventDefault();
    submit();
  }
}
</script>

<template>
  <Modal :show="show" :title="dialogTitle" maxWidth="56rem" @close="emit('close')">
    <div class="space-y-4">
      <div
        class="flex items-start gap-2.5 p-3 rounded-lg bg-primary-light text-primary"
      >
        <Sparkles :size="16" class="shrink-0 mt-0.5" />
        <p class="text-xs leading-relaxed">{{ hintText }}</p>
      </div>

      <!-- Input stage: textarea + drag-and-drop / paste / preview.
           Drag-and-drop is wired through Tauri's `onDragDropEvent` (the
           webview suppresses HTML5 `drop` events). -->
      <div v-if="!result" class="space-y-4 relative">
        <div class="space-y-1.5">
          <label
            class="text-[11px] font-medium uppercase tracking-wide text-text-muted"
          >
            {{ t("ai_smart_text_label") }}
          </label>
          <textarea
            v-model="textInput"
            :placeholder="placeholderText"
            :disabled="isParsing"
            rows="3"
            class="w-full px-3 py-2.5 rounded-lg border border-border bg-surface text-sm text-text-primary placeholder:text-text-muted focus:outline-none focus:ring-2 focus:ring-primary disabled:opacity-50 resize-y min-h-[96px] max-h-[240px]"
            @keydown="onTextareaKeydown"
            @paste="onTextareaPaste"
          />
          <p class="flex items-center gap-1 text-[10px] text-text-muted">
            <ClipboardPaste :size="10" class="shrink-0" />
            {{ t("ai_smart_paste_hint") }}
          </p>
        </div>

        <div
          class="relative flex items-center gap-2 text-[10px] uppercase tracking-wide text-text-muted"
        >
          <span class="flex-1 h-px bg-border" />
          <span>{{ t("ai_smart_or_separator") }}</span>
          <span class="flex-1 h-px bg-border" />
        </div>

        <div class="space-y-1.5">
          <label
            class="text-[11px] font-medium uppercase tracking-wide text-text-muted"
          >
            {{ t("ai_smart_file_label") }}
          </label>

          <!-- Desktop: dashed drop target. Touch: tap-to-choose (no drag copy). -->
          <button
            v-if="!attached"
            type="button"
            :disabled="isParsing"
            class="w-full flex flex-col items-center justify-center gap-2 px-4 rounded-xl transition-colors text-center disabled:opacity-50 cursor-pointer focus:outline-none focus:ring-2 focus:ring-primary/40"
            :class="
              dragDropFromFilesystem
                ? isDragOver
                  ? 'py-6 sm:py-8 border-2 border-dashed border-primary bg-primary-light/50 text-primary'
                  : 'py-6 sm:py-8 border-2 border-dashed border-border bg-surface-secondary/40 hover:border-primary hover:bg-primary-light/30 text-text-secondary hover:text-primary'
                : 'py-5 sm:py-6 border border-border bg-surface-secondary/70 hover:bg-primary-light/25 hover:border-primary/60 text-text-primary active:scale-[0.99]'
            "
            @click="triggerPicker"
          >
            <UploadCloud
              :size="dragDropFromFilesystem ? 28 : 24"
              class="opacity-80"
            />
            <span class="text-sm font-medium">
              {{ fileAttachHeadline }}
            </span>
            <span class="text-[11px] text-text-muted max-w-md leading-snug">
              {{
                dragDropFromFilesystem
                  ? t("ai_smart_supported_formats")
                  : t("ai_smart_touch_upload_sub")
              }}
            </span>
          </button>

          <!-- Picked file preview -->
          <div
            v-else
            class="rounded-xl border border-border bg-surface-secondary overflow-hidden"
          >
            <!-- Image preview (PNG/JPEG/WebP/GIF) -->
            <div
              v-if="previewUrl"
              class="relative bg-checkerboard flex items-center justify-center max-h-[260px] sm:max-h-[320px]"
            >
              <img
                :src="previewUrl"
                :alt="t('ai_smart_image_preview_alt')"
                class="max-h-[260px] sm:max-h-[320px] max-w-full object-contain"
                draggable="false"
              />
              <button
                type="button"
                :disabled="isParsing"
                class="absolute top-2 right-2 inline-flex items-center justify-center w-9 h-9 rounded-full bg-slate-900/70 text-white hover:bg-slate-900/90 disabled:opacity-50 backdrop-blur-sm"
                :aria-label="t('ai_smart_clear_file')"
                @click="clearFile"
              >
                <X :size="16" />
              </button>
            </div>

            <!-- Non-image file: rich row -->
            <div
              v-else
              class="flex items-center gap-3 p-3 sm:p-3.5"
            >
              <div
                class="shrink-0 inline-flex items-center justify-center w-10 h-10 rounded-lg bg-primary-light text-primary"
              >
                <component
                  :is="
                    isImageName(attached.name, attached.mime)
                      ? ImageIcon
                      : FileText
                  "
                  :size="20"
                />
              </div>
              <div class="min-w-0 flex-1">
                <p class="text-sm font-medium text-text-primary truncate">
                  {{ fileName }}
                </p>
                <p class="text-[11px] text-text-muted">
                  {{ formatFileSize(attached.size) }}
                </p>
              </div>
              <button
                type="button"
                :disabled="isParsing"
                class="shrink-0 inline-flex items-center justify-center w-10 h-10 rounded-full text-text-muted hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 disabled:opacity-50"
                :aria-label="t('ai_smart_clear_file')"
                @click="clearFile"
              >
                <X :size="18" />
              </button>
            </div>

            <!-- Image file metadata footer -->
            <div
              v-if="previewUrl"
              class="flex items-center gap-2 px-3 py-2 text-[11px] text-text-muted border-t border-border"
            >
              <ImageIcon :size="12" />
              <span class="truncate flex-1">{{ fileName }}</span>
              <span class="tabular-nums shrink-0">
                {{ formatFileSize(attached.size) }}
              </span>
            </div>
          </div>
        </div>

        <div v-if="isParsing && progressPercent !== null" class="space-y-1.5">
          <div class="h-1.5 w-full bg-surface-secondary rounded-full overflow-hidden">
            <div
              class="h-full bg-primary transition-all duration-200"
              :style="{ width: `${progressPercent}%` }"
            />
          </div>
          <div class="flex items-center justify-between gap-2">
            <p class="text-[11px] text-text-secondary truncate">
              {{
                progressText ||
                t("ai_import_progress_label", { percent: progressPercent })
              }}
            </p>
            <button
              v-if="!cancelRequested"
              type="button"
              class="text-[10px] text-text-muted hover:text-red-500 min-h-[32px] px-2 -mr-2"
              @click="requestCancel"
            >
              {{ t("ai_import_cancel") }}
            </button>
          </div>
        </div>

        <!-- Drag overlay (desktop only — phones use tap-to-upload) -->
        <div
          v-if="dragDropFromFilesystem && isDragOver && !isParsing"
          class="pointer-events-none absolute inset-0 -m-2 rounded-2xl border-2 border-dashed border-primary bg-primary-light/70 backdrop-blur-sm flex items-center justify-center"
        >
          <div class="flex flex-col items-center gap-2 text-primary">
            <UploadCloud :size="48" />
            <span class="text-base font-semibold">
              {{ t("ai_smart_drop_active") }}
            </span>
          </div>
        </div>
      </div>

      <!-- Preview stage: header summary + per-surface preview list -->
      <div v-else class="space-y-3">
        <div
          class="flex flex-wrap items-center gap-2 p-2.5 sm:p-3 rounded-lg border border-border bg-surface-secondary"
        >
          <!-- Thumbnail (image previews keep showing in the result stage) -->
          <div
            v-if="previewUrl"
            class="shrink-0 w-10 h-10 rounded-md overflow-hidden border border-border bg-checkerboard"
          >
            <img
              :src="previewUrl"
              :alt="t('ai_smart_image_preview_alt')"
              class="w-full h-full object-cover"
            />
          </div>
          <component
            v-else
            :is="result.format === 'image' ? ImageIcon : FileText"
            :size="14"
            class="text-text-muted shrink-0"
          />
          <span class="text-xs font-medium text-text-primary truncate min-w-0 flex-1 sm:flex-initial">
            {{ fileName || sourceLabel }}
          </span>
          <span
            class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-medium border"
            :class="
              isReceiptKind || isSingleSubscriptionKind
                ? 'bg-amber-100 text-amber-800 border-amber-200 dark:bg-amber-900/30 dark:text-amber-200 dark:border-amber-800'
                : 'bg-blue-100 text-blue-800 border-blue-200 dark:bg-blue-900/30 dark:text-blue-200 dark:border-blue-800'
            "
          >
            <ReceiptIcon v-if="isReceiptKind" :size="10" />
            <CreditCard v-else-if="isSingleSubscriptionKind" :size="10" />
            <ListChecks v-else :size="10" />
            {{ kindLabel }}
          </span>
          <span
            class="inline-flex items-center gap-1 px-1.5 py-0 rounded text-[9px] font-medium bg-surface-hover text-text-muted border border-border"
          >
            {{ result.format.toUpperCase() }}
          </span>
          <div class="hidden sm:block flex-1" />
          <span class="text-[11px] text-text-muted basis-full sm:basis-auto">
            <span
              v-if="
                result.surface === 'expense' &&
                result.stats.matchedByHeuristic > 0
              "
              class="text-emerald-600 dark:text-emerald-400"
            >
              {{ result.stats.matchedByHeuristic }}
            </span>
            <span
              v-if="
                result.surface === 'expense' &&
                result.stats.matchedByHeuristic > 0
              "
              >+</span
            >
            <span class="text-blue-600 dark:text-blue-400">
              {{ result.stats.matchedByLlm }}
            </span>
            <span v-if="result.stats.failed > 0">
              ·
              <span class="text-amber-600 dark:text-amber-400">
                {{ result.stats.failed }} {{ t("ai_import_failed_short") }}
              </span>
            </span>
          </span>
        </div>

        <!-- Receipt metadata: merchant + line items (expense surface only) -->
        <div
          v-if="
            result.surface === 'expense' &&
            isReceiptKind &&
            (result.metadata.merchantName || result.metadata.lineItems.length > 0)
          "
          class="rounded-lg border border-amber-200 dark:border-amber-800 bg-amber-50/50 dark:bg-amber-900/10 p-2.5 space-y-1.5"
        >
          <div
            v-if="result.metadata.merchantName"
            class="flex items-center justify-between text-xs"
          >
            <span class="text-text-muted">{{ t("ai_import_merchant") }}</span>
            <span class="font-medium text-text-primary truncate ml-2">
              {{ result.metadata.merchantName }}
            </span>
          </div>
          <div
            v-if="result.metadata.lineItems.length > 0"
            class="space-y-0.5 pt-1 border-t border-amber-200/50 dark:border-amber-800/50"
          >
            <div
              v-for="(li, i) in result.metadata.lineItems"
              :key="i"
              class="flex items-center justify-between text-[11px] gap-2"
            >
              <span class="text-text-secondary truncate">{{ li.name }}</span>
              <span class="text-text-primary tabular-nums shrink-0">
                {{
                  merchantCurrencyCode
                    ? fmtCurrency(li.amount, merchantCurrencyCode)
                    : li.amount.toFixed(2)
                }}
              </span>
            </div>
          </div>
        </div>

        <AiImportPreviewList
          v-if="result.surface === 'expense'"
          :rows="expenseRows"
          :categories="metaRefs.categories.value ?? []"
          :currencies="metaRefs.currencies.value ?? []"
          :payment-methods="metaRefs.paymentMethods.value ?? []"
          @update:row="onExpenseRowUpdate"
          @toggle-selection="toggleExpenseRow"
          @remove="removeExpenseRow"
          @select-all="selectAll"
          @deselect-all="deselectAll"
        />
        <AiSubscriptionImportPreviewList
          v-else
          :rows="subscriptionRows"
          :categories="metaRefs.categories.value ?? []"
          :currencies="metaRefs.currencies.value ?? []"
          :payment-methods="metaRefs.paymentMethods.value ?? []"
          @update:row="onSubscriptionRowUpdate"
          @toggle-selection="toggleSubscriptionRow"
          @remove="removeSubscriptionRow"
          @select-all="selectAll"
          @deselect-all="deselectAll"
        />
      </div>
    </div>

    <template #footer>
      <button
        type="button"
        :disabled="isParsing || isImporting"
        class="px-3 sm:px-4 py-2 rounded-lg border border-border text-xs sm:text-sm font-medium text-text-secondary hover:bg-surface-hover disabled:opacity-50"
        @click="emit('close')"
      >
        {{ t("cancel") }}
      </button>
      <button
        v-if="!result"
        type="button"
        :disabled="!canSubmit"
        class="flex items-center gap-1.5 px-3 sm:px-4 py-2 rounded-lg bg-primary text-white text-xs sm:text-sm font-medium hover:bg-primary-hover disabled:opacity-50"
        @click="submit"
      >
        <Loader2 v-if="isParsing" :size="14" class="animate-spin" />
        <Send v-else :size="14" />
        {{ t("ai_smart_submit") }}
      </button>
      <button
        v-else
        type="button"
        :disabled="isImporting || selectedCount === 0"
        class="flex items-center gap-1.5 px-3 sm:px-4 py-2 rounded-lg bg-primary text-white text-xs sm:text-sm font-medium hover:bg-primary-hover disabled:opacity-50"
        @click="confirmImport"
      >
        <Loader2 v-if="isImporting" :size="14" class="animate-spin" />
        <CheckCircle2 v-else :size="14" />
        {{ t("ai_import_confirm", { count: selectedCount }) }}
      </button>
    </template>

    <input
      ref="fileInput"
      type="file"
      :accept="acceptedFileTypes"
      class="hidden"
      @change="onPickFile"
    />
  </Modal>
</template>
