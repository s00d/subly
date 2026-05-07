<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import { clearRateHistory, rateHistoryCount } from "@/services/ratesClient";
import type { Settings } from "@/schemas/appData";
import { useAppMetaStore } from "@/stores/appMetaStore";
import Toast from "@/components/ui/Toast.vue";
import { History, Trash2 } from "@lucide/vue";
import { ui } from "@/lib/tv";

const props = defineProps<{ lookupSettings: Settings | null }>();
const { t } = useI18n();
const { toastMsg, toastType, showToast, toast, closeToast } = useToast();
const metaStore = useAppMetaStore();
const settings = ref<Settings | null>(null);

const recordCount = ref(0);
const showConfirm = ref(false);

onMounted(async () => {
  settings.value = props.lookupSettings;
  await loadCount();
});

async function updateSettings(updates: Partial<Settings>) {
  if (!settings.value) return;
  const next = { ...settings.value, ...updates };
  settings.value = next;
  await metaStore.updateSettings(next);
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
  await clearRateHistory();
  recordCount.value = 0;
  showConfirm.value = false;
  toast(t("history_cleared"));
}

const dayPresets = [30, 60, 90, 180, 365];
</script>

<template>
  <section class="bg-surface rounded-xl border border-border p-3 sm:p-5">
    <div class="flex items-center gap-2 mb-1">
      <History :size="16" class="text-primary" />
      <h2 :class="ui.sectionTitle()">{{ t('rate_history') }}</h2>
    </div>
    <p class="text-xs text-text-muted mb-4">{{ t('rate_history_desc') }}</p>

    <div class="space-y-4">
      <!-- Toggle -->
      <div class="flex items-center justify-between">
        <span class="text-sm text-text-primary">{{ t('rate_history_enabled') }}</span>
          <button
          @click="toggleEnabled"
          :class="[
            'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
            settings?.rateHistoryEnabled
              ? 'bg-primary'
              : 'bg-border',
          ]"
        >
          <span
            :class="[
              'inline-block h-4 w-4 transform rounded-full bg-white transition-transform shadow',
              settings?.rateHistoryEnabled ? 'translate-x-6' : 'translate-x-1',
            ]"
          />
        </button>
      </div>

      <!-- Days limit -->
      <div v-if="settings?.rateHistoryEnabled" class="space-y-2">
        <div class="flex items-center justify-between">
          <span class="text-sm text-text-primary">{{ t('rate_history_days') }}</span>
          <input
            type="number"
            min="7"
            max="365"
            :value="settings?.rateHistoryDays"
            @change="setDays(($event.target as HTMLInputElement).value)"
            class="w-20 px-2 py-1 text-sm text-right rounded-lg border border-border bg-surface text-text-primary focus:outline-none focus:ring-2 focus:ring-primary tabular-nums"
          />
        </div>
        <div class="flex flex-wrap gap-1">
          <button
            v-for="d in dayPresets"
            :key="d"
            @click="setDays(String(d))"
            :class="[
              'px-2.5 py-0.5 text-[11px] rounded border transition-colors',
              settings?.rateHistoryDays === d
                ? 'bg-primary text-white border-primary'
                : 'bg-surface-secondary text-text-secondary border-border hover:border-primary hover:text-primary',
            ]"
          >{{ d }}d</button>
        </div>
      </div>

      <!-- Stats + Clear -->
      <div class="flex items-center justify-between pt-2 border-t border-border">
        <span class="text-xs text-text-muted">
          {{ t('rate_history_records') }}: <strong class="text-text-primary tabular-nums">{{ recordCount.toLocaleString() }}</strong>
        </span>
        <div v-if="!showConfirm">
          <button
            @click="showConfirm = true"
            :disabled="recordCount === 0"
            class="flex items-center gap-1 px-2.5 py-1 text-xs rounded-lg text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
          >
            <Trash2 :size="13" />
            {{ t('clear_history') }}
          </button>
        </div>
        <div v-else class="flex items-center gap-2">
          <span class="text-xs text-text-muted">{{ t('clear_history_confirm') }}</span>
          <button
            @click="clearHistory"
            class="px-2.5 py-1 text-xs rounded-lg bg-red-500 text-white hover:bg-red-600 transition-colors"
          >{{ t('confirm') }}</button>
          <button
            @click="showConfirm = false"
            class="px-2.5 py-1 text-xs rounded-lg border border-border text-text-secondary hover:border-primary transition-colors"
          >{{ t('cancel') }}</button>
        </div>
      </div>
    </div>

    <Toast :show="showToast" :message="toastMsg" :type="toastType" @close="closeToast" />
  </section>
</template>
