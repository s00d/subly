<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import type { Expense, Currency, Category, PaymentMethod, HouseholdMember, Settings } from "@/schemas/appData";
import { expenseToIsoDate } from "@/schemas/appData";
import Modal from "@/components/ui/Modal.vue";
import IconDisplay from "@/components/ui/IconDisplay.vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import { Pencil, Trash2, Calendar, CreditCard, Tag, User, FileText, Hash, Wallet, Link2, ExternalLink, Link } from "@lucide/vue";
import { openUrl } from "@tauri-apps/plugin-opener";

const props = defineProps<{
  show: boolean;
  expense: Expense | null;
  lookupData: {
    categories: Category[];
    paymentMethods: PaymentMethod[];
    household: HouseholdMember[];
    currencies: Currency[];
    settings: Settings;
  };
}>();

const emit = defineEmits<{
  close: [];
  edit: [exp: Expense];
  delete: [id: string];
  openUrl: [url: string];
}>();

const categories = ref<Category[]>([]);
const paymentMethods = ref<PaymentMethod[]>([]);
const household = ref<HouseholdMember[]>([]);
const currencies = ref<Currency[]>([]);
const settings = ref<Settings | null>(null);
const { t } = useI18n();
const { fmtDateFull, fmtCurrency } = useLocaleFormat();

const exp = computed(() => props.expense);

function fmt(price: number, currencyId: string): string {
  const c = currencies.value.find((cur) => cur.id === currencyId);
  return fmtCurrency(price, c?.code || "USD");
}

function fmtCur(amount: number, currency: Currency): string {
  return fmtCurrency(amount, currency.code);
}

const categoryName = computed(() =>
  exp.value ? (categories.value.find((c) => c.id === exp.value!.categoryId)?.name || "") : ""
);
const categoryIcon = computed(() =>
  exp.value ? (categories.value.find((c) => c.id === exp.value!.categoryId)?.icon || "") : ""
);
const paymentMethod = computed(() =>
  exp.value ? paymentMethods.value.find((p) => p.id === exp.value!.paymentMethodId) : null
);
const payerName = computed(() =>
  exp.value ? (household.value.find((h) => h.id === exp.value!.payerUserId)?.name || "") : ""
);

const targetCurrencies = computed(() => {
  const mainId = settings.value?.mainCurrencyId;
  const targets = settings.value?.currencyUpdateTargets ?? [];
  const ids = new Set(targets);
  if (mainId) ids.add(mainId);
  return [...ids]
    .map((id) => currencies.value.find((c) => c.id === id))
    .filter((c): c is Currency => !!c && c.rate > 0);
});

function convertAmount(amount: number, fromCurId: string, toCurrency: Currency): number {
  const fromCur = currencies.value.find((c) => c.id === fromCurId);
  if (!fromCur || fromCur.rate <= 0) return 0;
  return (amount / fromCur.rate) * toCurrency.rate;
}

const convertedPrices = computed(() => {
  if (!exp.value || targetCurrencies.value.length === 0) return [];
  return targetCurrencies.value
    .filter((tc) => tc.id !== exp.value!.currencyId)
    .map((tc) => ({
      currency: tc,
      amount: convertAmount(exp.value!.amount, exp.value!.currencyId, tc),
    }));
});

const linkedSubscription = computed(() => {
  if (!exp.value?.subscriptionId) return null;
  return { id: exp.value.subscriptionId };
});

async function handleOpenUrl(url: string) {
  const fullUrl = url.startsWith("http") ? url : `https://${url}`;
  try { await openUrl(fullUrl); } catch (e) { console.error("Failed to open URL:", e); }
}
watch(
  () => props.lookupData,
  (lookup) => {
    categories.value = lookup.categories;
    paymentMethods.value = lookup.paymentMethods;
    household.value = lookup.household;
    currencies.value = lookup.currencies;
    settings.value = lookup.settings;
  },
  { immediate: true, deep: true },
);
</script>

