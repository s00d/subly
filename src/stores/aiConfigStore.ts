import { defineStore } from "pinia";
import { computed, ref } from "vue";

import {
  aiApiKeyName,
  aiGetProviders,
  DEFAULT_AI_FEATURES,
  type AiFeatureToggles,
  type AiProviderMeta,
  type AiProviderType,
} from "@/services/aiClient";
import { getConfigValue, setConfigValue } from "@/services/configClient";
import {
  deleteSecureValue,
  getSecureValue,
  setSecureValue,
} from "@/services/secureStorageClient";

/**
 * Reactive single source of truth for AI settings (provider, model, endpoint,
 * feature toggles, master enable). The actual API key is never held in state —
 * we only mirror a `hasApiKey` boolean so the UI can show a "configured" badge.
 * `load()` does not read the OS keyring; call {@link refreshHasApiKey} when you
 * need an accurate key-present flag (e.g. after clicking the AI assistant).
 *
 * Pages displaying AI-related buttons (`SubscriptionsPage`, `ExpensesPage`,
 * `DataManagementSection`) subscribe via `storeToRefs(useAiConfigStore())` so
 * they react instantly when the user saves settings.
 *
 * IMPORTANT: every piece of state must be a `ref()` rather than `reactive()` —
 * `storeToRefs()` on a setup-store only wraps refs; reactive proxies are
 * returned as-is, which breaks `aiFeatures.value.subscriptionInput` access
 * patterns on consumer pages.
 */
export const useAiConfigStore = defineStore("aiConfig", () => {
  const providers = ref<AiProviderMeta[]>([]);
  const enabled = ref(false);
  const features = ref<AiFeatureToggles>({ ...DEFAULT_AI_FEATURES });
  const providerType = ref<AiProviderType>("openrouter");
  const model = ref("");
  const endpoint = ref("");
  const hasApiKey = ref(false);
  const loaded = ref(false);
  const loading = ref(false);

  const activeProvider = computed<AiProviderMeta | undefined>(() =>
    providers.value.find((p) => p.type === providerType.value),
  );

  /** True iff master + per-feature flag are on AND API key present (when required). */
  function featureAvailable(key: keyof AiFeatureToggles): boolean {
    if (!enabled.value) return false;
    if (!features.value[key]) return false;
    const provider = activeProvider.value;
    if (provider?.requiresKey && !hasApiKey.value) return false;
    return true;
  }

  async function load(force = false): Promise<void> {
    if (loaded.value && !force) return;
    if (loading.value) return;
    loading.value = true;
    try {
      try {
        providers.value = await aiGetProviders();
      } catch {
        providers.value = [];
      }

      const savedProvider = (await getConfigValue<string>(
        "aiProvider",
      )) as AiProviderType | null;
      if (
        savedProvider &&
        providers.value.some((p) => p.type === savedProvider)
      ) {
        providerType.value = savedProvider;
      } else if (providers.value.length > 0) {
        providerType.value = providers.value[0].type;
      }

      const desc = providers.value.find((p) => p.type === providerType.value);
      model.value =
        (await getConfigValue<string>("aiModel")) || desc?.defaultModel || "";
      endpoint.value =
        (await getConfigValue<string>("aiCustomEndpoint")) ||
        desc?.defaultBaseUrl ||
        "";

      enabled.value = !!(await getConfigValue<boolean>("aiEnabled"));
      const savedFeatures = await getConfigValue<AiFeatureToggles>(
        "aiFeatures",
      );
      features.value = {
        ...DEFAULT_AI_FEATURES,
        ...(savedFeatures || {}),
      };

      loaded.value = true;
    } finally {
      loading.value = false;
    }
  }

  async function setEnabled(next: boolean): Promise<void> {
    enabled.value = next;
    try {
      await setConfigValue("aiEnabled", next);
    } catch (e) {
      enabled.value = !next;
      throw e;
    }
  }

  async function setFeatures(next: AiFeatureToggles): Promise<void> {
    const prev = features.value;
    // Replace the whole object so Vue picks up the change in a single tick.
    features.value = { ...next };
    try {
      await setConfigValue("aiFeatures", { ...features.value });
    } catch (e) {
      features.value = prev;
      throw e;
    }
  }

  /**
   * Switch the active provider and (optionally) persist a new API key for it.
   *
   * The function is the single canonical place that writes
   * `aiProvider` / `aiModel` / `aiCustomEndpoint` to config and the API key
   * to secure storage. A read-back self-test guards against keychain backends
   * that silently swallow writes. For a key presence check without saving, use
   * {@link refreshHasApiKey}.
   */
  async function setProvider(opts: {
    type: AiProviderType;
    model: string;
    endpoint?: string;
    apiKey?: string;
  }): Promise<void> {
    const desc = providers.value.find((p) => p.type === opts.type);

    if (opts.apiKey !== undefined) {
      const trimmed = opts.apiKey.trim();
      await setSecureValue(aiApiKeyName(opts.type), trimmed);
      if (trimmed.length > 0) {
        // Keychain backends on dev builds occasionally accept the write but
        // refuse to surface it back. Round-trip to make sure the key actually
        // persisted; raise a typed error so the UI can show a clear message.
        let echo = (await getSecureValue(aiApiKeyName(opts.type))) || "";
        if (echo !== trimmed) {
          // Some backends (DBus/KWallet) latch asynchronously — retry once.
          await new Promise((resolve) => setTimeout(resolve, 60));
          echo = (await getSecureValue(aiApiKeyName(opts.type))) || "";
        }
        if (echo !== trimmed) {
          throw new Error("ai_keychain_unavailable");
        }
      }
    }

    await setConfigValue("aiProvider", opts.type);
    await setConfigValue(
      "aiModel",
      opts.model.trim() || desc?.defaultModel || "",
    );
    if (opts.endpoint !== undefined) {
      await setConfigValue("aiCustomEndpoint", opts.endpoint.trim());
    }

    providerType.value = opts.type;
    model.value = opts.model.trim() || desc?.defaultModel || "";
    if (opts.endpoint !== undefined) endpoint.value = opts.endpoint.trim();

    if (opts.apiKey !== undefined) {
      hasApiKey.value = opts.apiKey.trim().length > 0;
    } else {
      // Provider changed without supplying a new key — check whether one
      // already exists for the new provider.
      const existing = (await getSecureValue(aiApiKeyName(opts.type))) || "";
      hasApiKey.value = existing.trim().length > 0;
    }
  }

  /**
   * Refresh `hasApiKey` for the current provider without touching anything else.
   * Useful after the user navigates back to settings or the app resumes.
   */
  async function refreshHasApiKey(): Promise<void> {
    const key = (await getSecureValue(aiApiKeyName(providerType.value))) || "";
    hasApiKey.value = key.trim().length > 0;
  }

  /**
   * Delete the API key for the current provider from the OS keyring and clear
   * the `hasApiKey` indicator. Used by the "Clear" button next to the input
   * so the user can intentionally revoke a saved key.
   */
  async function clearApiKey(): Promise<void> {
    await deleteSecureValue(aiApiKeyName(providerType.value));
    hasApiKey.value = false;
  }

  return {
    providers,
    enabled,
    features,
    providerType,
    model,
    endpoint,
    hasApiKey,
    loaded,
    loading,
    activeProvider,
    featureAvailable,
    load,
    setEnabled,
    setFeatures,
    setProvider,
    refreshHasApiKey,
    clearApiKey,
  };
});
