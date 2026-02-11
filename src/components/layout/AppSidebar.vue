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
  <aside class="w-56 shrink-0 bg-[var(--color-surface)] border-r border-[var(--color-border)] hidden md:flex flex-col h-full">
    <!-- Logo -->
    <div class="h-16 flex items-center px-5 border-b border-[var(--color-border)]">
      <div class="flex items-center gap-2">
        <div class="w-8 h-8 rounded-lg bg-[var(--color-primary)] flex items-center justify-center">
          <span class="text-white font-bold text-sm">W</span>
        </div>
        <span class="font-semibold text-lg text-[var(--color-text-primary)]">Subly</span>
      </div>
    </div>

    <!-- Navigation -->
    <nav class="flex-1 p-3 space-y-1">
      <router-link
        v-for="item in navItems"
        :key="item.name"
        :to="item.path"
        class="flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all"
        :class="[
          isActive(item.name)
            ? 'bg-[var(--color-primary)] text-white shadow-sm'
            : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text-primary)]'
        ]"
      >
        <component :is="item.icon" :size="20" />
        <span>{{ item.label }}</span>
      </router-link>
    </nav>

    <!-- Footer -->
    <div class="p-4 border-t border-[var(--color-border)]">
      <p class="text-xs text-[var(--color-text-muted)]">Subly v0.1.1</p>
    </div>
  </aside>
</template>
