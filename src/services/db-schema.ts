/**
 * Kysely database schema.
 * All names are camelCase — CamelCasePlugin maps them to snake_case in SQL.
 * Table names: paymentRecords -> payment_records, householdMembers -> household_members, etc.
 * Column names: currencyId -> currency_id, sortOrder -> sort_order, etc.
 */

import type { Generated } from "kysely";

export interface SubscriptionsTable {
  id: string;
  name: string;
  logo: string;
  price: number;
  currencyId: string;
  nextPayment: string;
  startDate: string;
  cycle: number;
  frequency: number;
  notes: string;
  paymentMethodId: string;
  payerUserId: string;
  categoryId: string;
  notify: number;
  notifyDaysBefore: number;
  lastNotifiedDate: string;
  inactive: number;
  autoRenew: number;
  url: string;
  cancellationDate: string | null;
  replacementSubscriptionId: string | null;
  createdAt: string;
  favorite: number;
  tags: string;
}

export interface PaymentRecordsTable {
  id: string;
  subscriptionId: string;
  date: string;
  amount: number;
  currencyId: string;
  note: string;
}

export interface ExpensesTable {
  id: string;
  name: string;
  amount: number;
  currencyId: string;
  date: string;
  categoryId: string;
  paymentMethodId: string;
  payerUserId: string;
  tags: string;
  notes: string;
  createdAt: string;
  url: string;
  subscriptionId: string;
  paymentRecordId: string;
}

export interface CategoriesTable {
  id: string;
  name: string;
  icon: string;
  sortOrder: number;
  i18nKey: string;
}

export interface CurrenciesTable {
  id: string;
  name: string;
  symbol: string;
  code: string;
  rate: number;
  sortOrder: number;
  i18nKey: string;
}

export interface HouseholdMembersTable {
  id: string;
  name: string;
  email: string;
  sortOrder: number;
}

export interface PaymentMethodsTable {
  id: string;
  name: string;
  icon: string;
  enabled: number;
  sortOrder: number;
  i18nKey: string;
}

export interface TagsTable {
  id: string;
  name: string;
  favorite: number;
  sortOrder: number;
  i18nKey: string;
}

export interface CurrencyRateHistoryTable {
  id: Generated<number>;
  currencyId: string;
  rate: number;
  recordedAt: string;
}

export interface ConfigTable {
  key: string;
  value: string;
}

export interface DatabaseSchema {
  subscriptions: SubscriptionsTable;
  paymentRecords: PaymentRecordsTable;
  expenses: ExpensesTable;
  categories: CategoriesTable;
  currencies: CurrenciesTable;
  householdMembers: HouseholdMembersTable;
  paymentMethods: PaymentMethodsTable;
  tags: TagsTable;
  currencyRateHistory: CurrencyRateHistoryTable;
  config: ConfigTable;
}
