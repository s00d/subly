<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { storeToRefs } from "pinia";
import { Sparkles, Loader2, ArrowRight, ScanLine } from "@lucide/vue";

import Modal from "@/components/ui/Modal.vue";
import { useToast } from "@/composables/useToast";
import { formatErrorForToast } from "@/utils/formatError";
import { useAppMetaStore } from "@/stores/appMetaStore";
import {
  aiExtractExpenseFromText,
  aiExtractReceipt,
  aiModelSupportsVision,
  type ExpenseDraft,
} from "@/services/aiClient";
import { useAiConfigStore } from "@/stores/aiConfigStore";

const props = withDefaults(
  defineProps<{
    show: boolean;
    /**
     * When `true`, the receipt file picker is triggered automatically on
     * open. Used by the inline "Scan receipt" CTA so the user lands directly
     * in the file dialog without an extra click.
     */
    autoScanReceipt?: boolean;
  }>(),
  { autoScanReceipt: false },
);
const emit = defineEmits<{
  close: [];
  draft: [ExpenseDraft];
}>();

const { t, locale } = useI18n();
const { toast } = useToast();

const metaStore = useAppMetaStore();
const metaRefs = storeToRefs(metaStore);

const text = ref("");
const isLoading = ref(false);
const fileInput = ref<HTMLInputElement | null>(null);

const aiConfigStore = useAiConfigStore();
const { providers, providerType: activeProviderType, model: activeModel, features: aiFeatures } =
  storeToRefs(aiConfigStore);

const receiptEnabled = computed(() => aiFeatures.value.receiptImport);

/**
 * `true` only when the active model is a preset known to accept image
 * input. Custom models default to `false` so we don't silently send a
 * megabyte of base64 to a text-only endpoint.
 */
const visionVerified = computed(() =>
  aiModelSupportsVision(providers.value, activeProviderType.value, activeModel.value),
);

watch(
  () => props.show,
  async (v) => {
    if (!v) return;
    text.value = "";
    isLoading.value = false;
    try {
      await aiConfigStore.load();
    } catch (e) {
      console.warn("[AiQuickAddExpense] AI config load failed", e);
    }
    // Make sure the meta store is hydrated so the parent form receives
    // resolved currency / category data.
    metaStore.ensureLoaded().catch(() => {});
    void metaRefs;
    if (props.autoScanReceipt && receiptEnabled.value && visionVerified.value) {
      // Wait for the modal DOM (and the hidden <input ref="fileInput">) to
      // mount before triggering the picker.
      await nextTick();
      triggerReceiptPicker();
    }
  },
);

const placeholderExamples = computed(() =>
  [
    "Starbucks coffee $5.40 yesterday",
    "Пятёрочка 1280₽ продукты",
    "Uber 12.50€ ride home",
    "Pharmacy 45 PLN, tag: health",
  ].join("\n"),
);

async function submit() {
  const trimmed = text.value.trim();
  if (!trimmed) {
    toast(t("ai_input_empty_hint"), "error");
    return;
  }
  isLoading.value = true;
  try {
    const draft = await aiExtractExpenseFromText(trimmed, locale.value);
    emit("draft", draft);
    emit("close");
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  } finally {
    isLoading.value = false;
  }
}

function triggerReceiptPicker() {
  fileInput.value?.click();
}

async function onPickReceipt(event: Event) {
  const target = event.target as HTMLInputElement;
  const file = target.files?.[0];
  if (!file) return;
  isLoading.value = true;
  try {
    const buf = new Uint8Array(await file.arrayBuffer());
    const mime = file.type || guessMime(file.name);
    const draft = await aiExtractReceipt(buf, mime, locale.value);
    emit("draft", draft);
    emit("close");
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  } finally {
    isLoading.value = false;
    target.value = "";
  }
}

function guessMime(name: string): string {
  const lower = name.toLowerCase();
  if (lower.endsWith(".png")) return "image/png";
  if (lower.endsWith(".jpg") || lower.endsWith(".jpeg")) return "image/jpeg";
  if (lower.endsWith(".webp")) return "image/webp";
  if (lower.endsWith(".gif")) return "image/gif";
  if (lower.endsWith(".pdf")) return "application/pdf";
  return "application/octet-stream";
}
</script>

<template>
  <Modal :show="show" :title="t('ai_quick_add_expense')" @close="emit('close')" maxWidth="32rem">
    <div class="space-y-4">
      <div class="flex items-start gap-2.5 p-3 rounded-lg bg-primary-light text-primary">
        <Sparkles :size="16" class="shrink-0 mt-0.5" />
        <p class="text-xs leading-relaxed">{{ t("ai_quick_add_expense_hint") }}</p>
      </div>
      <textarea
        v-model="text"
        rows="4"
        :placeholder="placeholderExamples"
        class="w-full px-3 py-2.5 rounded-xl border border-border bg-surface text-sm text-text-primary placeholder-text-muted focus:outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary resize-none"
        autofocus
        :disabled="isLoading"
        @keydown.meta.enter.prevent="submit"
        @keydown.ctrl.enter.prevent="submit"
      />
      <div class="flex items-center justify-between gap-2">
        <p class="text-[11px] text-text-muted">{{ t("ai_keyboard_hint") }}</p>
        <button
          v-if="receiptEnabled"
          type="button"
          @click="triggerReceiptPicker"
          :disabled="isLoading || !visionVerified"
          :title="!visionVerified ? t('ai_model_no_vision') : undefined"
          class="inline-flex items-center gap-1 px-2 py-1 rounded-md border border-border text-[11px] text-text-secondary hover:bg-surface-hover disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <ScanLine :size="12" />
          {{ t("ai_scan_receipt") }}
        </button>
      </div>
    </div>

    <template #footer>
      <button
        type="button"
        @click="emit('close')"
        :disabled="isLoading"
        class="px-3 sm:px-4 py-2 rounded-lg border border-border text-xs sm:text-sm font-medium text-text-secondary hover:bg-surface-hover disabled:opacity-50"
      >
        {{ t("cancel") }}
      </button>
      <button
        type="button"
        @click="submit"
        :disabled="isLoading || !text.trim()"
        class="flex items-center gap-1.5 px-3 sm:px-4 py-2 rounded-lg bg-primary text-white text-xs sm:text-sm font-medium hover:bg-primary-hover disabled:opacity-50"
      >
        <Loader2 v-if="isLoading" :size="14" class="animate-spin" />
        <ArrowRight v-else :size="14" />
        {{ t("ai_extract") }}
      </button>
    </template>

    <input
      ref="fileInput"
      type="file"
      accept="image/png,image/jpeg,image/jpg,image/webp,image/gif,application/pdf"
      class="hidden"
      @change="onPickReceipt"
    />
  </Modal>
</template>
