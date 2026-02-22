<script setup lang="ts">
import { X, Wallet } from "lucide-vue-next";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import { useI18n } from "vue-i18n";

defineProps<{
  show: boolean;
  title: string;
  subs: { id: string; name: string; price: number; currencyId: string }[];
  expenses?: { id: string; name: string; amount: number; currencyId: string }[];
}>();

const emit = defineEmits<{
  close: [];
  selectSub: [id: string];
  selectExp: [id: string];
}>();

const { fmt } = useCurrencyFormat();
const { t } = useI18n();
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition ease-out duration-200"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition ease-in duration-150"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div v-if="show" class="fixed inset-0 z-50 flex items-end sm:items-center justify-center sm:p-4">
        <div class="absolute inset-0 bg-black/50" @click="emit('close')" />
        <div class="relative bg-[var(--color-surface)] w-full overflow-hidden rounded-t-2xl sm:rounded-xl shadow-2xl max-h-[80vh] sm:max-w-md">
          <div class="flex items-center justify-between px-4 sm:px-6 py-3 sm:py-4 border-b border-[var(--color-border)]">
            <h3 class="text-base sm:text-lg font-semibold text-[var(--color-text-primary)]">{{ title }}</h3>
            <button @click="emit('close')" class="p-1 rounded-lg hover:bg-[var(--color-surface-hover)] text-[var(--color-text-muted)]">
              <X :size="20" />
            </button>
          </div>
          <div class="px-4 sm:px-6 py-3 sm:py-4 space-y-2.5 sm:space-y-3 max-h-[60vh] overflow-y-auto">
            <!-- Subscriptions -->
            <div
              v-for="sub in subs"
              :key="sub.id"
              @click="emit('selectSub', sub.id)"
              class="flex items-center gap-2.5 sm:gap-3 p-2.5 sm:p-3 rounded-lg bg-[var(--color-surface-secondary)] border border-[var(--color-border)] cursor-pointer hover:border-[var(--color-primary)] hover:bg-[var(--color-primary-light)]/30 transition-colors"
            >
              <div class="w-8 h-8 sm:w-10 sm:h-10 rounded-lg bg-[var(--color-primary-light)] flex items-center justify-center text-xs sm:text-sm font-bold text-[var(--color-primary)] shrink-0">
                {{ sub.name.charAt(0).toUpperCase() }}
              </div>
              <div class="flex-1 min-w-0">
                <p class="text-xs sm:text-sm font-medium text-[var(--color-text-primary)] truncate">{{ sub.name }}</p>
              </div>
              <span class="text-xs sm:text-sm font-semibold text-[var(--color-text-primary)] whitespace-nowrap">
                {{ fmt(sub.price, sub.currencyId) }}
              </span>
            </div>

            <!-- Expenses -->
            <template v-if="expenses && expenses.length > 0">
              <p class="text-[10px] sm:text-xs font-medium text-[var(--color-text-muted)] uppercase tracking-wide pt-2 border-t border-[var(--color-border)]">{{ t('expenses') }}</p>
              <div
                v-for="exp in expenses"
                :key="exp.id"
                @click="emit('selectExp', exp.id)"
                class="flex items-center gap-2.5 sm:gap-3 p-2.5 sm:p-3 rounded-lg bg-[var(--color-surface-secondary)] border border-[var(--color-border)] cursor-pointer hover:border-orange-400 dark:hover:border-orange-500 transition-colors"
              >
                <div class="w-8 h-8 sm:w-10 sm:h-10 rounded-lg bg-orange-100 dark:bg-orange-900/30 flex items-center justify-center shrink-0">
                  <Wallet :size="14" class="text-orange-500 sm:[&]:w-4 sm:[&]:h-4" />
                </div>
                <div class="flex-1 min-w-0">
                  <p class="text-xs sm:text-sm font-medium text-[var(--color-text-primary)] truncate">{{ exp.name }}</p>
                  <p class="text-[10px] sm:text-xs text-orange-500">{{ t('expenses') }}</p>
                </div>
                <span class="text-xs sm:text-sm font-semibold text-[var(--color-text-primary)] whitespace-nowrap">
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
