<script setup lang="ts">
import { computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import { tv } from "@/lib/tv";
import { version } from "../../../package.json";
import {
  LayoutDashboard,
  CreditCard,
  Wallet,
  Calendar,
  Settings,
  ArrowRightLeft,
} from "lucide-vue-next";

const props = withDefaults(defineProps<{
  mobile?: boolean;
}>(), { mobile: false });

const emit = defineEmits<{
  navigate: [];
}>();

const route = useRoute();
const router = useRouter();
const { t } = useI18n();

const navItems = computed(() => [
  { path: "/", name: "dashboard", icon: LayoutDashboard, label: t("dashboard") },
  { path: "/subscriptions", name: "subscriptions", icon: CreditCard, label: t("subscriptions") },
  { path: "/expenses", name: "expenses", icon: Wallet, label: t("expenses") },
  { path: "/calendar", name: "calendar", icon: Calendar, label: t("calendar") },
  { path: "/currencies", name: "currencies", icon: ArrowRightLeft, label: t("exchange_rates") },
  { path: "/settings", name: "settings", icon: Settings, label: t("settings") },
]);

function isActive(name: string): boolean {
  return route.name === name;
}

function handleNav(path: string) {
  router.push(path);
  emit("navigate");
}

const sidebarTv = tv({
  slots: {
    root: "w-56 shrink-0 bg-[var(--color-surface)] border-r border-[var(--color-border)] flex flex-col h-full",
    logoWrap: "h-16 flex items-center px-5 border-b border-[var(--color-border)]",
    logoIcon: "w-8 h-8 rounded-lg bg-[var(--color-primary)] flex items-center justify-center",
    logoText: "font-semibold text-lg text-[var(--color-text-primary)]",
    nav: "flex-1 p-3 space-y-1",
    navItem: "flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all cursor-pointer",
    footer: "p-4 border-t border-[var(--color-border)]",
    footerText: "text-xs text-[var(--color-text-muted)]",
  },
  variants: {
    active: {
      true: { navItem: "bg-[var(--color-primary)] text-white shadow-sm" },
      false: { navItem: "text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text-primary)]" },
    },
  },
});

const slots = sidebarTv();
</script>

<template>
  <aside :class="slots.root()">
    <div :class="slots.logoWrap()">
      <div class="flex items-center gap-2">
        <div :class="slots.logoIcon()">
          <span class="text-white font-bold text-sm">S</span>
        </div>
        <span :class="slots.logoText()">Subly</span>
      </div>
    </div>

    <nav :class="slots.nav()">
      <template v-if="mobile">
        <button
          v-for="item in navItems"
          :key="item.name"
          @click="handleNav(item.path)"
          :class="sidebarTv({ active: isActive(item.name) }).navItem()"
          class="w-full text-left"
        >
          <component :is="item.icon" :size="20" />
          <span>{{ item.label }}</span>
        </button>
      </template>
      <template v-else>
        <router-link
          v-for="item in navItems"
          :key="item.name"
          :to="item.path"
          :class="sidebarTv({ active: isActive(item.name) }).navItem()"
        >
          <component :is="item.icon" :size="20" />
          <span>{{ item.label }}</span>
        </router-link>
      </template>
    </nav>

    <div :class="slots.footer()">
      <p :class="slots.footerText()">Subly v{{ version }}</p>
      <p :class="slots.footerText()">&copy; {{ new Date().getFullYear() }} Subly</p>
    </div>
  </aside>
</template>
