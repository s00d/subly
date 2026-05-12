<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { Sparkles, Loader2, ArrowRight } from "@lucide/vue";

import Modal from "@/components/ui/Modal.vue";
import { useToast } from "@/composables/useToast";
import { formatErrorForToast } from "@/utils/formatError";
import {
  aiExtractSubscriptionFromText,
  type SubscriptionDraft,
} from "@/services/aiClient";

const props = defineProps<{ show: boolean }>();
const emit = defineEmits<{
  close: [];
  draft: [SubscriptionDraft];
}>();

const { t, locale } = useI18n();
const { toast } = useToast();

const text = ref("");
const isLoading = ref(false);

watch(
  () => props.show,
  (v) => {
    if (!v) return;
    text.value = "";
    isLoading.value = false;
  },
);

const placeholderExamples = computed(() =>
  [
    "Telegram Premium ₽300 per month",
    "Netflix $15.49 monthly, family plan",
    "iCloud+ 200 GB €2.99/month",
    "JetBrains All Products Pack $289/year",
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
    const draft = await aiExtractSubscriptionFromText(trimmed, locale.value);
    emit("draft", draft);
    emit("close");
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  } finally {
    isLoading.value = false;
  }
}
</script>

<template>
  <Modal :show="show" :title="t('ai_quick_add_subscription')" @close="emit('close')" maxWidth="32rem">
    <div class="space-y-4">
      <div class="flex items-start gap-2.5 p-3 rounded-lg bg-primary-light text-primary">
        <Sparkles :size="16" class="shrink-0 mt-0.5" />
        <p class="text-xs leading-relaxed">{{ t("ai_quick_add_subscription_hint") }}</p>
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
      <p class="text-[11px] text-text-muted">{{ t("ai_keyboard_hint") }}</p>
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
  </Modal>
</template>
