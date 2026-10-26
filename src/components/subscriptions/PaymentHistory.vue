<script setup lang="ts">
import { localYmd } from "@/composables/useLocaleFormat";
import { ref, computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import type { PaymentRecord, Currency, MoneyString } from "@/schemas/appData";
import {
  convertAmountToCurrency,
  moneyFromNumber,
  moneyGt,
  parseMoney,
  sumMoney,
  toDisplayNumber,
} from "@/lib/money";
import { insertPaymentRecord, deletePaymentRecord } from "@/services/subscriptionsClient";
import { History, Plus, Trash2, ChevronDown, ChevronUp, Receipt } from "@lucide/vue";
import { button } from "@/lib/tv";
import Tooltip from "@/components/ui/Tooltip.vue";
import AppDatePicker from "@/components/ui/AppDatePicker.vue";
import AppInput from "@/components/ui/AppInput.vue";
import { formatErrorForToast } from "@/utils/formatError";

const props = defineProps<{
  subscriptionId: string;
  currencyId: string;
  price: MoneyString;
  history: PaymentRecord[];
  lookupData: {
    currencies: Currency[];
    mainCurrencyId: string;
    targetCurrencyIds: string[];
  };
}>();

// `recordPayment` stays in the contract (SubscriptionDetail listens), but the in-list "renew"
// button was removed: the detail footer already has "Record payment".
defineEmits<{
  recordPayment: [id: string];
}>();

const { t } = useI18n();
const { fmtDateMedium } = useLocaleFormat();
const { fmt, ratesMap } = useCurrencyFormat();
const currencies = ref<Currency[]>([]);
const mainCurrencyId = ref("cur-2");
const targetIds = ref<string[]>([]);

const expanded = ref(false);
const showAddForm = ref(false);
const localHistory = ref<PaymentRecord[]>([]);

// Add form
const addDate = ref(localYmd());
const addAmount = ref(props.price);
const addNote = ref("");
watch(() => props.history, (value) => { localHistory.value = [...value]; }, { immediate: true });

const targetCurrencies = computed(() => {
  const ids = new Set(targetIds.value);
  const mainId = mainCurrencyId.value;
  if (mainId) ids.add(mainId);
  return [...ids]
    .map((id) => currencies.value.find((c) => c.id === id))
    .filter((c): c is Currency => !!c && moneyGt(c.rate, "0"));
});

function getConvertTargets(curId: string) {
  return targetCurrencies.value.filter((tc) => tc.id !== curId);
}

function convertAmount(amount: MoneyString, fromCurId: string, toCurrency: Currency): number {
  return toDisplayNumber(
    convertAmountToCurrency(amount, fromCurId, toCurrency, ratesMap.value),
  );
}

const visibleRecords = computed(() =>
  expanded.value ? localHistory.value : localHistory.value.slice(0, 5),
);

/** Records may be in older currencies (the subscription's currency can change): convert before summing. */
const totalPaid = computed(() => {
  const target = currencies.value.find((c) => c.id === props.currencyId);
  if (!target) return sumMoney(localHistory.value.map((r) => r.amount));
  return localHistory.value.reduce(
    (acc, r) => acc.plus(convertAmountToCurrency(r.amount, r.currencyId || props.currencyId, target, ratesMap.value)),
    sumMoney([]),
  );
});

const actionError = ref("");

async function addRecord() {
  if (!addDate.value) return;
  actionError.value = "";
  const record: PaymentRecord = {
    id: crypto.randomUUID(),
    date: addDate.value,
    amount: addAmount.value || props.price,
    currencyId: props.currencyId,
    note: addNote.value,
  };
  try {
    await insertPaymentRecord(props.subscriptionId, record);
  } catch (e) {
    actionError.value = formatErrorForToast(e, t);
    return;
  }
  localHistory.value = [record, ...localHistory.value];
  showAddForm.value = false;
  addDate.value = localYmd();
  addAmount.value = props.price;
  addNote.value = "";
}

async function deleteRecord(recordId: string) {
  actionError.value = "";
  try {
    await deletePaymentRecord(recordId);
  } catch (e) {
    actionError.value = formatErrorForToast(e, t);
    return;
  }
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
    <p v-if="actionError" role="alert" class="px-3 py-2 text-xs text-danger">{{ actionError }}</p>
    <!-- Header -->
    <div class="flex items-center justify-between px-3 py-2.5">
      <div class="flex items-center gap-1.5">
        <History :size="14" class="text-text-muted" />
        <span class="text-[10px] uppercase tracking-wide font-medium text-text-muted">{{ t('payment_history') }}</span>
        <span v-if="localHistory.length > 0" class="text-[10px] text-text-muted ml-1">({{ localHistory.length }})</span>
      </div>
      <div class="flex items-center gap-1">
        <div v-if="localHistory.length > 0" class="text-right mr-1">
          <span class="text-[10px] font-medium text-primary">
            {{ t('total') }}: {{ fmt(totalPaid.toFixed(), currencyId) }}
          </span>
          <div v-if="getConvertTargets(currencyId).length > 0" class="flex flex-wrap justify-end gap-x-1.5">
            <span
              v-for="tc in getConvertTargets(currencyId)"
              :key="tc.id"
              class="text-[9px] text-text-muted"
            >≈ {{ fmt(convertAmount(totalPaid.toFixed(), currencyId, tc).toFixed(), tc.id) }}</span>
          </div>
        </div>
        <Tooltip :text="t('add_payment')" position="bottom">
          <button
            type="button"
            :aria-label="t('add_payment')"
            :aria-expanded="showAddForm"
            @click="showAddForm = !showAddForm"
            :class="button({ intent: 'ghost', size: 'icon', class: 'hover:text-primary' })"
          >
            <Plus :size="15" />
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
            @update:modelValue="(v) => addAmount = moneyFromNumber(Number(v))"
            type="number"
            step="0.01"
            min="0"
            :placeholder="String(price)"
            size="sm"
            class="w-24"
          />
        </div>
        <div class="flex gap-2 items-center">
          <AppInput
            :modelValue="addNote"
            @update:modelValue="(v) => addNote = String(v)"
            :placeholder="t('note_optional')"
            size="sm"
            class="flex-1"
          />
          <button
            type="button"
            @click="addRecord"
            :class="button({ intent: 'primary', size: 'md' })"
          >{{ t('add') }}</button>
        </div>
      </div>
    </Transition>

    <!-- Records list -->
    <div v-if="localHistory.length > 0">
      <div
        v-for="record in visibleRecords"
        :key="record.id"
        class="flex items-center gap-2.5 px-3 py-2.5 group hover:bg-surface-hover transition-colors"
      >
        <Receipt :size="14" class="text-text-muted shrink-0" />
        <span class="text-xs text-text-muted whitespace-nowrap">{{ formatDate(record.date) }}</span>
        <div class="min-w-0">
          <span class="text-xs font-semibold text-text-primary">{{ fmt(record.amount, record.currencyId) }}</span>
          <div v-if="getConvertTargets(record.currencyId).length > 0" class="flex flex-wrap gap-x-1.5">
            <span
              v-for="tc in getConvertTargets(record.currencyId)"
              :key="tc.id"
              class="text-[9px] text-text-muted"
            >≈ {{ fmt(moneyFromNumber(convertAmount(record.amount, record.currencyId, tc)), tc.id) }}</span>
          </div>
        </div>
        <span v-if="record.note" class="text-[10px] text-text-muted truncate flex-1 min-w-0">{{ record.note }}</span>
        <div v-else class="flex-1" />
        <Tooltip :text="t('delete')" position="left">
          <button
            type="button"
            :aria-label="t('delete')"
            @click="deleteRecord(record.id)"
            :class="button({ intent: 'ghost', size: 'icon', class: 'shrink-0 text-text-muted hover:text-danger hover:bg-danger/12' })"
          >
            <Trash2 :size="14" />
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
      type="button"
      :class="button({ intent: 'ghost', size: 'sm', class: 'w-full rounded-none text-primary hover:text-primary' })"
    >
      <component :is="expanded ? ChevronUp : ChevronDown" :size="14" />
      {{ expanded ? t('show_less') : t('show_all') }} ({{ localHistory.length }})
    </button>
  </div>
</template>
