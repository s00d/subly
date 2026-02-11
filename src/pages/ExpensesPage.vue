<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useAppStore } from "@/stores/appStore";
import { useI18n } from "@/i18n";
import { useHeaderActions } from "@/composables/useHeaderActions";
import type { Expense } from "@/schemas/appData";
import ExpenseForm from "@/components/expenses/ExpenseForm.vue";
import Modal from "@/components/ui/Modal.vue";
import AppInput from "@/components/ui/AppInput.vue";
import AppSelect from "@/components/ui/AppSelect.vue";
import IconDisplay from "@/components/ui/IconDisplay.vue";
import type { SelectOption } from "@/components/ui/AppSelect.vue";
import {
  Plus, Search, Trash2, Edit3, CheckSquare, Square, X, Hash,
  ArrowUpDown, Calendar as CalendarIcon, Wallet,
} from "lucide-vue-next";

const store = useAppStore();
const { t } = useI18n();
const { setActions } = useHeaderActions();

// ---- Form state ----
const showForm = ref(false);
const editingExpense = ref<Expense | null>(null);

function openAdd() { editingExpense.value = null; showForm.value = true; }
function openEdit(exp: Expense) { editingExpense.value = exp; showForm.value = true; }

// ---- Search & filters ----
const searchQuery = ref("");
const filterCategory = ref("");
const filterPayment = ref("");
const filterTag = ref("");
const dateFrom = ref("");
const dateTo = ref("");
const sortBy = ref<"date_desc" | "date_asc" | "amount_desc" | "amount_asc">("date_desc");

// ---- Selection for batch ----
const selectionMode = ref(false);
const selectedIds = ref<Set<string>>(new Set());
const showDeleteModal = ref(false);

function toggleSelect(id: string) {
  if (selectedIds.value.has(id)) selectedIds.value.delete(id);
  else selectedIds.value.add(id);
}

function selectAll() {
  filteredExpenses.value.forEach((e) => selectedIds.value.add(e.id));
}

function deselectAll() {
  selectedIds.value.clear();
  selectionMode.value = false;
}

function handleBatchDelete() {
  store.batchDeleteExpenses([...selectedIds.value]);
  selectedIds.value.clear();
  selectionMode.value = false;
  showDeleteModal.value = false;
}

// ---- Computed options ----
const categoryOptions = computed<SelectOption[]>(() => {
  const opts: SelectOption[] = [{ label: t("filter"), value: "" }];
  store.sortedCategories.value.forEach((c) => opts.push({ label: c.name, value: c.id, icon: c.icon || undefined }));
  return opts;
});

const paymentOptions = computed<SelectOption[]>(() => {
  const opts: SelectOption[] = [{ label: t("filter"), value: "" }];
  store.state.paymentMethods.filter((p) => p.enabled).forEach((p) => opts.push({ label: p.name, value: p.id, icon: p.icon }));
  return opts;
});

const tagOptions = computed<SelectOption[]>(() => {
  const opts: SelectOption[] = [{ label: t("filter_by_tag"), value: "" }];
  store.state.tags.forEach((tg) => opts.push({ label: tg.name, value: tg.name }));
  return opts;
});

const sortOptions = computed<SelectOption[]>(() => [
  { label: `${t("expense_date")} ↓`, value: "date_desc" },
  { label: `${t("expense_date")} ↑`, value: "date_asc" },
  { label: `${t("expense_amount")} ↓`, value: "amount_desc" },
  { label: `${t("expense_amount")} ↑`, value: "amount_asc" },
]);

// ---- Filtered & sorted ----
const filteredExpenses = computed(() => {
  let list = [...store.state.expenses];

  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase();
    list = list.filter((e) => e.name.toLowerCase().includes(q) || e.notes.toLowerCase().includes(q));
  }
  if (filterCategory.value) list = list.filter((e) => e.categoryId === filterCategory.value);
  if (filterPayment.value) list = list.filter((e) => e.paymentMethodId === filterPayment.value);
  if (filterTag.value) list = list.filter((e) => e.tags.includes(filterTag.value));
  if (dateFrom.value) list = list.filter((e) => e.date >= dateFrom.value);
  if (dateTo.value) list = list.filter((e) => e.date <= dateTo.value);

  switch (sortBy.value) {
    case "date_desc": list.sort((a, b) => b.date.localeCompare(a.date)); break;
    case "date_asc": list.sort((a, b) => a.date.localeCompare(b.date)); break;
    case "amount_desc": list.sort((a, b) => b.amount - a.amount); break;
    case "amount_asc": list.sort((a, b) => a.amount - b.amount); break;
  }
  return list;
});

