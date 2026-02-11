<script setup lang="ts">
import { ref } from "vue";
import { useAppStore } from "@/stores/appStore";
import { useI18n } from "@/i18n";
import { useToast } from "@/composables/useToast";
import AppInput from "@/components/ui/AppInput.vue";
import AppToggle from "@/components/ui/AppToggle.vue";
import { sendTelegramTestMessage } from "@/services/telegram";
import { Send } from "lucide-vue-next";

const store = useAppStore();
const { t } = useI18n();
const { toast } = useToast();

const botToken = ref(store.state.telegramBotToken);
const chatId = ref(store.state.telegramChatId);
const enabled = ref(store.state.telegramEnabled);
const isTesting = ref(false);

function saveTelegram() {
  store.setTelegramConfig(botToken.value, chatId.value, enabled.value);
  toast(t("success"));
}

async function testTelegram() {
  if (!botToken.value || !chatId.value) {
    toast(t("telegram_fill_fields"), "error");
    return;
  }
  isTesting.value = true;
  try {
    const ok = await sendTelegramTestMessage({ botToken: botToken.value, chatId: chatId.value });
    if (ok) toast(t("telegram_test_sent"));
    else toast(t("telegram_test_failed"), "error");
  } catch {
    toast(t("telegram_test_failed"), "error");
  } finally {
    isTesting.value = false;
  }
}
</script>

<template>
  <section class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-5">
    <h2 class="text-lg font-semibold text-[var(--color-text-primary)] mb-4">{{ t('telegram_notifications') }}</h2>
    <div class="space-y-3">
      <AppToggle v-model="enabled" :label="t('telegram_enabled')" :description="t('telegram_enabled_info')" />
      <AppInput v-model="botToken" :label="t('telegram_bot_token')" :placeholder="t('telegram_bot_token_placeholder')" />
      <AppInput v-model="chatId" :label="t('telegram_chat_id')" :placeholder="t('telegram_chat_id_placeholder')" />
      <div class="flex gap-2">
        <button @click="saveTelegram" class="px-4 py-2 rounded-lg bg-[var(--color-primary)] text-white text-sm font-medium hover:bg-[var(--color-primary-hover)] transition-colors">{{ t('save') }}</button>
        <button
          @click="testTelegram"
          :disabled="isTesting || !botToken || !chatId"
          class="flex items-center gap-1.5 px-4 py-2 rounded-lg border border-[var(--color-border)] text-sm font-medium text-[var(--color-text-secondary)] hover:border-[var(--color-primary)] hover:text-[var(--color-primary)] transition-colors disabled:opacity-50"
        >
          <Send :size="14" />
          {{ t('telegram_test') }}
        </button>
      </div>
      <p class="text-xs text-[var(--color-text-muted)]">{{ t('telegram_info') }}</p>
    </div>
  </section>
</template>
