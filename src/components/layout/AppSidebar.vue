<script setup lang="ts">
import { computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import { tv, iconSize } from "@/lib/tv";
import { version } from "../../../package.json";
import {
  LayoutDashboard,
  CreditCard,
  Wallet,
  Calendar,
  Settings,
  ArrowRightLeft,
} from "@lucide/vue";

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
    root: "app-sidebar w-[232px] shrink-0 bg-sidebar border-r border-border flex flex-col h-full",
    logoWrap: "h-14 flex items-center px-4",
    logoIcon:
      "w-7 h-7 rounded-[9px] bg-gradient-to-br from-primary to-accent flex items-center justify-center shadow-sm",
    logoText: "font-semibold text-[15px] text-text-primary tracking-[-0.01em]",
    nav: "flex-1 px-2.5 py-2 space-y-0.5 overflow-y-auto",
    navItem:
      "flex items-center gap-2.5 h-9 px-2.5 rounded-lg text-[13px] font-medium transition-colors cursor-pointer",
    footer: "px-4 py-3",
    footerText: "text-[11px] text-text-muted",
  },
  variants: {
    active: {
      true: {
        navItem:
          "relative bg-primary-light text-primary font-semibold before:absolute before:left-0 before:top-2 before:bottom-2 before:w-[3px] before:rounded-r-full before:bg-primary",
      },
      false: {
        navItem: "text-text-secondary hover:bg-surface-hover hover:text-text-primary [&>svg]:opacity-80",
      },
    },
  },
});

const slots = sidebarTv();
</script>

<template>
  <aside :class="[slots.root(), { 'app-sidebar--mobile': mobile }]">
    <div :class="slots.logoWrap()">
      <div class="flex items-center gap-2">
        <div :class="slots.logoIcon()">
          <span class="text-white font-semibold text-[13px]">S</span>
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
          <component :is="item.icon" :size="iconSize.sm" :stroke-width="2" />
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
          <component :is="item.icon" :size="iconSize.sm" :stroke-width="2" />
          <span>{{ item.label }}</span>
        </router-link>
      </template>
    </nav>

    <div :class="slots.footer()">
      <p :class="slots.footerText()">Subly {{ version }}</p>
    </div>
  </aside>
</template>

<style scoped>
/* Mobile drawer покрывает status bar / home indicator: padding'ом сдвигаем
   контент ниже часов и выше home indicator. На desktop инсеты = 0, эффекта нет. */
.app-sidebar--mobile {
  padding-top: var(--sat, 0px);
  padding-bottom: var(--sab, 0px);
}
</style>
