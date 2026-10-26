import { z } from "zod";
import {
  cycleFieldSchema,
  isoDateYmdSchema,
  nonEmptyTrimmedStringSchema,
  moneyStringSchema,
  notifyDaysBeforeFieldSchema,
  positiveIntCoerced,
} from "./common";

/**
 * Fields validated before `upsertSubscription` (after the same coercions as the modal used to apply manually).
 * Does not include credentials (optional, stored separately).
 */
export const subscriptionFormFieldsSchema = z.object({
  name: nonEmptyTrimmedStringSchema,
  price: moneyStringSchema,
  currencyId: nonEmptyTrimmedStringSchema,
  nextPayment: isoDateYmdSchema,
  startDate: isoDateYmdSchema,
  frequency: positiveIntCoerced,
  notifyDaysBefore: notifyDaysBeforeFieldSchema,
  cycle: cycleFieldSchema,
});

export type SubscriptionFormFields = z.infer<typeof subscriptionFormFieldsSchema>;

/**
 * Same coercions as `SubscriptionForm` before Zod (HTML inputs, defaults).
 */
export function coerceSubscriptionFormForValidation(
  base: Record<string, unknown>,
): Record<string, unknown> {
  return {
    name: String(base.name ?? ""),
    price: typeof base.price === "string" ? base.price : String(Number(base.price) || 0),
    // Raw value: `|| 1` here hid 0 / blank from the schema and saved them as 1.
    frequency: base.frequency,
    notifyDaysBefore: Number(base.notifyDaysBefore ?? -1),
    cycle: Number(base.cycle) || 3,
    currencyId: String(base.currencyId ?? ""),
    nextPayment: String(base.nextPayment ?? ""),
    startDate: String(base.startDate ?? ""),
  };
}
