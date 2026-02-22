<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useSettingsStore } from "@/stores/settings";
import { useCatalogStore } from "@/stores/catalog";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import AppInput from "@/components/ui/AppInput.vue";
import AppToggle from "@/components/ui/AppToggle.vue";
import type { SelectOption } from "@/components/ui/AppSelect.vue";
import Toast from "@/components/ui/Toast.vue";
import { getRatesProviders, updateCurrencyRates } from "@/services/rates";
import type { RatesProviderType, RatesProviderMeta } from "@/services/rates";
import { RefreshCw, Check, ChevronDown, ExternalLink, ArrowRightLeft } from "lucide-vue-next";
import { tv } from "@/lib/tv";
import { openUrl } from "@tauri-apps/plugin-opener";

const settingsStore = useSettingsStore();
const catalogStore = useCatalogStore();
const { t } = useI18n();
const { toastMsg, toastType, showToast, toast, closeToast } = useToast();

const providers = getRatesProviders();
const selectedProvider = ref<RatesProviderType>(settingsStore.ratesProvider as RatesProviderType || "frankfurter");
const apiKey = ref(settingsStore.ratesApiKey);
const expandedProvider = ref<RatesProviderType | null>(null);
const showChangeProvider = ref(false);

onMounted(() => {
  selectedProvider.value = (settingsStore.ratesProvider as RatesProviderType) || "frankfurter";
  apiKey.value = settingsStore.ratesApiKey;
});

const activeProvider = computed<RatesProviderMeta | undefined>(() =>
  providers.find((p) => p.type === selectedProvider.value),
);

function selectProvider(type: RatesProviderType) {
  if (type === selectedProvider.value && !providers.find((p) => p.type === type)?.requiresKey) return;

  const provider = providers.find((p) => p.type === type);
  if (!provider) return;

  if (!provider.requiresKey) {
    selectedProvider.value = type;
    apiKey.value = "";
    settingsStore.setRatesConfig("", type);
    toast(t("success"));
    expandedProvider.value = null;
    showChangeProvider.value = false;
    return;
  }

  expandedProvider.value = expandedProvider.value === type ? null : type;
}

function saveProviderConfig(type: RatesProviderType) {
  selectedProvider.value = type;
  settingsStore.setRatesConfig(apiKey.value, type);
  expandedProvider.value = null;
  showChangeProvider.value = false;
  toast(t("success"));
}

const otherProviders = computed(() =>
  providers.filter((p) => p.type !== selectedProvider.value),
);

const autoUpdate = ref(settingsStore.settings.currencyAutoUpdate);
const updateTargets = ref<string[]>([...settingsStore.settings.currencyUpdateTargets]);

const targetCurrencyOptions = computed<SelectOption[]>(() =>
  catalogStore.currencies
    .filter((c) => c.id !== settingsStore.settings.mainCurrencyId)
    .map((c) => ({ value: c.id, label: `${c.name} (${c.code})` })),
);

function toggleAutoUpdate() {
  autoUpdate.value = !autoUpdate.value;
  settingsStore.updateSettings({ currencyAutoUpdate: autoUpdate.value });
}

function toggleTarget(curId: string) {
  const idx = updateTargets.value.indexOf(curId);
  if (idx >= 0) {
    updateTargets.value.splice(idx, 1);
  } else {
    updateTargets.value.push(curId);
  }
  settingsStore.updateSettings({ currencyUpdateTargets: [...updateTargets.value] });
}

function selectAllTargets() {
  updateTargets.value = targetCurrencyOptions.value.map((o) => String(o.value));
  settingsStore.updateSettings({ currencyUpdateTargets: [...updateTargets.value] });
}

function deselectAllTargets() {
  updateTargets.value = [];
  settingsStore.updateSettings({ currencyUpdateTargets: [] });
}

const lastUpdate = computed(() => settingsStore.settings.lastCurrencyUpdate || t("never"));

const isUpdating = ref(false);

