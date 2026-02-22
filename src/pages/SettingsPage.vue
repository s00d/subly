<script setup lang="ts">
import { ref, computed, defineAsyncComponent, onMounted, type Component } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import { useHeaderActions } from "@/composables/useHeaderActions";
import { tv } from "@/lib/tv";
import Toast from "@/components/ui/Toast.vue";
import {
  Palette, Bell, FolderTree, Coins, CreditCard, Tags,
  Users, Cloud, Database, ArrowRightLeft, Send,
} from "lucide-vue-next";

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

const tabsTv = tv({
  slots: {
    nav: "flex border-b border-[var(--color-border)] mb-4 sm:mb-6 overflow-x-auto scrollbar-none -mx-1 px-1",
    tab: [
      "flex items-center gap-1.5 px-3 sm:px-4 py-2.5 text-xs sm:text-sm font-medium whitespace-nowrap",
      "border-b-2 transition-colors cursor-pointer shrink-0",
    ],
    body: "space-y-4 sm:space-y-6",
  },
  variants: {
    active: {
      true: {
        tab: "border-[var(--color-primary)] text-[var(--color-primary)]",
      },
      false: {
        tab: "border-transparent text-[var(--color-text-muted)] hover:text-[var(--color-text-secondary)] hover:border-[var(--color-border)]",
      },
    },
  },
});

const slots = tabsTv();
</script>

<template>
  <div class="max-w-3xl mx-auto">
    <!-- Tab navigation -->
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

    <!-- Tab content -->
    <div :class="slots.body()">
      <template v-if="activeTab === 'general'">
        <AppearanceSection />
        <BudgetNotificationsSection />
        <HouseholdSection />
      </template>

      <template v-if="activeTab === 'catalogs'">
        <CategoriesSection />
        <CurrenciesSection />
        <PaymentMethodsSection />
        <TagsSection />
        <ConverterPresetsSection />
      </template>

      <template v-if="activeTab === 'integrations'">
        <ExchangeRatesSection />
        <RateHistorySection />
        <TelegramSection />
        <CloudSyncSection />
      </template>

      <template v-if="activeTab === 'data'">
        <DataManagementSection />
      </template>
    </div>

    <Toast :show="showToast" :message="toastMsg" :type="toastType" @close="closeToast" />
  </div>
</template>
