<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import { formatErrorForToast } from "@/utils/formatError";
import { clearRateHistory, rateHistoryCount } from "@/services/ratesClient";
import type { Settings } from "@/schemas/appData";
import { useAppMetaStore } from "@/stores/appMetaStore";
import Toast from "@/components/ui/Toast.vue";
import { History, Trash2 } from "@lucide/vue";
import { button, ui } from "@/lib/tv";
import UiCard from "@/components/ui/UiCard.vue";
import AppToggle from "@/components/ui/AppToggle.vue";
import ConfirmDialog from "@/components/ui/ConfirmDialog.vue";
import SavedIndicator, { useAutoSave } from "@/components/settings/SavedIndicator.vue";

const props = defineProps<{ lookupSettings: Settings | null }>();
const { t } = useI18n();
const { toastMsg, toastType, showToast, toast, closeToast } = useToast();
const metaStore = useAppMetaStore();
const settings = ref<Settings | null>(null);
const { savedAt, run } = useAutoSave();

const recordCount = ref(0);
const showConfirm = ref(false);

watch(() => props.lookupSettings, (v) => (settings.value = v), { immediate: true, deep: true });

onMounted(async () => {
  await loadCount();
});

function updateSettings(updates: Partial<Settings>) {
  if (!settings.value) return;
  const next = { ...settings.value, ...updates };
  settings.value = next;
  void run(() => metaStore.updateSettings(next));
}

async function loadCount() {
  recordCount.value = await rateHistoryCount();
}

function toggleEnabled() {
  updateSettings({
    rateHistoryEnabled: !settings.value?.rateHistoryEnabled,
  });
}

function setDays(val: string) {
  const n = parseInt(val, 10);
  if (isNaN(n) || n < 7) return;
  updateSettings({ rateHistoryDays: Math.min(n, 365) });
}

async function clearHistory() {
  showConfirm.value = false;
  try {
    await clearRateHistory();
    recordCount.value = 0;
    toast(t("history_cleared"));
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  }
}

const dayPresets = [30, 60, 90, 180, 365];
</script>

<template>
  <UiCard :title="t('rate_history')" :icon="History" icon-tone="text-primary">
    <template #actions>
      <SavedIndicator :at="savedAt" />
    </template>
    <p class="text-xs text-text-muted mb-4">{{ t('rate_history_desc') }}</p>

    <div class="space-y-4">
      <AppToggle
        :modelValue="Boolean(settings?.rateHistoryEnabled)"
        @update:modelValue="toggleEnabled"
        :label="t('rate_history_enabled')"
      />

      <!-- Days limit -->
      <div v-if="settings?.rateHistoryEnabled" class="space-y-2">
        <div class="flex items-center justify-between gap-3">
          <span class="text-sm text-text-secondary">{{ t('rate_history_days') }}</span>
          <input
            type="number"
            min="7"
            max="365"
            :value="settings?.rateHistoryDays"
            @change="setDays(($event.target as HTMLInputElement).value)"
            :class="ui.field({ class: 'w-20 h-8 py-1 text-right tabular-nums' })"
          />
        </div>
        <div class="flex flex-wrap gap-1">
          <button
            v-for="d in dayPresets"
            :key="d"
            type="button"
            @click="setDays(String(d))"
            :aria-pressed="settings?.rateHistoryDays === d"
            :class="button({
              intent: settings?.rateHistoryDays === d ? 'primary' : 'secondary',
              size: 'sm',
              class: 'tabular-nums',
            })"
          >{{ d }}d</button>
        </div>
      </div>

      <!-- Stats + Clear -->
      <div class="flex items-center justify-between pt-2 border-t border-border">
        <span class="text-xs text-text-muted">
          {{ t('rate_history_records') }}: <strong class="font-semibold text-text-primary tabular-nums">{{ recordCount.toLocaleString() }}</strong>
        </span>
        <button
          type="button"
          @click="showConfirm = true"
          :disabled="recordCount === 0"
          :class="button({ intent: 'ghost', size: 'sm', class: 'text-danger hover:bg-danger/12 hover:text-danger' })"
        >
          <Trash2 :size="13" />
          {{ t('clear_history') }}
        </button>
      </div>
    </div>

    <ConfirmDialog
      :show="showConfirm"
      :title="t('clear_history')"
      :message="t('clear_history_confirm')"
      :confirm-label="t('confirm')"
      @confirm="clearHistory"
      @cancel="showConfirm = false"
    />
    <Toast :show="showToast" :message="toastMsg" :type="toastType" @close="closeToast" />
  </UiCard>
</template>
