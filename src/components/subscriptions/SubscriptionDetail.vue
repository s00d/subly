<script setup lang="ts">
import { computed } from "vue";
import { useAppStore } from "@/stores/appStore";
import { useI18n } from "@/i18n";
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import { getPricePerMonth, getDaysUntilPayment, getBillingCycleText, formatCurrency, isOverdue } from "@/services/calculations";
import type { Subscription } from "@/schemas/appData";
import Modal from "@/components/ui/Modal.vue";
import IconDisplay from "@/components/ui/IconDisplay.vue";
import PaymentHistory from "@/components/subscriptions/PaymentHistory.vue";
import { Pencil, Copy, RefreshCw, ExternalLink, Trash2, Calendar, CreditCard, Tag, User, Bell, BellOff, Link, FileText, Clock, AlertTriangle, Power, Star, Hash, CircleDollarSign } from "lucide-vue-next";

const props = defineProps<{
  show: boolean;
  subscription: Subscription | null;
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

const store = useAppStore();
const { t } = useI18n();
const { fmtDateFull, fmtDateMedium } = useLocaleFormat();

const sub = computed(() => props.subscription);

function fmt(price: number, currencyId: string): string {
  const c = store.state.currencies.find((cur) => cur.id === currencyId);
  return formatCurrency(price, c?.code || "USD", c?.symbol);
}

const categoryName = computed(() =>
  sub.value ? (store.state.categories.find((c) => c.id === sub.value!.categoryId)?.name || "") : ""
);

const categoryIcon = computed(() =>
  sub.value ? (store.state.categories.find((c) => c.id === sub.value!.categoryId)?.icon || "") : ""
);

const paymentMethod = computed(() =>
  sub.value ? store.state.paymentMethods.find((p) => p.id === sub.value!.paymentMethodId) : null
);

const payerName = computed(() =>
  sub.value ? (store.state.household.find((h) => h.id === sub.value!.payerUserId)?.name || "") : ""
);

const monthlyPrice = computed(() =>
  sub.value ? getPricePerMonth(sub.value.cycle, sub.value.frequency, sub.value.price) : 0
);

const daysLeft = computed(() =>
  sub.value ? getDaysUntilPayment(sub.value.nextPayment) : 0
);

const overdue = computed(() =>
  sub.value ? isOverdue(sub.value) : false
);

const formatDate = fmtDateFull;
const formatDateShort = fmtDateMedium;
</script>

<template>
  <Modal :show="show && !!sub" :title="t('subscription_details')" @close="emit('close')" maxWidth="32rem">
    <div v-if="sub" class="space-y-4 sm:space-y-5">
      <!-- Header: Logo + Name + Price -->
      <div class="flex items-center gap-3 sm:gap-4">
        <div class="w-14 h-14 rounded-xl bg-[var(--color-primary-light)] flex items-center justify-center text-lg font-bold text-[var(--color-primary)] shrink-0 overflow-hidden">
          <img v-if="sub.logo" :src="sub.logo" class="w-full h-full object-contain" />
          <span v-else>{{ sub.name.charAt(0).toUpperCase() }}</span>
        </div>
        <div class="flex-1 min-w-0">
          <h3 class="text-lg font-semibold text-[var(--color-text-primary)] truncate">{{ sub.name }}</h3>
          <p class="text-xs text-[var(--color-text-muted)]">
            {{ getBillingCycleText(sub.cycle, sub.frequency, t) }}
            <span v-if="!sub.autoRenew" class="ml-1 text-orange-500">({{ t('manual_renewal') }})</span>
          </p>
        </div>
        <div class="text-right shrink-0">
          <p class="text-xl font-bold text-[var(--color-text-primary)]">{{ fmt(sub.price, sub.currencyId) }}</p>
          <p v-if="sub.cycle !== 3 || sub.frequency !== 1" class="text-xs text-[var(--color-text-muted)]">
            ≈ {{ fmt(monthlyPrice, sub.currencyId) }}/{{ t('monthly').toLowerCase() }}
          </p>
        </div>
      </div>

      <!-- Status badges -->
      <div class="flex flex-wrap gap-2">
        <button
          @click="emit('toggleFavorite', sub.id)"
          class="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-medium transition-colors cursor-pointer"
          :class="sub.favorite ? 'bg-yellow-100 text-yellow-600 dark:bg-yellow-900/30 dark:text-yellow-400' : 'bg-gray-100 text-gray-400 dark:bg-gray-800 dark:text-gray-500 hover:text-yellow-500'"
          :title="sub.favorite ? t('remove_from_favorites') : t('add_to_favorites')"
        >
          <Star :size="12" :fill="sub.favorite ? 'currentColor' : 'none'" /> {{ t('favorite') }}
        </button>
        <span v-if="sub.inactive" class="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-medium bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400">
          <Power :size="12" /> {{ t('inactive') }}
        </span>
        <span v-if="overdue" class="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-medium bg-red-100 text-red-600 dark:bg-red-900/30 dark:text-red-400">
          <AlertTriangle :size="12" /> {{ t('overdue') }}
        </span>
        <span v-if="!sub.inactive"
          class="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-medium"
          :class="daysLeft <= 3 ? 'bg-red-100 text-red-600 dark:bg-red-900/30 dark:text-red-400' : daysLeft <= 7 ? 'bg-orange-100 text-orange-600 dark:bg-orange-900/30 dark:text-orange-400' : 'bg-[var(--color-primary-light)] text-[var(--color-primary)]'"
        >
          <Clock :size="12" /> {{ daysLeft }}{{ t('days_short') }} {{ t('next_payment').toLowerCase() }}
        </span>
      </div>

      <!-- Tags -->
      <div v-if="sub.tags && sub.tags.length > 0" class="flex items-center gap-2 flex-wrap">
        <Hash :size="13" class="text-[var(--color-text-muted)] shrink-0" />
        <span
          v-for="tag in sub.tags"
          :key="tag"
          class="inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-medium bg-[var(--color-surface-secondary)] text-[var(--color-text-secondary)] border border-[var(--color-border)]"
        >{{ tag }}</span>
      </div>

      <!-- Info grid -->
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-2 sm:gap-3">
        <!-- Next Payment -->
        <div class="bg-[var(--color-surface-secondary)] rounded-lg p-3">
          <div class="flex items-center gap-1.5 mb-1">
            <Calendar :size="13" class="text-[var(--color-text-muted)]" />
            <span class="text-[10px] uppercase tracking-wide font-medium text-[var(--color-text-muted)]">{{ t('next_payment') }}</span>
          </div>
          <p class="text-sm font-medium text-[var(--color-text-primary)]" :class="{ 'text-red-500': overdue }">{{ formatDate(sub.nextPayment) }}</p>
        </div>

        <!-- Start Date -->
        <div class="bg-[var(--color-surface-secondary)] rounded-lg p-3">
          <div class="flex items-center gap-1.5 mb-1">
            <Calendar :size="13" class="text-[var(--color-text-muted)]" />
            <span class="text-[10px] uppercase tracking-wide font-medium text-[var(--color-text-muted)]">{{ t('start_date') }}</span>
          </div>
          <p class="text-sm font-medium text-[var(--color-text-primary)]">{{ formatDateShort(sub.startDate) }}</p>
        </div>

        <!-- Category -->
        <div class="bg-[var(--color-surface-secondary)] rounded-lg p-3">
          <div class="flex items-center gap-1.5 mb-1">
            <Tag :size="13" class="text-[var(--color-text-muted)]" />
            <span class="text-[10px] uppercase tracking-wide font-medium text-[var(--color-text-muted)]">{{ t('category') }}</span>
          </div>
          <p class="text-sm font-medium text-[var(--color-text-primary)] flex items-center gap-1.5">
            <IconDisplay v-if="categoryIcon" :icon="categoryIcon" :size="16" />
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
            <IconDisplay v-if="paymentMethod" :icon="paymentMethod.icon" :size="18" />
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

        <!-- Notifications -->
        <div class="bg-[var(--color-surface-secondary)] rounded-lg p-3">
          <div class="flex items-center gap-1.5 mb-1">
            <component :is="sub.notify ? Bell : BellOff" :size="13" class="text-[var(--color-text-muted)]" />
            <span class="text-[10px] uppercase tracking-wide font-medium text-[var(--color-text-muted)]">{{ t('notifications') }}</span>
          </div>
          <p class="text-sm font-medium text-[var(--color-text-primary)]">
            {{ sub.notify ? (sub.notifyDaysBefore === 0 ? t('on_due_date') : sub.notifyDaysBefore === -1 ? t('default_value_from_settings') : sub.notifyDaysBefore + ' ' + t('days_before')) : t('off') }}
          </p>
        </div>
      </div>

      <!-- URL -->
      <div v-if="sub.url" class="bg-[var(--color-surface-secondary)] rounded-lg p-3">
        <div class="flex items-center gap-1.5 mb-1">
          <Link :size="13" class="text-[var(--color-text-muted)]" />
          <span class="text-[10px] uppercase tracking-wide font-medium text-[var(--color-text-muted)]">{{ t('url') }}</span>
        </div>
        <button @click="emit('openUrl', sub.url)" class="text-sm text-[var(--color-primary)] hover:underline truncate block max-w-full text-left">{{ sub.url }}</button>
      </div>

      <!-- Notes -->
      <div v-if="sub.notes" class="bg-[var(--color-surface-secondary)] rounded-lg p-3">
        <div class="flex items-center gap-1.5 mb-1">
          <FileText :size="13" class="text-[var(--color-text-muted)]" />
          <span class="text-[10px] uppercase tracking-wide font-medium text-[var(--color-text-muted)]">{{ t('notes') }}</span>
        </div>
        <p class="text-sm text-[var(--color-text-secondary)] whitespace-pre-wrap">{{ sub.notes }}</p>
      </div>

      <!-- Payment History -->
      <PaymentHistory
        :subscriptionId="sub.id"
        :currencyId="sub.currencyId"
        :price="sub.price"
        :history="sub.paymentHistory || []"
      />

      <!-- Cancellation -->
      <div v-if="sub.cancellationDate" class="bg-orange-50 dark:bg-orange-900/20 rounded-lg p-3 border border-orange-200 dark:border-orange-800">
        <p class="text-xs font-medium text-orange-600 dark:text-orange-400">
          {{ t('cancellation_notification') }}: {{ formatDateShort(sub.cancellationDate) }}
        </p>
      </div>
    </div>

    <template #footer>
      <div class="flex items-center gap-2 w-full">
        <!-- Left actions -->
        <button
          v-if="sub && !sub.inactive"
          @click="emit('recordPayment', sub!.id)"
          class="p-2 rounded-lg text-[var(--color-text-muted)] hover:text-emerald-600 hover:bg-emerald-50 dark:hover:bg-emerald-900/20 transition-colors"
          :title="t('record_payment')"
        >
          <CircleDollarSign :size="16" />
        </button>
        <button
          v-if="sub && !sub.autoRenew"
          @click="emit('renew', sub!.id)"
          class="p-2 rounded-lg text-[var(--color-text-muted)] hover:text-green-600 hover:bg-green-50 dark:hover:bg-green-900/20 transition-colors"
          :title="t('renew')"
        >
          <RefreshCw :size="16" />
        </button>
        <button
          v-if="sub"
          @click="emit('clone', sub!.id)"
          class="p-2 rounded-lg text-[var(--color-text-muted)] hover:text-[var(--color-primary)] hover:bg-[var(--color-primary-light)] transition-colors"
          :title="t('clone')"
        >
          <Copy :size="16" />
        </button>
        <button
          v-if="sub"
          @click="emit('delete', sub!.id)"
          class="p-2 rounded-lg text-[var(--color-text-muted)] hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors"
          :title="t('delete')"
        >
          <Trash2 :size="16" />
        </button>

        <div class="flex-1" />

        <!-- Right actions -->
        <button
          @click="emit('close')"
          class="px-4 py-2 rounded-lg border border-[var(--color-border)] text-sm font-medium text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)] transition-colors"
        >{{ t('cancel') }}</button>
        <button
          v-if="sub"
          @click="emit('edit', sub!)"
          class="px-5 py-2 rounded-lg bg-[var(--color-primary)] text-white text-sm font-medium hover:bg-[var(--color-primary-hover)] transition-colors flex items-center gap-1.5"
        >
          <Pencil :size="14" />
          {{ t('edit_subscription') }}
        </button>
      </div>
    </template>
  </Modal>
</template>
