<script setup lang="ts">
import { ref, watch, computed } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
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

const props = defineProps<{
  show: boolean;
  editExpense?: Expense | null;
  prefill?: { amount?: number; currencyId?: string; name?: string } | null;
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

function nextExpenseNumber(): number {
  return expensesCount.value + 1;
}

function localYmd(d = new Date()): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

function resetForm() {
  form.value = {
    name: `${t("expenses")} #${nextExpenseNumber()}`,
    amount: 0,
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
        if (props.prefill.amount != null) form.value.amount = props.prefill.amount;
        if (props.prefill.currencyId) form.value.currencyId = props.prefill.currencyId;
        if (props.prefill.name) form.value.name = props.prefill.name;
      }
    }
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
      toast("Could not load icon from this domain", "error");
      return;
    }
    iconPreview.value = faviconUrl;
    await copyToClipboard(faviconUrl);
    toast("Icon URL copied from domain");
  } finally {
    isResolvingIcon.value = false;
  }
}

async function handleSave() {
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
    amount: rawAmount,
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

  try {
    await upsertExpense(payload);
    toast(t("success"));
    emit("saved");
    emit("close");
  } catch (e) {
    console.error("Expense save failed:", e);
    toast(t("save_error"), "error");
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
    <form @submit.prevent="handleSave" class="space-y-4 sm:space-y-5">
      <!-- Name -->
      <AppInput
        :label="t('expense_name')"
        :modelValue="form.name || ''"
        @update:modelValue="(v) => { form.name = String(v); markExpenseFieldDirty('name'); }"
        :placeholder="t('expense_name')"
        :error="expenseErrors.name"
        required
      />

      <!-- Amount + Currency -->
      <div class="flex flex-col sm:flex-row gap-3">
        <div class="flex-1">
          <AppInput
            :label="t('expense_amount')"
            :modelValue="form.amount ?? 0"
            @update:modelValue="(v) => form.amount = Number(v)"
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
          class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-border text-xs font-medium text-text-secondary hover:bg-surface-hover transition-colors disabled:opacity-60 disabled:cursor-not-allowed"
        >
          <Loader2 v-if="isResolvingIcon" :size="14" class="animate-spin" />
          <Globe v-else :size="14" />
          Get icon from domain
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

      <!-- Actions -->
      <div class="flex justify-end gap-3 pt-2">
        <button type="button" @click="emit('close')"
          class="px-4 py-2 text-sm rounded-lg bg-surface-hover text-text-secondary hover:bg-border">
          {{ t('cancel') }}
        </button>
        <button type="submit"
          class="px-4 py-2 text-sm rounded-lg bg-primary text-white hover:opacity-90">
          {{ t('save') }}
        </button>
      </div>
    </form>
  </Modal>
</template>
