<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, reactive, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import {
  syncStatus,
  oauthPending,
  openOAuthModal,
  getProviders,
  getSyncSettings,
  saveProviderSettings,
  enableProvider,
  disableProvider,
  pullRemote,
  pushLocal,
  pushLocalForce,
  checkRemote,
  dismissPendingUpdate,
  testWebdavConnection,
} from "@/services/syncClient";
import type { SyncProviderType, SyncProviderSchema } from "@/services/syncClient";
import Modal from "@/components/ui/Modal.vue";
import UiCard from "@/components/ui/UiCard.vue";
import ConfirmDialog from "@/components/ui/ConfirmDialog.vue";
import { getLocaleTag } from "@/composables/useLocaleFormat";
import SyncOAuthModal from "@/components/settings/SyncOAuthModal.vue";
import SecretInput from "@/components/ui/SecretInput.vue";
import { ChevronRight, Cloud, CloudOff, Download, RefreshCw, Save, Upload } from "@lucide/vue";
import { ui, button } from "@/lib/tv";
import { formatErrorForToast } from "@/utils/formatError";
import { buildSyncCredentialsSchema } from "@/schemas/zod/syncCredentials";
import { zodIssueToMessageKey, type ZodFieldMeta } from "@/composables/useZodErrors";

const { t, locale } = useI18n();
const { toast } = useToast();
const isChecking = ref(false);
const confirmAction = ref<"disconnect" | "pull" | null>(null);

// "Last synced" label: relative time, re-rendered every 30s.
const now = ref(Date.now());
const nowTimer = setInterval(() => (now.value = Date.now()), 30_000);
onBeforeUnmount(() => clearInterval(nowTimer));
const lastSyncedLabel = computed(() => {
  const ts = syncStatus.lastSynced;
  if (!ts) return t("sync_never");
  const sec = Math.round((ts - now.value) / 1000);
  const abs = Math.abs(sec);
  const rtf = new Intl.RelativeTimeFormat(getLocaleTag(locale.value), { numeric: "auto" });
  if (abs < 60) return rtf.format(0, "second");
  if (abs < 3600) return rtf.format(Math.round(sec / 60), "minute");
  if (abs < 86400) return rtf.format(Math.round(sec / 3600), "hour");
  return rtf.format(Math.round(sec / 86400), "day");
});
const allProviders = computed(() => getProviders());
const expandedProvider = ref<SyncProviderType | null>(null);
const isConnecting = ref(false);
const isSaving = ref<SyncProviderType | null>(null);
const formValues = reactive<Record<string, Record<string, string>>>({});
const fieldErrors = reactive<Record<string, string>>({});
const isTestingWebdav = ref(false);

onMounted(() => {
  const cfg = getSyncSettings();
  allProviders.value.forEach((provider) => {
    const values: Record<string, string> = {};
    (provider.fields ?? []).forEach((f) => {
      values[f.key] = (cfg as unknown as Record<string, string>)[f.key] ?? "";
    });
    formValues[provider.type] = values;
  });
});

const showRevisionConflict = ref(false);

const isConnected = computed(() => syncStatus.enabled && !syncStatus.oauthPending);
const activeProvider = computed(() =>
  isConnected.value ? allProviders.value.find((p) => p.type === syncStatus.provider) ?? null : null,
);
const visibleProviders = computed(() => {
  if (isConnected.value && syncStatus.provider) {
    return allProviders.value.filter((p) => p.type !== syncStatus.provider);
  }
  return allProviders.value;
});
const oauthPendingProvider = computed(() => {
  const type = syncStatus.oauthPendingProvider ?? oauthPending.provider;
  if (!type) return null;
  return allProviders.value.find((p) => p.type === type) ?? null;
});

function providerLabel(type: SyncProviderType): string {
  return t(`sync_provider_${type}`);
}

function syncMetaForProvider(provider: SyncProviderSchema): ZodFieldMeta {
  const m: ZodFieldMeta = {};
  for (const f of provider.fields ?? []) {
    m[f.key] = "string";
  }
  return m;
}

function validateProviderFields(provider: SyncProviderSchema, silentClear = false): boolean {
  const fields = provider.fields ?? [];
  if (fields.length === 0) return true;
  const schema = buildSyncCredentialsSchema(provider);
  const raw = formValues[provider.type] ?? {};
  const meta = syncMetaForProvider(provider);
  for (const f of fields) {
    fieldErrors[`${provider.type}:${f.key}`] = "";
  }
  const parsed = schema.safeParse(raw);
  if (parsed.success) return true;
  for (const issue of parsed.error.issues) {
    const fk = issue.path[0];
    if (typeof fk !== "string") continue;
    const ek = `${provider.type}:${fk}`;
    if (!fieldErrors[ek]) {
      fieldErrors[ek] = zodIssueToMessageKey(issue, meta, t);
    }
  }
  return false;
}

