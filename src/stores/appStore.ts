import { defineStore } from "pinia";
import { ref } from "vue";
import type { AppData } from "@/schemas/appData";
import { loadAppData, saveAppData } from "@/services/database";
import { setLanguage } from "@/i18n";
import { useSubscriptionsStore } from "./subscriptions";
import { useExpensesStore } from "./expenses";
import { useCatalogStore } from "./catalog";
import { useSettingsStore } from "./settings";

export const useAppStore = defineStore("app", () => {
  const isLoading = ref(true);

  async function init() {
    isLoading.value = true;
    try {
      const data = await loadAppData();
      useSubscriptionsStore().$hydrate(data);
      useExpensesStore().$hydrate(data);
      useCatalogStore().$hydrate(data);
      useSettingsStore().$hydrate(data);

      const lang = useSettingsStore().settings.language;
      await setLanguage(lang);
      if (lang !== "en") {
        useCatalogStore().retranslateDefaults();
      }

      // load first page of expenses from SQL
      await useExpensesStore().fetchPage(1);
    } catch (e) {
      console.error("Failed to load data:", e);
    } finally {
      isLoading.value = false;
    }
  }

  async function importData(data: AppData) {
    await saveAppData(data);
    useSubscriptionsStore().$hydrate(data);
    useExpensesStore().$hydrate(data);
    useCatalogStore().$hydrate(data);
    useSettingsStore().$hydrate(data);
    await setLanguage(useSettingsStore().settings.language);
    await useExpensesStore().fetchPage(1);
  }

  function getExportData(): AppData {
    const subs = useSubscriptionsStore();
    const catalog = useCatalogStore();
    const stgs = useSettingsStore();
    const exps = useExpensesStore();

    return {
      subscriptions: [...subs.subscriptions],
      expenses: [...exps.items],
      categories: [...catalog.categories],
      currencies: [...catalog.currencies],
      household: [...catalog.household],
      paymentMethods: [...catalog.paymentMethods],
      tags: [...catalog.tags],
      settings: { ...stgs.settings },
      ratesApiKey: stgs.ratesApiKey,
      ratesProvider: stgs.ratesProvider,
      fixerApiKey: stgs.fixerApiKey,
      fixerProvider: stgs.fixerProvider,
      telegramBotToken: stgs.telegramBotToken,
      telegramChatId: stgs.telegramChatId,
      telegramEnabled: stgs.telegramEnabled,
      initialized: true,
    };
  }

  return {
    isLoading,
    init,
    importData,
    getExportData,
  };
});
