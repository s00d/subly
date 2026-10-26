<script setup lang="ts">
import { ref, computed, defineAsyncComponent, onMounted, watch, type Component } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter } from "vue-router";
import { useToast } from "@/composables/useToast";
import { useHeaderActions } from "@/composables/useHeaderActions";
import { tv } from "@/lib/tv";
import { storeToRefs } from "pinia";
import { useAppMetaStore } from "@/stores/appMetaStore";
import { useCatalogsUsageStore } from "@/stores/catalogsUsageStore";
import Toast from "@/components/ui/Toast.vue";
import {
  Palette, Bell, FolderTree, Coins, Database, Sparkles,
} from "@lucide/vue";

import AppearanceSection from "@/components/settings/AppearanceSection.vue";
import BudgetNotificationsSection from "@/components/settings/BudgetNotificationsSection.vue";

const CategoriesSection = defineAsyncComponent(() => import("@/components/settings/CategoriesSection.vue"));
const CurrenciesSection = defineAsyncComponent(() => import("@/components/settings/CurrenciesSection.vue"));
const PaymentMethodsSection = defineAsyncComponent(() => import("@/components/settings/PaymentMethodsSection.vue"));
const TagsSection = defineAsyncComponent(() => import("@/components/settings/TagsSection.vue"));
const HouseholdSection = defineAsyncComponent(() => import("@/components/settings/HouseholdSection.vue"));
const ExchangeRatesSection = defineAsyncComponent(() => import("@/components/settings/ExchangeRatesSection.vue"));
const AiSection = defineAsyncComponent(() => import("@/components/settings/AiSection.vue"));
const TelegramSection = defineAsyncComponent(() => import("@/components/settings/TelegramSection.vue"));
const CloudSyncSection = defineAsyncComponent(() => import("@/components/settings/CloudSyncSection.vue"));
const ConverterPresetsSection = defineAsyncComponent(() => import("@/components/settings/ConverterPresetsSection.vue"));
const RateHistorySection = defineAsyncComponent(() => import("@/components/settings/RateHistorySection.vue"));
const DataManagementSection = defineAsyncComponent(() => import("@/components/settings/DataManagementSection.vue"));

const { t } = useI18n();
const { clearActions } = useHeaderActions();
onMounted(() => clearActions());
const { showToast, toastMsg, toastType, closeToast } = useToast();
const metaStore = useAppMetaStore();
const catalogsUsageStore = useCatalogsUsageStore();
const { settings, categories, currencies, paymentMethods, household, tags } = storeToRefs(metaStore);
const { usage: catalogsUsage } = storeToRefs(catalogsUsageStore);
onMounted(async () => {
  await metaStore.ensureLoaded();
});

interface SettingsTab {
  id: string;
  labelKey: string;
  icon: Component;
}

/** Grouped by what the user is trying to do, one topic per tab. */
const tabs: SettingsTab[] = [
  { id: "appearance", labelKey: "appearance", icon: Palette },
  { id: "notifications", labelKey: "notifications", icon: Bell },
  { id: "catalogs", labelKey: "settings_tab_catalogs", icon: FolderTree },
  { id: "currencies", labelKey: "currencies", icon: Coins },
  { id: "ai", labelKey: "ai_section_title", icon: Sparkles },
  { id: "data", labelKey: "settings_tab_data", icon: Database },
];

const route = useRoute();
const router = useRouter();
const tabIds = new Set(tabs.map((t) => t.id));
/** `/settings?section=<tab>` deep-links (e.g. "set up AI first") and keeps the tab across reloads. */
const activeTab = ref(tabIds.has(String(route.query.section)) ? String(route.query.section) : "appearance");
watch(activeTab, (tab) => {
  if (route.query.section !== tab) void router.replace({ query: { ...route.query, section: tab } });
});
watch(
  () => route.query.section,
  (section) => {
    if (typeof section === "string" && tabIds.has(section)) activeTab.value = section;
  },
);

watch(activeTab, async (tab) => {
  // Usage counts drive the "in use" badges / delete guards in catalog and currency lists.
  if (tab === "catalogs" || tab === "currencies") {
    await catalogsUsageStore.ensureLoaded();
  }
}, { immediate: true }); // `?section=catalogs` deep links start on that tab

const ready = computed(() => !!settings.value);

const tabsTv = tv({
  slots: {
    layout: "md:grid md:grid-cols-[200px_minmax(0,1fr)] md:gap-6 md:items-start",
    // Phones: horizontal scrolling tabs; desktop: sticky vertical section nav
    nav: [
      "flex gap-1 overflow-x-auto scrollbar-none mb-4 p-1 rounded-xl bg-surface-hover",
      "md:sticky md:top-2 md:flex-col md:mb-0 md:p-0 md:bg-transparent md:rounded-none",
    ],
    tab: [
      "flex items-center gap-2 h-9 px-3 rounded-lg text-[13px] font-medium whitespace-nowrap shrink-0",
      "transition-colors cursor-pointer",
    ],
    body: "space-y-4 min-w-0",
  },
  variants: {
    active: {
      true: { tab: "bg-surface text-text-primary shadow-sm md:bg-primary-light md:text-primary md:shadow-none" },
      false: { tab: "text-text-muted hover:text-text-primary md:hover:bg-surface-hover" },
    },
  },
});

const slots = tabsTv();
</script>

<template>
  <div :class="slots.layout()">
    <nav :class="slots.nav()" :aria-label="t('settings')">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        @click="activeTab = tab.id"
        :class="tabsTv({ active: activeTab === tab.id }).tab()"
        :aria-current="activeTab === tab.id ? 'page' : undefined"
        data-compact
      >
        <component :is="tab.icon" :size="15" />
        {{ t(tab.labelKey) }}
      </button>
    </nav>

    <!-- Tab content -->
    <div :class="slots.body()">
      <template v-if="ready">
        <template v-if="activeTab === 'appearance'">
          <AppearanceSection :lookupSettings="settings!" />
        </template>

        <template v-if="activeTab === 'notifications'">
          <BudgetNotificationsSection :lookupSettings="settings!" />
          <TelegramSection />
        </template>

        <template v-if="activeTab === 'catalogs'">
          <CategoriesSection :lookupData="{ categories, settings: settings!, categoryUsage: catalogsUsage?.categoryUsage ?? {} }" />
          <PaymentMethodsSection :lookupData="{ paymentMethods, settings: settings!, paymentMethodUsage: catalogsUsage?.paymentMethodUsage ?? {} }" />
          <TagsSection :lookupData="{ tags, tagUsage: catalogsUsage?.tagUsage ?? {} }" />
          <HouseholdSection :lookupHousehold="household" />
        </template>

        <template v-if="activeTab === 'currencies'">
          <CurrenciesSection :lookupData="{ currencies, settings: settings!, currencyUsage: catalogsUsage?.currencyUsage ?? {} }" />
          <ExchangeRatesSection :lookupData="{ settings: settings!, currencies }" />
          <RateHistorySection :lookupSettings="settings!" />
          <ConverterPresetsSection :lookupSettings="settings!" />
        </template>

        <template v-if="activeTab === 'ai'">
          <AiSection />
        </template>

        <template v-if="activeTab === 'data'">
          <CloudSyncSection />
          <DataManagementSection />
        </template>
      </template>
      <template v-else>
        <div class="text-sm text-text-muted py-4">
          {{ t("loading") }}…
        </div>
      </template>
    </div>

    <Toast :show="showToast" :message="toastMsg" :type="toastType" @close="closeToast" />
  </div>
</template>
