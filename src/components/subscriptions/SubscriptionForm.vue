<script setup lang="ts">
import { ref, watch, computed, reactive } from "vue";
import Papa from "papaparse";
import { useSubscriptionsStore } from "@/stores/subscriptions";
import { useSettingsStore } from "@/stores/settings";
import { useCatalogStore } from "@/stores/catalog";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import type { Subscription, CycleType } from "@/schemas/appData";
import { parseSubscription, SubscriptionSchema } from "@/schemas/appData";
import { mapZodErrors, type ZodFieldMeta } from "@/composables/useZodErrors";
import { useClipboard } from "@/composables/useClipboard";
import Modal from "@/components/ui/Modal.vue";
import AppInput from "@/components/ui/AppInput.vue";
import AppDatePicker from "@/components/ui/AppDatePicker.vue";
import AppTextarea from "@/components/ui/AppTextarea.vue";
import AppSelect from "@/components/ui/AppSelect.vue";
import AppCheckbox from "@/components/ui/AppCheckbox.vue";
import LogoPicker from "@/components/ui/LogoPicker.vue";
import TagInput from "@/components/ui/TagInput.vue";
import type { SelectOption } from "@/components/ui/AppSelect.vue";
import { Sparkles, Globe, Copy, Table2 } from "lucide-vue-next";
import { resolveFaviconFromInputUrl } from "@/services/logoAssets";
import { getNextCycleDate } from "@/services/calculations";

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
const { copyToClipboard } = useClipboard();
const BULK_COLUMNS = ["name", "price", "currency", "nextPayment", "cycle", "frequency", "category", "paymentMethod", "tags", "url", "notes"] as const;

function createDefaultForm(): Partial<Subscription> {
  return {
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

const form = ref<Partial<Subscription>>(createDefaultForm());
const errors = reactive<Record<string, string>>({});
const bulkMode = ref(false);
const bulkCsv = ref("");
const bulkImportErrors = ref<string[]>([]);
let bulkValidationTimer: ReturnType<typeof setTimeout> | null = null;

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
  form.value = createDefaultForm();
  bulkMode.value = false;
  bulkCsv.value = "";
  bulkImportErrors.value = [];
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
const bulkTemplate = computed(() => {
  const tomorrow = new Date(Date.now() + 86400000).toISOString().split("T")[0];
  return `${BULK_COLUMNS.join(",")}
Netflix Premium,15.99,USD,${tomorrow},3,1,Entertainment,Visa,home|video,https://netflix.com,Family plan`;
});

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
    next = getNextCycleDate(next, form.value.cycle as CycleType, form.value.frequency);
  }

  form.value.nextPayment = next.toISOString().split("T")[0];
}

function clearErrors() {
  Object.keys(errors).forEach((k) => delete errors[k]);
}

async function applyDomainIcon() {
  const faviconUrl = await resolveFaviconFromInputUrl(form.value.url || "");
  if (!faviconUrl) {
    toast("Could not load icon from this domain", "error");
    return;
  }
  form.value.logo = faviconUrl;
  toast("Icon loaded from domain");
}

async function handleSubmit() {
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
      await subsStore.updateSubscription(result.data);
    } else {
      await subsStore.addSubscription(result.data);
    }
    toast(t("success"));
    emit("saved");
    emit("close");
  } catch (e) {
    console.error("Subscription save failed:", e);
    toast(t("save_error"), "error");
  }
}

function normalize(value: unknown): string {
  return String(value ?? "").trim();
}

function parseCycle(value: string): CycleType | null {
  const normalized = value.trim().toLowerCase();
  if (!normalized) return 3;
  if (normalized === "1" || normalized.startsWith("day")) return 1;
  if (normalized === "2" || normalized.startsWith("week")) return 2;
  if (normalized === "3" || normalized.startsWith("month")) return 3;
  if (normalized === "4" || normalized.startsWith("year")) return 4;
  return null;
}

