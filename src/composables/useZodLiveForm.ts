import { ref, watch, type Ref } from "vue";
import type { z } from "zod";
import { mapZodErrors, type ZodFieldMeta } from "./useZodErrors";

export interface UseZodLiveFormOptions {
  /** Current form payload for `schema.safeParse` */
  getValues: () => Record<string, unknown>;
  schema: z.ZodType<unknown>;
  fieldMeta: ZodFieldMeta;
  t: (key: string) => string;
  debounceMs?: number;
  /**
   * For `string` fields: hide "field_required" until the user has edited the field once.
   * Reduces red empty state when opening a modal.
   */
  guardEmptyRequiredStrings?: boolean;
  /** Field names to apply guard to (default: all string keys in fieldMeta) */
  guardedStringFields?: string[];
}

/**
 * Debounced live validation + optional dirty-guard for required string fields.
 */
export function useZodLiveForm(opts: UseZodLiveFormOptions) {
  const errors = ref<Record<string, string>>({});
  const dirtyFields = ref<Record<string, boolean>>({});
  const debounceMs = opts.debounceMs ?? 120;

  function markDirty(field: string) {
    dirtyFields.value = { ...dirtyFields.value, [field]: true };
  }

  function clearDirty() {
    dirtyFields.value = {};
  }

  function validateSync(): boolean {
    const raw = opts.getValues();
    const parsed = opts.schema.safeParse(raw);
    if (parsed.success) {
      errors.value = {};
      return true;
    }

    const next: Record<string, string> = {};
    mapZodErrors(parsed.error.issues, next, opts.fieldMeta, opts.t);

    if (opts.guardEmptyRequiredStrings) {
      const meta = opts.fieldMeta;
      const guardKeys =
        opts.guardedStringFields ??
        Object.keys(meta).filter((k) => meta[k] === "string");
      const reqKey = opts.t("field_required");
      for (const k of guardKeys) {
        if (!dirtyFields.value[k] && next[k] === reqKey) {
          delete next[k];
        }
      }
    }

    errors.value = next;
    return false;
  }

  /** Submit validation: same schema, no dirty-guard (shows all field errors). */
  function validateStrict(): boolean {
    const raw = opts.getValues();
    const parsed = opts.schema.safeParse(raw);
    if (parsed.success) {
      errors.value = {};
      return true;
    }
    const next: Record<string, string> = {};
    mapZodErrors(parsed.error.issues, next, opts.fieldMeta, opts.t);
    errors.value = next;
    return false;
  }

  let timer: ReturnType<typeof setTimeout> | null = null;

  function scheduleValidate() {
    if (timer != null) clearTimeout(timer);
    timer = setTimeout(() => {
      validateSync();
      timer = null;
    }, debounceMs);
  }

  function watchSource(source: Ref<unknown>) {
    watch(
      source,
      () => {
        scheduleValidate();
      },
      { deep: true },
    );
  }

  return {
    errors,
    dirtyFields,
    markDirty,
    clearDirty,
    validateSync,
    validateStrict,
    scheduleValidate,
    watchSource,
  };
}
