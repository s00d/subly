<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import AppInput from "@/components/ui/AppInput.vue";
import AppToggle from "@/components/ui/AppToggle.vue";
import type { SelectOption } from "@/components/ui/AppSelect.vue";
import Toast from "@/components/ui/Toast.vue";
import { ratesGetProviders, ratesRunBackendUpdate } from "@/services/ratesClient";
import type { RatesProviderType, RatesProviderMeta } from "@/services/ratesClient";
import { getConfigValue, setConfigValue } from "@/services/configClient";
import { getSecureValue, setSecureValue } from "@/services/secureStorageClient";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { Currency, Settings } from "@/schemas/appData";
import { useAppMetaStore } from "@/stores/appMetaStore";
import { RefreshCw, Check, CheckSquare, Square, ChevronDown, ArrowRightLeft, Search } from "@lucide/vue";
import { tv, ui } from "@/lib/tv";
import { formatErrorForToast } from "@/utils/formatError";

const props = defineProps<{
  lookupData: {
    settings: Settings;
    currencies: Currency[];
  } | null;
}>();
const { t } = useI18n();
const { toastMsg, toastType, showToast, toast, closeToast } = useToast();
const metaStore = useAppMetaStore();
const settings = ref<Settings | null>(null);
const currencies = ref<Currency[]>([]);

const providers = ref<RatesProviderMeta[]>([]);
const selectedProvider = ref<RatesProviderType>("frankfurter");
const apiKey = ref("");
const expandedProvider = ref<RatesProviderType | null>(null);
const showChangeProvider = ref(false);
const targetSearch = ref("");

async function updateSettings(updates: Partial<Settings>) {
  if (!settings.value) return;
  const next = { ...settings.value, ...updates };
  settings.value = next;
  await metaStore.updateSettings(next);
}

onMounted(async () => {
  settings.value = props.lookupData?.settings ?? null;
  currencies.value = props.lookupData?.currencies ?? [];
  selectedProvider.value = ((await getConfigValue<string>("ratesProvider")) || "frankfurter") as RatesProviderType;
  apiKey.value = (await getSecureValue("ratesApiKey")) || "";
  ratesGetProviders()
    .then((items) => { providers.value = items; })
    .catch((e) => {
      providers.value = [];
      console.error("[ExchangeRatesSection] ratesGetProviders failed", e);
      toast(formatErrorForToast(e, t), "error");
    });
});

const activeProvider = computed<RatesProviderMeta | undefined>(() =>
  providers.value.find((p) => p.type === selectedProvider.value),
);

async function selectProvider(type: RatesProviderType) {
  if (type === selectedProvider.value && !providers.value.find((p) => p.type === type)?.requiresKey) return;

  const provider = providers.value.find((p) => p.type === type);
  if (!provider) return;

  if (!provider.requiresKey) {
    try {
      selectedProvider.value = type;
      await setConfigValue("ratesProvider", type);
      toast(t("success"));
      expandedProvider.value = null;
      showChangeProvider.value = false;
    } catch (e) {
      toast(formatErrorForToast(e, t), "error");
    }
    return;
  }

  expandedProvider.value = expandedProvider.value === type ? null : type;
}

async function saveProviderConfig(type: RatesProviderType) {
  try {
    selectedProvider.value = type;
    await setSecureValue("ratesApiKey", apiKey.value);
    await setConfigValue("ratesProvider", type);
    expandedProvider.value = null;
    showChangeProvider.value = false;
    toast(t("success"));
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  }
}

const otherProviders = computed(() =>
  providers.value.filter((p) => p.type !== selectedProvider.value),
);

const autoUpdate = computed(() => Boolean(settings.value?.currencyAutoUpdate));
const updateTargets = computed(() => settings.value?.currencyUpdateTargets ?? []);

type TargetCurrencyOption = SelectOption & { searchText: string };