function canConnect(provider: SyncProviderSchema): boolean {
  if (!provider.requiresSaveBeforeConnect) return true;
  return canSave(provider);
}

function canSave(provider: SyncProviderSchema): boolean {
  const fields = provider.fields ?? [];
  if (fields.length === 0) return true;
  const schema = buildSyncCredentialsSchema(provider);
  return schema.safeParse(formValues[provider.type] ?? {}).success;
}

let syncValidateTimer: ReturnType<typeof setTimeout> | null = null;
function scheduleValidateExpandedProvider() {
  if (syncValidateTimer != null) clearTimeout(syncValidateTimer);
  syncValidateTimer = setTimeout(() => {
    syncValidateTimer = null;
    const type = expandedProvider.value;
    if (!type) return;
    const provider = allProviders.value.find((p) => p.type === type);
    if (provider) validateProviderFields(provider);
  }, 120);
}

watch(
  () => formValues,
  () => {
    if (!expandedProvider.value) return;
    scheduleValidateExpandedProvider();
  },
  { deep: true },
);

watch(expandedProvider, (type) => {
  if (!type) return;
  const provider = allProviders.value.find((p) => p.type === type);
  if (provider) validateProviderFields(provider);
});

/**
 * Strip empty secret values from the payload before pushing it to the
 * backend. The dispatcher interprets a missing secret key as "keep the
 * already-saved keyring value", so leaving the input blank simply leaves
 * the previous secret untouched. Non-secret fields are always sent.
 */
function buildSaveCredentials(provider: SyncProviderSchema): Record<string, string> {
  const raw = formValues[provider.type] ?? {};
  const out: Record<string, string> = {};
  for (const f of provider.fields ?? []) {
    const v = raw[f.key];
    if (f.secret && !(v ?? "").trim()) continue;
    out[f.key] = v ?? "";
  }
  return out;
}

async function saveProvider(provider: SyncProviderSchema) {
  if (!validateProviderFields(provider)) return;
  isSaving.value = provider.type;
  try {
    await saveProviderSettings(provider.type, buildSaveCredentials(provider));
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
    return;
  } finally {
    isSaving.value = null;
  }
  toast(t("sync_credentials_saved"));
  // Clear any secret inputs the user typed — `saveProviderSettings` already
  // refreshed `providersSchema`, so the corresponding `hasSavedValue` flag
  // will surface the mask through `SecretInput` again.
  for (const f of provider.fields ?? []) {
    if (f.secret) formValues[provider.type][f.key] = "";
  }
}

async function connectProvider(type: SyncProviderType) {
  isConnecting.value = true;
  try {
    const res = await enableProvider(type);
    if (res.ok) {
      if (res.awaitingOAuth) {
        // OAuth modal opened by syncClient; browser opened once.
      } else {
        toast(t("sync_connected"));
        expandedProvider.value = null;
      }
    } else {
      toast((res.messageKey && t(res.messageKey)) || t("sync_not_available"), "error");
    }
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  } finally {
    isConnecting.value = false;
  }
}

async function cancelOAuthPending() {
  await disableProvider();
}

function resumeOAuthModal() {
  if (oauthPending.authUrl && oauthPending.provider) {
    openOAuthModal(oauthPending.authUrl, oauthPending.provider);
  }
}

async function handleTestWebdav() {
  isTestingWebdav.value = true;
  try {
    const res = await testWebdavConnection();
    if (res.ok) toast(t("sync_webdav_test_ok"));
    else toast((res.messageKey && t(res.messageKey)) || t("sync_operation_failed"), "error");
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  } finally {
    isTestingWebdav.value = false;
  }
}

async function handleDisconnect() {
  try {
    await disableProvider();
    toast(t("sync_disabled"));
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  }
}

/** Only checks the cloud copy — report what was found, not a fake "synced". */
async function handleCheckRemote() {
  isChecking.value = true;
  try {
    const hasUpdate = await checkRemote();
    toast(hasUpdate ? t("sync_remote_newer") : t("sync_up_to_date"), "info");
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  } finally {
    isChecking.value = false;
  }
}

function onConfirmAction() {
  const action = confirmAction.value;
  confirmAction.value = null;
  if (action === "disconnect") void handleDisconnect();
  else if (action === "pull") void handlePull();
}

function toastSyncFailure(res: { messageKey?: string }) {
  const msg =
    (res.messageKey && t(res.messageKey)) ||
    syncStatus.error ||
    t("sync_operation_failed");
  toast(msg, "error");
}

