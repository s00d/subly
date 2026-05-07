<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import type { PaymentRecord, Currency } from "@/schemas/appData";
import { insertPaymentRecord, deletePaymentRecord } from "@/services/subscriptionsClient";
import { History, Plus, Trash2, ChevronDown, ChevronUp, Receipt, RefreshCw } from "@lucide/vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import AppDatePicker from "@/components/ui/AppDatePicker.vue";
import AppInput from "@/components/ui/AppInput.vue";

const props = defineProps<{
  subscriptionId: string;
  currencyId: string;
  price: number;
  history: PaymentRecord[];
  lookupData: {
    currencies: Currency[];
    mainCurrencyId: string;
    targetCurrencyIds: string[];
  };
}>();

const emit = defineEmits<{
  recordPayment: [id: string];
}>();

const { t } = useI18n();
const { fmtDateMedium, fmtCurrency } = useLocaleFormat();
const currencies = ref<Currency[]>([]);
const mainCurrencyId = ref("cur-2");
const targetIds = ref<string[]>([]);

const expanded = ref(false);
const showAddForm = ref(false);
const localHistory = ref<PaymentRecord[]>([]);

// Add form
const addDate = ref(new Date().toISOString().split("T")[0]);
const addAmount = ref(props.price);
const addNote = ref("");
watch(() => props.history, (value) => { localHistory.value = [...value]; }, { immediate: true });

function fmt(amount: number, curId: string): string {
  const c = currencies.value.find((cur) => cur.id === curId);
  return fmtCurrency(amount, c?.code || "USD");
}

function fmtCur(amount: number, currency: Currency): string {
  return fmtCurrency(amount, currency.code);
}

const targetCurrencies = computed(() => {
  const ids = new Set(targetIds.value);
  const mainId = mainCurrencyId.value;
  if (mainId) ids.add(mainId);
  return [...ids]
    .map((id) => currencies.value.find((c) => c.id === id))
    .filter((c): c is Currency => !!c && c.rate > 0);
});

function getConvertTargets(curId: string) {
  return targetCurrencies.value.filter((tc) => tc.id !== curId);
}

function convertAmount(amount: number, fromCurId: string, toCurrency: Currency): number {
  const fromCur = currencies.value.find((c) => c.id === fromCurId);
  if (!fromCur || fromCur.rate <= 0) return 0;
  const inBase = amount / fromCur.rate;
  return inBase * toCurrency.rate;
}

const visibleRecords = computed(() =>
  expanded.value ? localHistory.value : localHistory.value.slice(0, 5),
);

const totalPaid = computed(() =>
  localHistory.value.reduce((sum, r) => sum + r.amount, 0),
);

async function addRecord() {
  if (!addDate.value) return;
  const record: PaymentRecord = {
    id: crypto.randomUUID(),
    date: addDate.value,
    amount: addAmount.value || props.price,
    currencyId: props.currencyId,
    note: addNote.value,
  };
  await insertPaymentRecord(props.subscriptionId, record);
  localHistory.value = [record, ...localHistory.value];
  showAddForm.value = false;
  addDate.value = new Date().toISOString().split("T")[0];
  addAmount.value = props.price;
  addNote.value = "";
}

async function deleteRecord(recordId: string) {
  await deletePaymentRecord(recordId);
  localHistory.value = localHistory.value.filter((item) => item.id !== recordId);
}

const formatDate = fmtDateMedium;
watch(
  () => props.lookupData,
  (lookup) => {
    currencies.value = lookup.currencies;
    mainCurrencyId.value = lookup.mainCurrencyId;
    targetIds.value = lookup.targetCurrencyIds ?? [];
  },
  { immediate: true, deep: true },
);
</script>

