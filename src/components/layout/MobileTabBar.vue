<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";
import { useI18n } from "vue-i18n";
import { tv } from "@/lib/tv";
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

const tabBarTv = tv({
  slots: {
    root: "fixed bottom-0 left-0 right-0 z-40 bg-[var(--color-surface)] border-t border-[var(--color-border)] safe-area-bottom",
    inner: "flex items-center justify-around h-14",
    tab: "flex flex-col items-center justify-center gap-0.5 flex-1 h-full transition-colors",
    tabLabel: "text-[10px] font-medium leading-tight",
  },
  variants: {
    active: {
      true: { tab: "text-[var(--color-primary)]" },
      false: { tab: "text-[var(--color-text-muted)]" },
    },
  },
});

const slots = tabBarTv();
</script>

<template>
  <nav :class="slots.root()">
    <div :class="slots.inner()">
      <router-link
        v-for="item in navItems"
        :key="item.name"
        :to="item.path"
        :class="tabBarTv({ active: isActive(item.name) }).tab()"
      >
        <component :is="item.icon" :size="20" />
        <span :class="slots.tabLabel()">{{ item.label }}</span>
      </router-link>
    </div>
  </nav>
</template>

<style scoped>
.safe-area-bottom {
  padding-bottom: env(safe-area-inset-bottom, 0px);
}
</style>
