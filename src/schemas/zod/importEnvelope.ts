import { z } from "zod";

const subscriptionsArraySchema = z.preprocess(
  (v) => (v == null || v === undefined ? [] : v),
  z.array(z.unknown()),
);

/** Aligns with Rust `validate_import_payload` in export.rs (core catalogs + main currency + currency codes). */
const settingsImportSchema = z
  .object({
    mainCurrencyId: z.preprocess((v) => (v == null ? "" : String(v)), z.string().min(1)),
  })
  .passthrough();

export const importAppDataDocShapeSchema = z
  .object({
    subscriptions: subscriptionsArraySchema,
    expenses: subscriptionsArraySchema,
    categories: subscriptionsArraySchema,
    currencies: subscriptionsArraySchema,
    household: subscriptionsArraySchema,
    paymentMethods: subscriptionsArraySchema,
    tags: subscriptionsArraySchema,
    settings: settingsImportSchema,
  })
  .passthrough()
  .superRefine((data, ctx) => {
    if (data.categories.length < 1) {
      ctx.addIssue({ code: "custom", message: "import_categories_required", path: ["categories"] });
    }
    if (data.currencies.length < 1) {
      ctx.addIssue({ code: "custom", message: "import_currencies_required", path: ["currencies"] });
    }
    if (data.household.length < 1) {
      ctx.addIssue({ code: "custom", message: "import_household_required", path: ["household"] });
    }
    if (data.paymentMethods.length < 1) {
      ctx.addIssue({ code: "custom", message: "import_payment_methods_required", path: ["paymentMethods"] });
    }
    for (let i = 0; i < data.currencies.length; i++) {
      const row = data.currencies[i];
      if (row == null || typeof row !== "object") {
        ctx.addIssue({ code: "custom", message: "import_currency_row", path: ["currencies", i] });
        continue;
      }
      const code = (row as Record<string, unknown>).code;
      if (typeof code !== "string" || code.trim() === "") {
        ctx.addIssue({ code: "custom", message: "import_currency_code", path: ["currencies", i, "code"] });
      }
    }
    const ids = new Set<string>();
    for (const row of data.currencies) {
      if (row != null && typeof row === "object") {
        const id = (row as Record<string, unknown>).id;
        if (typeof id === "string" && id.trim() !== "") ids.add(id.trim());
      }
    }
    const main = String(data.settings.mainCurrencyId ?? "").trim();
    if (!ids.has(main)) {
      ctx.addIssue({ code: "custom", message: "import_main_currency_id", path: ["settings", "mainCurrencyId"] });
    }
  });

/** Same shape as `parse_import_payload_json` in Rust: flat `AppDataDoc` or `{ appData, appConfig? }`. */
export const importJsonPayloadSchema = z.union([
  z.object({ appData: importAppDataDocShapeSchema }).passthrough(),
  importAppDataDocShapeSchema,
]);

/**
 * @deprecated Minimal envelope — use `importJsonPayloadSchema` or `importAppDataDocShapeSchema`.
 * Kept for any external reference to the old name.
 */
export const importAppDataEnvelopeSchema = importAppDataDocShapeSchema;