// ---- Summary ----
const totalFiltered = computed(() =>
  filteredExpenses.value.reduce((s, e) => {
    const cur = store.state.currencies.find((c) => c.id === e.currencyId);
    const mainCur = store.mainCurrency.value;
    if (!cur || !mainCur) return s + e.amount;
    if (cur.id === mainCur.id) return s + e.amount;
    if (cur.rate && mainCur.rate) return s + (e.amount * mainCur.rate / cur.rate);
    return s + e.amount;
  }, 0)
);

// ---- Helpers ----
function getCurrency(id: string) { return store.state.currencies.find((c) => c.id === id); }
function getCategory(id: string) { return store.state.categories.find((c) => c.id === id); }
function getPaymentMethod(id: string) { return store.state.paymentMethods.find((p) => p.id === id); }
function formatDate(d: string) { return d; }
function formatAmount(amount: number, currencyId: string) {
  const cur = getCurrency(currencyId);
  return `${cur?.symbol || ""}${amount.toFixed(2)}`;
}

function handleDelete(id: string) {
  store.deleteExpense(id);
}

// ---- Header action ----
onMounted(() => {
  setActions([{ id: "add-expense", icon: Plus, title: t("add_expense"), onClick: openAdd }]);
});
</script>

<template>
  <div class="max-w-5xl mx-auto space-y-4 sm:space-y-5">
    <!-- Filters Row -->
    <div class="space-y-3">
      <!-- Search + sort -->
      <div class="flex items-center gap-2">
        <div class="relative flex-1">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)]" :size="16" />
          <input v-model="searchQuery" :placeholder="t('search')"
            class="w-full pl-9 pr-3 py-2 text-sm rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)] text-[var(--color-text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]/40" />
        </div>
        <button @click="selectionMode = !selectionMode"
          class="p-2 rounded-lg border border-[var(--color-border)] hover:bg-[var(--color-surface-hover)]"
          :class="selectionMode ? 'bg-[var(--color-primary)] text-white' : ''">
          <CheckSquare :size="16" />
        </button>
        <button @click="openAdd"
          class="p-2 rounded-lg bg-[var(--color-primary)] text-white hover:opacity-90 sm:hidden">
          <Plus :size="16" />
        </button>
      </div>

      <!-- Date range + filters (scrollable on mobile) -->
      <div class="flex gap-2 overflow-x-auto pb-1 scrollbar-none">
        <input v-model="dateFrom" type="date"
          class="shrink-0 px-2 py-1.5 text-xs rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)] text-[var(--color-text-primary)]" />
        <input v-model="dateTo" type="date"
          class="shrink-0 px-2 py-1.5 text-xs rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)] text-[var(--color-text-primary)]" />
        <AppSelect :options="categoryOptions" :modelValue="filterCategory" @update:modelValue="(v) => filterCategory = String(v)" class="shrink-0 w-36" size="sm" />
        <AppSelect :options="paymentOptions" :modelValue="filterPayment" @update:modelValue="(v) => filterPayment = String(v)" class="shrink-0 w-36" size="sm" />
        <AppSelect :options="tagOptions" :modelValue="filterTag" @update:modelValue="(v) => filterTag = String(v)" class="shrink-0 w-32" size="sm" />
        <AppSelect :options="sortOptions" :modelValue="sortBy" @update:modelValue="(v) => sortBy = v as any" class="shrink-0 w-36" size="sm" />
      </div>
    </div>

    <!-- Batch toolbar -->
    <div v-if="selectionMode && selectedIds.size > 0"
      class="flex items-center gap-2 p-3 rounded-lg bg-[var(--color-primary)]/10 border border-[var(--color-primary)]/30">
      <span class="text-sm font-medium text-[var(--color-primary)]">{{ selectedIds.size }} {{ t('selected_count') }}</span>
      <button @click="selectAll" class="text-xs px-2 py-1 rounded bg-[var(--color-surface)] border border-[var(--color-border)]">{{ t('select_all') }}</button>
      <button @click="deselectAll" class="text-xs px-2 py-1 rounded bg-[var(--color-surface)] border border-[var(--color-border)]">{{ t('deselect_all') }}</button>
      <div class="flex-1" />
      <button @click="showDeleteModal = true" class="flex items-center gap-1 text-xs px-3 py-1.5 rounded-lg bg-red-500 text-white hover:bg-red-600">
        <Trash2 :size="12" /> {{ t('batch_delete_expenses') }}
      </button>
    </div>

    <!-- Summary bar -->
    <div class="flex items-center justify-between p-3 rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)]">
      <div class="flex items-center gap-2 text-sm text-[var(--color-text-secondary)]">
        <Wallet :size="16" />
        <span>{{ filteredExpenses.length }} {{ t('expenses').toLowerCase() }}</span>
      </div>
      <div class="text-sm font-semibold text-[var(--color-text-primary)]">
        {{ store.mainCurrency.value?.symbol }}{{ totalFiltered.toFixed(2) }}
      </div>
    </div>

    <!-- List -->
    <div v-if="filteredExpenses.length === 0"
      class="flex flex-col items-center justify-center py-16 text-center text-[var(--color-text-muted)]">
      <Wallet :size="48" class="mb-4 opacity-30" />
      <p class="text-lg font-medium">{{ t('no_expenses_yet') }}</p>
      <p class="text-sm mt-1">{{ t('add_first_expense') }}</p>
      <button @click="openAdd"
        class="mt-4 px-4 py-2 text-sm rounded-lg bg-[var(--color-primary)] text-white hover:opacity-90">
        {{ t('add_expense') }}
      </button>
    </div>

    <div v-else class="space-y-2">
      <div
        v-for="exp in filteredExpenses"
        :key="exp.id"
        class="flex items-center gap-3 p-3 sm:p-4 rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)] hover:border-[var(--color-primary)]/40 transition-all cursor-pointer group"
        @click="selectionMode ? toggleSelect(exp.id) : openEdit(exp)"
      >
        <!-- Checkbox (selection mode) -->
        <button v-if="selectionMode" @click.stop="toggleSelect(exp.id)" class="shrink-0">
          <component :is="selectedIds.has(exp.id) ? CheckSquare : Square" :size="18"
            :class="selectedIds.has(exp.id) ? 'text-[var(--color-primary)]' : 'text-[var(--color-text-muted)]'" />
        </button>

        <!-- Category icon -->
        <div class="shrink-0 w-8 h-8 rounded-lg bg-[var(--color-primary)]/10 flex items-center justify-center">
          <IconDisplay v-if="getCategory(exp.categoryId)?.icon" :icon="getCategory(exp.categoryId)!.icon" :size="16" />
          <Wallet v-else :size="16" class="text-[var(--color-primary)]" />
        </div>

        <!-- Info -->
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2">
            <span class="text-sm font-medium text-[var(--color-text-primary)] truncate">{{ exp.name }}</span>
            <span v-for="tag in exp.tags.slice(0, 2)" :key="tag"
              class="hidden sm:inline-flex items-center gap-0.5 text-[10px] px-1.5 py-0.5 rounded-full bg-[var(--color-primary)]/10 text-[var(--color-primary)]">
              <Hash :size="8" />{{ tag }}
            </span>
          </div>
          <div class="flex items-center gap-2 mt-0.5 text-xs text-[var(--color-text-muted)]">
            <span>{{ formatDate(exp.date) }}</span>
            <span v-if="getCategory(exp.categoryId)" class="hidden sm:inline">• {{ getCategory(exp.categoryId)!.name }}</span>
            <span v-if="getPaymentMethod(exp.paymentMethodId)" class="hidden sm:inline">• {{ getPaymentMethod(exp.paymentMethodId)!.name }}</span>
          </div>
        </div>

        <!-- Amount -->
        <div class="shrink-0 text-right">
          <span class="text-sm font-semibold text-[var(--color-text-primary)]">
            {{ formatAmount(exp.amount, exp.currencyId) }}
          </span>
        </div>

        <!-- Actions -->
        <div v-if="!selectionMode" class="shrink-0 flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
          <button @click.stop="openEdit(exp)" class="p-1.5 rounded hover:bg-[var(--color-surface-hover)]">
            <Edit3 :size="14" class="text-[var(--color-text-muted)]" />
          </button>
          <button @click.stop="handleDelete(exp.id)" class="p-1.5 rounded hover:bg-red-500/10">
            <Trash2 :size="14" class="text-red-400" />
          </button>
        </div>
      </div>
    </div>

    <!-- Expense Form Modal -->
    <ExpenseForm :show="showForm" :editExpense="editingExpense" @close="showForm = false" @saved="showForm = false" />

    <!-- Batch Delete Modal -->
    <Modal :show="showDeleteModal" :title="t('confirm')" @close="showDeleteModal = false">
      <p class="text-sm text-[var(--color-text-secondary)] mb-4">
        {{ t('batch_confirm_delete_expenses').replace('{count}', String(selectedIds.size)) }}
      </p>
      <div class="flex justify-end gap-3">
        <button @click="showDeleteModal = false"
          class="px-4 py-2 text-sm rounded-lg bg-[var(--color-surface-hover)]">{{ t('cancel') }}</button>
        <button @click="handleBatchDelete"
          class="px-4 py-2 text-sm rounded-lg bg-red-500 text-white hover:bg-red-600">{{ t('delete') }}</button>
      </div>
    </Modal>
  </div>
</template>