<template>
  <Modal :show="show && !!exp" :title="t('expense_details')" @close="emit('close')" maxWidth="28rem">
    <div v-if="exp" class="space-y-4">
      <!-- Header: Name + Amount -->
      <div class="flex items-center gap-3">
        <div class="w-11 h-11 rounded-xl bg-primary-light flex items-center justify-center shrink-0">
          <IconDisplay v-if="categoryIcon" :icon="categoryIcon" :size="20" />
          <Wallet v-else :size="20" class="text-primary" />
        </div>
        <div class="flex-1 min-w-0">
          <h3 class="text-base font-semibold text-text-primary truncate">{{ exp.name }}</h3>
          <p class="text-xs text-text-muted">{{ fmtDateFull(expenseToIsoDate(exp)) }}</p>
        </div>
        <div class="text-right shrink-0">
          <p class="text-lg font-bold text-text-primary">{{ fmt(exp.amount, exp.currencyId) }}</p>
          <div v-if="convertedPrices.length > 0" class="mt-0.5 space-y-0">
            <p v-for="cp in convertedPrices" :key="cp.currency.id" class="text-[11px] text-text-muted tabular-nums">
              ≈ {{ fmtCur(cp.amount, cp.currency) }}
            </p>
          </div>
        </div>
      </div>

      <!-- Tags -->
      <div v-if="exp.tags && exp.tags.length > 0" class="flex items-center gap-2 flex-wrap">
        <Hash :size="13" class="text-text-muted shrink-0" />
        <span
          v-for="tag in exp.tags"
          :key="tag"
          class="inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-medium bg-surface-secondary text-text-secondary border border-border"
        >{{ tag }}</span>
      </div>

      <!-- Info grid -->
      <div class="grid grid-cols-2 gap-2">
        <!-- Date -->
        <div class="bg-surface-secondary rounded-lg p-3">
          <div class="flex items-center gap-1.5 mb-1">
            <Calendar :size="13" class="text-text-muted" />
            <span class="text-[10px] uppercase tracking-wide font-medium text-text-muted">{{ t('expense_date') }}</span>
          </div>
          <p class="text-sm font-medium text-text-primary">{{ fmtDateFull(expenseToIsoDate(exp)) }}</p>
        </div>

        <!-- Category -->
        <div class="bg-surface-secondary rounded-lg p-3">
          <div class="flex items-center gap-1.5 mb-1">
            <Tag :size="13" class="text-text-muted" />
            <span class="text-[10px] uppercase tracking-wide font-medium text-text-muted">{{ t('category') }}</span>
          </div>
          <p class="text-sm font-medium text-text-primary flex items-center gap-1.5">
            <IconDisplay v-if="categoryIcon" :icon="categoryIcon" :size="14" />
            {{ categoryName || '—' }}
          </p>
        </div>

        <!-- Payment Method -->
        <div class="bg-surface-secondary rounded-lg p-3">
          <div class="flex items-center gap-1.5 mb-1">
            <CreditCard :size="13" class="text-text-muted" />
            <span class="text-[10px] uppercase tracking-wide font-medium text-text-muted">{{ t('payment_method') }}</span>
          </div>
          <div class="flex items-center gap-1.5">
            <IconDisplay v-if="paymentMethod" :icon="paymentMethod.icon" :size="16" />
            <p class="text-sm font-medium text-text-primary">{{ paymentMethod?.name || '—' }}</p>
          </div>
        </div>

        <!-- Payer -->
        <div class="bg-surface-secondary rounded-lg p-3">
          <div class="flex items-center gap-1.5 mb-1">
            <User :size="13" class="text-text-muted" />
            <span class="text-[10px] uppercase tracking-wide font-medium text-text-muted">{{ t('paid_by') }}</span>
          </div>
          <p class="text-sm font-medium text-text-primary">{{ payerName || '—' }}</p>
        </div>
      </div>

      <!-- URL -->
      <div v-if="exp.url" class="bg-surface-secondary rounded-lg p-3">
        <div class="flex items-center gap-1.5 mb-1">
          <Link :size="13" class="text-text-muted" />
          <span class="text-[10px] uppercase tracking-wide font-medium text-text-muted">{{ t('url') }}</span>
        </div>
        <button @click="handleOpenUrl(exp.url)" class="text-sm text-primary hover:underline truncate block max-w-full text-left">{{ exp.url }}</button>
      </div>

      <!-- Linked subscription -->
      <div v-if="linkedSubscription" class="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-3 border border-blue-200 dark:border-blue-800">
        <div class="flex items-center gap-1.5">
          <Link2 :size="13" class="text-blue-500" />
          <span class="text-xs font-medium text-blue-600 dark:text-blue-400">{{ t('subscriptions') }}</span>
        </div>
      </div>

      <!-- Notes -->
      <div v-if="exp.notes" class="bg-surface-secondary rounded-lg p-3">
        <div class="flex items-center gap-1.5 mb-1">
          <FileText :size="13" class="text-text-muted" />
          <span class="text-[10px] uppercase tracking-wide font-medium text-text-muted">{{ t('notes') }}</span>
        </div>
        <p class="text-sm text-text-secondary whitespace-pre-wrap">{{ exp.notes }}</p>
      </div>
    </div>

    <template #footer>
      <div class="w-full flex items-center gap-1.5">
        <div class="flex items-center gap-1.5 overflow-x-auto scrollbar-none">
          <Tooltip v-if="exp && exp.url" :text="t('url')" position="top">
            <button
              @click="handleOpenUrl(exp!.url)"
              class="w-9 h-9 rounded-xl border border-border text-text-secondary bg-surface-hover hover:text-text-primary hover:bg-surface-secondary transition-colors inline-flex items-center justify-center shrink-0"
            >
              <ExternalLink :size="16" />
            </button>
          </Tooltip>
          <Tooltip v-if="exp" :text="t('edit_expense')" position="top">
            <button
              @click="emit('edit', exp!)"
              class="w-9 h-9 rounded-xl border border-border text-text-secondary bg-surface-hover hover:text-text-primary hover:bg-surface-secondary transition-colors inline-flex items-center justify-center shrink-0"
            >
              <Pencil :size="16" />
            </button>
          </Tooltip>
          <Tooltip v-if="exp" :text="t('delete')" position="top">
            <button
              @click="emit('delete', exp!.id)"
              class="w-9 h-9 rounded-xl border border-border text-text-secondary bg-surface-hover hover:text-text-primary hover:bg-surface-secondary transition-colors inline-flex items-center justify-center shrink-0"
            >
              <Trash2 :size="16" />
            </button>
          </Tooltip>
        </div>
        <div class="flex-1" />
        <button
          @click="emit('close')"
          class="h-9 px-3 sm:px-4 rounded-xl border border-border text-xs sm:text-sm font-medium text-text-secondary hover:bg-surface-hover transition-colors shrink-0"
        >{{ t('cancel') }}</button>
      </div>
    </template>
  </Modal>
</template>
