<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";
import { useI18n } from "@/i18n";
import {
  LayoutDashboard,
  CreditCard,
  Wallet,
  Calendar,
  Settings,
} from "lucide-vue-next";

const route = useRoute();
const { t } = useI18n();

const navItems = computed(() => [
  { path: "/", name: "dashboard", icon: LayoutDashboard, label: t("dashboard") },
  { path: "/subscriptions", name: "subscriptions", icon: CreditCard, label: t("subscriptions") },
  { path: "/expenses", name: "expenses", icon: Wallet, label: t("expenses") },
  { path: "/calendar", name: "calendar", icon: Calendar, label: t("calendar") },
  { path: "/settings", name: "settings", icon: Settings, label: t("settings") },
]);

function isActive(name: string): boolean {
  return route.name === name;
}
</script>

<template>
  <nav class="fixed bottom-0 left-0 right-0 z-40 bg-[var(--color-surface)] border-t border-[var(--color-border)] safe-area-bottom">
    <div class="flex items-center justify-around h-14">
      <router-link
        v-for="item in navItems"
        :key="item.name"
        :to="item.path"
        class="flex flex-col items-center justify-center gap-0.5 flex-1 h-full transition-colors"
        :class="isActive(item.name)
          ? 'text-[var(--color-primary)]'
          : 'text-[var(--color-text-muted)]'"
      >
        <component :is="item.icon" :size="20" />
        <span class="text-[10px] font-medium leading-tight">{{ item.label }}</span>
      </router-link>
    </div>
  </nav>
</template>

<style scoped>
.safe-area-bottom {
  padding-bottom: env(safe-area-inset-bottom, 0px);
}
</style>
