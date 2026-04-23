<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { useSubscriptionsStore } from "@/stores/subscriptions";
import { useExpensesStore } from "@/stores/expenses";
import { useSettingsStore } from "@/stores/settings";
import { useCatalogStore } from "@/stores/catalog";
import { useI18n } from "vue-i18n";
import { useHeaderActions } from "@/composables/useHeaderActions";
import { useCurrencyFormat } from "@/composables/useCurrencyFormat";
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import { useToast } from "@/composables/useToast";
import { useScrollLock } from "@/composables/useScrollLock";
import { getPaymentDatesInMonth, convertPrice } from "@/services/calculations";
import { openUrl } from "@tauri-apps/plugin-opener";
import CalendarHeader from "@/components/calendar/CalendarHeader.vue";
import CalendarGrid from "@/components/calendar/CalendarGrid.vue";
import type { CalendarCell } from "@/components/calendar/CalendarGrid.vue";
import CalendarMonthStats from "@/components/calendar/CalendarMonthStats.vue";
import CalendarDayModal from "@/components/calendar/CalendarDayModal.vue";
import SubscriptionDetail from "@/components/subscriptions/SubscriptionDetail.vue";
import SubscriptionForm from "@/components/subscriptions/SubscriptionForm.vue";
import ExpenseDetail from "@/components/expenses/ExpenseDetail.vue";
import ExpenseForm from "@/components/expenses/ExpenseForm.vue";
import Toast from "@/components/ui/Toast.vue";
import type { Subscription, Expense } from "@/schemas/appData";
import { dbGetExpensesForMonth } from "@/services/database";
import { AlertTriangle, LayoutList, LayoutGrid, Rows3, CreditCard, Wallet } from "lucide-vue-next";

const subsStore = useSubscriptionsStore();
const expsStore = useExpensesStore();
const settingsStore = useSettingsStore();
const catalogStore = useCatalogStore();
const { t } = useI18n();
const { setActions, clearActions } = useHeaderActions();
const { fmt: fmtMain, getCurrencyRate } = useCurrencyFormat();
const { fmtMonthYear, fmtDateMedium } = useLocaleFormat();
const { toastMsg, toastType, showToast, toast, closeToast } = useToast();
const pageLogPrefix = "[CalendarPage]";

function logPageError(scope: string, error: unknown, extra?: Record<string, unknown>) {
  console.error(`${pageLogPrefix} ${scope}`, {
    error,
    ...extra,
  });
}

onMounted(() => {
  updateHeaderActions();
});
onUnmounted(() => {
  clearActions();
});

// --- Navigation ---
const now = new Date();
const currentMonth = ref(now.getMonth());
const currentYear = ref(now.getFullYear());
const compactWeekIndex = ref(0);

const currentMonthLabel = computed(() => {
  if (isCompactView.value) {
    const week = compactWeekCells.value;
    if (week.length === 0) return fmtMonthYear(new Date(currentYear.value, currentMonth.value, 1));
    const firstDay = week.find((cell) => !cell.isEmpty)?.day;
    const lastDay = [...week].reverse().find((cell) => !cell.isEmpty)?.day;
    if (!firstDay || !lastDay) return fmtMonthYear(new Date(currentYear.value, currentMonth.value, 1));
    const start = fmtDateMedium(new Date(currentYear.value, currentMonth.value, firstDay).toISOString());
    const end = fmtDateMedium(new Date(currentYear.value, currentMonth.value, lastDay).toISOString());
    return `${start} - ${end}`;
  }
  return fmtMonthYear(new Date(currentYear.value, currentMonth.value, 1));
});
const viewMode = computed(() => settingsStore.settings.calendarViewMode || "default");
const isCompactView = computed(() => viewMode.value === "compact");

const isCurrentMonth = computed(() =>
  isCompactView.value
    ? currentWeekMonthIndex.value === compactWeekIndex.value
      && currentMonth.value === now.getMonth()
      && currentYear.value === now.getFullYear()
    : currentMonth.value === now.getMonth() && currentYear.value === now.getFullYear()
);

