<script setup lang="ts">
import { ref, computed, onMounted, reactive, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import {
  syncStatus,
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
} from "@/services/syncClient";
import type { SyncProviderType, SyncProviderSchema } from "@/services/syncClient";
import Modal from "@/components/ui/Modal.vue";
import { ChevronRight, Cloud, CloudOff, Download, Save, Upload } from "@lucide/vue";
import { ui } from "@/lib/tv";
import { formatErrorForToast } from "@/utils/formatError";
import { buildSyncCredentialsSchema } from "@/schemas/zod/syncCredentials";
import { zodIssueToMessageKey, type ZodFieldMeta } from "@/composables/useZodErrors";

const { t } = useI18n();
const { toast } = useToast();
const allProviders = computed(() => getProviders());
const expandedProvider = ref<SyncProviderType | null>(null);
const isConnecting = ref(false);
const isSaving = ref<SyncProviderType | null>(null);
const formValues = reactive<Record<string, Record<string, string>>>({});
const fieldErrors = reactive<Record<string, string>>({});
const showRevisionConflict = ref(false);

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

const activeProvider = computed(() => allProviders.value.find((p) => p.type === syncStatus.provider) ?? null);
const visibleProviders = computed(() => allProviders.value.filter((p) => p.type !== syncStatus.provider));

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

async function saveProvider(provider: SyncProviderSchema) {
  if (!validateProviderFields(provider)) return;
  isSaving.value = provider.type;
  await saveProviderSettings(provider.type, formValues[provider.type] ?? {});
  isSaving.value = null;
  toast(t("sync_credentials_saved"));
}

async function connectProvider(type: SyncProviderType) {
  isConnecting.value = true;
  try {
    const res = await enableProvider(type);
    if (res.ok) {
      toast(t("sync_success"));
      expandedProvider.value = null;
    } else {
      toast((res.messageKey && t(res.messageKey)) || t("sync_not_available"), "error");
    }
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  } finally {
    isConnecting.value = false;
  }
}

async function handleDisconnect() {
  await disableProvider();
  toast(t("sync_disabled"));
}

async function handleCheckRemote() {
  try {
    const hasUpdate = await checkRemote();
    toast(hasUpdate ? t("sync_remote_newer") : t("sync_success"));
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  }
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
  <section class="bg-surface rounded-xl border border-border p-4 sm:p-5">
    <div class="flex items-center gap-2 mb-1">
      <Cloud :size="18" class="text-primary" />
      <h2 :class="ui.sectionTitle()">{{ t("cloud_sync") }}</h2>
    </div>
    <p class="text-xs sm:text-sm text-text-muted mb-4">{{ t("cloud_sync_desc") }}</p>

    <div v-if="activeProvider" class="mb-4 p-4 rounded-xl bg-surface-secondary border border-border">
      <div class="flex items-center justify-between">
        <div class="text-sm text-text-primary font-medium">{{ providerLabel(activeProvider.type) }}</div>
        <button @click="handleDisconnect" class="px-3 py-1.5 rounded-lg text-xs border border-red-300 text-red-600">
          <CloudOff :size="12" class="inline-block mr-1" />{{ t("sync_disconnect") }}
        </button>
      </div>
      <div class="mt-3 flex gap-2">
        <button @click="handleCheckRemote" class="px-3 py-1.5 rounded-lg text-xs border border-border">{{ t("sync_now") }}</button>
        <button @click="handlePull" class="px-3 py-1.5 rounded-lg text-xs border border-blue-300 text-blue-700">
          <Download :size="12" class="inline-block mr-1" />{{ t("sync_pull") }}
        </button>
        <button @click="handlePush" class="px-3 py-1.5 rounded-lg text-xs border border-green-300 text-green-700">
          <Upload :size="12" class="inline-block mr-1" />{{ t("sync_push") }}
        </button>
      </div>
      <p v-if="activeProvider" class="mt-2 text-[11px] text-text-muted leading-snug">
        {{ t("sync_connected_push_hint") }}
      </p>
      <div v-if="syncStatus.pendingUpdate" class="mt-3 text-xs text-blue-700">
        {{ t("sync_remote_newer_desc") }}
        <button @click="dismissPendingUpdate()" class="ml-2 underline">{{ t("sync_dismiss") }}</button>
      </div>
    </div>

    <div class="space-y-3">
      <div v-for="provider in visibleProviders" :key="provider.type" class="rounded-xl border border-border">
        <button
          type="button"
          class="w-full p-3 text-left flex items-center justify-between gap-2"
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
              <label class="block text-[10px] text-text-muted mb-1">{{ t(field.label) }}</label>
              <input
                v-model="formValues[provider.type][field.key]"
                :type="field.inputType || (field.secret ? 'password' : 'text')"
                :placeholder="field.placeholder ? t(field.placeholder) : ''"
                class="w-full px-2.5 py-1.5 rounded-lg border border-border bg-surface text-xs text-text-primary"
              />
              <p v-if="field.helpText" class="mt-1 text-[10px] text-text-muted">{{ t(field.helpText) }}</p>
              <p v-if="fieldErrors[`${provider.type}:${field.key}`]" class="mt-1 text-[10px] text-red-500">
                {{ fieldErrors[`${provider.type}:${field.key}`] }}
              </p>
            </div>
          </template>
          <div class="flex gap-2 pt-1">
            <button
              v-if="(provider.fields ?? []).length > 0"
              @click="saveProvider(provider)"
              :disabled="isSaving === provider.type || !canSave(provider)"
              class="px-3 py-1.5 rounded-lg text-xs border border-border disabled:opacity-50"
            >
              <Save :size="12" class="inline-block mr-1" />{{ isSaving === provider.type ? t("sync_saving") : t("sync_save_credentials") }}
            </button>
            <button
              @click="connectProvider(provider.type)"
              :disabled="isConnecting || !canSave(provider)"
              class="px-3 py-1.5 rounded-lg text-xs bg-primary text-white disabled:opacity-50"
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
      <p class="text-sm text-text-primary mb-4">{{ t("sync_push_revision_body") }}</p>
      <div class="flex flex-col sm:flex-row gap-2 justify-end">
        <button
          type="button"
          class="px-3 py-2 rounded-lg text-xs border border-border order-2 sm:order-1"
          @click="showRevisionConflict = false"
        >
          {{ t("cancel") }}
        </button>
        <button
          type="button"
          class="px-3 py-2 rounded-lg text-xs border border-blue-300 text-blue-700 order-1 sm:order-2"
          @click="handleConflictPullMerge"
        >
          {{ t("sync_conflict_pull_merge") }}
        </button>
        <button
          type="button"
          class="px-3 py-2 rounded-lg text-xs bg-amber-600 text-white order-3"
          @click="handleConflictForcePush"
        >
          {{ t("sync_conflict_force_push") }}
        </button>
      </div>
    </Modal>
  </section>
</template>
