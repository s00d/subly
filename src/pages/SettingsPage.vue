<script setup lang="ts">
import { ref, computed, defineAsyncComponent, onMounted, watch, type Component } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import { useHeaderActions } from "@/composables/useHeaderActions";
import { tv } from "@/lib/tv";
import { storeToRefs } from "pinia";
import { useAppMetaStore } from "@/stores/appMetaStore";
import { useCatalogsUsageStore } from "@/stores/catalogsUsageStore";
import Toast from "@/components/ui/Toast.vue";
import {
  Palette, Bell, FolderTree, Coins, CreditCard, Tags,
  Users, Cloud, Database, ArrowRightLeft, Send,
} from "@lucide/vue";

import AppearanceSection from "@/components/settings/AppearanceSection.vue";
import BudgetNotificationsSection from "@/components/settings/BudgetNotificationsSection.vue";

const CategoriesSection = defineAsyncComponent(() => import("@/components/settings/CategoriesSection.vue"));
const CurrenciesSection = defineAsyncComponent(() => import("@/components/settings/CurrenciesSection.vue"));
const PaymentMethodsSection = defineAsyncComponent(() => import("@/components/settings/PaymentMethodsSection.vue"));
const TagsSection = defineAsyncComponent(() => import("@/components/settings/TagsSection.vue"));
const HouseholdSection = defineAsyncComponent(() => import("@/components/settings/HouseholdSection.vue"));
const ExchangeRatesSection = defineAsyncComponent(() => import("@/components/settings/ExchangeRatesSection.vue"));
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

const tabs: SettingsTab[] = [
  { id: "general", labelKey: "settings_tab_general", icon: Palette },
  { id: "catalogs", labelKey: "settings_tab_catalogs", icon: FolderTree },
  { id: "integrations", labelKey: "settings_tab_integrations", icon: ArrowRightLeft },
  { id: "data", labelKey: "settings_tab_data", icon: Database },
];

const activeTab = ref("general");

watch(activeTab, async (tab) => {
  if (tab === "catalogs") {
    await catalogsUsageStore.ensureLoaded();
  }
});

const ready = computed(() => !!settings.value);

const tabsTv = tv({
  slots: {
    navWrap: "settings-tabs-sticky mb-4 sm:mb-6",
    nav: "settings-tabs-nav",
    tab: [
      "flex items-center gap-1.5 px-3 sm:px-4 py-2.5 text-xs sm:text-sm font-medium whitespace-nowrap",
      "border-b-2 transition-colors cursor-pointer shrink-0",
    ],
    body: "space-y-4 sm:space-y-6",
  },
  variants: {
    active: {
      true: {
        tab: "border-primary text-primary",
      },
      false: {
        tab: "border-transparent text-text-muted hover:text-text-secondary hover:border-border",
      },
    },
  },
});

const slots = tabsTv();
</script>

<template>
  <div class="max-w-3xl mx-auto">
    <!-- Tab navigation -->
    <div :class="slots.navWrap()">
      <nav :class="slots.nav()">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          @click="activeTab = tab.id"
          :class="tabsTv({ active: activeTab === tab.id }).tab()"
          data-compact
        >
          <component :is="tab.icon" :size="15" />
          {{ t(tab.labelKey) }}
        </button>
      </nav>
    </div>

    <!-- Tab content -->
    <div :class="slots.body()">
      <template v-if="ready">
        <template v-if="activeTab === 'general'">
          <AppearanceSection :lookupSettings="settings!" />
          <BudgetNotificationsSection :lookupSettings="settings!" />
          <HouseholdSection :lookupHousehold="household" />
        </template>

        <template v-if="activeTab === 'catalogs'">
          <CategoriesSection :lookupData="{ categories, settings: settings!, categoryUsage: catalogsUsage?.categoryUsage ?? {} }" />
          <CurrenciesSection :lookupData="{ currencies, settings: settings!, currencyUsage: catalogsUsage?.currencyUsage ?? {} }" />
          <PaymentMethodsSection :lookupData="{ paymentMethods, settings: settings!, paymentMethodUsage: catalogsUsage?.paymentMethodUsage ?? {} }" />
          <TagsSection :lookupData="{ tags, tagUsage: catalogsUsage?.tagUsage ?? {} }" />
          <ConverterPresetsSection :lookupSettings="settings!" />
        </template>

        <template v-if="activeTab === 'integrations'">
          <ExchangeRatesSection :lookupData="{ settings: settings!, currencies }" />
          <RateHistorySection :lookupSettings="settings!" />
          <TelegramSection />
          <CloudSyncSection />
        </template>

        <template v-if="activeTab === 'data'">
          <DataManagementSection />
        </template>
      </template>
      <template v-else>
        <div class="text-sm text-text-muted py-4">
          {{ t("loading") }}...
        </div>
      </template>
    </div>

    <Toast :show="showToast" :message="toastMsg" :type="toastType" @close="closeToast" />
  </div>
</template>

<style scoped>
.settings-tabs-sticky {
  position: sticky;
  top: -0.75rem;
  z-index: 20;
  background: var(--color-surface-secondary);
  border-bottom: 1px solid var(--color-border);
}

.settings-tabs-nav {
  display: flex;
  overflow-x: auto;
  scrollbar-width: none;
  -ms-overflow-style: none;
  padding: 0 0.25rem;
}

.settings-tabs-nav::-webkit-scrollbar {
  display: none;
}

@media (min-width: 640px) {
  .settings-tabs-sticky {
    top: -1rem;
  }
}

@media (min-width: 768px) {
  .settings-tabs-sticky {
    top: -1.5rem;
  }
}
</style>
