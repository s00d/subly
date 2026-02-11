<script setup lang="ts">
import { ref, computed } from "vue";
import { useAppStore } from "@/stores/appStore";
import { useI18n } from "@/i18n";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import { useToast } from "@/composables/useToast";
import { getPaymentDatesInMonth, convertPrice } from "@/services/calculations";
import { openUrl } from "@tauri-apps/plugin-opener";
import CalendarHeader from "@/components/calendar/CalendarHeader.vue";
import CalendarGrid from "@/components/calendar/CalendarGrid.vue";
import type { CalendarCell } from "@/components/calendar/CalendarGrid.vue";
import CalendarMonthStats from "@/components/calendar/CalendarMonthStats.vue";
import CalendarDayModal from "@/components/calendar/CalendarDayModal.vue";
import SubscriptionDetail from "@/components/subscriptions/SubscriptionDetail.vue";
import SubscriptionForm from "@/components/subscriptions/SubscriptionForm.vue";
import Toast from "@/components/ui/Toast.vue";
import type { Subscription } from "@/schemas/appData";
import { AlertTriangle } from "lucide-vue-next";

const store = useAppStore();
const { t } = useI18n();
const { fmt: fmtMain, getCurrencyRate } = useCurrencyFormat();
const { fmtMonthYear, fmtDateMedium } = useLocaleFormat();
const { toastMsg, toastType, showToast, toast, closeToast } = useToast();

// --- Navigation ---
const now = new Date();
const currentMonth = ref(now.getMonth());
const currentYear = ref(now.getFullYear());

const currentMonthLabel = computed(() =>
  fmtMonthYear(new Date(currentYear.value, currentMonth.value, 1)),
);

const isCurrentMonth = computed(() =>
  currentMonth.value === now.getMonth() && currentYear.value === now.getFullYear()
);

function prevMonth() {
  if (isCurrentMonth.value) return;
  if (currentMonth.value === 0) {
    currentMonth.value = 11;
    currentYear.value--;
  } else {
    currentMonth.value--;
  }
  const cur = new Date(now.getFullYear(), now.getMonth(), 1);
  const sel = new Date(currentYear.value, currentMonth.value, 1);
  if (sel < cur) {
    currentMonth.value = now.getMonth();
    currentYear.value = now.getFullYear();
  }
}

function nextMonth() {
  if (currentMonth.value === 11) {
    currentMonth.value = 0;
    currentYear.value++;
  } else {
    currentMonth.value++;
  }
}

function resetMonth() {
  currentMonth.value = now.getMonth();
  currentYear.value = now.getFullYear();
}

// --- Grid ---
const calendarGrid = computed<CalendarCell[]>(() => {
  const year = currentYear.value;
  const month = currentMonth.value;
  const daysInMonth = new Date(year, month + 1, 0).getDate();
  const firstDayOfWeek = (new Date(year, month, 1).getDay() + 6) % 7;

  const today = new Date();
  const todayDay = today.getDate();
  const todayMonth = today.getMonth();
  const todayYear = today.getFullYear();

  const activeSubs = store.state.subscriptions.filter((s) => !s.inactive);
  const subsByDay: Record<number, CalendarCell["subs"]> = {};

  for (const sub of activeSubs) {
    const days = getPaymentDatesInMonth(sub, year, month);
    for (const d of days) {
      if (!subsByDay[d]) subsByDay[d] = [];
      subsByDay[d].push({ id: sub.id, name: sub.name, price: sub.price, currencyId: sub.currencyId });
    }
  }

  // Expenses for this month
  const expByDay: Record<number, { id: string; name: string; amount: number; currencyId: string }[]> = {};
  const monthStr = `${year}-${String(month + 1).padStart(2, "0")}`;
  for (const exp of store.state.expenses) {
    if (exp.date.startsWith(monthStr)) {
      const d = parseInt(exp.date.split("-")[2], 10);
      if (!expByDay[d]) expByDay[d] = [];
      expByDay[d].push({ id: exp.id, name: exp.name, amount: exp.amount, currencyId: exp.currencyId });
    }
  }

  const cells: CalendarCell[] = [];
  for (let i = 0; i < firstDayOfWeek; i++) {
    cells.push({ day: 0, isEmpty: true, isToday: false, subs: [], expenses: [] });
  }
  for (let d = 1; d <= daysInMonth; d++) {
    cells.push({
      day: d,
      isEmpty: false,
      isToday: d === todayDay && month === todayMonth && year === todayYear,
      subs: subsByDay[d] || [],
      expenses: expByDay[d] || [],
    });
  }
  while (cells.length % 7 !== 0) {
    cells.push({ day: 0, isEmpty: true, isToday: false, subs: [], expenses: [] });
  }
  return cells;
});

// --- Stats ---
const monthStats = computed(() => {
  let totalCost = 0;
  let totalDue = 0;
  let count = 0;
  const today = new Date();

  for (const cell of calendarGrid.value) {
    if (cell.isEmpty) continue;
    for (const sub of cell.subs) {
      const converted = convertPrice(sub.price, getCurrencyRate(sub.currencyId));
      totalCost += converted;
      count++;
      const cellDate = new Date(currentYear.value, currentMonth.value, cell.day);
      if (cellDate > today) {
        totalDue += converted;
      }
    }
    for (const exp of (cell.expenses || [])) {
      const converted = convertPrice(exp.amount, getCurrencyRate(exp.currencyId));
      totalCost += converted;
      count++;
    }
  }
  return { totalCost, totalDue, count };
});

