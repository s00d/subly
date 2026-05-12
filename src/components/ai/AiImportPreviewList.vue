<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useLocaleFormat } from "@/composables/useLocaleFormat";
import AppSelect, { type SelectOption } from "@/components/ui/AppSelect.vue";
import { Sparkles, Cpu, AlertTriangle, CheckSquare, Square } from "@lucide/vue";
import type { Category, Currency, PaymentMethod } from "@/schemas/appData";
import type { StatementDraftRow } from "@/services/aiClient";

interface RowState {
  row: StatementDraftRow;
  selected: boolean;
}

const props = defineProps<{
  rows: RowState[];
  categories: Category[];
  currencies: Currency[];
  paymentMethods: PaymentMethod[];
}>();

const emit = defineEmits<{
  "update:row": [index: number, patch: Partial<StatementDraftRow["draft"]>];
  "toggle-selection": [index: number];
  "select-all": [];
  "deselect-all": [];
}>();

const { t } = useI18n();
const { fmtCurrency, fmtDateMedium } = useLocaleFormat();

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

function currencyCode(id: string): string {
  return props.currencies.find((c) => c.id === id)?.code ?? "USD";
}

function rowWarning(row: StatementDraftRow): string | null {
  if (!row.draft.warnings || row.draft.warnings.length === 0) return null;
  return row.draft.warnings
    .map((w) => {
      // We translate the well-known keys, fall back to the raw value.
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
                :value="state.row.draft.amount"
                @input="
                  emit('update:row', idx, {
                    amount: Number(
                      ($event.target as HTMLInputElement).value || 0,
                    ),
                  })
                "
                type="number"
                step="0.01"
                inputmode="decimal"
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
            <div class="col-span-12 sm:col-span-3">
              <input
                :value="state.row.draft.date"
                @input="
                  emit('update:row', idx, {
                    date: ($event.target as HTMLInputElement).value,
                  })
                "
                type="date"
                class="w-full px-2 py-1.5 rounded-md border border-border bg-surface text-xs text-text-primary focus:outline-none focus:ring-2 focus:ring-primary/30 focus:border-primary"
              />
            </div>

            <div class="col-span-6">
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
            <div class="col-span-6">
              <AppSelect
                :modelValue="state.row.draft.paymentMethodId"
                @update:modelValue="
                  (v) =>
                    emit('update:row', idx, { paymentMethodId: v as string })
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
            class="inline-flex items-center gap-1 px-1.5 py-0 rounded text-[9px] font-medium"
            :class="
              state.row.source === 'heuristic'
                ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-300'
                : 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-300'
            "
          >
            <Cpu v-if="state.row.source === 'heuristic'" :size="9" />
            <Sparkles v-else :size="9" />
            {{
              state.row.source === "heuristic"
                ? t("ai_source_heuristic")
                : t("ai_source_llm")
            }}
          </span>
          <span
            v-if="rowWarning(state.row)"
            class="inline-flex items-center gap-1 text-[10px] text-amber-600 dark:text-amber-400 truncate"
            :title="rowWarning(state.row) ?? ''"
          >
            <AlertTriangle :size="10" />
            {{ rowWarning(state.row) }}
          </span>
          <div class="flex-1" />
          <span class="text-[10px] text-text-muted tabular-nums">
            {{ fmtDateMedium(state.row.draft.date || new Date().toISOString()) }}
            ·
            {{
              fmtCurrency(
                state.row.draft.amount,
                currencyCode(state.row.draft.currencyId),
              )
            }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>
