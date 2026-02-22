import { defineStore } from "pinia";
import { ref } from "vue";
import type { Settings, AppData } from "@/schemas/appData";
import { SettingsSchema, parseSettings } from "@/schemas/appData";
import { setLanguage } from "@/i18n";
import { setConfigValue } from "@/services/database";
import { useCatalogStore } from "./catalog";

export const useSettingsStore = defineStore("settings", () => {
  const settings = ref<Settings>(SettingsSchema.parse({}));
  const ratesApiKey = ref("");
  const ratesProvider = ref("frankfurter");
  const fixerApiKey = ref("");
  const fixerProvider = ref(0);
  const telegramBotToken = ref("");
  const telegramChatId = ref("");
  const telegramEnabled = ref(false);

  function $hydrate(data: Partial<AppData>) {
    if (data.settings) settings.value = { ...data.settings };
    if (data.ratesApiKey !== undefined) ratesApiKey.value = data.ratesApiKey;
    else if (data.fixerApiKey) ratesApiKey.value = data.fixerApiKey;
    if (data.ratesProvider !== undefined) ratesProvider.value = data.ratesProvider;
    else if (data.fixerProvider !== undefined) ratesProvider.value = data.fixerProvider === 1 ? "apilayer" : "fixer";
    if (data.fixerApiKey !== undefined) fixerApiKey.value = data.fixerApiKey;
    if (data.fixerProvider !== undefined) fixerProvider.value = data.fixerProvider;
    if (data.telegramBotToken !== undefined) telegramBotToken.value = data.telegramBotToken;
    if (data.telegramChatId !== undefined) telegramChatId.value = data.telegramChatId;
    if (data.telegramEnabled !== undefined) telegramEnabled.value = data.telegramEnabled;
  }

  async function updateSettings(updates: Partial<Settings>, opts?: { skipRetranslate?: boolean }) {
    const merged = parseSettings({ ...settings.value, ...updates });
    Object.assign(settings.value, merged);
    if (updates.language) {
      await setLanguage(updates.language);
      if (!opts?.skipRetranslate) {
        useCatalogStore().retranslateDefaults();
      }
    }
    await setConfigValue("settings", settings.value);
  }

  async function setRatesConfig(key: string, provider: string) {
    ratesApiKey.value = key;
    ratesProvider.value = provider;
    await setConfigValue("ratesApiKey", key);
    await setConfigValue("ratesProvider", provider);
  }

  async function setFixerApiKey(key: string, provider: number) {
    fixerApiKey.value = key;
    fixerProvider.value = provider;
    await setConfigValue("fixerApiKey", key);
    await setConfigValue("fixerProvider", String(provider));
  }

  async function setTelegramConfig(botToken: string, chatId: string, enabled: boolean) {
    telegramBotToken.value = botToken;
    telegramChatId.value = chatId;
    telegramEnabled.value = enabled;
    await setConfigValue("telegramBotToken", botToken);
    await setConfigValue("telegramChatId", chatId);
    await setConfigValue("telegramEnabled", enabled ? "1" : "0");
  }

  return {
    settings, ratesApiKey, ratesProvider, fixerApiKey, fixerProvider,
    telegramBotToken, telegramChatId, telegramEnabled,
    $hydrate, updateSettings, setRatesConfig, setFixerApiKey, setTelegramConfig,
  };
});
