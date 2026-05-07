import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type {
  Category,
  Currency,
  HouseholdMember,
  PaymentMethod,
  ConfigSettings,
  Tag,
} from "@/schemas/appData";
import { loadCatalogs } from "@/services/catalogClient";
import { getConfigValue, setConfigValue } from "@/services/configClient";

export const useAppMetaStore = defineStore("appMeta", () => {
  const settings = ref<ConfigSettings | null>(null);
  const categories = ref<Category[]>([]);
  const currencies = ref<Currency[]>([]);
  const paymentMethods = ref<PaymentMethod[]>([]);
  const household = ref<HouseholdMember[]>([]);
  const tags = ref<Tag[]>([]);
  const initialized = ref(false);
  const loading = ref(false);

  async function fetchMeta() {
    loading.value = true;
    try {
      const [catalogs, savedSettings] = await Promise.all([
        loadCatalogs(),
        getConfigValue<ConfigSettings>("settings"),
      ]);
      categories.value = catalogs.categories;
      currencies.value = catalogs.currencies;
      paymentMethods.value = catalogs.paymentMethods;
      household.value = catalogs.household;
      tags.value = catalogs.tags;
      settings.value = savedSettings;
      initialized.value = true;
    } finally {
      loading.value = false;
    }
  }

  async function ensureLoaded() {
    if (initialized.value || loading.value) return;
    await fetchMeta();
  }

  async function refresh() {
    if (loading.value) return;
    initialized.value = false;
    await fetchMeta();
  }

  async function updateSettings(patch: Partial<ConfigSettings>) {
    if (!settings.value) {
      await ensureLoaded();
    }
    const next = { ...(settings.value || ({} as ConfigSettings)), ...patch } as ConfigSettings;
    settings.value = next;
    await setConfigValue("settings", next);
  }

  const hasCoreMeta = computed(
    () => categories.value.length > 0 || currencies.value.length > 0 || paymentMethods.value.length > 0,
  );

  return {
    settings,
    categories,
    currencies,
    paymentMethods,
    household,
    tags,
    initialized,
    loading,
    hasCoreMeta,
    ensureLoaded,
    refresh,
    updateSettings,
  };
});