function resolveCurrencyId(value: string): string | null {
  const normalized = value.trim().toLowerCase();
  if (!normalized) return form.value.currencyId || settingsStore.settings.mainCurrencyId || null;
  const byId = catalogStore.currencies.find((item) => item.id.toLowerCase() === normalized);
  if (byId) return byId.id;
  const byCode = catalogStore.currencies.find((item) => item.code.toLowerCase() === normalized);
  if (byCode) return byCode.id;
  const byName = catalogStore.currencies.find((item) => item.name.toLowerCase() === normalized);
  return byName?.id || null;
}

function resolveCategoryId(value: string): string | null {
  const normalized = value.trim().toLowerCase();
  if (!normalized) return form.value.categoryId || settingsStore.settings.defaultCategoryId || "cat-1";
  const byId = catalogStore.categories.find((item) => item.id.toLowerCase() === normalized);
  if (byId) return byId.id;
  const byName = catalogStore.categories.find((item) => item.name.toLowerCase() === normalized);
  return byName?.id || null;
}

function resolvePaymentMethodId(value: string): string | null {
  const normalized = value.trim().toLowerCase();
  if (!normalized) {
    return form.value.paymentMethodId || settingsStore.settings.defaultPaymentMethodId || catalogStore.enabledPaymentMethods[0]?.id || null;
  }
  const byId = catalogStore.paymentMethods.find((item) => item.id.toLowerCase() === normalized);
  if (byId) return byId.id;
  const byName = catalogStore.paymentMethods.find((item) => item.name.toLowerCase() === normalized);
  return byName?.id || null;
}

function parseTags(value: string): string[] {
  if (!value.trim()) return [];
  return value.split("|").map((item) => item.trim()).filter(Boolean);
}

function validateHeader(fields: string[]): boolean {
  if (fields.length !== BULK_COLUMNS.length) return false;
  return BULK_COLUMNS.every((column, idx) => fields[idx] === column);
}

function validateBulkCsvRows() {
  const csvSource = bulkCsv.value.trim();
  if (!csvSource) {
    return {
      ok: false,
      rows: [] as Array<{ rowNumber: number; raw: Record<string, string> }>,
      errors: [t("bulk_add_import_failed")],
    };
  }

  const parsed = Papa.parse<Record<string, string>>(csvSource, {
    header: true,
    skipEmptyLines: "greedy",
    transformHeader: (header) => header.trim(),
  });

  const fields = parsed.meta.fields?.map((field) => field.trim()) || [];
  if (!validateHeader(fields)) {
    return {
      ok: false,
      rows: [] as Array<{ rowNumber: number; raw: Record<string, string> }>,
      errors: [t("bulk_add_invalid_header")],
    };
  }

  const rows: Array<{ rowNumber: number; raw: Record<string, string> }> = [];
  const errors: string[] = [];

  for (let index = 0; index < parsed.data.length; index += 1) {
    const row = parsed.data[index];
    const rowNumber = index + 2;

    try {
      const name = normalize(row.name);
      const price = Number(normalize(row.price));
      const nextPayment = normalize(row.nextPayment);
      const frequency = Number(normalize(row.frequency || "1"));
      const cycle = parseCycle(normalize(row.cycle));
      const currencyId = resolveCurrencyId(normalize(row.currency));
      const categoryId = resolveCategoryId(normalize(row.category));
      const paymentMethodId = resolvePaymentMethodId(normalize(row.paymentMethod));

      if (!name) throw new Error(t("field_required"));
      if (!Number.isFinite(price) || price < 0) throw new Error(t("field_invalid_number"));
      if (!/^\d{4}-\d{2}-\d{2}$/.test(nextPayment)) throw new Error(t("field_invalid_date"));
      if (!Number.isFinite(frequency) || frequency < 1) throw new Error(t("field_invalid_number"));
      if (!cycle) throw new Error(t("field_invalid_number"));
      if (!currencyId) throw new Error(`Unknown currency: ${normalize(row.currency)}`);
      if (!categoryId) throw new Error(`Unknown category: ${normalize(row.category)}`);
      if (!paymentMethodId) throw new Error(`Unknown payment method: ${normalize(row.paymentMethod)}`);

      rows.push({ rowNumber, raw: row });
    } catch (error) {
      const message = error instanceof Error ? error.message : t("save_error");
      errors.push(t("bulk_add_invalid_row", { row: rowNumber, error: message }));
    }
  }

  return {
    ok: errors.length === 0,
    rows,
    errors,
  };
}

