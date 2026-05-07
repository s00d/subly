import type { ZodIssue } from "zod";

/**
 * Expected kind for i18n message selection (aligned with old useZodErrors).
 */
export type ZodFieldKind = "string" | "number" | "date";

export type ZodFieldMeta = Record<string, ZodFieldKind | undefined>;

/**
 * Convert a Zod v4 issue to a translation key (caller applies `t()`).
 */
export function zodIssueToMessageKey(issue: ZodIssue, meta: ZodFieldMeta, t: (key: string) => string): string {
  const field = issue.path[0];
  const fieldKey = typeof field === "string" ? field : undefined;
  const kind = fieldKey ? meta[fieldKey] : undefined;
  const code = issue.code as string;

  if (code === "invalid_type") {
    const rec = issue as ZodIssue & { received?: string; expected?: string };
    if (rec.received === "NaN" || (kind === "number" && rec.expected === "number")) {
      return t("field_invalid_number");
    }
    if (kind === "number") return t("field_invalid_number");
    if (kind === "date") return t("field_invalid_date");
    return t("field_required");
  }

  if (code === "too_small") {
    if (kind === "number") {
      const min = (issue as { minimum?: unknown }).minimum;
      return t("field_min_value").replace("{min}", String(min ?? 1));
    }
    return t("field_required");
  }

  if (code === "too_big") {
    return issue.message;
  }

  if (code === "invalid_format" || code === "invalid_string") {
    if (kind === "date") return t("field_invalid_date");
    if (kind === "number") return t("field_invalid_number");
    return t("field_invalid_date");
  }

  if (code === "custom") {
    const msg = String(issue.message || "");
    if (msg === "required" || msg === "validation_too_short" || msg === "validation_invalid_format") {
      return t(msg);
    }
  }

  if (code === "unrecognized_keys") {
    return issue.message;
  }

  const m = String(issue.message || "");
  if (m === "field_invalid_number" || m === "field_invalid_date" || m === "field_required") {
    return t(m);
  }

  return m || t("field_required");
}

/**
 * Map issues to a flat `errors[field]` object (first issue per field wins).
 */
export function mapZodErrors(
  issues: ZodIssue[],
  errors: Record<string, string>,
  meta: ZodFieldMeta,
  t: (key: string) => string,
): void {
  for (const issue of issues) {
    const field = issue.path[0];
    if (typeof field !== "string" || !field) continue;
    if (errors[field]) continue;
    errors[field] = zodIssueToMessageKey(issue, meta, t);
  }
}

export function issuesToFieldMap(issues: ZodIssue[], meta: ZodFieldMeta, t: (key: string) => string): Record<string, string> {
  const out: Record<string, string> = {};
  mapZodErrors(issues, out, meta, t);
  return out;
}
