<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { openUrl } from "@tauri-apps/plugin-opener";
import Modal from "@/components/ui/Modal.vue";
import { oauthPending, closeOAuthModal, disableProvider } from "@/services/syncClient";
import type { SyncProviderType } from "@/services/syncClient";

const props = defineProps<{
  providerLabel: (type: SyncProviderType) => string;
}>();

const { t } = useI18n();

async function openBrowser() {
  if (oauthPending.authUrl) {
    await openUrl(oauthPending.authUrl);
  }
}

async function cancelOAuth() {
  closeOAuthModal();
  await disableProvider();
}
</script>

<template>
  <Modal
    :show="oauthPending.open"
    :title="t('sync_oauth_pending_title')"
    maxWidth="24rem"
    @close="cancelOAuth"
  >
    <p class="text-sm text-text-primary mb-2">
      {{ t("sync_oauth_pending_body", { provider: oauthPending.provider ? props.providerLabel(oauthPending.provider) : "" }) }}
    </p>
    <p class="text-xs text-text-muted mb-4">{{ t("sync_oauth_pending_hint") }}</p>
    <div class="flex flex-col sm:flex-row gap-2 justify-end">
      <button
        type="button"
        class="px-3 py-2 rounded-lg text-xs border border-border order-2 sm:order-1"
        @click="cancelOAuth"
      >
        {{ t("sync_oauth_cancel") }}
      </button>
      <button
        type="button"
        class="px-3 py-2 rounded-lg text-xs bg-primary text-white order-1 sm:order-2"
        @click="openBrowser"
      >
        {{ t("sync_oauth_open_browser") }}
      </button>
    </div>
  </Modal>
</template>
