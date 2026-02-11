<script setup lang="ts">
import { ref, computed } from "vue";
import { useAppStore } from "@/stores/appStore";
import { useI18n } from "@/i18n";
import { useToast } from "@/composables/useToast";
import AppInput from "@/components/ui/AppInput.vue";
import AppSelect from "@/components/ui/AppSelect.vue";
import AppToggle from "@/components/ui/AppToggle.vue";
import type { SelectOption } from "@/components/ui/AppSelect.vue";
import Toast from "@/components/ui/Toast.vue";
import { updateCurrencyRates } from "@/services/currencyUpdater";
import { RefreshCw } from "lucide-vue-next";

const store = useAppStore();
const { t } = useI18n();
const { toastMsg, toastType, showToast, toast, closeToast } = useToast();

const fixerKey = ref(store.state.fixerApiKey);
const fixerProvider = ref(store.state.fixerProvider);

const fixerProviderOptions: SelectOption[] = [
  { value: 0, label: "fixer.io" },
  { value: 1, label: "apilayer.com" },
];

function saveFixerKey() {
  store.setFixerApiKey(fixerKey.value, fixerProvider.value);
  toast(t("success"));
}

// --- Auto-update settings ---
const autoUpdate = ref(store.state.settings.currencyAutoUpdate);
const updateTargets = ref<string[]>([...store.state.settings.currencyUpdateTargets]);

const targetCurrencyOptions = computed<SelectOption[]>(() =>
  store.state.currencies
    .filter((c) => c.id !== store.state.settings.mainCurrencyId)
    .map((c) => ({ value: c.id, label: `${c.name} (${c.code})` })),
);

function toggleAutoUpdate() {
  autoUpdate.value = !autoUpdate.value;
  store.updateSettings({ currencyAutoUpdate: autoUpdate.value });
}

function toggleTarget(curId: string) {
  const idx = updateTargets.value.indexOf(curId);
  if (idx >= 0) {
    updateTargets.value.splice(idx, 1);
  } else {
    updateTargets.value.push(curId);
  }
  store.updateSettings({ currencyUpdateTargets: [...updateTargets.value] });
}

function selectAllTargets() {
  updateTargets.value = targetCurrencyOptions.value.map((o) => String(o.value));
  store.updateSettings({ currencyUpdateTargets: [...updateTargets.value] });
}

function deselectAllTargets() {
  updateTargets.value = [];
  store.updateSettings({ currencyUpdateTargets: [] });
}

const lastUpdate = computed(() => store.state.settings.lastCurrencyUpdate || t("never"));

// --- Manual update ---
const isUpdating = ref(false);

async function manualUpdate() {
  if (!fixerKey.value) {
    toast(t("fixer_key_required"), "error");
    return;
  }
  isUpdating.value = true;
  try {
    const result = await updateCurrencyRates(
      fixerKey.value,
      fixerProvider.value,
      store.state.currencies,
      store.state.settings.mainCurrencyId,
      updateTargets.value,
    );
    if (result.error) {
      toast(result.error, "error");
    } else if (result.updated > 0) {
      store.updateSettings({ lastCurrencyUpdate: new Date().toISOString().split("T")[0] });
      toast(t("rates_updated").replace("{count}", String(result.updated)));
    } else {
      toast(t("no_rates_updated"));
    }
  } catch (e) {
    toast(String(e), "error");
  } finally {
    isUpdating.value = false;
  }
}
</script>

<template>
  <section class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-5">
    <h2 class="text-lg font-semibold text-[var(--color-text-primary)] mb-4">{{ t('fixer_api_key') }}</h2>
    <div class="space-y-3">
      <AppInput v-model="fixerKey" :placeholder="t('api_key')" />
      <AppSelect v-model="fixerProvider" :options="fixerProviderOptions" />
      <button @click="saveFixerKey" class="px-4 py-2 rounded-lg bg-[var(--color-primary)] text-white text-sm font-medium hover:bg-[var(--color-primary-hover)] transition-colors">{{ t('save') }}</button>
      <p class="text-xs text-[var(--color-text-muted)]">{{ t('fixer_info') }}</p>
    </div>

    <!-- Auto-update -->
    <div class="pt-4 mt-4 border-t border-[var(--color-border)]">
      <h3 class="text-sm font-semibold text-[var(--color-text-primary)] mb-3">{{ t('currency_auto_update') }}</h3>

      <AppToggle
        :modelValue="autoUpdate"
        @update:modelValue="toggleAutoUpdate"
        :label="t('auto_update_rates')"
        :description="t('auto_update_rates_info')"
      />

      <div class="mt-3 flex items-center gap-2 text-xs text-[var(--color-text-muted)]">
        <span>{{ t('last_update') }}: {{ lastUpdate }}</span>
        <button
          @click="manualUpdate"
          :disabled="isUpdating || !fixerKey"
          class="inline-flex items-center gap-1 px-2.5 py-1 rounded-md text-xs font-medium border border-[var(--color-border)] text-[var(--color-text-secondary)] hover:border-[var(--color-primary)] hover:text-[var(--color-primary)] disabled:opacity-30 transition-colors"
        >
          <RefreshCw :size="12" :class="{ 'animate-spin': isUpdating }" />
          {{ t('update_now') }}
        </button>
      </div>

      <!-- Target currencies -->
      <div v-if="autoUpdate" class="mt-4">
        <div class="flex items-center justify-between mb-2">
          <label class="text-xs font-medium text-[var(--color-text-secondary)]">{{ t('target_currencies') }}</label>
          <div class="flex gap-2">
            <button @click="selectAllTargets" class="text-[10px] text-[var(--color-primary)] hover:underline">{{ t('select_all') }}</button>
            <button @click="deselectAllTargets" class="text-[10px] text-[var(--color-text-muted)] hover:underline">{{ t('deselect_all') }}</button>
          </div>
        </div>
        <p class="text-[10px] text-[var(--color-text-muted)] mb-2">{{ t('target_currencies_info') }}</p>
        <div class="flex flex-wrap gap-1.5 max-h-32 overflow-auto">
          <button
            v-for="opt in targetCurrencyOptions"
            :key="String(opt.value)"
            @click="toggleTarget(String(opt.value))"
            class="px-2 py-1 rounded-md text-[11px] font-medium border transition-colors"
            :class="updateTargets.includes(String(opt.value))
              ? 'bg-[var(--color-primary-light)] border-[var(--color-primary)] text-[var(--color-primary)]'
              : 'bg-[var(--color-surface)] border-[var(--color-border)] text-[var(--color-text-muted)] hover:border-[var(--color-text-muted)]'"
          >
            {{ opt.label }}
          </button>
        </div>
      </div>
    </div>

    <Toast :show="showToast" :message="toastMsg" :type="toastType" @close="closeToast" />
  </section>
</template>