watch([bulkCsv, bulkMode], () => {
  if (bulkValidationTimer) {
    clearTimeout(bulkValidationTimer);
  }

  bulkValidationTimer = setTimeout(() => {
    if (!bulkMode.value) {
      bulkImportErrors.value = [];
      return;
    }

    if (!bulkCsv.value.trim()) {
      bulkImportErrors.value = [];
      return;
    }

    const result = validateBulkCsvRows();
    bulkImportErrors.value = result.errors;
  }, 250);
});

watch(() => props.show, (isOpen) => {
  if (!isOpen && bulkValidationTimer) {
    clearTimeout(bulkValidationTimer);
    bulkValidationTimer = null;
  }
});

async function handleBulkImport() {
  bulkImportErrors.value = [];
  const csvSource = bulkCsv.value.trim();
  if (!csvSource) {
    toast(t("bulk_add_import_failed"), "error");
    return;
  }

  const validation = validateBulkCsvRows();
  if (!validation.ok && validation.rows.length === 0) {
    bulkImportErrors.value = validation.errors;
    toast(t("bulk_add_import_failed"), "error");
    return;
  }

  let successCount = 0;
  const rowErrors: string[] = [...validation.errors];

  for (const item of validation.rows) {
    const row = item.raw;
    const rowNumber = item.rowNumber;

    try {
      const name = normalize(row.name);
      const price = Number(normalize(row.price));
      const nextPayment = normalize(row.nextPayment);
      const frequency = Number(normalize(row.frequency || "1"));
      const cycle = parseCycle(normalize(row.cycle));
      const currencyId = resolveCurrencyId(normalize(row.currency));
      const categoryId = resolveCategoryId(normalize(row.category));
      const paymentMethodId = resolvePaymentMethodId(normalize(row.paymentMethod));

      if (!name) throw new Error(t("field_required"));
      if (!Number.isFinite(price) || price < 0) throw new Error(t("field_invalid_number"));
      if (!/^\d{4}-\d{2}-\d{2}$/.test(nextPayment)) throw new Error(t("field_invalid_date"));
      if (!Number.isFinite(frequency) || frequency < 1) throw new Error(t("field_invalid_number"));
      if (!cycle) throw new Error(t("field_invalid_number"));
      if (!currencyId) throw new Error(`Unknown currency: ${normalize(row.currency)}`);
      if (!categoryId) throw new Error(`Unknown category: ${normalize(row.category)}`);
      if (!paymentMethodId) throw new Error(`Unknown payment method: ${normalize(row.paymentMethod)}`);

      const raw = {
        id: crypto.randomUUID(),
        createdAt: new Date().toISOString(),
        name,
        logo: "",
        price,
        currencyId,
        nextPayment,
        startDate: nextPayment,
        cycle,
        frequency: Math.trunc(frequency),
        notes: normalize(row.notes),
        paymentMethodId,
        payerUserId: form.value.payerUserId || catalogStore.household[0]?.id || "",
        categoryId,
        notify: true,
        notifyDaysBefore: -1,
        lastNotifiedDate: "",
        inactive: false,
        autoRenew: true,
        url: normalize(row.url),
        cancellationDate: null,
        replacementSubscriptionId: null,
        tags: parseTags(normalize(row.tags)),
        favorite: false,
        paymentHistory: [],
      };

      const validated = SubscriptionSchema.safeParse(raw);
      if (!validated.success) {
        throw new Error(validated.error.issues[0]?.message || t("save_error"));
      }

      await subsStore.addSubscription(validated.data);
      successCount += 1;
    } catch (error) {
      const message = error instanceof Error ? error.message : t("save_error");
      rowErrors.push(t("bulk_add_invalid_row", { row: rowNumber, error: message }));
    }
  }

  bulkImportErrors.value = rowErrors;

  if (successCount > 0 && rowErrors.length === 0) {
    toast(t("bulk_add_import_success", { count: successCount }));
    emit("saved");
    emit("close");
    return;
  }

  if (successCount > 0) {
    toast(t("bulk_add_import_partial", { success: successCount, failed: rowErrors.length }), "error");
    emit("saved");
    return;
  }

  toast(t("bulk_add_import_failed"), "error");
}

