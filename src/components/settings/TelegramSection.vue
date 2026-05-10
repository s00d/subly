<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import AppInput from "@/components/ui/AppInput.vue";
import { ui } from "@/lib/tv";
import AppToggle from "@/components/ui/AppToggle.vue";
import { notificationsEvent } from "@/services/notificationsClient";
import { getConfigValue, setConfigValue } from "@/services/configClient";
import { getSecureValue, setSecureValue } from "@/services/secureStorageClient";
import { Send } from "@lucide/vue";
import { formatErrorForToast } from "@/utils/formatError";

const { t } = useI18n();
const { toast } = useToast();

const botToken = ref("");
const chatId = ref("");
const proxyUrl = ref("");
const enabled = ref(false);
const isTesting = ref(false);

onMounted(async () => {
  const [savedToken, savedChatId, savedProxyUrl, savedEnabled] = await Promise.all([
    getSecureValue("telegramBotToken"),
    getConfigValue<string>("telegramChatId"),
    getConfigValue<string>("telegramProxyUrl"),
    getConfigValue<boolean>("telegramEnabled"),
  ]);
  botToken.value = savedToken || "";
  chatId.value = savedChatId || "";
  proxyUrl.value = savedProxyUrl || "";
  enabled.value = !!savedEnabled;
});

async function saveTelegram() {
  await setSecureValue("telegramBotToken", botToken.value);
  await setConfigValue("telegramChatId", chatId.value);
  await setConfigValue("telegramProxyUrl", proxyUrl.value);
  await setConfigValue("telegramEnabled", enabled.value);
  toast(t("success"));
}

async function testTelegram() {
  if (!botToken.value || !chatId.value) {
    toast(t("telegram_fill_fields"), "error");
    return;
  }
  isTesting.value = true;
  try {
    const result = await notificationsEvent<{ system: boolean; sound: boolean; telegram: boolean }>("dispatch", {
      title: "Subly",
      body: "Subly test message",
      showSystem: false,
      playSound: false,
      sendTelegram: true,
      telegramBotToken: botToken.value,
      telegramChatId: chatId.value,
      telegramProxyBaseUrl: proxyUrl.value,
    });
    const ok = result.data.telegram;
    if (ok) toast(t("telegram_test_sent"));
    else toast(t("telegram_test_failed"), "error");
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  } finally {
    isTesting.value = false;
  }
}
</script>

<template>
  <section class="bg-surface rounded-xl border border-border p-3 sm:p-5">
    <h2 :class="[ui.sectionTitle(), 'mb-3 sm:mb-4']">{{ t('telegram_notifications') }}</h2>
    <div class="space-y-3">
      <AppToggle v-model="enabled" :label="t('telegram_enabled')" :description="t('telegram_enabled_info')" />
      <AppInput v-model="botToken" :label="t('telegram_bot_token')" :placeholder="t('telegram_bot_token_placeholder')" />
      <AppInput v-model="chatId" :label="t('telegram_chat_id')" :placeholder="t('telegram_chat_id_placeholder')" />
      <AppInput v-model="proxyUrl" label="Telegram Proxy URL" placeholder="https://your-proxy.example.com" />
      <div class="flex gap-2">
        <button @click="saveTelegram" class="px-4 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-primary-hover transition-colors">{{ t('save') }}</button>
        <button
          @click="testTelegram"
          :disabled="isTesting || !botToken || !chatId"
          class="flex items-center gap-1.5 px-4 py-2 rounded-lg border border-border text-sm font-medium text-text-secondary hover:border-primary hover:text-primary transition-colors disabled:opacity-50"
        >
          <Send :size="14" />
          {{ t('telegram_test') }}
        </button>
      </div>
      <p class="text-xs text-text-muted">{{ t('telegram_info') }}</p>
    </div>
  </section>
</template>
