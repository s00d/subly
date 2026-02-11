<script setup lang="ts">
import { ref, computed } from "vue";
import { useAppStore } from "@/stores/appStore";
import { useI18n } from "@/i18n";
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import { formatCurrency } from "@/services/calculations";
import type { PaymentRecord } from "@/schemas/appData";
import { History, Plus, Trash2, ChevronDown, ChevronUp, Receipt } from "lucide-vue-next";

const props = defineProps<{
  subscriptionId: string;
  currencyId: string;
  price: number;
  history: PaymentRecord[];
}>();

const store = useAppStore();
const { t } = useI18n();
const { fmtDateMedium } = useLocaleFormat();

const expanded = ref(false);
const showAddForm = ref(false);

// Add form
const addDate = ref(new Date().toISOString().split("T")[0]);
const addAmount = ref(props.price);
const addNote = ref("");

function fmt(amount: number, curId: string): string {
  const c = store.state.currencies.find((cur) => cur.id === curId);
  return formatCurrency(amount, c?.code || "USD", c?.symbol);
}

const visibleRecords = computed(() =>
  expanded.value ? props.history : props.history.slice(0, 5),
);

const totalPaid = computed(() =>
  props.history.reduce((sum, r) => {
    // Convert all to the subscription's currency for simplicity
    return sum + r.amount;
  }, 0),
);

function addRecord() {
  if (!addDate.value) return;
  store.addPaymentRecord(props.subscriptionId, {
    id: crypto.randomUUID(),
    date: addDate.value,
    amount: addAmount.value || props.price,
    currencyId: props.currencyId,
    note: addNote.value,
  });
  showAddForm.value = false;
  addDate.value = new Date().toISOString().split("T")[0];
  addAmount.value = props.price;
  addNote.value = "";
}

function deleteRecord(recordId: string) {
  store.deletePaymentRecord(props.subscriptionId, recordId);
}

const formatDate = fmtDateMedium;
</script>

<template>
  <div class="bg-[var(--color-surface-secondary)] rounded-lg overflow-hidden">
    <!-- Header -->
    <div class="flex items-center justify-between px-3 py-2.5">
      <div class="flex items-center gap-1.5">
        <History :size="13" class="text-[var(--color-text-muted)]" />
        <span class="text-[10px] uppercase tracking-wide font-medium text-[var(--color-text-muted)]">{{ t('payment_history') }}</span>
        <span v-if="history.length > 0" class="text-[10px] text-[var(--color-text-muted)] ml-1">({{ history.length }})</span>
      </div>
      <div class="flex items-center gap-1">
        <span v-if="history.length > 0" class="text-[10px] font-medium text-[var(--color-primary)]">
          {{ t('total') }}: {{ fmt(totalPaid, currencyId) }}
        </span>
        <button
          @click="showAddForm = !showAddForm"
          class="p-1 rounded hover:bg-[var(--color-surface-hover)] text-[var(--color-text-muted)] hover:text-[var(--color-primary)] transition-colors"
          :title="t('add_payment')"
        >
          <Plus :size="13" />
        </button>
      </div>
    </div>

    <!-- Add form -->
    <Transition
      enter-active-class="transition ease-out duration-150"
      enter-from-class="opacity-0 -translate-y-1"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition ease-in duration-100"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div v-if="showAddForm" class="px-3 pb-3 space-y-2 border-b border-[var(--color-border)]">
        <div class="flex gap-2">
          <input
            v-model="addDate"
            type="date"
            class="flex-1 px-2 py-1 rounded border border-[var(--color-border)] bg-[var(--color-surface)] text-xs text-[var(--color-text-primary)] focus:outline-none focus:ring-1 focus:ring-[var(--color-primary)]"
          />
          <input
            v-model.number="addAmount"
            type="number"
            step="0.01"
            min="0"
            :placeholder="String(price)"
            class="w-24 px-2 py-1 rounded border border-[var(--color-border)] bg-[var(--color-surface)] text-xs text-[var(--color-text-primary)] focus:outline-none focus:ring-1 focus:ring-[var(--color-primary)]"
          />
        </div>
        <div class="flex gap-2">
          <input
            v-model="addNote"
            type="text"
            :placeholder="t('note_optional')"
            class="flex-1 px-2 py-1 rounded border border-[var(--color-border)] bg-[var(--color-surface)] text-xs text-[var(--color-text-primary)] placeholder-[var(--color-text-muted)] focus:outline-none focus:ring-1 focus:ring-[var(--color-primary)]"
          />
          <button
            @click="addRecord"
            class="px-3 py-1 rounded bg-[var(--color-primary)] text-white text-xs font-medium hover:bg-[var(--color-primary-hover)] transition-colors"
          >{{ t('add') }}</button>
        </div>
      </div>
    </Transition>

    <!-- Records list -->
    <div v-if="history.length > 0" class="divide-y divide-[var(--color-border)]">
      <div
        v-for="record in visibleRecords"
        :key="record.id"
        class="flex items-center gap-2.5 px-3 py-2 group hover:bg-[var(--color-surface-hover)] transition-colors"
      >
        <Receipt :size="12" class="text-[var(--color-text-muted)] shrink-0" />
        <span class="text-xs text-[var(--color-text-muted)] whitespace-nowrap">{{ formatDate(record.date) }}</span>
        <span class="text-xs font-semibold text-[var(--color-text-primary)]">{{ fmt(record.amount, record.currencyId) }}</span>
        <span v-if="record.note" class="text-[10px] text-[var(--color-text-muted)] truncate flex-1 min-w-0">{{ record.note }}</span>
        <div v-else class="flex-1" />
        <button
          @click="deleteRecord(record.id)"
          class="p-0.5 rounded opacity-0 group-hover:opacity-100 hover:bg-red-100 dark:hover:bg-red-900/30 text-red-400 hover:text-red-600 transition-all shrink-0"
          :title="t('delete')"
        >
          <Trash2 :size="11" />
        </button>
      </div>
    </div>

    <!-- Empty state -->
    <div v-else class="px-3 py-4 text-center">
      <p class="text-[11px] text-[var(--color-text-muted)]">{{ t('no_payment_records') }}</p>
    </div>

    <!-- Show more -->
    <button
      v-if="history.length > 5"
      @click="expanded = !expanded"
      class="w-full flex items-center justify-center gap-1 py-1.5 text-[10px] text-[var(--color-primary)] hover:bg-[var(--color-surface-hover)] transition-colors"
    >
      <component :is="expanded ? ChevronUp : ChevronDown" :size="12" />
      {{ expanded ? t('show_less') : t('show_all') }} ({{ history.length }})
    </button>
  </div>
</template>
