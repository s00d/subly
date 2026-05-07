<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import type { Subscription, SubscriptionCredentials, SubscriptionListItem, Currency, Category, PaymentMethod, HouseholdMember, Settings } from "@/schemas/appData";
import Modal from "@/components/ui/Modal.vue";
import IconDisplay from "@/components/ui/IconDisplay.vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import PaymentHistory from "@/components/subscriptions/PaymentHistory.vue";
import { Pencil, Copy, RefreshCw, Trash2, Calendar, CreditCard, Tag, User, Bell, BellOff, Link, FileText, Clock, AlertTriangle, Power, Star, Hash, CircleDollarSign, KeyRound } from "@lucide/vue";
import {
  subscriptionTotpCurrent,
  type SubscriptionTotpCurrentDto,
} from "@/services/subscriptionCredentialsClient";
import { useClipboard } from "@/composables/useClipboard";
import { useToast } from "@/composables/useToast";
import { ui, statValue } from "@/lib/tv";

const props = defineProps<{
  show: boolean;
  subscription: Subscription | null;
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
  edit: [sub: Subscription];
  clone: [id: string];
  renew: [id: string];
  delete: [id: string];
  openUrl: [url: string];
  toggleFavorite: [id: string];
  recordPayment: [id: string];
}>();

const categories = ref<Category[]>([]);
const paymentMethods = ref<PaymentMethod[]>([]);
const household = ref<HouseholdMember[]>([]);
const currencies = ref<Currency[]>([]);
const settings = ref<Settings | null>(null);
const { t } = useI18n();
const { fmtDateFull, fmtDateMedium, fmtCurrency } = useLocaleFormat();
const { copyToClipboard } = useClipboard();
const { toast } = useToast();

const totpCurrent = ref<SubscriptionTotpCurrentDto | null>(null);
let otpPollTimer: ReturnType<typeof setInterval> | null = null;

const sub = computed(() => props.subscription as SubscriptionListItem | null);

/** Учётные данные приходят в объекте подписки из списка (secure storage на бэкенде). */
const creds = computed<SubscriptionCredentials | null>(() => sub.value?.credentials ?? null);

const hasSavedCredentials = computed(() => {
  const c = creds.value;
  if (!c) return false;
  return Boolean(c.login?.trim() || c.password || c.totpSecret?.trim());
});

const otpSecondsLeft = computed(() => {
  if (!totpCurrent.value) return 0;
  const ms = totpCurrent.value.validUntilMs - Date.now();
  return Math.max(0, Math.ceil(ms / 1000));
});

function clearOtpPoll() {
  if (otpPollTimer) {
    clearInterval(otpPollTimer);
    otpPollTimer = null;
  }
}

async function refreshTotpOnly() {
  if (!sub.value) return;
  try {
    totpCurrent.value = await subscriptionTotpCurrent(sub.value.id);
  } catch {
    totpCurrent.value = null;
  }
}

async function copyLoginField() {
  const v = creds.value?.login?.trim() ?? "";
  if (!v) return;
  if (await copyToClipboard(v)) toast(t("copied_to_clipboard"));
  else toast(t("clipboard_copy_failed"), "error");
}

async function copyPasswordField() {
  const v = creds.value?.password ?? "";
  if (!v) return;
  if (await copyToClipboard(v)) toast(t("copied_to_clipboard"));
  else toast(t("clipboard_copy_failed"), "error");
}

async function copyOtpField() {
  const v = totpCurrent.value?.code ?? "";
  if (!v) return;
  if (await copyToClipboard(v)) toast(t("copied_to_clipboard"));
  else toast(t("clipboard_copy_failed"), "error");
}

watch(
  () => ({
    open: props.show,
    totpSecret: sub.value?.credentials?.totpSecret ?? "",
  }),
  () => {
    clearOtpPoll();
    totpCurrent.value = null;
    if (!props.show || !sub.value?.credentials?.totpSecret?.trim()) return;
    void refreshTotpOnly();
    otpPollTimer = setInterval(() => void refreshTotpOnly(), 2500);
  },
  { immediate: true },
);

