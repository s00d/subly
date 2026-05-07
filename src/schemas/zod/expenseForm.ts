import { z } from "zod";
import { isoDateYmdSchema, nonEmptyTrimmedStringSchema, nonNegativeFiniteCoerced } from "./common";

/** Modal fields validated before build payload for `expenses_upsert`. */
export const expenseFormFieldsSchema = z.object({
  name: nonEmptyTrimmedStringSchema,
  amount: nonNegativeFiniteCoerced,
  currencyId: nonEmptyTrimmedStringSchema,
  date: isoDateYmdSchema,
});

export type ExpenseFormFields = z.infer<typeof expenseFormFieldsSchema>;

/** Aligns with `ExpenseForm` coercions before submit. */
export function coerceExpenseFormForValidation(form: Record<string, unknown>): Record<string, unknown> {
  const iso = String(form.date ?? "").trim();
  const rawAmount = Number(form.amount) || 0;
  return {
    name: String(form.name ?? ""),
    amount: rawAmount,
    currencyId: String(form.currencyId ?? ""),
    date: iso,
  };
}
