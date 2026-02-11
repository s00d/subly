<script setup lang="ts">
import { useToast } from "@/composables/useToast";
import Toast from "@/components/ui/Toast.vue";
import AppearanceSection from "@/components/settings/AppearanceSection.vue";
import BudgetNotificationsSection from "@/components/settings/BudgetNotificationsSection.vue";
import FixerSection from "@/components/settings/FixerSection.vue";
import TelegramSection from "@/components/settings/TelegramSection.vue";
import HouseholdSection from "@/components/settings/HouseholdSection.vue";
import CategoriesSection from "@/components/settings/CategoriesSection.vue";
import CurrenciesSection from "@/components/settings/CurrenciesSection.vue";
import PaymentMethodsSection from "@/components/settings/PaymentMethodsSection.vue";
import TagsSection from "@/components/settings/TagsSection.vue";
import DataManagementSection from "@/components/settings/DataManagementSection.vue";

const { showToast, toastMsg, toastType, closeToast } = useToast();
</script>

<template>
  <div class="max-w-3xl mx-auto space-y-4 sm:space-y-6">
    <AppearanceSection />
    <BudgetNotificationsSection />
    <FixerSection />
    <TelegramSection />
    <HouseholdSection />
    <CategoriesSection />
    <CurrenciesSection />
    <PaymentMethodsSection />
    <TagsSection />
    <DataManagementSection />
    <Toast :show="showToast" :message="toastMsg" :type="toastType" @close="closeToast" />
  </div>
</template>
