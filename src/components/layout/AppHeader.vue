<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";
import { useI18n } from "vue-i18n";
import { useHeaderActions, type HeaderAction } from "@/composables/useHeaderActions";
import { useRouter } from "vue-router";
import { tv, typo, iconSize, button } from "@/lib/tv";
import { useAlerts } from "@/composables/useAlerts";
import Tooltip from "@/components/ui/Tooltip.vue";
import { Bell, Menu } from "@lucide/vue";

const emit = defineEmits<{
  toggleSidebar: [];
}>();

const route = useRoute();
const router = useRouter();
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

const { alerts } = useAlerts();
const alertTone = computed(() =>
  alerts.value.some((a) => a.type === "overdue" || a.type === "due_today") ? "bg-danger" : "bg-primary",
);

const headerTv = tv({
  slots: {
    root: "bg-surface-secondary shrink-0 max-md:border-b max-md:border-border",
    inner:
      "mx-auto w-full max-w-[1120px] min-h-14 md:min-h-16 flex items-center gap-2 px-3 sm:px-5 md:px-8 max-md:py-1.5",
    burgerBtn: `md:hidden -ml-1 ${button({ intent: "ghost", size: "icon" })} h-10 w-10`,
    title: `${typo.pageTitle()} flex-1 min-w-0`,
    actionsWrap: "flex items-center gap-1 sm:gap-1.5 ml-auto shrink-0",
    bellBadge:
      "absolute -top-1 -right-1 min-w-4 h-4 px-1 rounded-full text-[10px] font-semibold leading-4 text-white text-center ring-2 ring-surface-secondary",
  },
});

const slots = headerTv();

/** One primary (filled, labelled on sm+) action per page; everything else is a quiet secondary button. */
function hasLabel(action: HeaderAction): boolean {
  return action.style === "primary" || action.style === "accent";
}

function getActionBtnClass(action: Pick<HeaderAction, "style">): string {
  const labelled = hasLabel(action as HeaderAction);
  const size = labelled ? "max-sm:w-9 max-sm:px-0" : "";
  if (action.style === "primary") return button({ intent: "primary", class: size });
  if (action.style === "danger") return button({ intent: "danger", size: "icon" });
  if (action.style === "warning" || action.style === "success") return button({ size: "icon", pressed: true });
  // accent = AI / smart actions: same secondary button, violet icon carries the identity
  if (action.style === "accent") return button({ class: `${size} [&>svg]:text-violet-500 dark:[&>svg]:text-violet-400` });
  return button({ size: "icon", class: "text-text-secondary hover:text-text-primary" });
}
</script>

<template>
  <header :class="[slots.root(), 'app-header-safe']">
    <div :class="slots.inner()">
      <button data-compact :class="slots.burgerBtn()" aria-label="Menu" @click="emit('toggleSidebar')">
        <Menu :size="iconSize.nav" />
      </button>
      <h1 :class="slots.title()">{{ pageTitle }}</h1>
      <div :class="slots.actionsWrap()">
        <Tooltip v-for="action in actions" :key="action.id" :text="action.title" position="bottom">
          <button
            data-compact
            :aria-label="action.title"
            @click="action.onClick"
            :class="getActionBtnClass(action)"
          >
            <component :is="action.icon" :size="iconSize.sm" />
            <span v-if="hasLabel(action)" class="max-sm:hidden">{{ action.label ?? action.title }}</span>
          </button>
        </Tooltip>
        <!-- Alerts are listed on the Dashboard itself; phones keep header room for the title -->
        <span v-if="alerts.length > 0 && route.name !== 'dashboard'" class="max-sm:hidden">
        <Tooltip :text="t('notifications')" position="bottom">
          <button
            data-compact
            class="relative"
            :aria-label="t('notifications')"
            :class="getActionBtnClass({ style: 'neutral' })"
            @click="router.push('/')"
          >
            <Bell :size="iconSize.sm" />
            <span :class="[slots.bellBadge(), alertTone]">{{ alerts.length }}</span>
          </button>
        </Tooltip>
        </span>
      </div>
    </div>
  </header>
</template>

<style scoped>
.app-header-safe {
  padding-top: var(--sat, 0px);
}
</style>