const targetCurrencyOptions = computed<TargetCurrencyOption[]>(() => {
  const mainId = settings.value?.mainCurrencyId ?? "";
  const mainCode = currencies.value.find((c) => c.id === mainId)?.code?.trim().toUpperCase() ?? "";
  const seenCodes = new Set<string>();
  const out: TargetCurrencyOption[] = [];

  for (const currency of currencies.value) {
    const id = String(currency.id ?? "").trim();
    const code = String(currency.code ?? "").trim().toUpperCase();
    if (!id || !code) continue;
    if (id === mainId || code === mainCode) continue;
    if (seenCodes.has(code)) continue;
    seenCodes.add(code);
    out.push({
      value: id,
      label: `${currency.name} (${code})`,
      searchText: `${currency.name} ${code} ${currency.symbol || ""} ${id}`.toLowerCase(),
    });
  }

  return out;
});

const filteredTargetCurrencyOptions = computed<SelectOption[]>(() => {
  const q = targetSearch.value.trim().toLowerCase();
  if (!q) return targetCurrencyOptions.value;
  return targetCurrencyOptions.value.filter((opt) =>
    opt.searchText.includes(q) || opt.label.toLowerCase().includes(q),
  );
});

function toggleAutoUpdate() {
  updateSettings({ currencyAutoUpdate: !autoUpdate.value });
}

function toggleTarget(curId: string) {
  const current = [...updateTargets.value];
  const idx = current.indexOf(curId);
  if (idx >= 0) {
    current.splice(idx, 1);
  } else {
    current.push(curId);
  }
  updateSettings({ currencyUpdateTargets: current });
}

function selectAllTargets() {
  updateSettings({ currencyUpdateTargets: targetCurrencyOptions.value.map((o) => String(o.value)) });
}

function deselectAllTargets() {
  updateSettings({ currencyUpdateTargets: [] });
}

const lastUpdate = computed(() => settings.value?.lastCurrencyUpdate || t("never"));

const isUpdating = ref(false);

async function manualUpdate() {
  isUpdating.value = true;
  try {
    if (activeProvider.value?.requiresKey) {
      const key = apiKey.value.trim();
      if (!key) {
        toast(t("rates_key_required"), "error");
        return;
      }
      // Keep backend keyring/config in sync before manual update.
      await setSecureValue("ratesApiKey", key);
      await setConfigValue("ratesProvider", selectedProvider.value);
      apiKey.value = key;
    }

    console.log("[ExchangeRatesSection] manualUpdate start", {
      provider: selectedProvider.value,
      activeProvider: activeProvider.value?.type,
      mainCurrencyId: settings.value?.mainCurrencyId,
      targetCount: settings.value?.currencyUpdateTargets?.length ?? 0,
    });
    const result = await ratesRunBackendUpdate();
    console.log("[ExchangeRatesSection] manualUpdate backend result", result);
    await metaStore.refresh();
    settings.value = metaStore.settings;
    currencies.value = metaStore.currencies;
    console.log("[ExchangeRatesSection] manualUpdate post-refresh", {
      lastCurrencyUpdate: settings.value?.lastCurrencyUpdate,
      currencyCount: currencies.value.length,
    });
    if (result.error) {
      toast(result.error, "error");
    } else if (result.updated > 0) {
      toast(t("rates_updated").replace("{count}", String(result.updated)));
    } else {
      toast(t("no_rates_updated"), "error");
    }
  } catch (e) {
    console.error("[ExchangeRatesSection] manualUpdate failed", e);
    toast(formatErrorForToast(e, t), "error");
  } finally {
    isUpdating.value = false;
  }
}

