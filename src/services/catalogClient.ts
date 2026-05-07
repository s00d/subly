import type { Category, Currency, HouseholdMember, PaymentMethod, Tag } from "@/schemas/appData";
import { callCommand } from "@/services/commandClient";

export interface CatalogsPayload {
  categories: Category[];
  currencies: Currency[];
  household: HouseholdMember[];
  paymentMethods: PaymentMethod[];
  tags: Tag[];
}

export interface CatalogsUsageSummary {
  categoryUsage: Record<string, number>;
  currencyUsage: Record<string, number>;
  paymentMethodUsage: Record<string, number>;
  tagUsage: Record<string, number>;
}

export async function loadCatalogs(): Promise<CatalogsPayload> {
  return callCommand("catalogs_load");
}

export async function getCatalogsUsageSummary(): Promise<CatalogsUsageSummary> {
  return callCommand("catalogs_usage_summary");
}

export async function upsertCategory(category: Category): Promise<void> { return callCommand("catalogs_upsert_category", { category }); }
export async function deleteCategory(id: string): Promise<void> { return callCommand("catalogs_delete_category", { id }); }
export async function upsertCurrency(currency: Currency): Promise<void> { return callCommand("catalogs_upsert_currency", { currency }); }
export async function deleteCurrency(id: string): Promise<void> { return callCommand("catalogs_delete_currency", { id }); }
export async function updateCurrencyRates(updates: { id: string; rate: number }[]): Promise<void> { return callCommand("catalogs_update_currency_rates", { updates }); }
export async function upsertHouseholdMember(householdMember: HouseholdMember): Promise<void> { return callCommand("catalogs_upsert_household_member", { householdMember }); }
export async function deleteHouseholdMember(id: string): Promise<void> { return callCommand("catalogs_delete_household_member", { id }); }
export async function upsertPaymentMethod(paymentMethod: PaymentMethod): Promise<void> { return callCommand("catalogs_upsert_payment_method", { paymentMethod }); }
export async function deletePaymentMethod(id: string): Promise<void> { return callCommand("catalogs_delete_payment_method", { id }); }
export async function upsertTag(tag: Tag): Promise<void> { return callCommand("catalogs_upsert_tag", { tag }); }
export async function deleteTag(id: string): Promise<void> { return callCommand("catalogs_delete_tag", { id }); }
export async function maxSortOrder(table: "categories" | "currencies" | "householdMembers" | "paymentMethods" | "tags"): Promise<number> {
  return callCommand("catalogs_max_sort_order", { table });
}

