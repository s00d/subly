import type { z } from "zod";

/**
 * Describes the expected type for each field, so we can pick a fitting
 * human-readable error message instead of Zod's default "Invalid input".
 */
type FieldKind = "string" | "number" | "date";

export interface ZodFieldMeta {
  [fieldName: string]: FieldKind;
}

/**
 * Convert a single Zod issue into a translated, user-friendly message.
 *
 * @param issue   – Zod issue object
 * @param meta    – map of field name → expected kind
 * @param t       – translation function (key → string)
 */
export function zodIssueMessage(
  issue: z.core.$ZodIssue,
  meta: ZodFieldMeta,
  t: (key: string) => string,
): string {
  const field = issue.path[0] as string | undefined;
  const kind = field ? meta[field] : undefined;
  const code = issue.code;

  // invalid_type — happens when a required field is missing or has a wrong type
  if (code === "invalid_type") {
    if (kind === "number") return t("field_invalid_number");
    if (kind === "date") return t("field_invalid_date");
    return t("field_required");
  }

  // too_small — e.g. min(1) on a number or non-empty string
  if (code === "too_small") {
    if (kind === "number") {
      const min = (issue as unknown as Record<string, unknown>).minimum;
      return t("field_min_value").replace("{min}", String(min ?? 1));
    }
    return t("field_required");
  }

  // too_big
  if (code === "too_big") {
    return issue.message; // keep original — rare case
  }

  // Fallback — use Zod's message but wrap it just in case
  return issue.message || t("field_required");
}

/**
 * Populate a reactive errors record from a Zod safe-parse result.
 *
 * Returns `true` if there are errors.
 */
export function mapZodErrors(
  issues: z.core.$ZodIssue[],
  errors: Record<string, string>,
  meta: ZodFieldMeta,
  t: (key: string) => string,
): void {
  for (const issue of issues) {
    const field = issue.path[0] as string;
    if (field && !errors[field]) {
      errors[field] = zodIssueMessage(issue, meta, t);
    }
  }
}
