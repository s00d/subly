<script setup lang="ts">
import { computed } from "vue";
import { useCatalogStore } from "@/stores/catalog";
import { useSettingsStore } from "@/stores/settings";
import { useI18n } from "vue-i18n";
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import { formatCurrency } from "@/services/calculations";
import type { Expense, Currency } from "@/schemas/appData";
import Modal from "@/components/ui/Modal.vue";
import IconDisplay from "@/components/ui/IconDisplay.vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import { Pencil, Trash2, Calendar, CreditCard, Tag, User, FileText, Hash, Wallet, Link2, ExternalLink, Link } from "lucide-vue-next";
import { openUrl } from "@tauri-apps/plugin-opener";

const props = defineProps<{
  show: boolean;
  expense: Expense | null;
}>();

const emit = defineEmits<{
  close: [];
  edit: [exp: Expense];
  delete: [id: string];
  openUrl: [url: string];
}>();

const catalogStore = useCatalogStore();
const settingsStore = useSettingsStore();
const { t } = useI18n();
const { fmtDateFull } = useLocaleFormat();

const exp = computed(() => props.expense);

function fmt(price: number, currencyId: string): string {
  const c = catalogStore.currencies.find((cur) => cur.id === currencyId);
  return formatCurrency(price, c?.code || "USD", c?.symbol);
}

function fmtCur(amount: number, currency: Currency): string {
  return formatCurrency(amount, currency.code, currency.symbol);
}

const categoryName = computed(() =>
  exp.value ? (catalogStore.categories.find((c) => c.id === exp.value!.categoryId)?.name || "") : ""
);
const categoryIcon = computed(() =>
  exp.value ? (catalogStore.categories.find((c) => c.id === exp.value!.categoryId)?.icon || "") : ""
);
const paymentMethod = computed(() =>
  exp.value ? catalogStore.paymentMethods.find((p) => p.id === exp.value!.paymentMethodId) : null
);
const payerName = computed(() =>
  exp.value ? (catalogStore.household.find((h) => h.id === exp.value!.payerUserId)?.name || "") : ""
);

const targetCurrencies = computed(() => {
  const mainId = settingsStore.settings.mainCurrencyId;
  const targets = settingsStore.settings.currencyUpdateTargets ?? [];
  const ids = new Set(targets);
  if (mainId) ids.add(mainId);
  return [...ids]
    .map((id) => catalogStore.currencies.find((c) => c.id === id))
    .filter((c): c is Currency => !!c && c.rate > 0);
});

