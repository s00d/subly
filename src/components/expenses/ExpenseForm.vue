<script setup lang="ts">
import { ref, watch, computed } from "vue";
import { useAppStore } from "@/stores/appStore";
import { useI18n } from "@/i18n";
import type { Expense } from "@/schemas/appData";
import Modal from "@/components/ui/Modal.vue";
import AppInput from "@/components/ui/AppInput.vue";
import AppTextarea from "@/components/ui/AppTextarea.vue";
import AppSelect from "@/components/ui/AppSelect.vue";
import TagInput from "@/components/ui/TagInput.vue";
import type { SelectOption } from "@/components/ui/AppSelect.vue";

const props = defineProps<{
  show: boolean;
  editExpense?: Expense | null;
}>();

const emit = defineEmits<{
  close: [];
  saved: [];
}>();

const store = useAppStore();
const { t } = useI18n();

const form = ref<Partial<Expense>>({});

function resetForm() {
  form.value = {
    name: "",
    amount: 0,
    currencyId: store.state.settings.mainCurrencyId,
    date: new Date().toISOString().split("T")[0],
    categoryId: store.state.settings.defaultCategoryId || "cat-1",
    paymentMethodId: store.state.settings.defaultPaymentMethodId || store.enabledPaymentMethods.value[0]?.id || "",
    payerUserId: store.state.household[0]?.id || "",
    tags: [],
    notes: "",
  };
}

watch(() => props.show, (val) => {
  if (val) {
    if (props.editExpense) {
      form.value = { ...props.editExpense };
    } else {
      resetForm();
    }
  }
});

const isEditing = computed(() => !!props.editExpense);

const currencyOptions = computed<SelectOption[]>(() =>
  store.state.currencies.map((c) => ({ label: `${c.symbol} ${c.name} (${c.code})`, value: c.id }))
);

const categoryOptions = computed<SelectOption[]>(() =>
  store.sortedCategories.value.map((c) => ({ label: c.name, value: c.id, icon: c.icon || undefined }))
);

const paymentOptions = computed<SelectOption[]>(() => {
  const opts: SelectOption[] = [{ label: t("none"), value: "" }];
  store.enabledPaymentMethods.value.forEach((p) =>
    opts.push({ label: p.name, value: p.id, icon: p.icon })
  );
  return opts;
});

const payerOptions = computed<SelectOption[]>(() =>
  store.state.household.map((h) => ({ label: h.name, value: h.id }))
);

function handleSave() {
  if (!form.value.name?.trim() || !form.value.amount) return;
  if (isEditing.value && props.editExpense) {
    store.updateExpense({ ...form.value, id: props.editExpense.id } as Partial<Expense> & { id: string });
  } else {
    store.addExpense({
      ...form.value,
      id: crypto.randomUUID(),
      name: form.value.name!,
      currencyId: form.value.currencyId!,
      date: form.value.date!,
      createdAt: new Date().toISOString(),
    });
  }
  emit("saved");
  emit("close");
}
</script>

<template>
  <Modal :show="show" :title="isEditing ? t('edit_expense') : t('add_expense')" @close="emit('close')">
    <form @submit.prevent="handleSave" class="space-y-4 sm:space-y-5">
      <!-- Name -->
      <AppInput
        :label="t('expense_name')"
        :modelValue="form.name || ''"
        @update:modelValue="(v) => form.name = String(v)"
        :placeholder="t('expense_name')"
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
            required
          />
        </div>
        <div class="w-full sm:w-48">
          <AppSelect
            :label="t('currencies')"
            :options="currencyOptions"
            :modelValue="form.currencyId || ''"
            @update:modelValue="(v) => (form.currencyId = String(v))"
          />
        </div>
      </div>

      <!-- Date -->
      <AppInput
        :label="t('expense_date')"
        :modelValue="form.date || ''"
        @update:modelValue="(v) => form.date = String(v)"
        type="date"
        required
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
        v-if="store.state.household.length > 1"
        :label="t('paid_by')"
        :options="payerOptions"
        :modelValue="form.payerUserId || ''"
        @update:modelValue="(v) => (form.payerUserId = String(v))"
      />

      <!-- Tags -->
      <TagInput
        :label="t('tags')"
        :modelValue="form.tags || []"
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
          class="px-4 py-2 text-sm rounded-lg bg-[var(--color-surface-hover)] text-[var(--color-text-secondary)] hover:bg-[var(--color-border)]">
          {{ t('cancel') }}
        </button>
        <button type="submit"
          class="px-4 py-2 text-sm rounded-lg bg-[var(--color-primary)] text-white hover:opacity-90">
          {{ t('save') }}
        </button>
      </div>
    </form>
  </Modal>
</template>
