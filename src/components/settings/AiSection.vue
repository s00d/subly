<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { useI18n } from "vue-i18n";
import {
  AlertTriangle,
  Check,
  ExternalLink,
  KeyRound,
  Loader2,
  Sparkles,
  Zap,
} from "@lucide/vue";
import { openUrl } from "@tauri-apps/plugin-opener";

import AppInput from "@/components/ui/AppInput.vue";
import AppSelect from "@/components/ui/AppSelect.vue";
import AppToggle from "@/components/ui/AppToggle.vue";
import SecretInput from "@/components/ui/SecretInput.vue";
import Toast from "@/components/ui/Toast.vue";
import { useToast } from "@/composables/useToast";
import { tv, button } from "@/lib/tv";
import UiCard from "@/components/ui/UiCard.vue";

import {
  aiApiKeyName,
  aiTestConnection,
  type AiFeatureToggles,
  type AiModelPreset,
  type AiProviderMeta,
  type AiProviderType,
  type AiTestResult,
} from "@/services/aiClient";
import { getSecureValue } from "@/services/secureStorageClient";
import { useAiConfigStore } from "@/stores/aiConfigStore";
import { formatErrorForToast } from "@/utils/formatError";

const { t } = useI18n();
const { toast, toastMsg, toastType, showToast, closeToast } = useToast();

const store = useAiConfigStore();
const {
  providers,
  enabled,
  features,
  providerType,
  model,
  endpoint,
  hasApiKey,
} = storeToRefs(store);

// Provider picked in the form; committed to the store only on Save — writing the store directly
// made other pages evaluate an unsaved provider against the old key state.
const formProviderType = ref<AiProviderType>(providerType.value);
watch(providerType, (v) => (formProviderType.value = v));
const formProvider = computed(() => providers.value.find((p) => p.type === formProviderType.value));

const CUSTOM_MODEL_VALUE = "__custom__";

// Local form state — never lives in the store. Mutations get committed via
// `store.setProvider(...)` once the user clicks Save.
const formModel = ref("");
const formEndpoint = ref("");
/**
 * The freshly-typed API key. Empty unless the user actually edited the
 * password field — see `SecretInput.vue` for how the saved-key mask works.
 */
const formApiKey = ref("");
const useCustomModel = ref(false);
/** True when a key is already stored in the OS keyring for this provider. */
const keyConfigured = ref(false);

const isSaving = ref(false);
const isTesting = ref(false);
const lastTest = ref<AiTestResult | null>(null);

const recommendedModels = computed<readonly AiModelPreset[]>(
  () => formProvider.value?.recommendedModels ?? [],
);

const modelSelectOptions = computed(() => {
  const opts = recommendedModels.value.map((m) => ({
    value: m.id,
    label: `${m.label}${m.supportsVision ? " · 🖼" : ""}`,
  }));
  opts.push({ value: CUSTOM_MODEL_VALUE, label: t("ai_model_custom") });
  return opts;
});

const providerSelectOptions = computed(() =>
  providers.value.map((p) => ({ value: p.type, label: p.name })),
);

const activePreset = computed<AiModelPreset | null>(() => {
  const id = formModel.value.trim().toLowerCase();
  return (
    recommendedModels.value.find((m) => m.id.toLowerCase() === id) ?? null
  );
});

const hasSavedKey = computed(() => keyConfigured.value || hasApiKey.value);

/** True ⇔ user typed a fresh key into the input. */
const apiKeyEdited = computed(() => formApiKey.value.length > 0);

/** True ⇔ Save button should be enabled. */
const isDirty = computed(() => {
  const desc = formProvider.value;
  if (!desc) return false;
  if (desc.type !== providerType.value) return true;
  if (formModel.value.trim() !== model.value.trim()) return true;
  if (desc.requiresEndpoint && formEndpoint.value.trim() !== endpoint.value.trim()) return true;
  if (apiKeyEdited.value) return true;
  return false;
});

function applyProviderToForm(provider: AiProviderMeta | undefined, opts?: { reset?: boolean }) {
  if (!provider) return;
  if (opts?.reset) {
    formModel.value = provider.defaultModel;
    formEndpoint.value = provider.defaultBaseUrl ?? "";
  } else {
    formModel.value = model.value || provider.defaultModel;
    formEndpoint.value = endpoint.value || provider.defaultBaseUrl || "";
  }
  useCustomModel.value =
    !provider.recommendedModels?.some(
      (m) => m.id.toLowerCase() === formModel.value.trim().toLowerCase(),
    );
}

async function refreshKeyState(type: AiProviderType) {
  try {
    const key = (await getSecureValue(aiApiKeyName(type))) || "";
    keyConfigured.value = key.trim().length > 0;
  } catch {
    keyConfigured.value = false;
  }
  // The actual secret never lives in the form — `SecretInput` shows a mask
  // whenever `hasSavedKey` is true.
  formApiKey.value = "";
}

