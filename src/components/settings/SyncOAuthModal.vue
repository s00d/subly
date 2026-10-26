<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { openUrl } from "@tauri-apps/plugin-opener";
import Modal from "@/components/ui/Modal.vue";
import { button } from "@/lib/tv";
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
    <p class="text-xs text-text-muted">{{ t("sync_oauth_pending_hint") }}</p>
    <template #footer>
      <button type="button" :class="button()" @click="cancelOAuth">
        {{ t("sync_oauth_cancel") }}
      </button>
      <button type="button" :class="button({ intent: 'primary' })" @click="openBrowser">
        {{ t("sync_oauth_open_browser") }}
      </button>
    </template>
  </Modal>
</template>
