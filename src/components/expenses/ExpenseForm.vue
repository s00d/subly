<script setup lang="ts">
import { localYmd } from "@/composables/useLocaleFormat";
import { ref, watch, computed, nextTick, useId } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import { formatErrorForToast } from "@/utils/formatError";
import type { Expense, Settings, Currency, PaymentMethod, HouseholdMember, Category, Tag } from "@/schemas/appData";
import { expenseToIsoDate } from "@/schemas/appData";
import { useClipboard } from "@/composables/useClipboard";
import Modal from "@/components/ui/Modal.vue";
import AppInput from "@/components/ui/AppInput.vue";
import AppDatePicker from "@/components/ui/AppDatePicker.vue";
import AppTextarea from "@/components/ui/AppTextarea.vue";
import AppSelect from "@/components/ui/AppSelect.vue";
import TagInput from "@/components/ui/TagInput.vue";
import type { SelectOption } from "@/components/ui/AppSelect.vue";
import { Globe, Loader2 } from "@lucide/vue";
import { resolveFaviconFromInputUrl } from "@/services/logoClient";
import { upsertExpense } from "@/services/expensesClient";
import { expenseFormFieldsSchema, coerceExpenseFormForValidation } from "@/schemas/zod/expenseForm";
import { useZodLiveForm } from "@/composables/useZodLiveForm";
import type { ZodFieldMeta } from "@/composables/useZodErrors";
import { moneyFromNumber, parseMoney, toDisplayNumber } from "@/lib/money";
import { button } from "@/lib/tv";

/**
 * Subset of `Expense` fields the form can pre-fill. Used by the AI quick-add
 * flow and the existing "duplicate expense" / "import preview" paths.
 */
export interface ExpensePrefill {
  name?: string;
  amount?: number;
  currencyId?: string;
  /** UI date in `YYYY-MM-DD`. */
  date?: string;
  categoryId?: string;
  paymentMethodId?: string;
  payerUserId?: string;
  tags?: string[];
  notes?: string;
  url?: string;
}

const props = defineProps<{
  show: boolean;
  editExpense?: Expense | null;
  prefill?: ExpensePrefill | null;
  lookupData: {
    settings: Settings;
    currencies: Currency[];
    paymentMethods: PaymentMethod[];
    household: HouseholdMember[];
    categories: Category[];
    tags: Tag[];
    expensesCount: number;
  };
}>();

const emit = defineEmits<{
  close: [];
  saved: [];
}>();

const settings = ref<Settings | null>(null);
const currencies = ref<Currency[]>([]);
const paymentMethods = ref<PaymentMethod[]>([]);
const household = ref<HouseholdMember[]>([]);
const categories = ref<Category[]>([]);
const tags = ref<Tag[]>([]);
const expensesCount = ref(0);
const { t } = useI18n();
const { toast } = useToast();
const { copyToClipboard } = useClipboard();

/** UI-only `date` is `YYYY-MM-DD` for AppDatePicker; persisted moment is `createdAt` (RFC3339). */
type ExpenseFormModel = Partial<Expense> & { date?: string };

const form = ref<ExpenseFormModel>({});

const expenseFieldMeta: ZodFieldMeta = {
  name: "string",
  amount: "number",
  currencyId: "string",
  date: "date",
};

function buildExpenseValidationInput(): Record<string, unknown> {
  return coerceExpenseFormForValidation(form.value as Record<string, unknown>);
}

const {
  errors: expenseErrors,
  markDirty: markExpenseFieldDirty,
  clearDirty: clearExpenseZodDirty,
  validateStrict: validateExpenseStrict,
  watchSource: watchExpenseZod,
} = useZodLiveForm({
  getValues: buildExpenseValidationInput,
  schema: expenseFormFieldsSchema,
  fieldMeta: expenseFieldMeta,
  t,
  guardEmptyRequiredStrings: true,
  guardedStringFields: ["name"],
});

watchExpenseZod(form);

const iconPreview = ref("");
const isResolvingIcon = ref(false);
const isSaving = ref(false);
/** Footer submit button targets the form via `form=` so Enter-to-save keeps working. */
const formId = useId();
/** Wrapper around the Amount field; AppInput has no autofocus prop, so we focus its inner input. */
const amountWrap = ref<HTMLElement | null>(null);

function focusAmount() {
  void nextTick(() => {
    const input = amountWrap.value?.querySelector("input");
    input?.focus();
    input?.select();
  });
}

function nextExpenseNumber(): number {
  return expensesCount.value + 1;
}

function resetForm() {
  form.value = {
    name: `${t("expenses")} #${nextExpenseNumber()}`,
    amount: "0",
    currencyId: settings.value?.mainCurrencyId || "cur-2",
    date: localYmd(),
    categoryId: settings.value?.defaultCategoryId || "cat-1",
    paymentMethodId: settings.value?.defaultPaymentMethodId || paymentMethods.value.find((p) => p.enabled)?.id || "",
    payerUserId: household.value[0]?.id || "",
    tags: [],
    notes: "",
    url: "",
  };
}