onMounted(async () => {
  try {
    await store.load();
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  }
  applyProviderToForm(formProvider.value);
  await refreshKeyState(formProviderType.value);
});

watch(formProvider, (next, prev) => {
  if (!next) return;
  // Only react to provider switch, not in-store mutations of the same provider.
  if (prev?.type === next.type) return;
  applyProviderToForm(next, { reset: true });
  refreshKeyState(next.type);
});

async function onSelectProvider(value: string | number) {
  const type = String(value) as AiProviderType;
  if (type === formProviderType.value) return;
  const desc = providers.value.find((p) => p.type === type);
  if (!desc) return;
  // Form-only switch: the store (used by every page) changes on Save.
  formProviderType.value = type;
  applyProviderToForm(desc, { reset: true });
  await refreshKeyState(type);
  lastTest.value = null;
}

function onModelSelectChange(value: string | number) {
  const v = String(value);
  if (v === CUSTOM_MODEL_VALUE) {
    useCustomModel.value = true;
    return;
  }
  useCustomModel.value = false;
  formModel.value = v;
}

async function saveProviderConfig() {
  const desc = formProvider.value;
  if (!desc) return;

  // Push the key down to the keyring only when the user actually typed
  // something — otherwise the previously-saved key stays put.
  const wantsKeyUpdate = apiKeyEdited.value;
  const nextKey = wantsKeyUpdate ? formApiKey.value.trim() : "";

  if (desc.requiresKey && !hasSavedKey.value && !nextKey) {
    toast(t("ai_api_key_required"), "error");
    return;
  }
  if (desc.requiresEndpoint && !formEndpoint.value.trim()) {
    toast(t("ai_endpoint_required"), "error");
    return;
  }

  isSaving.value = true;
  try {
    await store.setProvider({
      type: desc.type as AiProviderType,
      model: formModel.value.trim() || desc.defaultModel,
      endpoint: desc.requiresEndpoint ? formEndpoint.value.trim() : undefined,
      apiKey: wantsKeyUpdate ? nextKey : undefined,
    });
    if (wantsKeyUpdate && nextKey.length > 0) {
      keyConfigured.value = true;
      formApiKey.value = "";
    }
    // Flip the master toggle on once a working provider+key combination is
    // saved. Without this users hit a common UX trap: they save the key,
    // hit "Test connection" (which succeeds), then wonder why the
    // assistance buttons never appear on data pages — because `aiEnabled`
    // is a separate flag that nobody told them to flip.
    if (!enabled.value && hasApiKey.value) {
      try {
        await store.setEnabled(true);
      } catch (e) {
        console.warn("[AiSection] auto-enable failed", e);
      }
    }
    toast(t("success"));
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    if (msg.includes("ai_keychain_unavailable")) {
      toast(t("ai_keychain_unavailable"), "error");
    } else {
      toast(formatErrorForToast(e, t), "error");
    }
  } finally {
    isSaving.value = false;
  }
}

async function toggleEnabled(next: boolean) {
  try {
    await store.setEnabled(next);
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  }
}

async function clearApiKey() {
  try {
    await store.clearApiKey();
    keyConfigured.value = false;
    formApiKey.value = "";
    lastTest.value = null;
    toast(t("ai_api_key_cleared"));
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  }
}

async function toggleFeature(key: keyof AiFeatureToggles, next: boolean) {
  try {
    await store.setFeatures({ ...features.value, [key]: next });
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  }
}

async function runTest() {
  if (isDirty.value) {
    await saveProviderConfig();
  }
  isTesting.value = true;
  lastTest.value = null;
  try {
    const result = await aiTestConnection();
    lastTest.value = result;
    if (result.ok) {
      toast(t("ai_test_ok"));
    } else {
      toast(result.error || t("ai_test_failed"), "error");
    }
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  } finally {
    isTesting.value = false;
  }
}

async function openDocs(url: string | undefined) {
  if (!url) return;
  try {
    await openUrl(url);
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  }
}

// ---- Provider help (long-form per-provider description) ----
const PROVIDER_HELP_KEYS: Record<string, string> = {
  openai: "ai_provider_openai_help",
  openrouter: "ai_provider_openrouter_help",
  gemini: "ai_provider_gemini_help",
  groq: "ai_provider_groq_help",
  deepseek: "ai_provider_deepseek_help",
  mistral: "ai_provider_mistral_help",
  openai_compat: "ai_provider_openai_compat_help",
};

const providerHelpText = computed(() => {
  const desc = formProvider.value;
  if (!desc) return "";
  const key = PROVIDER_HELP_KEYS[desc.type];
  return key ? t(key) : desc.description;
});