const sectionTv = tv({
  slots: {
    root: "bg-surface rounded-xl border border-border p-4 sm:p-5",
    header: "flex items-center gap-2 mb-1",
    title: ui.sectionTitle(),
    desc: "text-xs sm:text-sm text-text-muted mb-4",
    providerCard: "rounded-xl border overflow-hidden transition-all cursor-pointer select-none",
    providerRow: "flex items-center gap-3 p-3",
    providerName: "text-sm font-medium text-text-primary",
    providerNote: "text-[10px] text-text-muted",
    providerChevron: "text-text-muted transition-transform duration-200 shrink-0",
    credForm: "px-3 pb-3 pt-0",
    credFormInner: "space-y-2.5 p-3 rounded-lg bg-surface-secondary",
    credLabel: "block text-[10px] font-medium text-text-muted mb-1",
    credInput: [
      "w-full px-2.5 py-1.5 rounded-lg border border-border",
      "bg-surface text-xs text-text-primary",
      "focus:outline-none focus:ring-1 focus:ring-primary",
    ],
    saveBtn: [
      "flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors disabled:opacity-50",
      "bg-primary text-white hover:bg-primary-hover",
    ],
    linkBtn: "inline-flex items-center gap-1 text-[10px] text-primary hover:underline",
    divider: "pt-4 mt-4 border-t border-border",
  },
});

const s = sectionTv();

const providerDocs: Record<RatesProviderType, string> = {
  frankfurter: "https://www.frankfurter.app/",
  currencyapi: "https://currencyapi.com/",
  apilayer: "https://apilayer.com/",
  fixer: "https://fixer.io/",
  openexchangerates: "https://openexchangerates.org/",
  exchangerate: "https://www.exchangerate-api.com/",
};

const currentProviderDocsUrl = computed(() => providerDocs[selectedProvider.value] ?? providerDocs.frankfurter);

function providerDocsUrl(type: RatesProviderType): string {
  return providerDocs[type] ?? providerDocs.frankfurter;
}

async function openProviderDocs(type: RatesProviderType) {
  try {
    await openUrl(providerDocsUrl(type));
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  }
}
</script>

