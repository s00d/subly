<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";
import { useI18n } from "vue-i18n";
import { useHeaderActions } from "@/composables/useHeaderActions";
import { tv } from "@/lib/tv";
import Tooltip from "@/components/ui/Tooltip.vue";
import { Menu } from "lucide-vue-next";

const emit = defineEmits<{
  toggleSidebar: [];
}>();

const route = useRoute();
const { t } = useI18n();
const { actions } = useHeaderActions();

const pageTitle = computed(() => {
  const names: Record<string, string> = {
    dashboard: t("dashboard"),
    subscriptions: t("subscriptions"),
    expenses: t("expenses"),
    calendar: t("calendar"),
    currencies: t("exchange_rates"),
    settings: t("settings"),
  };
  return names[route.name as string] || "Subly";
});

const headerTv = tv({
  slots: {
    root: "h-14 sm:h-16 bg-surface border-b border-border flex items-center px-3 sm:px-6 shrink-0 gap-2",
    burgerBtn: "md:hidden p-1.5 -ml-1 rounded-lg text-text-secondary hover:bg-surface-hover transition-colors shrink-0",
    title: "text-base sm:text-xl font-semibold text-text-primary truncate",
    actionsWrap: "flex items-center gap-2 ml-auto",
    actionBtn: "w-8 h-8 rounded-lg flex items-center justify-center shadow-sm transition-colors shrink-0",
  },
});

const slots = headerTv();

function getActionBtnClass(style?: "primary" | "neutral" | "accent" | "success" | "warning" | "danger"): string {
  switch (style) {
    case "primary":
      return "bg-primary text-white hover:bg-primary-hover";
    case "accent":
      return "bg-indigo-500 text-white hover:bg-indigo-600 dark:bg-indigo-600 dark:hover:bg-indigo-500";
    case "success":
      return "bg-green-500 text-white hover:bg-green-600 dark:bg-green-600 dark:hover:bg-green-500";
    case "warning":
      return "bg-orange-500 text-white hover:bg-orange-600 dark:bg-orange-600 dark:hover:bg-orange-500";
    case "danger":
      return "bg-red-500 text-white hover:bg-red-600 dark:bg-red-600 dark:hover:bg-red-500";
    case "neutral":
    default:
      return "bg-surface-hover text-text-secondary hover:text-text-primary hover:bg-surface-secondary border border-border";
  }
}
</script>

<template>
  <header :class="slots.root()">
    <button :class="slots.burgerBtn()" @click="emit('toggleSidebar')">
      <Menu :size="22" />
    </button>
    <h1 :class="slots.title()">{{ pageTitle }}</h1>
    <div :class="slots.actionsWrap()">
      <Tooltip v-for="action in actions" :key="action.id" :text="action.title" position="bottom">
        <button
          @click="action.onClick"
          :class="[slots.actionBtn(), getActionBtnClass(action.style)]"
        >
          <component :is="action.icon" :size="18" />
        </button>
      </Tooltip>
    </div>
  </header>
</template>
