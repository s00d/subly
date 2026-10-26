<script setup lang="ts">
import Modal from "@/components/ui/Modal.vue";
import { ref, computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import { getAvailableLanguages } from "@/i18n";
import { setLanguage } from "@/i18n";
import type { Settings } from "@/schemas/appData";
import { useAppMetaStore } from "@/stores/appMetaStore";
import AppSelect from "@/components/ui/AppSelect.vue";
import AppToggle from "@/components/ui/AppToggle.vue";
import type { SelectOption } from "@/components/ui/AppSelect.vue";
import { Sun, Moon, Monitor, Check, AlertTriangle, Languages } from "@lucide/vue";
import { useScrollLock } from "@/composables/useScrollLock";
import { typo, button } from "@/lib/tv";
import UiCard from "@/components/ui/UiCard.vue";
import SavedIndicator, { useAutoSave } from "@/components/settings/SavedIndicator.vue";

const props = defineProps<{ lookupSettings: Settings | null }>();
const { t } = useI18n();
const metaStore = useAppMetaStore();
const settings = ref<Settings | null>(null);
const { savedAt, run } = useAutoSave();
watch(
  () => props.lookupSettings,
  (value) => {
    settings.value = value;
  },
  { immediate: true, deep: true },
);

async function updateSettings(updates: Partial<Settings>) {
  if (!settings.value) return;
  const next = { ...settings.value, ...updates };
  settings.value = next;
  await run(async () => {
    if (updates.language) await setLanguage(updates.language);
    await metaStore.updateSettings(next);
  });
}

const colorThemes = [
  { id: "blue", bg: "bg-blue-500" },
  { id: "indigo", bg: "bg-indigo-500" },
  { id: "purple", bg: "bg-purple-500" },
  { id: "pink", bg: "bg-pink-500" },
  { id: "rose", bg: "bg-rose-500" },
  { id: "red", bg: "bg-red-500" },
  { id: "orange", bg: "bg-orange-500" },
  { id: "yellow", bg: "bg-amber-500" },
  { id: "lime", bg: "bg-lime-500" },
  { id: "green", bg: "bg-green-500" },
  { id: "emerald", bg: "bg-emerald-500" },
  { id: "teal", bg: "bg-teal-500" },
  { id: "cyan", bg: "bg-cyan-500" },
  { id: "sky", bg: "bg-sky-500" },
  { id: "slate", bg: "bg-slate-500" },
] as const;

function setDarkTheme(val: 0 | 1 | 2) {
  updateSettings({ darkTheme: val });
}

function setColorTheme(theme: string) {
  updateSettings({ colorTheme: theme });
}

function toggleSetting(key: "monthlyPrice" | "convertCurrency" | "showOriginalPrice" | "showSubscriptionProgress") {
  if (!settings.value) return;
  updateSettings({ [key]: !settings.value[key] } as Partial<Settings>);
}

/** One choice over the two legacy flags (they conflicted as separate toggles). */
type InactiveMode = "show" | "bottom" | "hide";
const inactiveMode = computed<InactiveMode>(() =>
  settings.value?.hideDisabled ? "hide" : settings.value?.disabledToBottom ? "bottom" : "show",
);
const inactiveModeOptions = computed<SelectOption[]>(() => [
  { value: "show", label: t("inactive_subscriptions_show") },
  { value: "bottom", label: t("inactive_subscriptions_bottom") },
  { value: "hide", label: t("inactive_subscriptions_hide") },
]);
function setInactiveMode(mode: InactiveMode) {
  updateSettings({ disabledToBottom: mode === "bottom", hideDisabled: mode === "hide" });
}

const languages = getAvailableLanguages();
const languageSelectOptions = computed<SelectOption[]>(() =>
  languages.map((l) => ({ value: l.code, label: l.name }))
);

const pendingLang = ref<string | null>(null);
const showLangModal = ref(false);
useScrollLock(showLangModal);

function requestLangChange(lang: string) {
  if (lang === settings.value?.language) return;
  pendingLang.value = lang;
  showLangModal.value = true;
}

async function confirmWithTranslate() {
  if (pendingLang.value) {
    await updateSettings({ language: pendingLang.value });
  }
  closeLangModal();
}

async function confirmWithoutTranslate() {
  if (pendingLang.value) {
    await updateSettings({ language: pendingLang.value });
  }
  closeLangModal();
}

function closeLangModal() {
  showLangModal.value = false;
  pendingLang.value = null;
}
</script>

<template>
  <UiCard :title="t('appearance')">
    <template #actions>
      <SavedIndicator :at="savedAt" />
    </template>

    <h3 :class="[typo.subsection(), 'mb-2']">{{ t('theme') }}</h3>
    <div class="flex flex-wrap gap-2 mb-4 sm:mb-5">
      <button v-for="opt in ([{ val: 0 as const, icon: Sun, label: t('light_theme') }, { val: 1 as const, icon: Moon, label: t('dark_theme') }, { val: 2 as const, icon: Monitor, label: t('automatic') }])"
        :key="opt.val" type="button" @click="setDarkTheme(opt.val)"
        :aria-pressed="settings?.darkTheme === opt.val"
        :class="button({ pressed: settings?.darkTheme === opt.val, class: settings?.darkTheme === opt.val ? '' : 'text-text-secondary' })"
      >
        <component :is="opt.icon" :size="14" /> {{ opt.label }}
      </button>
    </div>

    <h3 :class="[typo.subsection(), 'mb-2']">{{ t('colors') }}</h3>
    <div class="flex flex-wrap gap-2.5 mb-5">
      <button v-for="theme in colorThemes" :key="theme.id" @click="setColorTheme(theme.id)"
        type="button"
        class="color-swatch appearance-none shrink-0 grow-0 basis-9 aspect-square w-9 h-9 min-w-9 min-h-9 p-0 m-0 leading-none rounded-full border-2 inline-flex items-center justify-center transition-transform duration-200 hover:scale-110 active:scale-95"
        :class="[settings?.colorTheme === theme.id ? 'border-text-primary scale-110 shadow-sm' : 'border-transparent', theme.bg]"
        :aria-pressed="settings?.colorTheme === theme.id"
        :aria-label="t('colors')"
      >
        <Check v-if="settings?.colorTheme === theme.id" :size="14" class="text-white drop-shadow-sm" />
      </button>
    </div>

    <div class="mb-5">
      <AppSelect :modelValue="settings?.language || 'en'" @update:modelValue="(v: string | number) => requestLangChange(String(v))" :options="languageSelectOptions" :label="t('language')" />
    </div>

    <h3 :class="[typo.subsection(), 'mb-2']">{{ t('display_settings') }}</h3>
    <div class="space-y-3">
      <AppToggle :modelValue="Boolean(settings?.monthlyPrice)" @update:modelValue="toggleSetting('monthlyPrice')" :label="t('calculate_monthly_price')" />
      <AppToggle :modelValue="Boolean(settings?.convertCurrency)" @update:modelValue="toggleSetting('convertCurrency')" :label="t('convert_prices')" />
      <AppToggle :modelValue="Boolean(settings?.showOriginalPrice)" @update:modelValue="toggleSetting('showOriginalPrice')" :label="t('show_original_price')" />
      <AppToggle :modelValue="Boolean(settings?.showSubscriptionProgress)" @update:modelValue="toggleSetting('showSubscriptionProgress')" :label="t('show_subscription_progress')" />
      <AppSelect
        :modelValue="inactiveMode"
        @update:modelValue="(v: string | number) => setInactiveMode(String(v) as InactiveMode)"
        :options="inactiveModeOptions"
        :label="t('inactive_subscriptions_mode')"
      />
    </div>
  </UiCard>

  <!-- Language change confirmation modal -->
  <Modal :show="showLangModal" :title="t('language_change')" max-width="28rem" @close="closeLangModal">
    <p class="text-sm text-text-secondary">{{ t('language_change_desc') }}</p>
    <p class="mt-1.5 text-xs text-text-muted">{{ t('language_change_hint') }}</p>
    <template #footer>
      <button :class="button({ intent: 'ghost' })" @click="closeLangModal">{{ t('cancel') }}</button>
      <button :class="button()" @click="confirmWithoutTranslate">{{ t('language_change_keep') }}</button>
      <button :class="button({ intent: 'primary' })" @click="confirmWithTranslate">{{ t('language_change_translate') }}</button>
    </template>
  </Modal>
</template>
