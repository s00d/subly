<script setup lang="ts">
import { ref, watch, computed, reactive } from "vue";
import { useSubscriptionsStore } from "@/stores/subscriptions";
import { useSettingsStore } from "@/stores/settings";
import { useCatalogStore } from "@/stores/catalog";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import type { Subscription, CycleType } from "@/schemas/appData";
import { parseSubscription, SubscriptionSchema } from "@/schemas/appData";
import { mapZodErrors, type ZodFieldMeta } from "@/composables/useZodErrors";
import Modal from "@/components/ui/Modal.vue";
import AppInput from "@/components/ui/AppInput.vue";
import AppTextarea from "@/components/ui/AppTextarea.vue";
import AppSelect from "@/components/ui/AppSelect.vue";
import AppCheckbox from "@/components/ui/AppCheckbox.vue";
import LogoPicker from "@/components/ui/LogoPicker.vue";
import TagInput from "@/components/ui/TagInput.vue";
import type { SelectOption } from "@/components/ui/AppSelect.vue";
import { Sparkles } from "lucide-vue-next";

const props = defineProps<{
  show: boolean;
  editSubscription?: Subscription | null;
}>();

const emit = defineEmits<{
  close: [];
  saved: [];
}>();

const subsStore = useSubscriptionsStore();
const settingsStore = useSettingsStore();
const catalogStore = useCatalogStore();
const { t } = useI18n();
const { toast } = useToast();

const form = ref<Partial<Subscription>>({});
const errors = reactive<Record<string, string>>({});

const fieldMeta: ZodFieldMeta = {
  name: "string",
  price: "number",
  currencyId: "string",
  nextPayment: "date",
  startDate: "date",
  frequency: "number",
  notifyDaysBefore: "number",
  cycle: "number",
};

function resetForm() {
  form.value = {
    name: "",
    logo: "",
    price: 0,
    currencyId: settingsStore.settings.mainCurrencyId,
    nextPayment: new Date().toISOString().split("T")[0],
    startDate: new Date().toISOString().split("T")[0],
    cycle: 3,
    frequency: 1,
    notes: "",
    paymentMethodId: settingsStore.settings.defaultPaymentMethodId || catalogStore.enabledPaymentMethods[0]?.id || "",
    payerUserId: catalogStore.household[0]?.id || "",
    categoryId: settingsStore.settings.defaultCategoryId || "cat-1",
    notify: true,
    notifyDaysBefore: -1,
    inactive: false,
    autoRenew: true,
    url: "",
    cancellationDate: null,
    replacementSubscriptionId: null,
    tags: [],
    favorite: false,
  };
}

watch(() => props.show, (val) => {
  if (val) {
    if (props.editSubscription) {
      form.value = { ...props.editSubscription };
    } else {
      resetForm();
    }
  }
});

const isEdit = computed(() => !!props.editSubscription);
const title = computed(() => isEdit.value ? t("edit_subscription") : t("add_subscription"));

// Select options
const currencyOptions = computed<SelectOption[]>(() =>
  catalogStore.currencies.map((c) => ({ value: c.id, label: `${c.name} (${c.code})` }))
);

const cycleOptions = computed<SelectOption[]>(() => [
  { value: 1, label: t("days") },
  { value: 2, label: t("weeks") },
  { value: 3, label: t("months") },
  { value: 4, label: t("years") },
]);

const paymentMethodOptions = computed<SelectOption[]>(() =>
  catalogStore.enabledPaymentMethods.map((pm) => ({ value: pm.id, label: pm.name, icon: pm.icon }))
);

const payerOptions = computed<SelectOption[]>(() =>
  catalogStore.household.map((m) => ({ value: m.id, label: m.name }))
);

const categoryOptions = computed<SelectOption[]>(() =>
  catalogStore.sortedCategories.map((c) => ({ value: c.id, label: c.name, icon: c.icon || undefined }))
);

function calculateNextPayment() {
  if (!form.value.startDate || !form.value.cycle || !form.value.frequency) return;
  const start = new Date(form.value.startDate);
  const now = new Date();
  let next = new Date(start);

  while (next < now) {
    switch (form.value.cycle) {
      case 1: next.setDate(next.getDate() + form.value.frequency); break;
      case 2: next.setDate(next.getDate() + 7 * form.value.frequency); break;
      case 3: next.setMonth(next.getMonth() + form.value.frequency); break;
      case 4: next.setFullYear(next.getFullYear() + form.value.frequency); break;
    }
  }

  form.value.nextPayment = next.toISOString().split("T")[0];
}

function clearErrors() {
  Object.keys(errors).forEach((k) => delete errors[k]);
}

