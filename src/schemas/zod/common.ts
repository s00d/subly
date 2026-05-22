import { z } from "zod";

/**
 * Coerce JSON `null` / `undefined` to `""` so Zod does not throw `invalid_type` on optional Rust
 * `String` fields serialized with `#[serde(default)]` (empty string, not null — but some clients
 * may still send null).
 */
export function preprocessEmptyString(v: unknown): string {
  if (v == null) return "";
  return String(v);
}

/**
 * Trims after coercion; use for required non-empty string fields (forms + IPC-shaped payloads).
 */
export const nonEmptyTrimmedStringSchema = z.preprocess(
  preprocessEmptyString,
  z.string().trim().min(1, { message: "field_required" }),
);

/** `YYYY-MM-DD` as used by date pickers and subscription date fields. */
export const isoDateYmdSchema = z.preprocess(
  preprocessEmptyString,
  z
    .string()
    .trim()
    .regex(/^\d{4}-\d{2}-\d{2}$/u, { message: "field_invalid_date" }),
);

function preprocessToNumber(v: unknown): number {
  if (v === "" || v === null || v === undefined) return Number.NaN;
  if (typeof v === "number") return v;
  return Number(String(v).trim());
}

const MONEY_STRING_RE = /^-?\d+(\.\d+)?$/u;

function preprocessMoneyString(v: unknown): string {
  if (v === "" || v === null || v === undefined) return "";
  if (typeof v === "number" && Number.isFinite(v)) return String(v);
  return String(v).trim();
}

/** IPC money field: non-negative decimal string (accepts legacy numbers from forms). */
export const moneyStringSchema = z
  .preprocess(preprocessMoneyString, z.string().trim())
  .refine((s) => s !== "" && MONEY_STRING_RE.test(s), { message: "field_invalid_number" })
  .refine((s) => !s.startsWith("-"), { message: "field_invalid_number" });

/** Number from inputs: empty string → NaN (fails next step), not silent 0. */
export const nonNegativeFiniteCoerced = z
  .preprocess(preprocessToNumber, z.number())
  .refine((n) => Number.isFinite(n) && n >= 0, { message: "field_invalid_number" });

export const positiveIntCoerced = z
  .preprocess(preprocessToNumber, z.number())
  .refine((n) => Number.isFinite(n) && Number.isInteger(n) && n >= 1, { message: "field_invalid_number" });

export const cycleFieldSchema = z
  .preprocess(preprocessToNumber, z.number())
  .refine((n) => [1, 2, 3, 4].includes(n), { message: "field_invalid_number" });

/** Backend allows any finite integer including `-1` (use global default). */
export const notifyDaysBeforeFieldSchema = z
  .preprocess(preprocessToNumber, z.number())
  .refine((n) => Number.isFinite(n), { message: "field_invalid_number" });