function fmt(price: number, currencyId: string): string {
  const c = currencies.value.find((cur) => cur.id === currencyId);
  return fmtCurrency(price, c?.code || "USD");
}

function fmtCur(amount: number, currency: Currency): string {
  return fmtCurrency(amount, currency.code);
}

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
  if (!sub.value || targetCurrencies.value.length === 0) return [];
  return targetCurrencies.value
    .filter((tc) => tc.id !== sub.value!.currencyId)
    .map((tc) => ({
      currency: tc,
      amount: convertAmount(sub.value!.price, sub.value!.currencyId, tc),
    }));
});

const convertedMonthly = computed(() => {
  if (!sub.value || targetCurrencies.value.length === 0) return [];
  return targetCurrencies.value
    .filter((tc) => tc.id !== sub.value!.currencyId)
    .map((tc) => ({
      currency: tc,
      amount: convertAmount(monthlyPrice.value, sub.value!.currencyId, tc),
    }));
});

const categoryName = computed(() =>
  sub.value ? (categories.value.find((c) => c.id === sub.value!.categoryId)?.name || "") : ""
);

const categoryIcon = computed(() =>
  sub.value ? (categories.value.find((c) => c.id === sub.value!.categoryId)?.icon || "") : ""
);

const paymentMethod = computed(() =>
  sub.value ? paymentMethods.value.find((p) => p.id === sub.value!.paymentMethodId) : null
);

const payerName = computed(() =>
  sub.value ? (household.value.find((h) => h.id === sub.value!.payerUserId)?.name || "") : ""
);

const monthlyPrice = computed(() =>
  sub.value ? Number(sub.value.monthlyPrice ?? 0) : 0
);

const daysLeft = computed(() =>
  sub.value ? Number(sub.value.daysLeft ?? 0) : 0
);

const overdue = computed(() =>
  sub.value ? Boolean(sub.value.overdue) : false
);

const formatDate = fmtDateFull;
const formatDateShort = fmtDateMedium;

