<script setup lang="ts">
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
import { ui, typo } from "@/lib/tv";

const props = defineProps<{ lookupSettings: Settings | null }>();
const { t } = useI18n();
const metaStore = useAppMetaStore();
const settings = ref<Settings | null>(null);
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
  if (updates.language) await setLanguage(updates.language);
  await metaStore.updateSettings(next);
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

function toggleSetting(key: "monthlyPrice" | "convertCurrency" | "showOriginalPrice" | "showSubscriptionProgress" | "disabledToBottom" | "hideDisabled") {
  if (!settings.value) return;
  updateSettings({ [key]: !settings.value[key] } as Partial<Settings>);
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
  <section class="bg-surface rounded-xl border border-border p-4 sm:p-5">
    <h2 :class="[ui.sectionTitle(), 'mb-3 sm:mb-4']">{{ t('appearance') }}</h2>

    <h3 :class="[typo.subsection(), 'mb-2']">{{ t('theme') }}</h3>
    <div class="flex flex-wrap gap-2 mb-4 sm:mb-5">
      <button v-for="opt in ([{ val: 0 as const, icon: Sun, label: t('light_theme') }, { val: 1 as const, icon: Moon, label: t('dark_theme') }, { val: 2 as const, icon: Monitor, label: t('automatic') }])"
        :key="opt.val" @click="setDarkTheme(opt.val)"
        class="flex items-center gap-1.5 sm:gap-2 px-3 sm:px-4 py-1.5 sm:py-2 rounded-lg border text-xs sm:text-sm font-medium"
        :class="settings?.darkTheme === opt.val ? 'border-primary bg-primary-light text-primary' : 'border-border text-text-secondary'"
      >
        <component :is="opt.icon" :size="14" /> {{ opt.label }}
      </button>
    </div>

    <h3 :class="[typo.subsection(), 'mb-2']">{{ t('colors') }}</h3>
    <div class="flex flex-wrap gap-2 mb-5">
      <button v-for="theme in colorThemes" :key="theme.id" @click="setColorTheme(theme.id)"
        class="w-8 h-8 rounded-full border-2 flex items-center justify-center transition-transform hover:scale-110"
        :class="[settings?.colorTheme === theme.id ? 'border-text-primary scale-110' : 'border-transparent', theme.bg]"
        :title="theme.id"
      >
        <Check v-if="settings?.colorTheme === theme.id" :size="14" class="text-white" />
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
      <AppToggle :modelValue="Boolean(settings?.disabledToBottom)" @update:modelValue="toggleSetting('disabledToBottom')" :label="t('show_disabled_subscriptions_at_the_bottom')" />
      <AppToggle :modelValue="Boolean(settings?.hideDisabled)" @update:modelValue="toggleSetting('hideDisabled')" :label="t('hide_disabled_subscriptions')" />
    </div>
  </section>

  <!-- Language change confirmation modal -->
  <Teleport to="body">
    <Transition name="app-modal">
      <div v-if="showLangModal" class="fixed inset-0 z-50 flex items-end sm:items-center justify-center sm:p-4">
        <div class="app-modal-backdrop absolute inset-0 bg-black/50" @click="closeLangModal" />
        <div class="app-modal-panel relative bg-surface w-full rounded-t-2xl sm:rounded-xl shadow-2xl sm:max-w-md p-4 sm:p-6">
          <div class="flex items-center gap-3 mb-3">
            <div class="w-10 h-10 rounded-full bg-blue-100 dark:bg-blue-900/30 flex items-center justify-center shrink-0">
              <Languages :size="20" class="text-blue-500" />
            </div>
            <div>
              <h3 :class="ui.sectionTitle()">{{ t('language_change') }}</h3>
            </div>
          </div>

          <p class="text-sm text-text-secondary mb-2">{{ t('language_change_desc') }}</p>
          <p class="text-xs text-text-muted mb-5">{{ t('language_change_hint') }}</p>

          <div class="flex flex-col sm:flex-row gap-2">
            <button
              @click="confirmWithTranslate"
              class="flex-1 px-4 py-2.5 rounded-lg bg-primary text-white text-sm font-medium hover:bg-primary-hover transition-colors"
            >
              {{ t('language_change_translate') }}
            </button>
            <button
              @click="confirmWithoutTranslate"
              class="flex-1 px-4 py-2.5 rounded-lg border border-border text-sm font-medium text-text-primary hover:bg-surface-hover transition-colors"
            >
              {{ t('language_change_keep') }}
            </button>
            <button
              @click="closeLangModal"
              class="flex-1 px-4 py-2.5 rounded-lg text-sm font-medium text-text-muted hover:bg-surface-hover transition-colors"
            >
              {{ t('cancel') }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
