<script setup lang="ts">
import { Wallet, CreditCard } from "@lucide/vue";
import Modal from "@/components/ui/Modal.vue";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import { useI18n } from "vue-i18n";
import IconDisplay from "@/components/ui/IconDisplay.vue";
import type { MoneyString } from "@/lib/money";

defineProps<{
  show: boolean;
  title: string;
  subs: { id: string; name: string; price: MoneyString; currencyId: string; logo?: string }[];
  expenses?: { id: string; name: string; amount: MoneyString; currencyId: string; icon?: string }[];
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
  <Modal :show="show" :title="title" max-width="28rem" @close="emit('close')">
    <div class="-mx-2 space-y-0.5">
    <!-- Subscriptions -->
    <div
      v-for="sub in subs"
      :key="sub.id"
      @click="emit('selectSub', sub.id)"
      class="flex items-center gap-3 px-2 py-2.5 rounded-lg cursor-pointer hover:bg-surface-hover transition-colors"
    >
        <div class="w-8 h-8 rounded-[9px] bg-primary/12 flex items-center justify-center shrink-0">
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
      <p class="px-2 pt-3 pb-1 text-[11px] font-semibold text-text-muted uppercase tracking-wide border-t border-border">{{ t('expenses') }}</p>
      <div
        v-for="exp in expenses"
        :key="exp.id"
        @click="emit('selectExp', exp.id)"
        class="flex items-center gap-3 px-2 py-2.5 rounded-lg cursor-pointer hover:bg-surface-hover transition-colors"
      >
        <div class="w-8 h-8 rounded-[9px] bg-surface-hover flex items-center justify-center shrink-0">
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
  </Modal>
</template>
