import type { Expense, ExpenseUpsertPayload } from "@/schemas/appData";
import { callCommand } from "@/services/commandClient";

export interface ExpenseFilter {
  search?: string;
  categoryId?: string;
  paymentMethodId?: string;
  tag?: string;
  dateFrom?: string;
  dateTo?: string;
  sortBy?: "date_desc" | "date_asc" | "amount_desc" | "amount_asc";
}

export interface ExpensePage {
  items: Expense[];
  total: number;
}

export async function listExpensesPage(): Promise<Expense[]> {
  const res = await callCommand<ExpensePage>("list_expenses_page", { request: {} });
  return res.items;
}

export async function listExpensesFiltered(filter: ExpenseFilter, limit: number, offset: number): Promise<ExpensePage> {
  return callCommand("list_expenses_page", { request: { ...filter, limit, offset } });
}

export async function getExpenseById(id: string): Promise<Expense | null> {
  return callCommand("expenses_get_by_id", { id });
}

export async function insertExpense(expense: Expense): Promise<void> {
  return callCommand("expenses_insert", { expense });
}

export async function upsertExpense(expense: ExpenseUpsertPayload): Promise<void> {
  return callCommand("expenses_upsert", { expense });
}

export async function updateExpense(expense: Expense): Promise<void> {
  return callCommand("expenses_update", { expense });
}

export async function deleteExpense(id: string): Promise<void> {
  return callCommand("expenses_delete", { id });
}

export async function deleteExpensesBatch(ids: string[]): Promise<void> {
  return callCommand("expenses_delete_batch", { ids });
}

export async function deleteExpenseByPaymentRecord(subId: string, prId: string): Promise<void> {
  return callCommand("expenses_delete_by_payment_record", { subId, prId });
}

export async function getExpenseCount(): Promise<number> {
  return callCommand("expenses_count");
}

export async function getExpenseTotalFiltered(filter: ExpenseFilter): Promise<number> {
  return callCommand("expenses_total_filtered", { filter });
}

export async function getExpensesForMonth(year: number, month: number): Promise<Expense[]> {
  return callCommand("expenses_for_month", { year, month });
}

export async function updateExpenseTagsBatch(oldName: string, newName: string): Promise<void> {
  return callCommand("expenses_update_tags_batch", { oldName, newName });
}

export async function removeExpenseTagBatch(tagName: string): Promise<void> {
  return callCommand("expenses_remove_tag_batch", { tagName });
}

