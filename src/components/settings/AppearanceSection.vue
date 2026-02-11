<script setup lang="ts">
import { computed } from "vue";
import { useAppStore } from "@/stores/appStore";
import { useI18n } from "@/i18n";
import { getAvailableLanguages } from "@/i18n";
import AppSelect from "@/components/ui/AppSelect.vue";
import AppToggle from "@/components/ui/AppToggle.vue";
import type { SelectOption } from "@/components/ui/AppSelect.vue";
import { Sun, Moon, Monitor, Check } from "lucide-vue-next";

const store = useAppStore();
const { t } = useI18n();

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
  store.updateSettings({ darkTheme: val });
  if (val === 1) document.documentElement.classList.add("dark");
  else if (val === 0) document.documentElement.classList.remove("dark");
  else {
    const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    document.documentElement.classList.toggle("dark", prefersDark);
  }
}

function setColorTheme(theme: string) {
  store.updateSettings({ colorTheme: theme });
  document.body.className = document.body.className.replace(/theme-\w+/g, "");
  if (theme !== "blue") document.body.classList.add(`theme-${theme}`);
}

function toggleSetting(key: "monthlyPrice" | "convertCurrency" | "showOriginalPrice" | "showSubscriptionProgress" | "disabledToBottom" | "hideDisabled") {
  store.updateSettings({ [key]: !store.state.settings[key] });
}

const languages = getAvailableLanguages();
const languageSelectOptions = computed<SelectOption[]>(() =>
  languages.map((l) => ({ value: l.code, label: l.name }))
);

function setLang(lang: string) {
  store.updateSettings({ language: lang });
}
</script>

<template>
  <section class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-4 sm:p-5">
    <h2 class="text-base sm:text-lg font-semibold text-[var(--color-text-primary)] mb-3 sm:mb-4">{{ t('appearance') }}</h2>

    <h3 class="text-xs sm:text-sm font-medium text-[var(--color-text-secondary)] mb-2">{{ t('theme') }}</h3>
    <div class="flex flex-wrap gap-2 mb-4 sm:mb-5">
      <button v-for="opt in ([{ val: 0 as const, icon: Sun, label: t('light_theme') }, { val: 1 as const, icon: Moon, label: t('dark_theme') }, { val: 2 as const, icon: Monitor, label: t('automatic') }])"
        :key="opt.val" @click="setDarkTheme(opt.val)"
        class="flex items-center gap-1.5 sm:gap-2 px-3 sm:px-4 py-1.5 sm:py-2 rounded-lg border text-xs sm:text-sm font-medium"
        :class="store.state.settings.darkTheme === opt.val ? 'border-[var(--color-primary)] bg-[var(--color-primary-light)] text-[var(--color-primary)]' : 'border-[var(--color-border)] text-[var(--color-text-secondary)]'"
      >
        <component :is="opt.icon" :size="14" /> {{ opt.label }}
      </button>
    </div>

    <h3 class="text-sm font-medium text-[var(--color-text-secondary)] mb-2">{{ t('colors') }}</h3>
    <div class="flex flex-wrap gap-2 mb-5">
      <button v-for="theme in colorThemes" :key="theme.id" @click="setColorTheme(theme.id)"
        class="w-8 h-8 rounded-full border-2 flex items-center justify-center transition-transform hover:scale-110"
        :class="[store.state.settings.colorTheme === theme.id ? 'border-[var(--color-text-primary)] scale-110' : 'border-transparent', theme.bg]"
        :title="theme.id"
      >
        <Check v-if="store.state.settings.colorTheme === theme.id" :size="14" class="text-white" />
      </button>
    </div>

    <div class="mb-5">
      <AppSelect :modelValue="store.state.settings.language" @update:modelValue="(v: any) => setLang(String(v))" :options="languageSelectOptions" :label="t('language')" />
    </div>

    <h3 class="text-sm font-medium text-[var(--color-text-secondary)] mb-2">{{ t('display_settings') }}</h3>
    <div class="space-y-3">
      <AppToggle :modelValue="store.state.settings.monthlyPrice" @update:modelValue="toggleSetting('monthlyPrice')" :label="t('calculate_monthly_price')" />
      <AppToggle :modelValue="store.state.settings.convertCurrency" @update:modelValue="toggleSetting('convertCurrency')" :label="t('convert_prices')" />
      <AppToggle :modelValue="store.state.settings.showOriginalPrice" @update:modelValue="toggleSetting('showOriginalPrice')" :label="t('show_original_price')" />
      <AppToggle :modelValue="store.state.settings.showSubscriptionProgress" @update:modelValue="toggleSetting('showSubscriptionProgress')" :label="t('show_subscription_progress')" />
      <AppToggle :modelValue="store.state.settings.disabledToBottom" @update:modelValue="toggleSetting('disabledToBottom')" :label="t('show_disabled_subscriptions_at_the_bottom')" />
      <AppToggle :modelValue="store.state.settings.hideDisabled" @update:modelValue="toggleSetting('hideDisabled')" :label="t('hide_disabled_subscriptions')" />
    </div>
  </section>
</template>