const visionPresets = computed(() =>
  recommendedModels.value.filter((m) => m.supportsVision),
);

const s = tv({
  slots: {
    desc: "text-xs sm:text-sm text-text-muted mb-4",
    divider: "pt-4 mt-4 border-t border-border",
    badge:
      "inline-flex items-center gap-1 px-1.5 py-0.5 rounded-full text-[10px] font-medium",
    helpCard:
      "mt-3 rounded-xl border border-primary/30 bg-primary-light p-3 space-y-2",
    warningBox:
      "mt-3 rounded-lg border border-warning/30 bg-warning/12 p-3 text-xs text-text-primary flex items-start gap-2",
    privacyBox:
      "mt-4 rounded-lg border border-warning/25 bg-warning/8 p-3 text-[11px] text-text-secondary",
    saveBtn: button({ intent: "primary", size: "sm" }),
    testBtn: button({ size: "sm" }),
  },
})();
</script>

<template>
  <UiCard :title="t('ai_section_title')" :icon="Sparkles" icon-tone="text-primary">
    <p :class="s.desc()">{{ t("ai_section_desc") }}</p>

    <!-- Master toggle -->
    <AppToggle
      :modelValue="enabled"
      @update:modelValue="toggleEnabled"
      :label="t('ai_master_enabled')"
      :description="t('ai_master_enabled_info')"
    />

    <!-- No-key warning banner -->
    <div
      v-if="enabled && formProvider?.requiresKey && !hasSavedKey"
      :class="s.warningBox()"
    >
      <AlertTriangle :size="16" class="shrink-0 mt-px text-warning" />
      <span>{{ t("ai_no_key_warning") }}</span>
    </div>

    <!-- Configured-but-disabled banner. We auto-enable on first Save, but
         the user may have explicitly switched the master toggle off again. -->
    <div
      v-else-if="!enabled && hasSavedKey"
      :class="s.warningBox()"
      class="cursor-pointer hover:opacity-80 transition-opacity"
      @click="toggleEnabled(true)"
    >
      <AlertTriangle :size="16" class="shrink-0 mt-px text-warning" />
      <span class="flex-1">{{ t("ai_configured_but_disabled") }}</span>
      <span class="font-semibold underline shrink-0">{{ t("ai_enable_now") }}</span>
    </div>

    <!-- Provider selector -->
    <div class="mt-4 space-y-3">
      <AppSelect
        :modelValue="formProviderType"
        :options="providerSelectOptions"
        :label="t('ai_provider_label')"
        :placeholder="t('ai_provider_label')"
        @update:modelValue="onSelectProvider"
      />

      <!-- Provider help card -->
      <div v-if="formProvider" :class="s.helpCard()">
        <div class="flex items-start gap-2">
          <Sparkles :size="14" class="text-primary mt-0.5 shrink-0" />
          <div class="flex-1 min-w-0">
            <p class="text-xs text-text-primary leading-relaxed">
              {{ providerHelpText }}
            </p>
            <div class="flex flex-wrap gap-1.5 mt-2">
              <span
                v-if="!formProvider.requiresKey"
                :class="[s.badge(), 'bg-success/12 text-success']"
              >
                {{ t("ai_free_tier") }}
              </span>
              <span
                v-else
                :class="[s.badge(), 'bg-info/12 text-info']"
              >
                <KeyRound :size="9" />
                {{ t("ai_paid_tier") }}
              </span>
              <span
                v-if="visionPresets.length > 0"
                :class="[s.badge(), 'bg-primary/12 text-primary']"
              >
                {{ t("ai_vision_badge") }}
              </span>
            </div>
          </div>
        </div>
        <button
          type="button"
          class="inline-flex items-center gap-1 text-[11px] text-primary hover:underline"
          @click="openDocs(formProvider.docsUrl)"
        >
          <ExternalLink :size="11" />
          {{ t("ai_open_docs") }}
        </button>
      </div>

      <!-- Model select -->
      <AppSelect
        v-if="recommendedModels.length > 0"
        :modelValue="useCustomModel ? CUSTOM_MODEL_VALUE : formModel"
        :options="modelSelectOptions"
        :label="t('ai_model')"
        searchable
        @update:modelValue="onModelSelectChange"
      />

      <!-- Custom model name (free input) -->
      <AppInput
        v-if="useCustomModel || recommendedModels.length === 0"
        v-model="formModel"
        type="text"
        :label="recommendedModels.length === 0 ? t('ai_model') : t('ai_model_custom')"
        :placeholder="formProvider?.defaultModel"
      />

      <p class="text-[11px] text-text-muted -mt-1.5">
        {{
          activePreset && activePreset.supportsVision
            ? t("ai_model_vision_supported")
            : useCustomModel
              ? t("ai_model_vision_unknown")
              : t("ai_model_hint")
        }}
      </p>

      <!-- Endpoint -->
      <AppInput
        v-if="formProvider?.requiresEndpoint"
        v-model="formEndpoint"
        type="text"
        :label="t('ai_endpoint_url')"
        placeholder="http://localhost:11434/v1"
      />

      <!-- API key -->
      <div v-if="formProvider && (formProvider.requiresKey || formProvider.requiresEndpoint)">
        <SecretInput
          v-model="formApiKey"
          :has-saved-value="hasSavedKey"
          :label="t('ai_api_key')"
          :placeholder="
            formProvider.requiresKey
              ? t('ai_api_key')
              : t('ai_api_key_optional')
          "
        />
        <div
          v-if="hasSavedKey && !apiKeyEdited"
          class="mt-1 flex items-center justify-between gap-2"
        >
          <p class="text-[11px] text-success flex items-center gap-1">
            <Check :size="11" />
            {{ t("ai_api_key_configured") }}
          </p>
          <button
            type="button"
            class="text-[11px] text-text-muted hover:text-danger transition-colors"
            @click="clearApiKey"
          >
            {{ t("ai_api_key_clear") }}
          </button>
        </div>
        <p
          v-else-if="hasSavedKey && apiKeyEdited"
          class="mt-1 text-[11px] text-warning"
        >
          {{ t("ai_api_key_will_replace") }}
        </p>
      </div>

      <!-- Action row -->
      <div class="flex items-center gap-2 flex-wrap pt-1">
        <button
          type="button"
          @click="saveProviderConfig"
          :disabled="isSaving || !isDirty"
          :class="s.saveBtn()"
        >
          <Loader2 v-if="isSaving" :size="12" class="animate-spin" />
          <Check v-else :size="12" />
          {{ t("save") }}
        </button>
        <button
          type="button"
          @click="runTest"
          :disabled="isTesting"
          :class="s.testBtn()"
        >
          <Loader2 v-if="isTesting" :size="12" class="animate-spin" />
          <Zap v-else :size="12" />
          {{ t("ai_test") }}
        </button>
        <span
          v-if="isDirty"
          class="text-[11px] text-warning"
        >
          {{ t("ai_unsaved_changes") }}
        </span>
        <span
          v-else-if="hasSavedKey && lastTest?.ok"
          class="text-[11px] text-success flex items-center gap-1"
        >
          <Check :size="11" />
          {{ t("ai_saved_indicator") }}
        </span>
      </div>

      <!-- Last test feedback -->
      <div
        v-if="lastTest && lastTest.ok"
        class="text-[11px] text-success flex items-center gap-1.5"
      >
        <Check :size="12" />
        {{ t("ai_test_ok_with_latency", { ms: lastTest.latencyMs }) }}
        <span class="text-text-muted" v-if="lastTest.echo">— "{{ lastTest.echo }}"</span>
      </div>
      <div
        v-else-if="lastTest && !lastTest.ok"
        class="text-[11px] text-danger wrap-break-word"
      >
        {{ lastTest.error || t("ai_test_failed") }}
      </div>
    </div>

    <!-- Per-feature toggles -->
    <div :class="s.divider()">
      <h3 class="text-sm font-semibold text-text-primary mb-3">
        {{ t("ai_features_title") }}
      </h3>
      <div class="space-y-3">
        <AppToggle
          :modelValue="features.subscriptionInput"
          @update:modelValue="(v) => toggleFeature('subscriptionInput', v)"
          :label="t('ai_feature_subscription_input')"
          :description="t('ai_feature_subscription_input_info')"
          :disabled="!enabled"
        />
        <AppToggle
          :modelValue="features.expenseInput"
          @update:modelValue="(v) => toggleFeature('expenseInput', v)"
          :label="t('ai_feature_expense_input')"
          :description="t('ai_feature_expense_input_info')"
          :disabled="!enabled"
        />
        <AppToggle
          :modelValue="features.statementImport"
          @update:modelValue="(v) => toggleFeature('statementImport', v)"
          :label="t('ai_feature_statement_import')"
          :description="t('ai_feature_statement_import_info')"
          :disabled="!enabled"
        />
        <AppToggle
          :modelValue="features.receiptImport"
          @update:modelValue="(v) => toggleFeature('receiptImport', v)"
          :label="t('ai_feature_receipt_import')"
          :description="t('ai_feature_receipt_import_info')"
          :disabled="!enabled"
        />
      </div>
    </div>

    <div :class="s.privacyBox()">
      {{ t("ai_privacy_warning") }}
    </div>

    <Toast :show="showToast" :message="toastMsg" :type="toastType" @close="closeToast" />
  </UiCard>
</template>
