<script setup lang="ts">
import { ref, computed } from "vue";
import { useSubscriptionsStore } from "@/stores/subscriptions";
import { useSettingsStore } from "@/stores/settings";
import { useCatalogStore } from "@/stores/catalog";
import { useI18n } from "vue-i18n";
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import { formatCurrency } from "@/services/calculations";
import type { PaymentRecord, Currency } from "@/schemas/appData";
import { History, Plus, Trash2, ChevronDown, ChevronUp, Receipt, RefreshCw } from "lucide-vue-next";
import Tooltip from "@/components/ui/Tooltip.vue";

const props = defineProps<{
  subscriptionId: string;
  currencyId: string;
  price: number;
  history: PaymentRecord[];
}>();

const emit = defineEmits<{
  recordPayment: [id: string];
}>();

const subsStore = useSubscriptionsStore();
const settingsStore = useSettingsStore();
const catalogStore = useCatalogStore();
const { t } = useI18n();
const { fmtDateMedium } = useLocaleFormat();

const expanded = ref(false);
const showAddForm = ref(false);

// Add form
const addDate = ref(new Date().toISOString().split("T")[0]);
const addAmount = ref(props.price);
const addNote = ref("");

function fmt(amount: number, curId: string): string {
  const c = catalogStore.currencies.find((cur) => cur.id === curId);
  return formatCurrency(amount, c?.code || "USD", c?.symbol);
}

function fmtCur(amount: number, currency: Currency): string {
  return formatCurrency(amount, currency.code, currency.symbol);
}

const targetCurrencies = computed(() => {
  const mainId = settingsStore.settings.mainCurrencyId;
  const targets = settingsStore.settings.currencyUpdateTargets ?? [];
  const ids = new Set(targets);
  if (mainId) ids.add(mainId);
  return [...ids]
    .map((id) => catalogStore.currencies.find((c) => c.id === id))
    .filter((c): c is Currency => !!c && c.rate > 0);
});

function getConvertTargets(curId: string) {
  return targetCurrencies.value.filter((tc) => tc.id !== curId);
}

function convertAmount(amount: number, fromCurId: string, toCurrency: Currency): number {
  const fromCur = catalogStore.currencies.find((c) => c.id === fromCurId);
  if (!fromCur || fromCur.rate <= 0) return 0;
  const inBase = amount / fromCur.rate;
  return inBase * toCurrency.rate;
}

const visibleRecords = computed(() =>
  expanded.value ? props.history : props.history.slice(0, 5),
);

const totalPaid = computed(() =>
  props.history.reduce((sum, r) => sum + r.amount, 0),
);

function addRecord() {
  if (!addDate.value) return;
  subsStore.addPaymentRecord(props.subscriptionId, {
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
  subsStore.deletePaymentRecord(props.subscriptionId, recordId);
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
        <div v-if="history.length > 0" class="text-right mr-1">
          <span class="text-[10px] font-medium text-[var(--color-primary)]">
            {{ t('total') }}: {{ fmt(totalPaid, currencyId) }}
          </span>
          <div v-if="getConvertTargets(currencyId).length > 0" class="flex flex-wrap justify-end gap-x-1.5">
            <span
              v-for="tc in getConvertTargets(currencyId)"
              :key="tc.id"
              class="text-[9px] text-[var(--color-text-muted)]"
            >≈ {{ fmtCur(convertAmount(totalPaid, currencyId, tc), tc) }}</span>
          </div>
        </div>
        <Tooltip :text="t('record_payment')" position="bottom">
          <button
            @click="emit('recordPayment', subscriptionId)"
            class="p-1 rounded hover:bg-green-100 dark:hover:bg-green-900/30 text-[var(--color-text-muted)] hover:text-green-600 transition-colors"
          >
            <RefreshCw :size="13" />
          </button>
        </Tooltip>
        <Tooltip :text="t('add_payment')" position="bottom">
          <button
            @click="showAddForm = !showAddForm"
            class="p-1 rounded hover:bg-[var(--color-surface-hover)] text-[var(--color-text-muted)] hover:text-[var(--color-primary)] transition-colors"
          >
            <Plus :size="13" />
          </button>
        </Tooltip>
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
        <div class="min-w-0">
          <span class="text-xs font-semibold text-[var(--color-text-primary)]">{{ fmt(record.amount, record.currencyId) }}</span>
          <div v-if="getConvertTargets(record.currencyId).length > 0" class="flex flex-wrap gap-x-1.5">
            <span
              v-for="tc in getConvertTargets(record.currencyId)"
              :key="tc.id"
              class="text-[9px] text-[var(--color-text-muted)]"
            >≈ {{ fmtCur(convertAmount(record.amount, record.currencyId, tc), tc) }}</span>
          </div>
        </div>
        <span v-if="record.note" class="text-[10px] text-[var(--color-text-muted)] truncate flex-1 min-w-0">{{ record.note }}</span>
        <div v-else class="flex-1" />
        <Tooltip :text="t('delete')" position="left">
          <button
            @click="deleteRecord(record.id)"
            class="p-0.5 rounded hover:bg-red-100 dark:hover:bg-red-900/30 text-red-400 hover:text-red-600 transition-all shrink-0"
          >
            <Trash2 :size="11" />
          </button>
        </Tooltip>
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
