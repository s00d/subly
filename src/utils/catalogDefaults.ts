import type { Category, PaymentMethod } from "@/schemas/appData";

const PROTECTED_CATEGORY_ID = "cat-1";

/** Preferred category from settings if it exists, else cat-1, else first. */
export function resolveCategoryId(
  preferred: string | undefined,
  categories: Category[],
): string {
  const id = preferred?.trim() ?? "";
  if (id && categories.some((c) => c.id === id)) {
    return id;
  }
  if (categories.some((c) => c.id === PROTECTED_CATEGORY_ID)) {
    return PROTECTED_CATEGORY_ID;
  }
  return categories[0]?.id ?? "";
}

/** Preferred enabled payment method from settings, else first enabled, else any. */
export function resolvePaymentMethodId(
  preferred: string | undefined,
  methods: PaymentMethod[],
): string {
  const id = preferred?.trim() ?? "";
  if (id) {
    const match = methods.find((p) => p.id === id);
    if (match?.enabled) {
      return id;
    }
  }
  return (
    methods.find((p) => p.enabled)?.id ??
    methods[0]?.id ??
    ""
  );
}

/** Keep id when it exists in the catalog; otherwise resolve from fallback. */
export function ensureCategoryId(
  id: string | undefined,
  fallback: string | undefined,
  categories: Category[],
): string {
  const trimmed = id?.trim() ?? "";
  if (trimmed && categories.some((c) => c.id === trimmed)) {
    return trimmed;
  }
  return resolveCategoryId(fallback, categories);
}

/** Keep id when it references an enabled method; otherwise resolve from fallback. */
export function ensurePaymentMethodId(
  id: string | undefined,
  fallback: string | undefined,
  methods: PaymentMethod[],
): string {
  const trimmed = id?.trim() ?? "";
  if (trimmed) {
    const match = methods.find((p) => p.id === trimmed);
    if (match?.enabled) {
      return trimmed;
    }
  }
  return resolvePaymentMethodId(fallback, methods);
}