<template>
  <section :class="s.root()">
    <div :class="s.header()">
      <ArrowRightLeft :size="18" class="text-primary" />
      <h2 :class="s.title()">{{ t('exchange_rates') }}</h2>
    </div>
    <p :class="s.desc()">{{ t('exchange_rates_desc') }}</p>
    <p class="text-[11px] text-text-muted mb-4">
      {{ t("exchange_rates_links_desc") }}
      <button
        type="button"
        class="text-primary hover:underline"
        @click="openProviderDocs(selectedProvider)"
      >
        {{ activeProvider?.name || selectedProvider }}
      </button>
    </p>

    <!-- Active provider card -->
    <div v-if="activeProvider" class="rounded-xl border border-primary bg-primary-light p-3 mb-3">
      <div class="flex items-center gap-3">
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2">
            <p class="text-sm font-medium text-text-primary">{{ activeProvider.name }}</p>
            <span class="flex items-center gap-0.5 px-1.5 py-0.5 rounded-full bg-green-100 dark:bg-green-900/30 text-green-600 dark:text-green-400 text-[9px] font-medium">
              <Check :size="9" />
              {{ t('active') }}
            </span>
          </div>
          <p class="text-[10px] text-text-muted">{{ activeProvider.freeTierNote }}</p>
        </div>
      </div>
      <!-- Change provider button -->
      <div class="flex items-center gap-2 mt-2 pt-2 border-t border-border">
        <button
          @click="showChangeProvider = !showChangeProvider"
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors text-text-secondary hover:bg-surface-hover border border-border bg-surface"
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
          class="border-border"
        >
          <div :class="s.providerRow()" @click="selectProvider(provider.type)">
            <div class="flex-1 min-w-0">
              <p :class="s.providerName()">{{ provider.name }}</p>
              <p :class="s.providerNote()">{{ provider.freeTierNote }}</p>
              <button
                v-if="provider.requiresKey"
                type="button"
                class="inline-flex items-center mt-1 text-[10px] text-primary hover:underline"
                @click.stop="openProviderDocs(provider.type)"
              >
                {{ t('open_url') }} (API key)
              </button>
            </div>
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
                  </div>
                </div>
              </div>
            </div>
          </Transition>
        </div>
      </div>
    </Transition>

    <!-- Active provider credentials -->
    <div
      v-if="activeProvider"
      class="rounded-xl border border-border bg-surface p-3 mb-3"
    >
      <p class="text-xs font-medium text-text-primary mb-2">
        {{ t('api_key') }} — {{ activeProvider.name }}
      </p>
      <button
        v-if="activeProvider.requiresKey"
        type="button"
        class="inline-flex items-center mb-2 text-[11px] text-primary hover:underline"
        @click="openProviderDocs(activeProvider.type)"
      >
        {{ t('open_url') }} (API key)
      </button>
      <div class="flex flex-col sm:flex-row gap-2">
        <input
          v-model="apiKey"
          type="password"
          :class="[s.credInput(), 'flex-1']"
          :placeholder="t('api_key')"
        />
        <button
          @click="saveProviderConfig(activeProvider.type)"
          :class="s.saveBtn()"
        >
          {{ t('save') }}
        </button>
      </div>
    </div>

    <!-- Target currencies -->
    <div :class="s.divider()">
      <div class="flex items-center justify-between mb-2">
        <label class="text-sm font-semibold text-text-primary">{{ t('target_currencies') }}</label>
        <div class="flex gap-2">
          <button @click="selectAllTargets" class="p-1 rounded text-primary hover:bg-primary/10 transition-colors" :title="t('select_all')">
            <CheckSquare :size="13" />
          </button>
          <button @click="deselectAllTargets" class="p-1 rounded text-text-muted hover:bg-surface-hover transition-colors" :title="t('deselect_all')">
            <Square :size="13" />
          </button>
        </div>
      </div>
      <p class="text-[10px] text-text-muted mb-2">{{ t('target_currencies_info') }}</p>
      <div class="relative mb-2">
        <Search :size="13" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-text-muted" />
        <input
          v-model="targetSearch"
          type="text"
          :placeholder="t('search')"
          class="w-full pl-8 pr-3 py-1.5 rounded-lg border border-border bg-surface text-xs text-text-primary placeholder-text-muted focus:outline-none focus:ring-2 focus:ring-primary transition-shadow"
        />
      </div>
      <div class="flex flex-wrap gap-1.5 max-h-32 overflow-auto">
        <button
          v-for="opt in filteredTargetCurrencyOptions"
          :key="String(opt.value)"
          @click="toggleTarget(String(opt.value))"
          class="px-2 py-1 rounded-md text-[11px] font-medium border transition-colors"
          :class="updateTargets.includes(String(opt.value))
            ? 'bg-primary-light border-primary text-primary'
            : 'bg-surface border-border text-text-muted hover:border-text-muted'"
        >
          {{ opt.label }}
        </button>
      </div>
    </div>

    <!-- Auto-update settings -->
    <div :class="s.divider()">
      <h3 class="text-sm font-semibold text-text-primary mb-3">{{ t('currency_auto_update') }}</h3>

      <AppToggle
        :modelValue="autoUpdate"
        @update:modelValue="toggleAutoUpdate"
        :label="t('auto_update_rates')"
        :description="t('auto_update_rates_info')"
      />

      <div class="mt-3 flex items-center gap-2 text-xs text-text-muted">
        <span>{{ t('last_update') }}: {{ lastUpdate }}</span>
        <button
          @click="manualUpdate"
          :disabled="isUpdating"
          class="inline-flex items-center gap-1 px-2.5 py-1 rounded-md text-xs font-medium border border-border text-text-secondary hover:border-primary hover:text-primary disabled:opacity-30 transition-colors"
        >
          <RefreshCw :size="12" :class="{ 'animate-spin': isUpdating }" />
          {{ t('update_now') }}
        </button>
      </div>
    </div>

    <Toast :show="showToast" :message="toastMsg" :type="toastType" @close="closeToast" />
  </section>
</template>