function billingCycleText(cycle: number, frequency: number): string {
  switch (cycle) {
    case 1: return frequency === 1 ? t("daily") : `${frequency} ${t("days")}`;
    case 2: return frequency === 1 ? t("weekly") : `${frequency} ${t("weeks")}`;
    case 3: return frequency === 1 ? t("monthly") : `${frequency} ${t("months")}`;
    case 4: return frequency === 1 ? t("yearly") : `${frequency} ${t("years")}`;
    default: return "";
  }
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

onUnmounted(() => clearOtpPoll());
</script>

<template>
  <Modal :show="show && !!sub" :title="t('subscription_details')" @close="emit('close')" maxWidth="32rem">
    <div v-if="sub" class="space-y-4 sm:space-y-5">
      <!-- Header: Logo + Name + Price -->
      <div class="flex items-center gap-2.5 sm:gap-4">
        <div class="w-11 h-11 sm:w-14 sm:h-14 rounded-xl bg-primary-light flex items-center justify-center text-base sm:text-lg font-bold text-primary shrink-0 overflow-hidden">
          <img v-if="sub.logo" :src="sub.logo" class="w-full h-full object-contain" />
          <span v-else>{{ sub.name.charAt(0).toUpperCase() }}</span>
        </div>
        <div class="flex-1 min-w-0">
          <h3 :class="[ui.sectionTitle(), 'truncate']">{{ sub.name }}</h3>
          <p class="text-[10px] sm:text-xs text-text-muted">
            {{ billingCycleText(sub.cycle, sub.frequency) }}
            <span v-if="!sub.autoRenew" class="ml-1 text-orange-500">({{ t('manual_renewal') }})</span>
          </p>
        </div>
        <div class="text-right shrink-0">
          <p :class="statValue()">{{ fmt(sub.price, sub.currencyId) }}</p>
          <p v-if="sub.cycle !== 3 || sub.frequency !== 1" class="text-[10px] sm:text-xs text-text-muted">
            ≈ {{ fmt(monthlyPrice, sub.currencyId) }}/{{ t('monthly').toLowerCase() }}
          </p>
          <div v-if="convertedPrices.length > 0" class="mt-1 space-y-0.5">
            <p v-for="cp in convertedPrices" :key="cp.currency.id" class="text-[10px] sm:text-xs text-text-muted">
              ≈ {{ fmtCur(cp.amount, cp.currency) }}
              <span v-if="sub.cycle !== 3 || sub.frequency !== 1">
                ({{ fmtCur(convertedMonthly.find(cm => cm.currency.id === cp.currency.id)?.amount || 0, cp.currency) }}/{{ t('monthly').toLowerCase().charAt(0) }})
              </span>
            </p>
          </div>
        </div>
      </div>

      <!-- Status badges -->
      <div class="flex flex-wrap gap-2">
        <button
          @click="emit('toggleFavorite', sub.id)"
          class="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-medium transition-colors cursor-pointer border"
          :class="sub.favorite
            ? 'bg-surface-hover border-border text-text-primary'
            : 'bg-surface-hover border-border text-text-secondary hover:text-text-primary hover:bg-surface-secondary'"
          :title="sub.favorite ? t('remove_from_favorites') : t('add_to_favorites')"
        >
          <Star :size="12" :fill="sub.favorite ? 'currentColor' : 'none'" class="text-amber-500" />
          {{ t('favorite') }}
        </button>
        <span v-if="sub.inactive" class="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-medium bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400">
          <Power :size="12" /> {{ t('inactive') }}
        </span>
        <span v-if="overdue" class="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-medium bg-red-100 text-red-600 dark:bg-red-900/30 dark:text-red-400">
          <AlertTriangle :size="12" /> {{ t('overdue') }}
        </span>
        <span v-if="!sub.inactive"
          class="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-medium bg-surface-hover text-text-primary border"
          :class="daysLeft <= 3
            ? 'border-red-200 dark:border-red-800'
            : daysLeft <= 7
              ? 'border-orange-200 dark:border-orange-800'
              : 'border-border'"
        >
          <Clock
            :size="12"
            :class="daysLeft <= 3
              ? 'text-red-500 dark:text-red-400'
              : daysLeft <= 7
                ? 'text-orange-500 dark:text-orange-400'
                : 'text-primary'"
          />
          {{ daysLeft }}{{ t('days_short') }} {{ t('next_payment').toLowerCase() }}
        </span>
      </div>

      <!-- Tags -->
      <div v-if="sub.tags && sub.tags.length > 0" class="flex items-center gap-2 flex-wrap">
        <Hash :size="13" class="text-text-muted shrink-0" />
        <span
          v-for="tag in sub.tags"
          :key="tag"
          class="inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-medium bg-surface-secondary text-text-secondary border border-border"
        >{{ tag }}</span>
      </div>

      <!-- Saved credentials -->
      <div
        v-if="sub"
        class="rounded-xl border border-border p-3 bg-surface-secondary/80 space-y-3"
      >
        <div class="flex items-center justify-between gap-2">
          <div class="flex items-center gap-2 min-w-0">
            <KeyRound :size="15" class="text-primary shrink-0" />
            <span class="text-xs font-semibold text-text-primary uppercase tracking-wide">{{ t("credentials_section") }}</span>
          </div>
          <button
            type="button"
            class="shrink-0 inline-flex items-center gap-1 px-2 py-1 rounded-lg border border-border text-xs font-medium text-text-secondary hover:bg-surface-hover"
            @click="emit('edit', sub)"
          >
            <Pencil :size="12" />
            {{ t("edit_credentials") }}
          </button>
        </div>
        <template v-if="hasSavedCredentials">
          <div class="flex flex-wrap gap-2">
            <button
              v-if="creds?.login?.trim()"
              type="button"
              class="inline-flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg border border-border text-xs font-medium text-text-secondary hover:bg-surface-hover"
              @click="copyLoginField"
            >
              <Copy :size="12" /> {{ t("copy_login") }}
            </button>
            <button
              v-if="creds?.password"
              type="button"
              class="inline-flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg border border-border text-xs font-medium text-text-secondary hover:bg-surface-hover"
              @click="copyPasswordField"
            >
              <Copy :size="12" /> {{ t("copy_password") }}
            </button>
          </div>
          <p v-if="creds?.login?.trim()" class="text-sm text-text-primary font-mono truncate">{{ creds.login }}</p>
          <p v-if="creds?.password" class="text-sm font-mono tracking-widest text-text-secondary">
            ••••••••
          </p>
          <div v-if="creds?.totpSecret?.trim() && totpCurrent" class="rounded-lg border border-border p-2 bg-surface">
            <p class="text-[10px] text-text-muted mb-1">{{ t("current_otp") }}</p>
            <p class="font-mono text-lg tracking-widest text-center">{{ totpCurrent.code }}</p>
            <p class="text-[11px] text-center text-text-muted mt-1">{{ t("otp_expires_in", { s: otpSecondsLeft }) }}</p>
            <button
              type="button"
              class="mt-2 w-full inline-flex items-center justify-center gap-1 px-2 py-1.5 rounded-md bg-surface-hover text-xs font-medium"
              @click="copyOtpField"
            >
              <Copy :size="12" /> {{ t("copy_otp") }}
            </button>
          </div>
        </template>
        <p v-else class="text-xs text-text-muted">{{ t("credentials_none_hint") }}</p>
      </div>

      <!-- Info grid -->
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-2 sm:gap-3">
        <!-- Next Payment -->
        <div class="bg-surface-secondary rounded-lg p-3">
          <div class="flex items-center gap-1.5 mb-1">
            <Calendar :size="13" class="text-text-muted" />
            <span class="text-[10px] uppercase tracking-wide font-medium text-text-muted">{{ t('next_payment') }}</span>
          </div>
          <p class="text-sm font-medium text-text-primary" :class="{ 'text-red-500': overdue }">{{ formatDate(sub.nextPayment) }}</p>
        </div>

        <!-- Start Date -->
        <div class="bg-surface-secondary rounded-lg p-3">
          <div class="flex items-center gap-1.5 mb-1">
            <Calendar :size="13" class="text-text-muted" />
            <span class="text-[10px] uppercase tracking-wide font-medium text-text-muted">{{ t('start_date') }}</span>
          </div>
          <p class="text-sm font-medium text-text-primary">{{ formatDateShort(sub.startDate) }}</p>
        </div>

        <!-- Category -->
        <div class="bg-surface-secondary rounded-lg p-3">
          <div class="flex items-center gap-1.5 mb-1">
            <Tag :size="13" class="text-text-muted" />
            <span class="text-[10px] uppercase tracking-wide font-medium text-text-muted">{{ t('category') }}</span>
          </div>
          <p class="text-sm font-medium text-text-primary flex items-center gap-1.5">
            <IconDisplay v-if="categoryIcon" :icon="categoryIcon" :size="16" />
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
            <IconDisplay v-if="paymentMethod" :icon="paymentMethod.icon" :size="18" />
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

        <!-- Notifications -->
        <div class="bg-surface-secondary rounded-lg p-3">
          <div class="flex items-center gap-1.5 mb-1">
            <component :is="sub.notify ? Bell : BellOff" :size="13" class="text-text-muted" />
            <span class="text-[10px] uppercase tracking-wide font-medium text-text-muted">{{ t('notifications') }}</span>
          </div>
          <p class="text-sm font-medium text-text-primary">
            {{ sub.notify ? (sub.notifyDaysBefore === 0 ? t('on_due_date') : sub.notifyDaysBefore === -1 ? t('default_value_from_settings') : sub.notifyDaysBefore + ' ' + t('days_before')) : t('off') }}
          </p>
        </div>
      </div>

      <!-- URL -->
      <div v-if="sub.url" class="bg-surface-secondary rounded-lg p-3">
        <div class="flex items-center gap-1.5 mb-1">
          <Link :size="13" class="text-text-muted" />
          <span class="text-[10px] uppercase tracking-wide font-medium text-text-muted">{{ t('url') }}</span>
        </div>
        <button @click="emit('openUrl', sub.url)" class="text-sm text-primary hover:underline truncate block max-w-full text-left">{{ sub.url }}</button>
      </div>

      <!-- Notes -->
      <div v-if="sub.notes" class="bg-surface-secondary rounded-lg p-3">
        <div class="flex items-center gap-1.5 mb-1">
          <FileText :size="13" class="text-text-muted" />
          <span class="text-[10px] uppercase tracking-wide font-medium text-text-muted">{{ t('notes') }}</span>
        </div>
        <p class="text-sm text-text-secondary whitespace-pre-wrap">{{ sub.notes }}</p>
      </div>

      <!-- Payment History -->
      <PaymentHistory
        :subscriptionId="sub.id"
        :currencyId="sub.currencyId"
        :price="sub.price"
        :history="sub.paymentHistory || []"
        :lookupData="{
          currencies,
          mainCurrencyId: settings?.mainCurrencyId || 'cur-2',
          targetCurrencyIds: settings?.currencyUpdateTargets || [],
        }"
        @recordPayment="(id: string) => emit('recordPayment', id)"
      />

      <!-- Cancellation -->
      <div v-if="sub.cancellationDate" class="bg-orange-50 dark:bg-orange-900/20 rounded-lg p-3 border border-orange-200 dark:border-orange-800">
        <p class="text-xs font-medium text-orange-600 dark:text-orange-400">
          {{ t('cancellation_notification') }}: {{ formatDateShort(sub.cancellationDate) }}
        </p>
      </div>
    </div>

    <template #footer>
      <div class="w-full flex items-center gap-1.5">
        <div class="flex items-center gap-1.5 overflow-x-auto scrollbar-none">
          <Tooltip v-if="sub && !sub.inactive" :text="t('record_payment')" position="top">
            <button
              @click="emit('recordPayment', sub!.id)"
              class="w-9 h-9 rounded-xl border border-border text-text-secondary bg-surface-hover hover:text-text-primary hover:bg-surface-secondary transition-colors inline-flex items-center justify-center shrink-0"
            >
              <CircleDollarSign :size="16" />
            </button>
          </Tooltip>
          <Tooltip v-if="sub" :text="t('renew')" position="top">
            <button
              @click="emit('renew', sub!.id)"
              class="w-9 h-9 rounded-xl border border-border text-text-secondary bg-surface-hover hover:text-text-primary hover:bg-surface-secondary transition-colors inline-flex items-center justify-center shrink-0"
            >
              <RefreshCw :size="16" />
            </button>
          </Tooltip>
          <Tooltip v-if="sub" :text="t('clone')" position="top">
            <button
              @click="emit('clone', sub!.id)"
              class="w-9 h-9 rounded-xl border border-border text-text-secondary bg-surface-hover hover:text-text-primary hover:bg-surface-secondary transition-colors inline-flex items-center justify-center shrink-0"
            >
              <Copy :size="16" />
            </button>
          </Tooltip>
          <Tooltip v-if="sub" :text="t('edit_subscription')" position="top">
            <button
              @click="emit('edit', sub!)"
              class="w-9 h-9 rounded-xl border border-border text-text-secondary bg-surface-hover hover:text-text-primary hover:bg-surface-secondary transition-colors inline-flex items-center justify-center shrink-0"
            >
              <Pencil :size="16" />
            </button>
          </Tooltip>
          <Tooltip v-if="sub" :text="t('delete')" position="top">
            <button
              @click="emit('delete', sub!.id)"
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