watch(() => props.show, (val) => {
  if (val) {
    loadLookupData();
    clearExpenseZodDirty();
    if (props.editExpense) {
      form.value = { ...props.editExpense, date: expenseToIsoDate(props.editExpense) };
      iconPreview.value = "";
    } else {
      resetForm();
      if (props.prefill) {
        const p = props.prefill;
        if (p.name) form.value.name = p.name;
        if (p.amount != null && Number.isFinite(p.amount)) form.value.amount = moneyFromNumber(p.amount);
        if (p.currencyId) form.value.currencyId = p.currencyId;
        if (p.date) form.value.date = p.date;
        if (p.categoryId) form.value.categoryId = p.categoryId;
        if (p.paymentMethodId) form.value.paymentMethodId = p.paymentMethodId;
        if (p.payerUserId) form.value.payerUserId = p.payerUserId;
        if (p.tags && p.tags.length) form.value.tags = [...p.tags];
        if (p.notes) form.value.notes = p.notes;
        if (p.url) form.value.url = p.url;
      }
    }
    focusAmount();
  }
});

const isEditing = computed(() => !!props.editExpense);

const currencyOptions = computed<SelectOption[]>(() =>
  currencies.value.map((c) => ({ label: `${c.symbol} ${c.name} (${c.code})`, value: c.id }))
);

const categoryOptions = computed<SelectOption[]>(() =>
  [...categories.value].sort((a, b) => a.sortOrder - b.sortOrder).map((c) => ({ label: c.name, value: c.id, icon: c.icon || undefined }))
);

const paymentOptions = computed<SelectOption[]>(() => {
  const opts: SelectOption[] = [{ label: t("none"), value: "" }];
  paymentMethods.value.filter((p) => p.enabled).forEach((p) =>
    opts.push({ label: p.name, value: p.id, icon: p.icon })
  );
  return opts;
});

const payerOptions = computed<SelectOption[]>(() =>
  household.value.map((h) => ({ label: h.name, value: h.id }))
);

async function applyDomainIcon() {
  if (isResolvingIcon.value) return;
  isResolvingIcon.value = true;
  try {
    const faviconUrl = await resolveFaviconFromInputUrl(form.value.url || "");
    if (!faviconUrl) {
      toast(t("favicon_load_failed"), "error");
      return;
    }
    iconPreview.value = faviconUrl;
    await copyToClipboard(faviconUrl);
    toast(t("copied_to_clipboard"));
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  } finally {
    isResolvingIcon.value = false;
  }
}

async function handleSave(addAnother = false) {
  if (isSaving.value) return;
  if (!validateExpenseStrict()) {
    toast(t("fill_required_fields"), "error");
    return;
  }

  const base = isEditing.value && props.editExpense
    ? { ...props.editExpense, ...form.value }
    : { ...form.value, id: crypto.randomUUID(), createdAt: new Date().toISOString() };

  const iso = String(form.value.date ?? "").trim();

  // Coerce numeric fields — HTML inputs always return strings
  const rawAmount = Number(base.amount) || 0;

  const prev = String(base.createdAt ?? "").trim();
  const prevDay = prev.length >= 10 ? prev.slice(0, 10) : "";
  const createdAt =
    isEditing.value && prevDay === iso
      ? prev
      : new Date(`${iso}T12:00:00.000Z`).toISOString();

  const payload: Record<string, unknown> = {
    id: base.id,
    name: String(base.name).trim(),
    amount: moneyFromNumber(rawAmount),
    currencyId: String(base.currencyId),
    categoryId: String(base.categoryId ?? ""),
    paymentMethodId: String(base.paymentMethodId ?? ""),
    payerUserId: String(base.payerUserId ?? ""),
    tags: base.tags ?? [],
    notes: String(base.notes ?? ""),
    url: String(base.url ?? ""),
    createdAt,
    subscriptionId: String(base.subscriptionId ?? ""),
    paymentRecordId: String(base.paymentRecordId ?? ""),
  };

  isSaving.value = true;
  try {
    await upsertExpense(payload);
    toast(t("success"));
    emit("saved");
    if (addAnother) {
      // Keep the "context" of a batch entry (day, category, payment method); clear the rest.
      const keep = {
        date: form.value.date,
        categoryId: form.value.categoryId,
        paymentMethodId: form.value.paymentMethodId,
      };
      expensesCount.value += 1;
      resetForm();
      Object.assign(form.value, keep);
      iconPreview.value = "";
      clearExpenseZodDirty();
      focusAmount();
    } else {
      emit("close");
    }
  } catch (e) {
    console.error("Expense save failed:", e);
    toast(formatErrorForToast(e, t), "error");
  } finally {
    isSaving.value = false;
  }
}