async function handlePull() {
  try {
    const res = await pullRemote();
    if (res.ok) toast(t("sync_pull_success"));
    else toastSyncFailure(res);
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  }
}

async function handlePush() {
  try {
    const res = await pushLocal();
    if (res.ok) toast(t("sync_push_success"));
    else if (res.messageKey === "sync_push_revision_conflict") showRevisionConflict.value = true;
    else toastSyncFailure(res);
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  }
}

async function handleConflictPullMerge() {
  showRevisionConflict.value = false;
  await handlePull();
}

async function handleConflictForcePush() {
  showRevisionConflict.value = false;
  try {
    const res = await pushLocalForce();
    if (res.ok) toast(t("sync_push_success"));
    else toastSyncFailure(res);
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  }
}
</script>

<template>
  <UiCard :title="t('cloud_sync')" :icon="Cloud" icon-tone="text-primary">
    <p class="text-xs sm:text-sm text-text-muted mb-4">{{ t("cloud_sync_desc") }}</p>

    <div
      v-if="syncStatus.oauthPending && oauthPendingProvider"
      class="mb-4 p-3 rounded-xl bg-warning/12 border border-warning/30"
    >
      <p class="text-xs text-text-primary mb-2">
        {{ t("sync_oauth_pending_banner", { provider: providerLabel(oauthPendingProvider.type) }) }}
      </p>
      <div class="flex gap-2">
        <button type="button" :class="button({ intent: 'primary', size: 'sm' })" @click="resumeOAuthModal">
          {{ t("sync_oauth_open_browser") }}
        </button>
        <button type="button" :class="button({ size: 'sm' })" @click="cancelOAuthPending">
          {{ t("sync_oauth_cancel") }}
        </button>
      </div>
    </div>

    <div v-if="activeProvider" class="mb-4 p-3 sm:p-4 rounded-xl bg-surface-secondary border border-border">
      <div class="flex items-center justify-between gap-2">
        <div class="min-w-0">
          <div class="text-sm text-text-primary font-medium truncate">{{ providerLabel(activeProvider.type) }}</div>
          <div class="text-xs text-text-muted">
            {{ t("sync_last_synced") }}: {{ syncStatus.syncing ? t("sync_syncing") : lastSyncedLabel }}
          </div>
        </div>
        <button
          type="button"
          @click="confirmAction = 'disconnect'"
          :class="button({ intent: 'ghost', size: 'sm', class: 'text-danger hover:bg-danger/12 hover:text-danger' })"
        >
          <CloudOff :size="12" />{{ t("sync_disconnect") }}
        </button>
      </div>
      <div class="mt-3 flex flex-wrap gap-2">
        <button
          type="button"
          @click="handleCheckRemote"
          :disabled="isChecking || syncStatus.syncing"
          :class="button({ size: 'sm' })"
        >
          <RefreshCw :size="12" :class="{ 'animate-spin': isChecking }" />
          {{ isChecking ? t("sync_checking") : t("sync_check_remote") }}
        </button>
      </div>
      <p v-if="syncStatus.error" class="mt-2 text-xs text-danger wrap-break-word">{{ syncStatus.error }}</p>
      <div v-if="syncStatus.pendingUpdate" class="mt-3 rounded-lg bg-info/12 p-2.5 text-xs text-text-primary">
        <p>{{ t("sync_remote_newer_desc") }}</p>
        <div class="mt-2 flex gap-2">
          <button type="button" :class="button({ intent: 'primary', size: 'sm' })" :disabled="syncStatus.syncing" @click="confirmAction = 'pull'">
            <Download :size="12" />{{ t("sync_pull") }}
          </button>
          <button type="button" :class="button({ intent: 'ghost', size: 'sm' })" @click="dismissPendingUpdate()">{{ t("sync_dismiss") }}</button>
        </div>
      </div>

      <details class="group mt-3">
        <summary class="flex cursor-pointer select-none list-none items-center gap-1 text-xs font-medium text-text-secondary hover:text-text-primary [&::-webkit-details-marker]:hidden">
          <ChevronRight :size="14" class="transition-transform group-open:rotate-90" />
          {{ t("sync_advanced") }}
        </summary>
        <div class="mt-2 space-y-2">
          <div class="flex flex-wrap gap-2">
            <button type="button" :disabled="syncStatus.syncing" :class="button({ size: 'sm' })" @click="confirmAction = 'pull'">
              <Download :size="12" />{{ t("sync_pull") }}
            </button>
            <button type="button" :disabled="syncStatus.syncing" :class="button({ size: 'sm' })" @click="handlePush">
              <Upload :size="12" />{{ t("sync_push") }}
            </button>
          </div>
          <p class="text-[11px] text-text-muted leading-snug">{{ t("sync_connected_push_hint") }}</p>
        </div>
      </details>
    </div>

    <div class="space-y-3">
      <div v-for="provider in visibleProviders" :key="provider.type" class="rounded-xl border border-border">
        <button
          type="button"
          class="w-full p-3 text-left flex items-center justify-between gap-2 rounded-xl hover:bg-surface-hover transition-colors"
          :aria-expanded="expandedProvider === provider.type"
          @click="expandedProvider = expandedProvider === provider.type ? null : provider.type"
        >
          <span class="text-sm font-medium text-text-primary">{{ providerLabel(provider.type) }}</span>
          <ChevronRight
            :size="16"
            class="text-text-muted shrink-0 transition-transform"
            :class="expandedProvider === provider.type ? 'rotate-90' : ''"
          />
        </button>

        <div v-if="expandedProvider === provider.type" class="px-3 pb-3 space-y-2">
          <p class="text-xs text-text-muted leading-relaxed border-b border-border pb-3 mb-1">
            {{ t(`sync_setup_${provider.type}`) }}
          </p>
          <template v-for="field in (provider.fields ?? [])" :key="field.key">
            <div>
              <label :class="ui.fieldLabel()">{{ t(field.label) }}</label>
              <SecretInput
                v-if="field.secret"
                v-model="formValues[provider.type][field.key]"
                :has-saved-value="!!field.hasSavedValue"
                size="sm"
                :placeholder="field.placeholder ? t(field.placeholder) : ''"
              />
              <input
                v-else
                v-model="formValues[provider.type][field.key]"
                :type="field.inputType || 'text'"
                :placeholder="field.placeholder ? t(field.placeholder) : ''"
                :class="ui.field({ class: 'h-8 py-1 text-xs' })"
              />
              <p v-if="field.secret && field.hasSavedValue && !(formValues[provider.type][field.key] ?? '').trim()"
                class="mt-1 text-[11px] text-success">
                {{ t('ai_api_key_configured') }}
              </p>
              <p v-if="field.helpText" class="mt-1 text-[11px] text-text-muted">{{ t(field.helpText) }}</p>
              <p v-if="fieldErrors[`${provider.type}:${field.key}`]" :class="ui.fieldError()">
                {{ fieldErrors[`${provider.type}:${field.key}`] }}
              </p>
            </div>
          </template>
          <div class="flex flex-wrap gap-2 pt-1">
            <button
              v-if="(provider.fields ?? []).length > 0"
              type="button"
              @click="saveProvider(provider)"
              :disabled="isSaving === provider.type || !canSave(provider)"
              :class="button({ size: 'sm' })"
            >
              <Save :size="12" />{{ isSaving === provider.type ? t("sync_saving") : t("sync_save_credentials") }}
            </button>
            <button
              v-if="provider.type === 'webdav'"
              type="button"
              @click="handleTestWebdav"
              :disabled="isTestingWebdav || !canSave(provider)"
              :class="button({ size: 'sm' })"
            >
              {{ isTestingWebdav ? t("sync_checking") : t("sync_webdav_test_connection") }}
            </button>
            <button
              type="button"
              @click="connectProvider(provider.type)"
              :disabled="isConnecting || !canConnect(provider)"
              :class="button({ intent: 'primary', size: 'sm' })"
            >
              {{ t("sync_connect") }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <Modal
      :show="showRevisionConflict"
      :title="t('sync_push_revision_title')"
      maxWidth="24rem"
      @close="showRevisionConflict = false"
    >
      <p class="text-sm text-text-secondary">{{ t("sync_push_revision_body") }}</p>
      <template #footer>
        <button type="button" :class="button()" @click="showRevisionConflict = false">
          {{ t("cancel") }}
        </button>
        <button type="button" :class="button()" @click="handleConflictPullMerge">
          {{ t("sync_conflict_pull_merge") }}
        </button>
        <button type="button" :class="button({ intent: 'danger' })" @click="handleConflictForcePush">
          {{ t("sync_conflict_force_push") }}
        </button>
      </template>
    </Modal>

    <ConfirmDialog
      :show="confirmAction !== null"
      :title="confirmAction === 'pull' ? t('sync_pull') : t('sync_disconnect')"
      :message="confirmAction === 'pull' ? t('sync_pull_confirm') : t('sync_disconnect_confirm')"
      :confirm-label="confirmAction === 'pull' ? t('sync_pull') : t('sync_disconnect')"
      @confirm="onConfirmAction"
      @cancel="confirmAction = null"
    />

    <SyncOAuthModal :provider-label="providerLabel" />
  </UiCard>
</template>
