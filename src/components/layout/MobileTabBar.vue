<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";
import { useI18n } from "vue-i18n";
import { useMobileTabBarViewport } from "@/composables/useMobileTabBarViewport";
import { tv, iconSize } from "@/lib/tv";
import {
  LayoutDashboard,
  CreditCard,
  Wallet,
  Calendar,
  Settings,
} from "@lucide/vue";

const route = useRoute();
const { t } = useI18n();

const navItems = computed(() => [
  { path: "/", name: "dashboard", icon: LayoutDashboard, label: t("dashboard") },
  { path: "/subscriptions", name: "subscriptions", icon: CreditCard, label: t("subscriptions") },
  { path: "/expenses", name: "expenses", icon: Wallet, label: t("expenses") },
  { path: "/calendar", name: "calendar", icon: Calendar, label: t("calendar") },
  { path: "/settings", name: "settings", icon: Settings, label: t("settings") },
]);

const activeIndex = computed(() => {
  const i = navItems.value.findIndex((it) => it.name === route.name);
  return i >= 0 ? i : 0;
});

const hasActive = computed(() =>
  navItems.value.some((it) => it.name === route.name),
);

const indicatorStyle = computed(() => ({
  width: `${100 / navItems.value.length}%`,
  transform: `translate3d(${activeIndex.value * 100}%, 0, 0)`,
  opacity: hasActive.value ? 1 : 0,
}));

function isActive(name: string): boolean {
  return route.name === name;
}

function handleTap() {
  try {
    if ("vibrate" in navigator) navigator.vibrate(8);
  } catch {
    /* ignore */
  }
}

const tabBarTv = tv({
  slots: {
    root:
      "fixed left-0 right-0 z-40 bg-surface/95 backdrop-blur-md border-t border-border md:hidden pb-[env(safe-area-inset-bottom,0px)] will-change-[bottom] shadow-[0_-2px_12px_rgba(0,0,0,0.04)] dark:shadow-[0_-2px_18px_rgba(0,0,0,0.35)]",
    inner: "relative flex items-stretch h-16",
    indicator:
      "mtb-indicator pointer-events-none absolute top-2 bottom-2 left-0 rounded-2xl bg-primary/10 dark:bg-primary/20 ring-1 ring-primary/15 dark:ring-primary/25",
    tab:
      "mtb-tab relative z-10 flex-1 inline-flex flex-col items-center justify-center gap-1 select-none transition-[color,transform] duration-200 ease-out active:scale-[0.92]",
    tabIcon: "mtb-icon relative transition-transform duration-300 ease-out",
    tabLabel:
      "relative text-[10px] leading-tight font-medium tracking-tight transition-[color,font-weight,transform] duration-200",
  },
  variants: {
    active: {
      true: {
        tab: "text-primary",
        tabIcon: "-translate-y-0.5 scale-[1.08]",
        tabLabel: "font-semibold",
      },
      false: { tab: "text-text-muted" },
    },
  },
});

const slots = tabBarTv();
const { bottomPx } = useMobileTabBarViewport();
</script>

<template>
  <Teleport to="body">
    <nav :class="slots.root()" :style="{ bottom: `${bottomPx}px` }">
      <div :class="slots.inner()">
        <span :class="slots.indicator()" :style="indicatorStyle" aria-hidden="true" />
        <router-link
          v-for="item in navItems"
          :key="item.name"
          :to="item.path"
          :class="tabBarTv({ active: isActive(item.name) }).tab()"
          @click="handleTap"
        >
          <component
            :is="item.icon"
            :size="iconSize.nav"
            :class="tabBarTv({ active: isActive(item.name) }).tabIcon()"
          />
          <span :class="tabBarTv({ active: isActive(item.name) }).tabLabel()">{{
            item.label
          }}</span>
        </router-link>
      </div>
    </nav>
  </Teleport>
</template>

<style scoped>
:global(.mtb-tab) {
  min-height: 44px;
  -webkit-tap-highlight-color: transparent;
  touch-action: manipulation;
}

/* Snappy spring-like easing for the sliding pill. The duration is short
   enough not to feel sluggish, but the slight overshoot makes it feel
   "alive" — closer to a native iOS / Material You bottom nav. */
:global(.mtb-indicator) {
  transition:
    transform 360ms cubic-bezier(0.34, 1.32, 0.64, 1),
    opacity 240ms ease-out,
    width 240ms ease-out;
  will-change: transform;
}

@media (prefers-reduced-motion: reduce) {
  :global(.mtb-tab),
  :global(.mtb-indicator),
  :global(.mtb-icon) {
    transition: none !important;
  }
}
</style>