<template>
  <div class="rounded-lg border border-border bg-surface-secondary overflow-hidden divide-y divide-border">
    <!-- Header -->
    <div class="flex items-center justify-between px-3 py-2.5">
      <div class="flex items-center gap-1.5">
        <History :size="13" class="text-text-muted" />
        <span class="text-[10px] uppercase tracking-wide font-medium text-text-muted">{{ t('payment_history') }}</span>
        <span v-if="localHistory.length > 0" class="text-[10px] text-text-muted ml-1">({{ localHistory.length }})</span>
      </div>
      <div class="flex items-center gap-1">
        <div v-if="localHistory.length > 0" class="text-right mr-1">
          <span class="text-[10px] font-medium text-primary">
            {{ t('total') }}: {{ fmt(totalPaid, currencyId) }}
          </span>
          <div v-if="getConvertTargets(currencyId).length > 0" class="flex flex-wrap justify-end gap-x-1.5">
            <span
              v-for="tc in getConvertTargets(currencyId)"
              :key="tc.id"
              class="text-[9px] text-text-muted"
            >≈ {{ fmtCur(convertAmount(totalPaid, currencyId, tc), tc) }}</span>
          </div>
        </div>
        <Tooltip :text="t('record_payment')" position="bottom">
          <button
            @click="emit('recordPayment', subscriptionId)"
            class="p-1 rounded hover:bg-green-100 dark:hover:bg-green-900/30 text-text-muted hover:text-green-600 transition-colors"
          >
            <RefreshCw :size="13" />
          </button>
        </Tooltip>
        <Tooltip :text="t('add_payment')" position="bottom">
          <button
            @click="showAddForm = !showAddForm"
            class="p-1 rounded hover:bg-surface-hover text-text-muted hover:text-primary transition-colors"
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
      <div v-if="showAddForm" class="px-3 pb-3 space-y-2">
        <div class="flex gap-2">
          <div class="flex-1">
            <AppDatePicker v-model="addDate" />
          </div>
          <AppInput
            :modelValue="addAmount"
            @update:modelValue="(v) => addAmount = Number(v)"
            type="number"
            step="0.01"
            min="0"
            :placeholder="String(price)"
            size="sm"
            class="w-24"
          />
        </div>
        <div class="flex gap-2">
          <AppInput
            :modelValue="addNote"
            @update:modelValue="(v) => addNote = String(v)"
            :placeholder="t('note_optional')"
            size="sm"
            class="flex-1"
          />
          <button
            @click="addRecord"
            class="px-3 py-1 rounded-lg bg-primary text-white text-xs font-medium hover:bg-primary-hover transition-colors"
          >{{ t('add') }}</button>
        </div>
      </div>
    </Transition>

    <!-- Records list -->
    <div v-if="localHistory.length > 0">
      <div
        v-for="record in visibleRecords"
        :key="record.id"
        class="flex items-center gap-2.5 px-3 py-2.5 group hover:bg-surface dark:hover:bg-white/6 transition-colors"
      >
        <Receipt :size="12" class="text-text-muted shrink-0" />
        <span class="text-xs text-text-muted whitespace-nowrap">{{ formatDate(record.date) }}</span>
        <div class="min-w-0">
          <span class="text-xs font-semibold text-text-primary">{{ fmt(record.amount, record.currencyId) }}</span>
          <div v-if="getConvertTargets(record.currencyId).length > 0" class="flex flex-wrap gap-x-1.5">
            <span
              v-for="tc in getConvertTargets(record.currencyId)"
              :key="tc.id"
              class="text-[9px] text-text-muted"
            >≈ {{ fmtCur(convertAmount(record.amount, record.currencyId, tc), tc) }}</span>
          </div>
        </div>
        <span v-if="record.note" class="text-[10px] text-text-muted truncate flex-1 min-w-0">{{ record.note }}</span>
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
      <p class="text-[11px] text-text-muted">{{ t('no_payment_records') }}</p>
    </div>

    <!-- Show more -->
    <button
      v-if="localHistory.length > 5"
      @click="expanded = !expanded"
      class="w-full flex items-center justify-center gap-1 py-1.5 text-[10px] text-primary hover:bg-surface-hover transition-colors"
    >
      <component :is="expanded ? ChevronUp : ChevronDown" :size="12" />
      {{ expanded ? t('show_less') : t('show_all') }} ({{ localHistory.length }})
    </button>
  </div>
</template>
