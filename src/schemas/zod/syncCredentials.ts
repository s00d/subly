import { z } from "zod";
import type { SyncProviderSchema } from "@/services/syncClient";
import { preprocessEmptyString } from "./common";

/**
 * One schema per provider: keys are `field.key` (not `type:key`); map errors to `type:key` in the component.
 * Values are coerced like Rust `String` + `#[serde(default)]` (null → "") before length/pattern checks.
 */
export function buildSyncCredentialsSchema(provider: SyncProviderSchema) {
  const fields = provider.fields ?? [];
  if (fields.length === 0) {
    return z.object({});
  }
  const shape: Record<string, z.ZodType<string>> = {};
  for (const f of fields) {
    const base = z.preprocess(preprocessEmptyString, z.string());
    shape[f.key] = base.superRefine((val, ctx) => {
      const t = val.trim();
      if (f.required && !t) {
        ctx.addIssue({ code: "custom", message: "required" });
        return;
      }
      if (!t) return;
      if (f.validation?.minLength != null && t.length < f.validation.minLength) {
        ctx.addIssue({ code: "custom", message: "validation_too_short" });
        return;
      }
      if (f.validation?.pattern) {
        const re = new RegExp(f.validation.pattern);
        if (!re.test(t)) {
          ctx.addIssue({ code: "custom", message: "validation_invalid_format" });
        }
      }
    });
  }
  return z.object(shape);
}
