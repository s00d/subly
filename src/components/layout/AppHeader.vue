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
    root: "h-14 sm:h-16 bg-[var(--color-surface)] border-b border-[var(--color-border)] flex items-center px-3 sm:px-6 shrink-0 gap-2",
    burgerBtn: "md:hidden p-1.5 -ml-1 rounded-lg text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)] transition-colors shrink-0",
    title: "text-base sm:text-xl font-semibold text-[var(--color-text-primary)] truncate",
    actionsWrap: "flex items-center gap-2 ml-auto",
    actionBtn: [
      "w-8 h-8 rounded-lg bg-[var(--color-primary)] text-white",
      "flex items-center justify-center hover:bg-[var(--color-primary-hover)]",
      "shadow-sm transition-colors shrink-0",
    ],
  },
});

const slots = headerTv();
</script>

<template>
  <header :class="slots.root()">
    <button :class="slots.burgerBtn()" @click="emit('toggleSidebar')">
      <Menu :size="22" />
    </button>
    <h1 :class="slots.title()">{{ pageTitle }}</h1>
    <div :class="slots.actionsWrap()">
      <Tooltip v-for="action in actions" :key="action.id" :text="action.title">
        <button
          @click="action.onClick"
          :class="slots.actionBtn()"
        >
          <component :is="action.icon" :size="18" />
        </button>
      </Tooltip>
    </div>
  </header>
</template>