function prevMonth() {
  if (isCompactView.value) {
    const weeks = monthWeeks.value;
    if (weeks.length === 0) return;
    if (currentMonth.value === now.getMonth() && currentYear.value === now.getFullYear() && compactWeekIndex.value <= currentWeekMonthIndex.value) {
      return;
    }
    if (compactWeekIndex.value > 0) {
      compactWeekIndex.value -= 1;
      return;
    }
    if (currentMonth.value === 0) {
      currentMonth.value = 11;
      currentYear.value -= 1;
    } else {
      currentMonth.value -= 1;
    }
    compactWeekIndex.value = Math.max(monthWeeks.value.length - 1, 0);
    return;
  }

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
  if (isCompactView.value) {
    if (compactWeekIndex.value < monthWeeks.value.length - 1) {
      compactWeekIndex.value += 1;
      return;
    }
    if (currentMonth.value === 11) {
      currentMonth.value = 0;
      currentYear.value++;
    } else {
      currentMonth.value++;
    }
    compactWeekIndex.value = 0;
    return;
  }

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
  compactWeekIndex.value = currentWeekMonthIndex.value;
}

function setViewMode(mode: "default" | "compact" | "expanded") {
  settingsStore.updateSettings({ calendarViewMode: mode });
}

function updateHeaderActions() {
  const viewIcon = viewMode.value === "compact" ? Rows3 : viewMode.value === "expanded" ? LayoutGrid : LayoutList;
  const nextViewMode = viewMode.value === "compact" ? "default" : viewMode.value === "default" ? "expanded" : "compact";
  const currentViewTitle = viewMode.value === "compact" ? t("view_compact") : viewMode.value === "expanded" ? t("view_expanded") : t("view_default");
  const nextViewTitle = nextViewMode === "compact" ? t("view_compact") : nextViewMode === "expanded" ? t("view_expanded") : t("view_default");

  setActions([
    { id: "cycle-calendar-view", icon: viewIcon, title: `${currentViewTitle} → ${nextViewTitle}`, onClick: () => setViewMode(nextViewMode), style: "accent" },
  ]);
}

watch(viewMode, updateHeaderActions);

// --- Grid ---
const monthExpenses = ref<Expense[]>([]);

async function loadMonthExpenses() {
  const monthStr = `${currentYear.value}-${String(currentMonth.value + 1).padStart(2, "0")}`;
  try {
    monthExpenses.value = await dbGetExpensesForMonth(monthStr);
  } catch (e) {
    logPageError("loadMonthExpenses failed", e, { month: monthStr });
  }
}

watch([currentYear, currentMonth], loadMonthExpenses);
onMounted(() => loadMonthExpenses());