async function manualUpdate() {
  const provider = activeProvider.value;
  if (!provider) return;
  if (provider.requiresKey && !apiKey.value) {
    toast(t("rates_key_required"), "error");
    return;
  }
  isUpdating.value = true;
  try {
    const result = await updateCurrencyRates(
      selectedProvider.value,
      apiKey.value,
      catalogStore.currencies,
      settingsStore.settings.mainCurrencyId,
      updateTargets.value,
      {
        historyEnabled: settingsStore.settings.rateHistoryEnabled,
        historyDays: settingsStore.settings.rateHistoryDays,
      },
    );
    if (result.error) {
      toast(result.error, "error");
    } else if (result.updated > 0) {
      settingsStore.updateSettings({ lastCurrencyUpdate: new Date().toISOString().split("T")[0] });
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

const sectionTv = tv({
  slots: {
    root: "bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-4 sm:p-5",
    header: "flex items-center gap-2 mb-1",
    title: "text-base sm:text-lg font-semibold text-[var(--color-text-primary)]",
    desc: "text-xs sm:text-sm text-[var(--color-text-muted)] mb-4",
    providerCard: "rounded-xl border overflow-hidden transition-all cursor-pointer select-none",
    providerRow: "flex items-center gap-3 p-3",
    providerName: "text-sm font-medium text-[var(--color-text-primary)]",
    providerNote: "text-[10px] text-[var(--color-text-muted)]",
    providerChevron: "text-[var(--color-text-muted)] transition-transform duration-200 shrink-0",
    credForm: "px-3 pb-3 pt-0",
    credFormInner: "space-y-2.5 p-3 rounded-lg bg-[var(--color-surface-secondary)]",
    credLabel: "block text-[10px] font-medium text-[var(--color-text-muted)] mb-1",
    credInput: [
      "w-full px-2.5 py-1.5 rounded-lg border border-[var(--color-border)]",
      "bg-[var(--color-surface)] text-xs text-[var(--color-text-primary)]",
      "focus:outline-none focus:ring-1 focus:ring-[var(--color-primary)]",
    ],
    saveBtn: [
      "flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors disabled:opacity-50",
      "bg-[var(--color-primary)] text-white hover:bg-[var(--color-primary-hover)]",
    ],
    linkBtn: "inline-flex items-center gap-1 text-[10px] text-[var(--color-primary)] hover:underline",
    divider: "pt-4 mt-4 border-t border-[var(--color-border)]",
  },
});

const s = sectionTv();
</script>

<template>
  <section :class="s.root()">
    <div :class="s.header()">
      <ArrowRightLeft :size="18" class="text-[var(--color-primary)]" />
      <h2 :class="s.title()">{{ t('exchange_rates') }}</h2>
    </div>
    <p :class="s.desc()">{{ t('exchange_rates_desc') }}</p>

    <!-- Active provider card -->
    <div v-if="activeProvider" class="rounded-xl border border-[var(--color-primary)] bg-[var(--color-primary-light)] p-3 mb-3">
      <div class="flex items-center gap-3">
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2">
            <p class="text-sm font-medium text-[var(--color-text-primary)]">{{ activeProvider.name }}</p>
            <span class="flex items-center gap-0.5 px-1.5 py-0.5 rounded-full bg-green-100 dark:bg-green-900/30 text-green-600 dark:text-green-400 text-[9px] font-medium">
              <Check :size="9" />
              {{ t('active') }}
            </span>
          </div>
          <p class="text-[10px] text-[var(--color-text-muted)]">{{ activeProvider.freeTierNote }}</p>
        </div>
        <button
          @click.stop="openUrl(activeProvider.url)"
          class="p-1.5 rounded-md text-[var(--color-text-muted)] hover:text-[var(--color-primary)] transition-colors shrink-0"
        >
          <ExternalLink :size="13" />
        </button>
      </div>
      <!-- Change provider button -->
      <div class="flex items-center gap-2 mt-2 pt-2 border-t border-[var(--color-border)]">
        <button
          @click="showChangeProvider = !showChangeProvider"
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)] border border-[var(--color-border)] bg-[var(--color-surface)]"
        >
          <RefreshCw :size="12" />
          {{ t('sync_change_provider') }}
          <ChevronDown :size="12" class="transition-transform duration-200" :style="{ transform: showChangeProvider ? 'rotate(180deg)' : '' }" />
        </button>
      </div>
    </div>

    <!-- Other providers (collapsible when active provider exists) -->
    <Transition
      enter-active-class="transition-all duration-200 ease-out"
      enter-from-class="max-h-0 opacity-0"
      enter-to-class="max-h-[1000px] opacity-100"
      leave-active-class="transition-all duration-150 ease-in"
      leave-from-class="max-h-[1000px] opacity-100"
      leave-to-class="max-h-0 opacity-0"
    >
      <div class="space-y-2 overflow-hidden" v-if="activeProvider ? showChangeProvider : true">
        <div
          v-for="provider in (activeProvider ? otherProviders : providers)"
          :key="provider.type"
          :class="s.providerCard()"
          class="border-[var(--color-border)]"
        >
          <div :class="s.providerRow()" @click="selectProvider(provider.type)">
            <div class="flex-1 min-w-0">
              <p :class="s.providerName()">{{ provider.name }}</p>
              <p :class="s.providerNote()">{{ provider.freeTierNote }}</p>
            </div>
            <button
              @click.stop="openUrl(provider.url)"
              class="p-1.5 rounded-md text-[var(--color-text-muted)] hover:text-[var(--color-primary)] transition-colors"
            >
              <ExternalLink :size="13" />
            </button>
            <ChevronDown
              v-if="provider.requiresKey"
              :size="16"
              :class="s.providerChevron()"
              :style="{ transform: expandedProvider === provider.type ? 'rotate(180deg)' : '' }"
            />
          </div>

          <!-- API key form -->
          <Transition
            enter-active-class="transition-all duration-200 ease-out"
            enter-from-class="max-h-0 opacity-0"
            enter-to-class="max-h-48 opacity-100"
            leave-active-class="transition-all duration-150 ease-in"
            leave-from-class="max-h-48 opacity-100"
            leave-to-class="max-h-0 opacity-0"
          >
            <div v-if="expandedProvider === provider.type && provider.requiresKey" class="overflow-hidden">
              <div :class="s.credForm()">
                <div :class="s.credFormInner()">
                  <div>
                    <label :class="s.credLabel()">{{ t('api_key') }}</label>
                    <input v-model="apiKey" type="password" :class="s.credInput()" :placeholder="t('api_key')" />
                  </div>
                  <div class="flex items-center gap-2">
                    <button @click.stop="saveProviderConfig(provider.type)" :disabled="!apiKey" :class="s.saveBtn()">
                      {{ t('sync_connect') }}
                    </button>
                    <button @click.stop="openUrl(provider.url)" :class="s.linkBtn()">
                      {{ t('rates_get_key') }} <ExternalLink :size="10" />
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </Transition>
        </div>
      </div>
    </Transition>

    <!-- Target currencies -->
    <div :class="s.divider()">
      <div class="flex items-center justify-between mb-2">
        <label class="text-sm font-semibold text-[var(--color-text-primary)]">{{ t('target_currencies') }}</label>
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

    <!-- Auto-update settings -->
    <div :class="s.divider()">
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
          :disabled="isUpdating"
          class="inline-flex items-center gap-1 px-2.5 py-1 rounded-md text-xs font-medium border border-[var(--color-border)] text-[var(--color-text-secondary)] hover:border-[var(--color-primary)] hover:text-[var(--color-primary)] disabled:opacity-30 transition-colors"
        >
          <RefreshCw :size="12" :class="{ 'animate-spin': isUpdating }" />
          {{ t('update_now') }}
        </button>
      </div>
    </div>

    <Toast :show="showToast" :message="toastMsg" :type="toastType" @close="closeToast" />
  </section>
</template>