function convertAmount(amount: number, fromCurId: string, toCurrency: Currency): number {
  const fromCur = catalogStore.currencies.find((c) => c.id === fromCurId);
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
</script>

<template>
  <Modal :show="show && !!exp" :title="t('expense_details')" @close="emit('close')" maxWidth="28rem">
    <div v-if="exp" class="space-y-4">
      <!-- Header: Name + Amount -->
      <div class="flex items-center gap-3">
        <div class="w-11 h-11 rounded-xl bg-[var(--color-primary-light)] flex items-center justify-center shrink-0">
          <IconDisplay v-if="categoryIcon" :icon="categoryIcon" :size="20" />
          <Wallet v-else :size="20" class="text-[var(--color-primary)]" />
        </div>
        <div class="flex-1 min-w-0">
          <h3 class="text-base font-semibold text-[var(--color-text-primary)] truncate">{{ exp.name }}</h3>
          <p class="text-xs text-[var(--color-text-muted)]">{{ fmtDateFull(exp.date) }}</p>
        </div>
        <div class="text-right shrink-0">
          <p class="text-lg font-bold text-[var(--color-text-primary)]">{{ fmt(exp.amount, exp.currencyId) }}</p>
          <div v-if="convertedPrices.length > 0" class="mt-0.5 space-y-0">
            <p v-for="cp in convertedPrices" :key="cp.currency.id" class="text-[11px] text-[var(--color-text-muted)] tabular-nums">
              ≈ {{ fmtCur(cp.amount, cp.currency) }}
            </p>
          </div>
        </div>
      </div>

      <!-- Tags -->
      <div v-if="exp.tags && exp.tags.length > 0" class="flex items-center gap-2 flex-wrap">
        <Hash :size="13" class="text-[var(--color-text-muted)] shrink-0" />
        <span
          v-for="tag in exp.tags"
          :key="tag"
          class="inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-medium bg-[var(--color-surface-secondary)] text-[var(--color-text-secondary)] border border-[var(--color-border)]"
        >{{ tag }}</span>
      </div>

      <!-- Info grid -->
      <div class="grid grid-cols-2 gap-2">
        <!-- Date -->
        <div class="bg-[var(--color-surface-secondary)] rounded-lg p-3">
          <div class="flex items-center gap-1.5 mb-1">
            <Calendar :size="13" class="text-[var(--color-text-muted)]" />
            <span class="text-[10px] uppercase tracking-wide font-medium text-[var(--color-text-muted)]">{{ t('expense_date') }}</span>
          </div>
          <p class="text-sm font-medium text-[var(--color-text-primary)]">{{ fmtDateFull(exp.date) }}</p>
        </div>

        <!-- Category -->
        <div class="bg-[var(--color-surface-secondary)] rounded-lg p-3">
          <div class="flex items-center gap-1.5 mb-1">
            <Tag :size="13" class="text-[var(--color-text-muted)]" />
            <span class="text-[10px] uppercase tracking-wide font-medium text-[var(--color-text-muted)]">{{ t('category') }}</span>
          </div>
          <p class="text-sm font-medium text-[var(--color-text-primary)] flex items-center gap-1.5">
            <IconDisplay v-if="categoryIcon" :icon="categoryIcon" :size="14" />
            {{ categoryName || '—' }}
          </p>
        </div>

        <!-- Payment Method -->
        <div class="bg-[var(--color-surface-secondary)] rounded-lg p-3">
          <div class="flex items-center gap-1.5 mb-1">
            <CreditCard :size="13" class="text-[var(--color-text-muted)]" />
            <span class="text-[10px] uppercase tracking-wide font-medium text-[var(--color-text-muted)]">{{ t('payment_method') }}</span>
          </div>
          <div class="flex items-center gap-1.5">
            <IconDisplay v-if="paymentMethod" :icon="paymentMethod.icon" :size="16" />
            <p class="text-sm font-medium text-[var(--color-text-primary)]">{{ paymentMethod?.name || '—' }}</p>
          </div>
        </div>

        <!-- Payer -->
        <div class="bg-[var(--color-surface-secondary)] rounded-lg p-3">
          <div class="flex items-center gap-1.5 mb-1">
            <User :size="13" class="text-[var(--color-text-muted)]" />
            <span class="text-[10px] uppercase tracking-wide font-medium text-[var(--color-text-muted)]">{{ t('paid_by') }}</span>
          </div>
          <p class="text-sm font-medium text-[var(--color-text-primary)]">{{ payerName || '—' }}</p>
        </div>
      </div>

      <!-- URL -->
      <div v-if="exp.url" class="bg-[var(--color-surface-secondary)] rounded-lg p-3">
        <div class="flex items-center gap-1.5 mb-1">
          <Link :size="13" class="text-[var(--color-text-muted)]" />
          <span class="text-[10px] uppercase tracking-wide font-medium text-[var(--color-text-muted)]">{{ t('url') }}</span>
        </div>
        <button @click="handleOpenUrl(exp.url)" class="text-sm text-[var(--color-primary)] hover:underline truncate block max-w-full text-left">{{ exp.url }}</button>
      </div>

      <!-- Linked subscription -->
      <div v-if="linkedSubscription" class="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-3 border border-blue-200 dark:border-blue-800">
        <div class="flex items-center gap-1.5">
          <Link2 :size="13" class="text-blue-500" />
          <span class="text-xs font-medium text-blue-600 dark:text-blue-400">{{ t('subscriptions') }}</span>
        </div>
      </div>

      <!-- Notes -->
      <div v-if="exp.notes" class="bg-[var(--color-surface-secondary)] rounded-lg p-3">
        <div class="flex items-center gap-1.5 mb-1">
          <FileText :size="13" class="text-[var(--color-text-muted)]" />
          <span class="text-[10px] uppercase tracking-wide font-medium text-[var(--color-text-muted)]">{{ t('notes') }}</span>
        </div>
        <p class="text-sm text-[var(--color-text-secondary)] whitespace-pre-wrap">{{ exp.notes }}</p>
      </div>
    </div>

    <template #footer>
      <div class="flex items-center gap-0.5 sm:gap-1 w-full overflow-x-auto scrollbar-none">
        <Tooltip v-if="exp && exp.url" :text="t('url')">
          <button
            @click="handleOpenUrl(exp!.url)"
            class="p-1.5 sm:p-2 rounded-lg text-[var(--color-text-muted)] hover:text-[var(--color-primary)] hover:bg-[var(--color-primary-light)] transition-colors shrink-0"
          >
            <ExternalLink :size="16" />
          </button>
        </Tooltip>
        <Tooltip v-if="exp" :text="t('edit')">
          <button
            @click="emit('edit', exp!)"
            class="p-1.5 sm:p-2 rounded-lg text-[var(--color-text-muted)] hover:text-[var(--color-primary)] hover:bg-[var(--color-primary-light)] transition-colors shrink-0"
          >
            <Pencil :size="16" />
          </button>
        </Tooltip>
        <Tooltip v-if="exp" :text="t('delete')">
          <button
            @click="emit('delete', exp!.id)"
            class="p-1.5 sm:p-2 rounded-lg text-[var(--color-text-muted)] hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors shrink-0"
          >
            <Trash2 :size="16" />
          </button>
        </Tooltip>
        <div class="flex-1" />
        <button
          @click="emit('close')"
          class="px-3 sm:px-4 py-1.5 sm:py-2 rounded-lg border border-[var(--color-border)] text-xs sm:text-sm font-medium text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)] transition-colors shrink-0"
        >{{ t('cancel') }}</button>
      </div>
    </template>
  </Modal>
</template>