// --- Day detail modal ---
const selectedDay = ref<CalendarCell | null>(null);

function openDayDetail(cell: CalendarCell) {
  if (cell.subs.length === 0 && (!cell.expenses || cell.expenses.length === 0)) return;
  selectedDay.value = cell;
}

const dayModalTitle = computed(() => {
  if (!selectedDay.value) return "";
  return fmtDateMedium(new Date(currentYear.value, currentMonth.value, selectedDay.value.day).toISOString());
});

// --- Subscription detail ---
const detailSub = ref<Subscription | null>(null);
const showDetail = ref(false);

function openSubDetail(subId: string) {
  const sub = store.state.subscriptions.find((s) => s.id === subId);
  if (sub) {
    selectedDay.value = null;
    detailSub.value = sub;
    showDetail.value = true;
  }
}

function closeDetail() {
  showDetail.value = false;
  detailSub.value = null;
}

// --- Edit form ---
const showForm = ref(false);
const editingSub = ref<Subscription | null>(null);

function handleEdit(sub: Subscription) {
  closeDetail();
  editingSub.value = sub;
  showForm.value = true;
}

function handleClone(id: string) {
  closeDetail();
  store.cloneSubscription(id);
  toast(t("subscription_added"));
}

function handleRenew(id: string) {
  closeDetail();
  store.renewSubscription(id);
  toast(t("success"));
}

// --- Delete ---
const deleteConfirmId = ref<string | null>(null);

function handleDelete(id: string) {
  closeDetail();
  deleteConfirmId.value = id;
}

function confirmDelete() {
  if (deleteConfirmId.value) {
    store.deleteSubscription(deleteConfirmId.value);
    toast(t("subscription_deleted"));
  }
  deleteConfirmId.value = null;
}

function cancelDelete() {
  deleteConfirmId.value = null;
}

function handleOpenUrl(url: string) {
  const fullUrl = url.startsWith("http") ? url : `https://${url}`;
  openUrl(fullUrl).catch(console.warn);
}

function onSaved() {
  toast(editingSub.value ? t("subscription_updated") : t("subscription_added"));
}
</script>

<template>
  <div class="max-w-5xl">
    <CalendarHeader
      :monthName="currentMonthLabel"
      :year="currentYear"
      :isCurrentMonth="isCurrentMonth"
      @prevMonth="prevMonth"
      @nextMonth="nextMonth"
      @resetMonth="resetMonth"
    />

    <CalendarGrid
      :cells="calendarGrid"
      @selectDay="openDayDetail"
    />

    <CalendarMonthStats
      :count="monthStats.count"
      :totalCost="fmtMain(monthStats.totalCost)"
      :totalDue="fmtMain(monthStats.totalDue)"
    />

    <!-- Day detail modal -->
    <CalendarDayModal
      :show="!!selectedDay"
      :title="dayModalTitle"
      :subs="selectedDay?.subs || []"
      :expenses="selectedDay?.expenses || []"
      @close="selectedDay = null"
      @selectSub="openSubDetail"
    />

    <!-- Subscription detail -->
    <SubscriptionDetail
      :show="showDetail"
      :subscription="detailSub"
      @close="closeDetail"
      @edit="handleEdit"
      @clone="handleClone"
      @renew="handleRenew"
      @delete="handleDelete"
      @openUrl="handleOpenUrl"
      @toggleFavorite="(id: string) => { store.toggleFavorite(id); if (detailSub && detailSub.id === id) detailSub = { ...store.state.subscriptions.find((s) => s.id === id)! }; }"
      @recordPayment="(id: string) => { store.recordPayment(id); if (detailSub && detailSub.id === id) detailSub = { ...store.state.subscriptions.find((s) => s.id === id)! }; }"
    />

    <!-- Edit / Add form -->
    <SubscriptionForm
      :show="showForm"
      :edit-subscription="editingSub"
      @close="showForm = false"
      @saved="onSaved"
    />

    <!-- Delete confirmation modal -->
    <Teleport to="body">
      <Transition
        enter-active-class="transition ease-out duration-200"
        enter-from-class="opacity-0"
        enter-to-class="opacity-100"
        leave-active-class="transition ease-in duration-150"
        leave-from-class="opacity-100"
        leave-to-class="opacity-0"
      >
        <div v-if="deleteConfirmId" class="fixed inset-0 z-50 flex items-center justify-center p-4">
          <div class="absolute inset-0 bg-black/50" @click="cancelDelete" />
          <div class="relative bg-[var(--color-surface)] rounded-xl shadow-2xl w-full max-w-sm p-6">
            <div class="flex items-center gap-3 mb-4">
              <div class="w-10 h-10 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center">
                <AlertTriangle :size="20" class="text-red-500" />
              </div>
              <h3 class="text-lg font-semibold text-[var(--color-text-primary)]">{{ t('delete') }}</h3>
            </div>
            <p class="text-sm text-[var(--color-text-secondary)] mb-6">{{ t('confirm_delete_subscription') }}</p>
            <div class="flex justify-end gap-3">
              <button @click="cancelDelete" class="px-4 py-2 rounded-lg border border-[var(--color-border)] text-sm font-medium text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)]">{{ t('cancel') }}</button>
              <button @click="confirmDelete" class="px-4 py-2 rounded-lg bg-red-600 text-white text-sm font-medium hover:bg-red-700">{{ t('delete') }}</button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <Toast :show="showToast" :message="toastMsg" :type="toastType" @close="closeToast" />
  </div>
</template>