async function copyBulkTemplate() {
  const text = bulkTemplate.value;
  const copied = await copyToClipboard(text);
  if (copied) {
    toast(t("copied_to_clipboard"));
    return;
  }
  toast(t("error"), "error");
}
</script>

<template>
  <Modal :show="show" :title="title" @close="emit('close')" maxWidth="42rem">
    <form @submit.prevent="bulkMode ? handleBulkImport() : handleSubmit()" class="space-y-4 sm:space-y-5">
      <div v-if="bulkMode" class="space-y-3">
        <p class="text-sm text-text-secondary">
          {{ t("bulk_add_csv_hint") }}
        </p>
        <AppTextarea
          v-model="bulkCsv"
          :label="t('bulk_add_csv')"
          :placeholder="bulkTemplate"
          :rows="12"
        />
        <ul v-if="bulkImportErrors.length" class="text-xs text-red-500 space-y-1 max-h-40 overflow-auto">
          <li v-for="(item, idx) in bulkImportErrors" :key="`bulk-error-${idx}`">{{ item }}</li>
        </ul>
        <div class="text-xs text-text-primary whitespace-pre-wrap border border-border rounded-lg p-2 bg-surface-secondary">
          <div class="flex flex-wrap items-center justify-between gap-2 mb-1">
            <span class="font-medium wrap-break-word">{{ t("bulk_add_csv_example") }}:</span>
            <div class="flex items-center gap-1">
              <button
                type="button"
                @click="copyBulkTemplate"
                class="inline-flex items-center gap-1 px-2 py-1 rounded-md border border-border text-xs font-medium text-text-secondary hover:bg-surface-hover transition-colors"
              >
                <Copy :size="12" />
                <span class="hidden sm:inline">{{ t("copy") }}</span>
              </button>
            </div>
          </div>
          <div class="overflow-x-auto">
            <pre class="font-mono text-[11px] leading-relaxed whitespace-pre-wrap wrap-break-word m-0">{{ bulkTemplate }}</pre>
          </div>
        </div>
      </div>

      <template v-else>
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
      <div class="flex flex-row gap-2 sm:gap-3 items-end">
        <div class="flex-1 min-w-0">
          <AppDatePicker
            v-model="form.startDate!"
            :label="t('start_date')"
            :error="errors.startDate"
          />
        </div>
        <button
          type="button"
          @click="calculateNextPayment"
          class="shrink-0 p-2 rounded-lg bg-surface-hover text-text-secondary hover:bg-primary hover:text-white transition-colors"
          :title="t('calculate_next_payment_date')"
        >
          <Sparkles :size="18" />
        </button>
        <div class="flex-1 min-w-0">
          <AppDatePicker
            v-model="form.nextPayment!"
            :label="t('next_payment') + ' *'"
            :error="errors.nextPayment"
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
      <div class="space-y-2">
        <AppInput
          v-model="form.url!"
          :label="t('url')"
          :placeholder="'https://...'"
        />
        <button
          type="button"
          @click="applyDomainIcon"
          class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-border text-xs font-medium text-text-secondary hover:bg-surface-hover transition-colors"
        >
          <Globe :size="14" />
          Get icon from domain
        </button>
      </div>

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
      </template>
    </form>

    <template #footer>
      <button
        @click="bulkMode = !bulkMode"
        class="mr-auto px-3 py-2 rounded-lg border border-border text-sm font-medium text-text-secondary hover:bg-surface-hover transition-colors inline-flex items-center"
        :title="bulkMode ? t('add_subscription') : t('bulk_add_mode')"
      ><Table2 :size="16" /></button>
      <div class="flex items-center gap-2">
        <button
          @click="emit('close')"
          class="px-4 py-2 rounded-lg border border-border text-sm font-medium text-text-secondary hover:bg-surface-hover transition-colors"
        >{{ t('cancel') }}</button>
        <button
          @click="bulkMode ? handleBulkImport() : handleSubmit()"
          class="px-5 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-primary-hover transition-colors"
        >{{ bulkMode ? t('bulk_add_import') : t('save') }}</button>
      </div>
    </template>
  </Modal>
</template>