function handleSubmit() {
  clearErrors();

  const base = isEdit.value && props.editSubscription
    ? { ...props.editSubscription, ...form.value }
    : { ...form.value, id: crypto.randomUUID(), createdAt: new Date().toISOString() };

  // Coerce numeric fields — HTML inputs always return strings
  const raw = {
    ...base,
    price: Number(base.price) || 0,
    frequency: Number(base.frequency) || 1,
    notifyDaysBefore: Number(base.notifyDaysBefore ?? 1),
    cycle: Number(base.cycle) || 3,
  };

  const result = SubscriptionSchema.safeParse(raw);

  if (!result.success) {
    mapZodErrors(result.error.issues, errors, fieldMeta, t);
    toast(t("fill_required_fields"), "error");
    return;
  }

  try {
    if (isEdit.value && props.editSubscription) {
      subsStore.updateSubscription(result.data);
    } else {
      subsStore.addSubscription(result.data);
    }
    toast(t("success"));
    emit("saved");
    emit("close");
  } catch (e) {
    console.error("Subscription save failed:", e);
    toast(t("save_error"), "error");
  }
}
</script>

<template>
  <Modal :show="show" :title="title" @close="emit('close')" maxWidth="42rem">
    <form @submit.prevent="handleSubmit" class="space-y-4 sm:space-y-5">
      <!-- Name + Logo -->
      <div class="flex gap-3 sm:gap-4 items-start">
        <div class="flex-1">
          <AppInput
            v-model="form.name!"
            :label="t('subscription_name') + ' *'"
            :placeholder="t('subscription_name')"
            :error="errors.name"
            required
          />
        </div>
        <div class="pt-5">
          <LogoPicker v-model="form.logo!" />
        </div>
      </div>

      <!-- Price + Currency -->
      <div class="flex flex-col sm:flex-row gap-3">
        <div class="flex-1">
          <AppInput
            v-model="form.price!"
            type="number"
            :label="t('price') + ' *'"
            :error="errors.price"
            step="0.01"
            min="0"
            required
          />
        </div>
        <div class="sm:w-44">
          <AppSelect
            v-model="form.currencyId!"
            :options="currencyOptions"
            :label="t('currencies')"
            :error="errors.currencyId"
            searchable
          />
        </div>
      </div>

      <!-- Cycle + Frequency + Auto Renew -->
      <div class="flex flex-wrap gap-3 items-end">
        <div class="w-20">
          <AppInput
            v-model="form.frequency!"
            type="number"
            :label="t('frequency')"
            :error="errors.frequency"
            min="1"
            max="366"
          />
        </div>
        <div class="flex-1 min-w-[120px]">
          <AppSelect
            v-model="form.cycle!"
            :options="cycleOptions"
            :label="t('payment_every')"
          />
        </div>
        <div class="pb-1">
          <AppCheckbox v-model="form.autoRenew!" :label="t('auto_renewal')" />
        </div>
      </div>

      <!-- Dates -->
      <div class="flex flex-col sm:flex-row gap-3 items-end">
        <div class="flex-1">
          <AppInput
            v-model="form.startDate!"
            type="date"
            :label="t('start_date')"
            :error="errors.startDate"
          />
        </div>
        <button
          type="button"
          @click="calculateNextPayment"
          class="p-2 rounded-lg bg-[var(--color-surface-hover)] text-[var(--color-text-secondary)] hover:bg-[var(--color-primary)] hover:text-white transition-colors"
          :title="t('calculate_next_payment_date')"
        >
          <Sparkles :size="18" />
        </button>
        <div class="flex-1">
          <AppInput
            v-model="form.nextPayment!"
            type="date"
            :label="t('next_payment') + ' *'"
            :error="errors.nextPayment"
            required
          />
        </div>
      </div>

      <!-- Payment Method + Payer -->
      <div class="flex flex-col sm:flex-row gap-3">
        <div class="flex-1">
          <AppSelect
            v-model="form.paymentMethodId!"
            :options="paymentMethodOptions"
            :label="t('payment_method')"
          />
        </div>
        <div class="flex-1">
          <AppSelect
            v-model="form.payerUserId!"
            :options="payerOptions"
            :label="t('paid_by')"
          />
        </div>
      </div>

      <!-- Category -->
      <AppSelect
        v-model="form.categoryId!"
        :options="categoryOptions"
        :label="t('category')"
        searchable
      />

      <!-- Tags -->
      <TagInput
        v-model="form.tags!"
        :label="t('tags')"
      />

      <!-- URL -->
      <AppInput
        v-model="form.url!"
        :label="t('url')"
        :placeholder="'https://...'"
      />

      <!-- Notes -->
      <AppTextarea
        v-model="form.notes!"
        :label="t('notes')"
        :placeholder="t('notes')"
        :rows="2"
      />

      <!-- Toggles -->
      <div class="flex flex-wrap gap-x-6 gap-y-3 pt-1">
        <AppCheckbox v-model="form.notify!" :label="t('enable_notifications')" />
        <AppCheckbox v-model="form.inactive!" :label="t('inactive')" />
      </div>
    </form>

    <template #footer>
      <button
        @click="emit('close')"
        class="px-4 py-2 rounded-lg border border-[var(--color-border)] text-sm font-medium text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)] transition-colors"
      >{{ t('cancel') }}</button>
      <button
        @click="handleSubmit"
        class="px-5 py-2 rounded-lg bg-[var(--color-primary)] text-white text-sm font-medium hover:bg-[var(--color-primary-hover)] transition-colors"
      >{{ t('save') }}</button>
    </template>
  </Modal>
</template>