const calendarGrid = computed<CalendarCell[]>(() => {
  const year = currentYear.value;
  const month = currentMonth.value;
  const daysInMonth = new Date(year, month + 1, 0).getDate();
  const firstDayOfWeek = (new Date(year, month, 1).getDay() + 6) % 7;

  const today = new Date();
  const todayDay = today.getDate();
  const todayMonth = today.getMonth();
  const todayYear = today.getFullYear();

  const activeSubs = subsStore.subscriptions.filter((s) => !s.inactive);
  const subsByDay: Record<number, CalendarCell["subs"]> = {};

  for (const sub of activeSubs) {
    const days = getPaymentDatesInMonth(sub, year, month);
    for (const d of days) {
      if (!subsByDay[d]) subsByDay[d] = [];
      subsByDay[d].push({ id: sub.id, name: sub.name, price: sub.price, currencyId: sub.currencyId, logo: sub.logo });
    }
  }

  const expByDay: Record<number, { id: string; name: string; amount: number; currencyId: string; icon?: string }[]> = {};
  for (const exp of monthExpenses.value) {
    const d = parseInt(exp.date.split("-")[2], 10);
    if (!expByDay[d]) expByDay[d] = [];
    const categoryIcon = catalogStore.categories.find((c) => c.id === exp.categoryId)?.icon || "";
    const paymentIcon = catalogStore.paymentMethods.find((p) => p.id === exp.paymentMethodId)?.icon || "";
    expByDay[d].push({ id: exp.id, name: exp.name, amount: exp.amount, currencyId: exp.currencyId, icon: categoryIcon || paymentIcon });
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
const monthWeeks = computed<CalendarCell[][]>(() => {
  const weeks: CalendarCell[][] = [];
  for (let i = 0; i < calendarGrid.value.length; i += 7) {
    weeks.push(calendarGrid.value.slice(i, i + 7));
  }
  return weeks;
});
const currentWeekMonthIndex = computed(() => {
  const todayDay = now.getDate();
  if (currentMonth.value !== now.getMonth() || currentYear.value !== now.getFullYear()) return 0;
  return monthWeeks.value.findIndex((week) => week.some((cell) => !cell.isEmpty && cell.day === todayDay));
});
const compactWeekCells = computed<CalendarCell[]>(() => {
  const weeks = monthWeeks.value;
  if (weeks.length === 0) return [];
  const safeIndex = Math.min(Math.max(compactWeekIndex.value, 0), weeks.length - 1);
  return weeks[safeIndex];
});
const weekDayLabels = computed(() => [
  t("mon"),
  t("tue"),
  t("wed"),
  t("thu"),
  t("fri"),
  t("sat"),
  t("sun"),
]);
watch([currentYear, currentMonth], () => {
  compactWeekIndex.value = currentMonth.value === now.getMonth() && currentYear.value === now.getFullYear()
    ? Math.max(currentWeekMonthIndex.value, 0)
    : 0;
}, { immediate: true });

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
const expandedDays = computed(() => calendarGrid.value.filter(
  (cell) => !cell.isEmpty && (cell.subs.length > 0 || (cell.expenses?.length || 0) > 0),
));

function expandedDayTitle(day: number): string {
  return fmtDateMedium(new Date(currentYear.value, currentMonth.value, day).toISOString());
}

// --- Subscription detail ---
const detailSub = ref<Subscription | null>(null);
const showDetail = ref(false);

function openSubDetail(subId: string) {
  const sub = subsStore.subscriptions.find((s) => s.id === subId);
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
  subsStore.cloneSubscription(id);
  toast(t("subscription_added"));
}

function handleRenew(id: string) {
  closeDetail();
  subsStore.renewSubscription(id);
  toast(t("payment_recorded"));
}

// --- Delete ---
const deleteConfirmId = ref<string | null>(null);

function handleDelete(id: string) {
  closeDetail();
  deleteConfirmId.value = id;
}

function confirmDelete() {
  if (deleteConfirmId.value) {
    subsStore.deleteSubscription(deleteConfirmId.value);
    toast(t("subscription_deleted"));
  }
  deleteConfirmId.value = null;
}

function cancelDelete() {
  deleteConfirmId.value = null;
}

function handleOpenUrl(url: string) {
  const fullUrl = url.startsWith("http") ? url : `https://${url}`;
  openUrl(fullUrl).catch((e) => {
    logPageError("handleOpenUrl failed", e, { url: fullUrl });
  });
}

function onSaved() {
  toast(editingSub.value ? t("subscription_updated") : t("subscription_added"));
}

// --- Expense detail ---
const detailExp = ref<Expense | null>(null);
const showExpDetail = ref(false);

function openExpDetail(expId: string) {
  const exp = monthExpenses.value.find((e) => e.id === expId);
  if (exp) {
    selectedDay.value = null;
    detailExp.value = exp;
    showExpDetail.value = true;
  }
}

function closeExpDetail() {
  showExpDetail.value = false;
  detailExp.value = null;
}

// --- Expense edit form ---
const showExpForm = ref(false);
const editingExp = ref<Expense | null>(null);

function handleExpEdit(exp: Expense) {
  closeExpDetail();
  editingExp.value = exp;
  showExpForm.value = true;
}

async function onExpSaved() {
  try {
    toast(editingExp.value ? t("expense_updated") : t("expense_added"));
    await loadMonthExpenses();
  } catch (e) {
    logPageError("onExpSaved failed", e);
    toast(t("error"), "error");
  }
}

// --- Expense delete ---
const deleteExpConfirmId = ref<string | null>(null);
const hasBlockingOverlay = computed(() => Boolean(deleteConfirmId.value) || Boolean(deleteExpConfirmId.value));
useScrollLock(hasBlockingOverlay);

function handleExpDelete(id: string) {
  closeExpDetail();
  deleteExpConfirmId.value = id;
}

async function confirmExpDelete() {
  try {
    if (deleteExpConfirmId.value) {
      await expsStore.deleteExpense(deleteExpConfirmId.value);
      toast(t("expense_deleted"));
      await loadMonthExpenses();
    }
  } catch (e) {
    logPageError("confirmExpDelete failed", e, { expenseId: deleteExpConfirmId.value });
    toast(t("error"), "error");
  } finally {
    deleteExpConfirmId.value = null;
  }
}

function cancelExpDelete() {
  deleteExpConfirmId.value = null;
}

async function handleExpOpenUrl(url: string) {
  const fullUrl = url.startsWith("http") ? url : `https://${url}`;
  openUrl(fullUrl).catch((e) => {
    logPageError("handleExpOpenUrl failed", e, { url: fullUrl });
  });
}
</script>

<template>
  <div class="max-w-5xl">
    <CalendarHeader
      :monthName="currentMonthLabel"
      :year="currentYear"
      :isCurrentMonth="isCurrentMonth"
      :compact="isCompactView"
      @prevMonth="prevMonth"
      @nextMonth="nextMonth"
      @resetMonth="resetMonth"
    />

    <div v-if="viewMode === 'expanded'" class="grid grid-cols-1 md:grid-cols-2 gap-3">
      <div
        v-for="cell in expandedDays"
        :key="`expanded-day-${cell.day}`"
        class="bg-surface rounded-xl border border-border p-3 sm:p-4 cursor-pointer hover:bg-surface-hover transition-colors"
        @click="openDayDetail(cell)"
      >
        <div class="flex items-center justify-between mb-2">
          <h3 class="text-sm font-semibold text-text-primary">{{ expandedDayTitle(cell.day) }}</h3>
          <span class="text-[11px] text-text-muted">{{ cell.subs.length + (cell.expenses?.length || 0) }}</span>
        </div>
        <div v-if="cell.subs.length" class="space-y-1.5 mb-2">
          <button
            v-for="sub in cell.subs"
            :key="sub.id"
            @click.stop="openSubDetail(sub.id)"
            class="w-full flex items-center justify-between gap-2 px-2 py-1.5 rounded-lg bg-blue-600 text-white text-xs font-medium hover:bg-blue-700 transition-colors"
          >
            <span class="inline-flex items-center gap-1.5 min-w-0">
              <CreditCard :size="12" class="shrink-0" />
              <span class="truncate">{{ sub.name }}</span>
            </span>
            <span class="shrink-0 opacity-90">{{ fmtMain(sub.price, sub.currencyId) }}</span>
          </button>
        </div>
        <div v-if="cell.expenses?.length" class="space-y-1.5">
          <button
            v-for="exp in cell.expenses"
            :key="exp.id"
            @click.stop="openExpDetail(exp.id)"
            class="w-full flex items-center justify-between gap-2 px-2 py-1.5 rounded-lg border border-border bg-surface text-text-primary text-xs font-medium hover:bg-surface-hover transition-colors"
          >
            <span class="inline-flex items-center gap-1.5 min-w-0">
              <Wallet :size="12" class="shrink-0 text-amber-500" />
              <span class="truncate">{{ exp.name }}</span>
            </span>
            <span class="shrink-0 text-amber-500">{{ fmtMain(exp.amount, exp.currencyId) }}</span>
          </button>
        </div>
      </div>
      <div v-if="expandedDays.length === 0" class="md:col-span-2 bg-surface rounded-xl border border-border p-6 text-center text-sm text-text-muted">
        {{ t("no_data") }}
      </div>
    </div>
    <div v-else-if="isCompactView" class="space-y-2">
      <button
        v-for="(cell, idx) in compactWeekCells"
        :key="`compact-week-${idx}-${cell.day}`"
        class="w-full text-left bg-surface rounded-xl border border-border px-3 py-2.5 transition-colors"
        :class="[
          cell.isToday ? 'border-primary/40 bg-primary-light/30' : 'hover:bg-surface-hover',
          !cell.isEmpty && (cell.subs.length > 0 || (cell.expenses?.length || 0) > 0) ? 'cursor-pointer' : 'cursor-default',
        ]"
        @click="!cell.isEmpty && (cell.subs.length > 0 || (cell.expenses?.length || 0) > 0) && openDayDetail(cell)"
      >
        <div class="flex items-center justify-between gap-2">
          <div class="flex items-center gap-2 min-w-0">
            <span class="text-xs text-text-muted uppercase shrink-0">{{ weekDayLabels[idx] }}</span>
            <span v-if="!cell.isEmpty" class="text-sm font-semibold text-text-primary">{{ cell.day }}</span>
            <span v-else class="text-sm text-text-muted">-</span>
          </div>
          <div class="text-[11px] text-text-muted shrink-0">
            {{ cell.subs.length }} / {{ cell.expenses?.length || 0 }}
          </div>
        </div>
        <div v-if="!cell.isEmpty && (cell.subs.length > 0 || (cell.expenses?.length || 0) > 0)" class="mt-1.5 space-y-1">
          <div
            v-for="sub in cell.subs.slice(0, 2)"
            :key="`compact-sub-${sub.id}`"
            class="flex items-center gap-1.5 px-2 py-1 rounded bg-blue-600 text-white text-[11px] font-medium"
          >
            <CreditCard :size="11" class="shrink-0" />
            <span class="truncate">{{ sub.name }}</span>
            <span class="ml-auto shrink-0 opacity-90">{{ fmtMain(sub.price, sub.currencyId) }}</span>
          </div>
          <div
            v-for="exp in (cell.expenses || []).slice(0, 2)"
            :key="`compact-exp-${exp.id}`"
            class="flex items-center gap-1.5 px-2 py-1 rounded border border-border bg-surface text-text-primary text-[11px] font-medium"
          >
            <Wallet :size="11" class="shrink-0 text-amber-500" />
            <span class="truncate">{{ exp.name }}</span>
            <span class="ml-auto shrink-0 text-amber-500">{{ fmtMain(exp.amount, exp.currencyId) }}</span>
          </div>
          <div v-if="cell.subs.length + (cell.expenses?.length || 0) > 4" class="text-[10px] text-text-muted pl-1">
            +{{ cell.subs.length + (cell.expenses?.length || 0) - 4 }}
          </div>
        </div>
      </button>
    </div>
    <CalendarGrid
      v-else
      :cells="calendarGrid"
      @selectDay="openDayDetail"
    />

    <CalendarMonthStats
      :count="monthStats.count"
      :totalCost="fmtMain(monthStats.totalCost)"
      :totalDue="fmtMain(monthStats.totalDue)"
      :compact="isCompactView"
    />

    <!-- Day detail modal -->
    <CalendarDayModal
      :show="!!selectedDay"
      :title="dayModalTitle"
      :subs="selectedDay?.subs || []"
      :expenses="selectedDay?.expenses || []"
      @close="selectedDay = null"
      @selectSub="openSubDetail"
      @selectExp="openExpDetail"
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
      @toggleFavorite="(id: string) => { subsStore.toggleFavorite(id); if (detailSub && detailSub.id === id) detailSub = { ...subsStore.subscriptions.find((s) => s.id === id)! }; }"
      @recordPayment="(id: string) => { subsStore.recordPayment(id); if (detailSub && detailSub.id === id) detailSub = { ...subsStore.subscriptions.find((s) => s.id === id)! }; }"
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
        <div v-if="deleteConfirmId" class="fixed inset-0 z-50 flex items-end sm:items-center justify-center sm:p-4">
          <div class="absolute inset-0 bg-black/50" @click="cancelDelete" />
          <div class="relative bg-surface w-full rounded-t-2xl sm:rounded-xl shadow-2xl sm:max-w-sm p-4 sm:p-6">
            <div class="flex items-center gap-3 mb-3 sm:mb-4">
              <div class="w-9 h-9 sm:w-10 sm:h-10 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center shrink-0">
                <AlertTriangle :size="18" class="text-red-500" />
              </div>
              <h3 class="text-base sm:text-lg font-semibold text-text-primary">{{ t('delete') }}</h3>
            </div>
            <p class="text-xs sm:text-sm text-text-secondary mb-4 sm:mb-6">{{ t('confirm_delete_subscription') }}</p>
            <div class="flex justify-end gap-2 sm:gap-3">
              <button @click="cancelDelete" class="px-3 sm:px-4 py-2 rounded-lg border border-border text-xs sm:text-sm font-medium text-text-secondary hover:bg-surface-hover">{{ t('cancel') }}</button>
              <button @click="confirmDelete" class="px-3 sm:px-4 py-2 rounded-lg bg-red-600 text-white text-xs sm:text-sm font-medium hover:bg-red-700">{{ t('delete') }}</button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- Expense detail -->
    <ExpenseDetail
      :show="showExpDetail"
      :expense="detailExp"
      @close="closeExpDetail"
      @edit="handleExpEdit"
      @delete="handleExpDelete"
      @openUrl="handleExpOpenUrl"
    />

    <!-- Expense edit form -->
    <ExpenseForm
      :show="showExpForm"
      :editExpense="editingExp"
      @close="showExpForm = false"
      @saved="onExpSaved"
    />

    <!-- Expense delete confirmation -->
    <Teleport to="body">
      <Transition
        enter-active-class="transition ease-out duration-200"
        enter-from-class="opacity-0"
        enter-to-class="opacity-100"
        leave-active-class="transition ease-in duration-150"
        leave-from-class="opacity-100"
        leave-to-class="opacity-0"
      >
        <div v-if="deleteExpConfirmId" class="fixed inset-0 z-50 flex items-end sm:items-center justify-center sm:p-4">
          <div class="absolute inset-0 bg-black/50" @click="cancelExpDelete" />
          <div class="relative bg-surface w-full rounded-t-2xl sm:rounded-xl shadow-2xl sm:max-w-sm p-4 sm:p-6">
            <div class="flex items-center gap-3 mb-3 sm:mb-4">
              <div class="w-9 h-9 sm:w-10 sm:h-10 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center shrink-0">
                <AlertTriangle :size="18" class="text-red-500" />
              </div>
              <h3 class="text-base sm:text-lg font-semibold text-text-primary">{{ t('delete') }}</h3>
            </div>
            <p class="text-xs sm:text-sm text-text-secondary mb-4 sm:mb-6">{{ t('confirm_delete_expense') }}</p>
            <div class="flex justify-end gap-2 sm:gap-3">
              <button @click="cancelExpDelete" class="px-3 sm:px-4 py-2 rounded-lg border border-border text-xs sm:text-sm font-medium text-text-secondary hover:bg-surface-hover">{{ t('cancel') }}</button>
              <button @click="confirmExpDelete" class="px-3 sm:px-4 py-2 rounded-lg bg-red-600 text-white text-xs sm:text-sm font-medium hover:bg-red-700">{{ t('delete') }}</button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <Toast :show="showToast" :message="toastMsg" :type="toastType" @close="closeToast" />
  </div>
</template>
