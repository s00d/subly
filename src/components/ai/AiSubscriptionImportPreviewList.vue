<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import AppSelect, { type SelectOption } from "@/components/ui/AppSelect.vue";
import { Sparkles, AlertTriangle, CheckSquare, Square, X } from "@lucide/vue";
import type { Category, Currency, PaymentMethod } from "@/schemas/appData";
import type { AiSubscriptionDraftRow } from "@/services/aiClient";

interface RowState {
  row: AiSubscriptionDraftRow;
  selected: boolean;
}

const props = defineProps<{
  rows: RowState[];
  categories: Category[];
  currencies: Currency[];
  paymentMethods: PaymentMethod[];
}>();

const emit = defineEmits<{
  "update:row": [index: number, patch: Partial<AiSubscriptionDraftRow["draft"]>];
  "toggle-selection": [index: number];
  remove: [index: number];
  "select-all": [];
  "deselect-all": [];
}>();

const { t } = useI18n();

const currencyOptions = computed<SelectOption[]>(() =>
  props.currencies.map((c) => ({
    label: `${c.symbol} ${c.code}`,
    value: c.id,
  })),
);

const categoryOptions = computed<SelectOption[]>(() => [
  { label: "—", value: "" },
  ...[...props.categories]
    .sort((a, b) => a.sortOrder - b.sortOrder)
    .map((c) => ({ label: c.name, value: c.id, icon: c.icon || undefined })),
]);

const paymentOptions = computed<SelectOption[]>(() => [
  { label: "—", value: "" },
  ...props.paymentMethods
    .filter((p) => p.enabled)
    .map((p) => ({ label: p.name, value: p.id, icon: p.icon })),
]);

const cycleOptions = computed<SelectOption[]>(() => [
  { label: t("days"), value: 1 },
  { label: t("weeks"), value: 2 },
  { label: t("months"), value: 3 },
  { label: t("years"), value: 4 },
]);

function rowWarning(row: AiSubscriptionDraftRow): string | null {
  if (!row.draft.warnings || row.draft.warnings.length === 0) return null;
  return row.draft.warnings
    .map((w) => {
      const key = w as string;
      const translated = t(key);
      return translated === key ? w : translated;
    })
    .join(", ");
}
</script>

<template>
  <div class="space-y-2">
    <div class="flex items-center gap-2 px-1">
      <button
        type="button"
        @click="emit('select-all')"
        class="text-[11px] text-text-secondary hover:text-text-primary inline-flex items-center gap-1"
      >
        <CheckSquare :size="12" /> {{ t("select_all") }}
      </button>
      <button
        type="button"
        @click="emit('deselect-all')"
        class="text-[11px] text-text-muted hover:text-text-secondary inline-flex items-center gap-1"
      >
        <Square :size="12" /> {{ t("deselect_all") }}
      </button>
      <div class="flex-1" />
      <span class="text-[11px] text-text-muted">
        {{ rows.filter((r) => r.selected).length }} / {{ rows.length }}
      </span>
    </div>

    <div class="space-y-2 max-h-[55vh] overflow-y-auto pr-1">
      <div
        v-for="(state, idx) in rows"
        :key="idx"
        class="rounded-lg border p-2.5 transition-colors"
        :class="state.selected ? 'border-primary bg-primary-light/30' : 'border-border bg-surface'"
      >
        <div class="flex items-start gap-2.5">
          <button
            type="button"
            class="shrink-0 mt-1 w-4 h-4 rounded border-2 flex items-center justify-center transition-colors cursor-pointer"
            :class="
              state.selected
                ? 'bg-primary border-primary text-white'
                : 'border-border hover:border-primary'
            "
            @click="emit('toggle-selection', idx)"
          >
            <svg
              v-if="state.selected"
              width="10"
              height="10"
              viewBox="0 0 12 12"
              fill="none"
            >
              <path
                d="M2 6L5 9L10 3"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
          </button>

          <button
            type="button"
            class="shrink-0 mt-1 w-4 h-4 rounded text-text-muted hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 flex items-center justify-center transition-colors cursor-pointer"
            :title="t('delete')"
            @click="emit('remove', idx)"
          >
            <X :size="12" />
          </button>

          <div class="min-w-0 flex-1 grid grid-cols-12 gap-2">
            <div class="col-span-12 sm:col-span-5">
              <input
                :value="state.row.draft.name"
                @input="
                  emit('update:row', idx, {
                    name: ($event.target as HTMLInputElement).value,
                  })
                "
                type="text"
                class="w-full px-2 py-1.5 rounded-md border border-border bg-surface text-xs text-text-primary focus:outline-none focus:ring-2 focus:ring-primary/30 focus:border-primary"
                :placeholder="t('name')"
              />
            </div>
            <div class="col-span-6 sm:col-span-2">
              <input
                :value="state.row.draft.price"
                @input="
                  emit('update:row', idx, {
                    price: Number(($event.target as HTMLInputElement).value || 0),
                  })
                "
                type="number"
                step="0.01"
                inputmode="decimal"
                :placeholder="t('price')"
                class="w-full px-2 py-1.5 rounded-md border border-border bg-surface text-xs text-text-primary text-right tabular-nums focus:outline-none focus:ring-2 focus:ring-primary/30 focus:border-primary"
              />
            </div>
            <div class="col-span-6 sm:col-span-2">
              <AppSelect
                :modelValue="state.row.draft.currencyId"
                @update:modelValue="
                  (v) => emit('update:row', idx, { currencyId: v as string })
                "
                :options="currencyOptions"
                size="sm"
              />
            </div>
            <div class="col-span-6 sm:col-span-3">
              <AppSelect
                :modelValue="state.row.draft.cycle"
                @update:modelValue="
                  (v) => emit('update:row', idx, { cycle: Number(v) })
                "
                :options="cycleOptions"
                size="sm"
              />
            </div>

            <div class="col-span-6 sm:col-span-3">
              <input
                :value="state.row.draft.nextPayment ?? ''"
                @input="
                  emit('update:row', idx, {
                    nextPayment: ($event.target as HTMLInputElement).value || undefined,
                  })
                "
                type="date"
                :placeholder="t('next_payment')"
                class="w-full px-2 py-1.5 rounded-md border border-border bg-surface text-xs text-text-primary focus:outline-none focus:ring-2 focus:ring-primary/30 focus:border-primary"
              />
            </div>
            <div class="col-span-6 sm:col-span-4">
              <AppSelect
                :modelValue="state.row.draft.categoryId"
                @update:modelValue="
                  (v) => emit('update:row', idx, { categoryId: v as string })
                "
                :options="categoryOptions"
                size="sm"
                :placeholder="t('category')"
              />
            </div>
            <div class="col-span-12 sm:col-span-5">
              <AppSelect
                :modelValue="state.row.draft.paymentMethodId"
                @update:modelValue="
                  (v) => emit('update:row', idx, { paymentMethodId: v as string })
                "
                :options="paymentOptions"
                size="sm"
                :placeholder="t('payment_method')"
              />
            </div>
          </div>
        </div>

        <div class="flex items-center gap-2 mt-1.5 pl-7">
          <span
            class="inline-flex items-center gap-1 px-1.5 py-0 rounded text-[9px] font-medium bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-300"
          >
            <Sparkles :size="9" />
            {{ t("ai_source_llm") }}
          </span>
          <span
            v-if="rowWarning(state.row)"
            class="inline-flex items-center gap-1 text-[10px] text-amber-600 dark:text-amber-400 truncate"
            :title="rowWarning(state.row) ?? ''"
          >
            <AlertTriangle :size="10" />
            {{ rowWarning(state.row) }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>
