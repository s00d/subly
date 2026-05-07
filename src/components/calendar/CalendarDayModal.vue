<script setup lang="ts">
import { computed } from "vue";
import { X, Wallet, CreditCard } from "@lucide/vue";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import { useI18n } from "vue-i18n";
import { useScrollLock } from "@/composables/useScrollLock";
import IconDisplay from "@/components/ui/IconDisplay.vue";
import { ui, iconSize } from "@/lib/tv";

const props = defineProps<{
  show: boolean;
  title: string;
  subs: { id: string; name: string; price: number; currencyId: string; logo?: string }[];
  expenses?: { id: string; name: string; amount: number; currencyId: string; icon?: string }[];
}>();

const emit = defineEmits<{
  close: [];
  selectSub: [id: string];
  selectExp: [id: string];
}>();

const { fmt } = useCurrencyFormat();
const { t } = useI18n();
useScrollLock(computed(() => props.show));
</script>

<template>
  <Teleport to="body">
    <Transition name="app-modal">
      <div v-if="show" class="fixed inset-0 z-50 flex items-end sm:items-center justify-center sm:p-4">
        <div class="app-modal-backdrop absolute inset-0 bg-black/50" @click="emit('close')" />
        <div class="app-modal-panel relative bg-surface w-full overflow-hidden rounded-t-2xl sm:rounded-xl shadow-2xl max-h-[80vh] sm:max-w-md">
          <div class="flex items-center justify-between px-4 sm:px-6 py-3 sm:py-4 border-b border-border">
            <h3 :class="ui.sectionTitle()">{{ title }}</h3>
            <button @click="emit('close')" class="p-1 rounded-lg hover:bg-surface-hover text-text-muted">
              <X :size="iconSize.nav" />
            </button>
          </div>
          <div class="px-4 sm:px-5 py-2.5 sm:py-3 space-y-2.5 sm:space-y-3 max-h-[60vh] overflow-y-auto">
            <!-- Subscriptions -->
            <div
              v-for="sub in subs"
              :key="sub.id"
              @click="emit('selectSub', sub.id)"
              class="flex items-center gap-2.5 sm:gap-3 px-3 py-2.5 sm:py-3 rounded-lg bg-surface-secondary border border-border cursor-pointer hover:bg-surface dark:hover:bg-white/6 transition-colors"
            >
                <div class="w-8 h-8 sm:w-9 sm:h-9 rounded-lg bg-primary-light flex items-center justify-center shrink-0">
                <IconDisplay v-if="sub.logo" :icon="sub.logo" :size="16" />
                <CreditCard v-else :size="14" class="text-primary sm:[&]:w-4 sm:[&]:h-4" />
              </div>
              <div class="flex-1 min-w-0">
                <p class="text-xs sm:text-sm font-medium text-text-primary truncate">{{ sub.name }}</p>
              </div>
              <span class="text-xs sm:text-sm font-semibold text-text-primary whitespace-nowrap">
                {{ fmt(sub.price, sub.currencyId) }}
              </span>
            </div>

            <!-- Expenses -->
            <template v-if="expenses && expenses.length > 0">
              <p class="text-[10px] sm:text-xs font-medium text-text-muted uppercase tracking-wide pt-2 border-t border-border">{{ t('expenses') }}</p>
              <div
                v-for="exp in expenses"
                :key="exp.id"
                @click="emit('selectExp', exp.id)"
                class="flex items-center gap-2.5 sm:gap-3 px-3 py-2.5 sm:py-3 rounded-lg bg-surface-secondary border border-border cursor-pointer hover:bg-surface dark:hover:bg-white/6 transition-colors"
              >
                <div class="w-8 h-8 sm:w-9 sm:h-9 rounded-lg bg-surface hover:bg-surface-hover flex items-center justify-center shrink-0">
                  <IconDisplay v-if="exp.icon" :icon="exp.icon" :size="16" />
                  <Wallet v-else :size="14" class="text-text-muted sm:[&]:w-4 sm:[&]:h-4" />
                </div>
                <div class="flex-1 min-w-0">
                  <p class="text-xs sm:text-sm font-medium text-text-primary truncate">{{ exp.name }}</p>
                  <p class="text-[10px] sm:text-xs text-text-muted">{{ t('expenses') }}</p>
                </div>
                <span class="text-xs sm:text-sm font-semibold text-text-primary whitespace-nowrap">
                  {{ fmt(exp.amount, exp.currencyId) }}
                </span>
              </div>
            </template>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