function loadLookupData() {
  settings.value = props.lookupData.settings;
  currencies.value = props.lookupData.currencies;
  paymentMethods.value = props.lookupData.paymentMethods;
  household.value = props.lookupData.household;
  categories.value = props.lookupData.categories;
  tags.value = props.lookupData.tags;
  expensesCount.value = props.lookupData.expensesCount;
}
</script>

<template>
  <Modal :show="show" :title="isEditing ? t('edit_expense') : t('add_expense')" @close="emit('close')">
    <form :id="formId" @submit.prevent="handleSave()" class="space-y-4 sm:space-y-5">
      <!-- Amount + Currency (first: quick entry) -->
      <div class="flex flex-col sm:flex-row gap-3">
        <div ref="amountWrap" class="flex-1">
          <AppInput
            :label="t('expense_amount')"
            :modelValue="toDisplayNumber(parseMoney(form.amount ?? '0'))"
            @update:modelValue="(v) => form.amount = moneyFromNumber(Number(v))"
            type="number"
            step="0.01"
            min="0"
            :error="expenseErrors.amount"
            required
          />
        </div>
        <div class="w-full sm:w-48">
          <AppSelect
            :label="t('currencies')"
            :options="currencyOptions"
            :modelValue="form.currencyId || ''"
            @update:modelValue="(v) => (form.currencyId = String(v))"
            :error="expenseErrors.currencyId"
          />
        </div>
      </div>

      <!-- Name -->
      <AppInput
        :label="t('expense_name')"
        :modelValue="form.name || ''"
        @update:modelValue="(v) => { form.name = String(v); markExpenseFieldDirty('name'); }"
        :placeholder="t('expense_name')"
        :error="expenseErrors.name"
        required
      />

      <!-- Date -->
      <AppDatePicker
        :label="t('expense_date')"
        :modelValue="form.date || ''"
        @update:modelValue="(v) => form.date = v"
        :error="expenseErrors.date"
      />

      <!-- Category + Payment Method -->
      <div class="flex flex-col sm:flex-row gap-3">
        <div class="flex-1">
          <AppSelect
            :label="t('category')"
            :options="categoryOptions"
            :modelValue="form.categoryId || 'cat-1'"
            @update:modelValue="(v) => (form.categoryId = String(v))"
          />
        </div>
        <div class="flex-1">
          <AppSelect
            :label="t('payment_method')"
            :options="paymentOptions"
            :modelValue="form.paymentMethodId || ''"
            @update:modelValue="(v) => (form.paymentMethodId = String(v))"
          />
        </div>
      </div>

      <!-- Payer (if more than 1 household member) -->
      <AppSelect
        v-if="household.length > 1"
        :label="t('paid_by')"
        :options="payerOptions"
        :modelValue="form.payerUserId || ''"
        @update:modelValue="(v) => (form.payerUserId = String(v))"
      />

      <!-- URL -->
      <AppInput
        :label="t('url')"
        :modelValue="form.url || ''"
        @update:modelValue="(v) => form.url = String(v)"
        :placeholder="'https://...'"
        type="url"
      />
      <div class="flex items-center gap-2">
        <button
          type="button"
          @click="applyDomainIcon"
          :disabled="isResolvingIcon"
          :class="button({ size: 'sm' })"
        >
          <Loader2 v-if="isResolvingIcon" :size="14" class="animate-spin" />
          <Globe v-else :size="14" />
          {{ t("get_icon_from_domain") }}
        </button>
        <img
          v-if="iconPreview"
          :src="iconPreview"
          alt="Domain icon preview"
          class="w-6 h-6 rounded border border-border object-contain"
        />
      </div>

      <!-- Tags -->
      <TagInput
        :label="t('tags')"
        :modelValue="form.tags || []"
        :availableTags="tags"
        @update:modelValue="(v) => (form.tags = v)"
      />

      <!-- Notes -->
      <AppTextarea
        :label="t('notes')"
        :modelValue="form.notes || ''"
        @update:modelValue="(v) => form.notes = v"
        :placeholder="t('notes')"
        :rows="2"
      />

    </form>

    <template #footer>
      <div class="flex items-center justify-end gap-2 w-full">
        <button type="button" :class="button({ intent: 'ghost' })" @click="emit('close')">
          {{ t('cancel') }}
        </button>
        <button
          v-if="!isEditing && !prefill"
          type="button"
          :class="button({ intent: 'secondary' })"
          :disabled="isSaving"
          @click="handleSave(true)"
        >
          {{ t('save_and_add_another') }}
        </button>
        <button
          type="submit"
          :form="formId"
          :class="button({ intent: 'primary' })"
          :disabled="isSaving"
        >
          {{ t('save') }}
        </button>
      </div>
    </template>
  </Modal>
</template>
