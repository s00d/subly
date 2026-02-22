import { Kysely, CamelCasePlugin, sql, type ExpressionBuilder, type SelectQueryBuilder } from "kysely";
import { TauriSqliteDialect, getRawDb } from "./kysely-tauri";
import type { DatabaseSchema, SubscriptionsTable, ExpensesTable } from "./db-schema";
import type {
  AppData, Subscription, PaymentRecord, Expense,
  Category, Currency, HouseholdMember, PaymentMethod, Tag, Settings,
} from "@/schemas/appData";
import { SettingsSchema, validateAppData } from "@/schemas/appData";
import { getDefaultData } from "./seed";
import { uploadNow, syncStatus, setLocalUpdatedAt } from "@/services/sync";

// =============================================
// Kysely instance
// =============================================

const db = new Kysely<DatabaseSchema>({
  dialect: new TauriSqliteDialect({ path: "sqlite:subly.db" }),
  plugins: [new CamelCasePlugin()],
});

// =============================================
// Sync helper
// =============================================

let _syncTimeout: ReturnType<typeof setTimeout> | null = null;

function triggerSync() {
  setLocalUpdatedAt(Date.now());
  if (syncStatus.enabled) {
    if (_syncTimeout) clearTimeout(_syncTimeout);
    _syncTimeout = setTimeout(() => { uploadNow().catch(console.warn); }, 5_000);
  }
}

// =============================================
// Helpers
// =============================================

function safeJsonParse<T>(str: string, fallback: T): T {
  try { return JSON.parse(str) as T; }
  catch { return fallback; }
}

function toSubscription(row: SubscriptionsTable, history: PaymentRecord[]): Subscription {
  return {
    ...row,
    cycle: row.cycle as 1 | 2 | 3 | 4,
    notify: !!row.notify,
    inactive: !!row.inactive,
    autoRenew: !!row.autoRenew,
    favorite: !!row.favorite,
    cancellationDate: row.cancellationDate ?? null,
    replacementSubscriptionId: row.replacementSubscriptionId ?? null,
    tags: safeJsonParse(row.tags, []),
    paymentHistory: history,
  };
}

function toExpense(row: ExpensesTable): Expense {
  return {
    ...row,
    url: row.url || "",
    tags: safeJsonParse(row.tags, []),
  };
}

function subValues(s: Subscription) {
  return {
    id: s.id, name: s.name, logo: s.logo, price: s.price,
    currencyId: s.currencyId, nextPayment: s.nextPayment, startDate: s.startDate,
    cycle: s.cycle, frequency: s.frequency, notes: s.notes,
    paymentMethodId: s.paymentMethodId, payerUserId: s.payerUserId,
    categoryId: s.categoryId, notify: s.notify ? 1 : 0,
    notifyDaysBefore: s.notifyDaysBefore, lastNotifiedDate: s.lastNotifiedDate,
    inactive: s.inactive ? 1 : 0, autoRenew: s.autoRenew ? 1 : 0,
    url: s.url, cancellationDate: s.cancellationDate ?? null,
    replacementSubscriptionId: s.replacementSubscriptionId ?? null,
    createdAt: s.createdAt, favorite: s.favorite ? 1 : 0,
    tags: JSON.stringify(s.tags),
  };
}

function expValues(e: Expense) {
  return {
    id: e.id, name: e.name, amount: e.amount, currencyId: e.currencyId,
    date: e.date, categoryId: e.categoryId, paymentMethodId: e.paymentMethodId,
    payerUserId: e.payerUserId, tags: JSON.stringify(e.tags),
    notes: e.notes, url: e.url || "", createdAt: e.createdAt,
    subscriptionId: e.subscriptionId, paymentRecordId: e.paymentRecordId,
  };
}

// =============================================
// Load all data (for init / import-export / sync)
// =============================================

export async function loadAllData(): Promise<AppData | null> {
  const cfgRows = await db.selectFrom("config").selectAll().execute();
  if (cfgRows.length === 0) return null;

  const cfg = new Map(cfgRows.map((r) => [r.key, r.value]));
  if (cfg.get("initialized") !== "1") return null;

  const [catRows, curRows, hmRows, pmRows, tagRows, subRows, prRows, expRows] = await Promise.all([
    db.selectFrom("categories").selectAll().orderBy("sortOrder", "asc").execute(),
    db.selectFrom("currencies").selectAll().orderBy("sortOrder", "asc").execute(),
    db.selectFrom("householdMembers").selectAll().orderBy("sortOrder", "asc").execute(),
    db.selectFrom("paymentMethods").selectAll().orderBy("sortOrder", "asc").execute(),
    db.selectFrom("tags").selectAll().orderBy("sortOrder", "asc").execute(),
    db.selectFrom("subscriptions").selectAll().execute(),
    db.selectFrom("paymentRecords").selectAll().orderBy("date", "desc").execute(),
    db.selectFrom("expenses").selectAll().execute(),
  ]);

  const prMap = new Map<string, PaymentRecord[]>();
  for (const r of prRows) {
    if (!prMap.has(r.subscriptionId)) prMap.set(r.subscriptionId, []);
    prMap.get(r.subscriptionId)!.push({
      id: r.id, date: r.date, amount: r.amount, currencyId: r.currencyId, note: r.note,
    });
  }

  const settingsJson = cfg.get("settings") || "{}";
  const settings: Settings = SettingsSchema.parse(safeJsonParse(settingsJson, {}));

  return {
    subscriptions: subRows.map((s) => toSubscription(s, prMap.get(s.id) || [])),
    expenses: expRows.map(toExpense),
    categories: catRows as Category[],
    currencies: curRows as Currency[],
    household: hmRows as HouseholdMember[],
    paymentMethods: pmRows.map((p) => ({ ...p, enabled: !!p.enabled })) as PaymentMethod[],
    tags: tagRows.map((t) => ({ ...t, favorite: !!t.favorite })) as Tag[],
    settings,
    ratesApiKey: cfg.get("ratesApiKey") || "",
    ratesProvider: cfg.get("ratesProvider") || "frankfurter",
    fixerApiKey: cfg.get("fixerApiKey") || "",
    fixerProvider: parseInt(cfg.get("fixerProvider") || "0", 10),
    telegramBotToken: cfg.get("telegramBotToken") || "",
    telegramChatId: cfg.get("telegramChatId") || "",
    telegramEnabled: cfg.get("telegramEnabled") === "1",
    initialized: true,
  };
}

// =============================================
// Save all data (for import / sync)
// =============================================

export async function saveAllData(data: AppData): Promise<void> {
  await sql`PRAGMA foreign_keys = OFF`.execute(db);
  try {
    await db.transaction().execute(async (trx) => {
      await trx.deleteFrom("paymentRecords").execute();
      await trx.deleteFrom("subscriptions").execute();
      await trx.deleteFrom("expenses").execute();
      await trx.deleteFrom("categories").execute();
      await trx.deleteFrom("currencies").execute();
      await trx.deleteFrom("householdMembers").execute();
      await trx.deleteFrom("paymentMethods").execute();
      await trx.deleteFrom("tags").execute();
      await trx.deleteFrom("config").execute();

      for (const c of data.categories) {
        await trx.insertInto("categories").values({
          id: c.id, name: c.name, icon: c.icon, sortOrder: c.sortOrder, i18nKey: c.i18nKey,
        }).execute();
      }

      for (const c of data.currencies) {
        await trx.insertInto("currencies").values({
          id: c.id, name: c.name, symbol: c.symbol, code: c.code, rate: c.rate, sortOrder: c.sortOrder, i18nKey: c.i18nKey,
        }).execute();
      }

      for (const h of data.household) {
        await trx.insertInto("householdMembers").values({
          id: h.id, name: h.name, email: h.email, sortOrder: h.sortOrder,
        }).execute();
      }

      for (const p of data.paymentMethods) {
        await trx.insertInto("paymentMethods").values({
          id: p.id, name: p.name, icon: p.icon, enabled: p.enabled ? 1 : 0, sortOrder: p.sortOrder, i18nKey: p.i18nKey,
        }).execute();
      }

      for (const t of data.tags) {
        await trx.insertInto("tags").values({
          id: t.id, name: t.name, favorite: t.favorite ? 1 : 0, sortOrder: t.sortOrder, i18nKey: t.i18nKey,
        }).execute();
      }

      for (const s of data.subscriptions) {
        await trx.insertInto("subscriptions").values(subValues(s)).execute();
        for (const pr of (s.paymentHistory || [])) {
          await trx.insertInto("paymentRecords").values({
            id: pr.id, subscriptionId: s.id, date: pr.date, amount: pr.amount, currencyId: pr.currencyId, note: pr.note,
          }).execute();
        }
      }

      for (const e of data.expenses) {
        await trx.insertInto("expenses").values(expValues(e)).execute();
      }

      const configEntries: [string, string][] = [
        ["settings", JSON.stringify(data.settings)],
        ["ratesApiKey", data.ratesApiKey],
        ["ratesProvider", data.ratesProvider],
        ["fixerApiKey", data.fixerApiKey],
        ["fixerProvider", String(data.fixerProvider)],
        ["telegramBotToken", data.telegramBotToken],
        ["telegramChatId", data.telegramChatId],
        ["telegramEnabled", data.telegramEnabled ? "1" : "0"],
        ["initialized", "1"],
      ];
      for (const [key, value] of configEntries) {
        await trx.insertInto("config").values({ key, value }).execute();
      }
    });
  } finally {
    await sql`PRAGMA foreign_keys = ON`.execute(db);
  }
}

// =============================================
// High-level API
// =============================================

export async function loadAppData(): Promise<AppData> {
  const sqlData = await loadAllData();
  if (sqlData) return sqlData;
  const defaultData = getDefaultData();
  await saveAllData(defaultData);
  return defaultData;
}

export async function saveAppData(data: AppData): Promise<void> {
  const validated = validateAppData(data);
  await saveAllData(validated ?? data);
  triggerSync();
}

export async function resetAppData(): Promise<AppData> {
  const defaultData = getDefaultData();
  await saveAllData(defaultData);
  return defaultData;
}

export async function flushDbQueue(): Promise<void> {}

// =============================================
// Granular: Subscriptions
// =============================================

async function buildSubscriptionsWithHistory(subRows: SubscriptionsTable[]): Promise<Subscription[]> {
  if (subRows.length === 0) return [];
  const subIds = subRows.map((s) => s.id);
  const prRows = await db.selectFrom("paymentRecords")
    .selectAll()
    .where("subscriptionId", "in", subIds)
    .orderBy("date", "desc")
    .execute();
  const prMap = new Map<string, PaymentRecord[]>();
  for (const r of prRows) {
    if (!prMap.has(r.subscriptionId)) prMap.set(r.subscriptionId, []);
    prMap.get(r.subscriptionId)!.push({
      id: r.id, date: r.date, amount: r.amount, currencyId: r.currencyId, note: r.note,
    });
  }
  return subRows.map((s) => toSubscription(s, prMap.get(s.id) || []));
}

export async function dbLoadSubscriptions(): Promise<Subscription[]> {
  const subRows = await db.selectFrom("subscriptions").selectAll().execute();
  return buildSubscriptionsWithHistory(subRows);
}

// ---- Filtered / sorted subscriptions (for SubscriptionsPage) ----

export interface SubscriptionFilter {
  search?: string;
  state?: "active" | "inactive";
  categoryId?: string;
  paymentMethodId?: string;
  tag?: string;
  sortBy?: "next_payment" | "name" | "price";
  disabledToBottom?: boolean;
  hideDisabled?: boolean;
}

export async function dbLoadSubscriptionsFiltered(f: SubscriptionFilter): Promise<Subscription[]> {
  let q = db.selectFrom("subscriptions").selectAll();

  if (f.hideDisabled || f.state === "active") {
    q = q.where("inactive", "=", 0);
  } else if (f.state === "inactive") {
    q = q.where("inactive", "=", 1);
  }
  if (f.categoryId) q = q.where("categoryId", "=", f.categoryId);
  if (f.paymentMethodId) q = q.where("paymentMethodId", "=", f.paymentMethodId);
  if (f.tag) q = q.where("tags", "like", `%"${f.tag}"%`);
  if (f.search) {
    const like = `%${f.search}%`;
    q = q.where((eb: ExpressionBuilder<DatabaseSchema, "subscriptions">) =>
      eb.or([eb("name", "like", like), eb("tags", "like", like)]),
    );
  }

  if (f.disabledToBottom) q = q.orderBy("inactive", "asc");
  q = q.orderBy("favorite", "desc");
  switch (f.sortBy) {
    case "name": q = q.orderBy("name", "asc"); break;
    case "price": q = q.orderBy("price", "desc"); break;
    default: q = q.orderBy("nextPayment", "asc");
  }

  const subRows = await q.execute();
  return buildSubscriptionsWithHistory(subRows);
}

// ---- Dashboard-specific queries ----

export async function dbLoadOverdueSubscriptions(): Promise<Subscription[]> {
  const today = new Date().toISOString().split("T")[0];
  const subRows = await db.selectFrom("subscriptions").selectAll()
    .where("inactive", "=", 0)
    .where("nextPayment", "<", today)
    .orderBy("nextPayment", "asc")
    .execute();
  return buildSubscriptionsWithHistory(subRows);
}

export async function dbLoadUpcomingSubscriptions(days: number, limit: number): Promise<Subscription[]> {
  const today = new Date().toISOString().split("T")[0];
  const futureDate = new Date(Date.now() + days * 86400000).toISOString().split("T")[0];
  const subRows = await db.selectFrom("subscriptions").selectAll()
    .where("inactive", "=", 0)
    .where("nextPayment", ">=", today)
    .where("nextPayment", "<=", futureDate)
    .orderBy("nextPayment", "asc")
    .limit(limit)
    .execute();
  return buildSubscriptionsWithHistory(subRows);
}

export async function dbInsertSubscription(s: Subscription): Promise<void> {
  await db.insertInto("subscriptions").values(subValues(s)).execute();
  triggerSync();
}

export async function dbUpdateSubscription(s: Subscription): Promise<void> {
  const { id, ...rest } = subValues(s);
  await db.updateTable("subscriptions").set(rest).where("id", "=", id).execute();
  triggerSync();
}

export async function dbDeleteSubscription(id: string): Promise<void> {
  await db.deleteFrom("paymentRecords").where("subscriptionId", "=", id).execute();
  await db.deleteFrom("subscriptions").where("id", "=", id).execute();
  await db.updateTable("subscriptions")
    .set({ replacementSubscriptionId: null })
    .where("replacementSubscriptionId", "=", id)
    .execute();
  triggerSync();
}

export async function dbDeleteSubscriptionsBatch(ids: string[]): Promise<void> {
  if (ids.length === 0) return;
  await db.deleteFrom("paymentRecords").where("subscriptionId", "in", ids).execute();
  await db.deleteFrom("subscriptions").where("id", "in", ids).execute();
  await db.updateTable("subscriptions")
    .set({ replacementSubscriptionId: null })
    .where("replacementSubscriptionId", "in", ids)
    .execute();
  triggerSync();
}

export async function dbInsertPaymentRecord(subId: string, pr: PaymentRecord): Promise<void> {
  await db.insertInto("paymentRecords").values({
    id: pr.id, subscriptionId: subId, date: pr.date, amount: pr.amount, currencyId: pr.currencyId, note: pr.note,
  }).execute();
  triggerSync();
}

export async function dbDeletePaymentRecord(id: string): Promise<void> {
  await db.deleteFrom("paymentRecords").where("id", "=", id).execute();
  triggerSync();
}

// =============================================
// Granular: Expenses (with pagination)
// =============================================

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

type ExpenseQuery<S> = SelectQueryBuilder<DatabaseSchema, "expenses", S>;

function applyExpenseFilters<S>(query: ExpenseQuery<S>, f: ExpenseFilter): ExpenseQuery<S> {
  let q = query;
  if (f.search) {
    const like = `%${f.search}%`;
    q = q.where((eb: ExpressionBuilder<DatabaseSchema, "expenses">) =>
      eb.or([eb("name", "like", like), eb("notes", "like", like)]),
    );
  }
  if (f.categoryId) q = q.where("categoryId", "=", f.categoryId);
  if (f.paymentMethodId) q = q.where("paymentMethodId", "=", f.paymentMethodId);
  if (f.tag) q = q.where("tags", "like", `%"${f.tag}"%`);
  if (f.dateFrom) q = q.where("date", ">=", f.dateFrom);
  if (f.dateTo) q = q.where("date", "<=", f.dateTo);
  return q;
}

export async function dbLoadExpenses(filter: ExpenseFilter, limit: number, offset: number): Promise<ExpensePage> {
  let countQ = db.selectFrom("expenses").select(sql<number>`count(*)`.as("cnt"));
  countQ = applyExpenseFilters(countQ, filter);
  const { cnt } = await countQ.executeTakeFirstOrThrow();

  let dataQ = db.selectFrom("expenses").selectAll();
  dataQ = applyExpenseFilters(dataQ, filter);

  switch (filter.sortBy) {
    case "date_asc": dataQ = dataQ.orderBy("date", "asc"); break;
    case "amount_desc": dataQ = dataQ.orderBy("amount", "desc"); break;
    case "amount_asc": dataQ = dataQ.orderBy("amount", "asc"); break;
    default: dataQ = dataQ.orderBy("date", "desc");
  }

  const rows = await dataQ.limit(limit).offset(offset).execute();
  return { items: rows.map(toExpense), total: cnt };
}

export async function dbGetExpenseById(id: string): Promise<Expense | null> {
  const row = await db.selectFrom("expenses").selectAll().where("id", "=", id).executeTakeFirst();
  return row ? toExpense(row) : null;
}

export async function dbInsertExpense(e: Expense): Promise<void> {
  await db.insertInto("expenses").values(expValues(e)).execute();
  triggerSync();
}

export async function dbUpdateExpense(e: Expense): Promise<void> {
  const { id, ...rest } = expValues(e);
  await db.updateTable("expenses").set(rest).where("id", "=", id).execute();
  triggerSync();
}

export async function dbDeleteExpense(id: string): Promise<void> {
  await db.deleteFrom("expenses").where("id", "=", id).execute();
  triggerSync();
}

export async function dbDeleteExpensesBatch(ids: string[]): Promise<void> {
  if (ids.length === 0) return;
  await db.deleteFrom("expenses").where("id", "in", ids).execute();
  triggerSync();
}

export async function dbDeleteExpenseByPaymentRecord(subId: string, prId: string): Promise<void> {
  await db.deleteFrom("expenses")
    .where("subscriptionId", "=", subId)
    .where("paymentRecordId", "=", prId)
    .execute();
  triggerSync();
}

export async function dbGetExpenseCount(): Promise<number> {
  const { cnt } = await db.selectFrom("expenses")
    .select(sql<number>`count(*)`.as("cnt"))
    .executeTakeFirstOrThrow();
  return cnt;
}

export async function dbUpdateExpenseTagsBatch(oldName: string, newName: string): Promise<void> {
  const rows = await db.selectFrom("expenses")
    .select(["id", "tags"])
    .where("tags", "like", `%"${oldName}"%`)
    .execute();
  for (const row of rows) {
    const tags: string[] = safeJsonParse(row.tags, []);
    const idx = tags.indexOf(oldName);
    if (idx !== -1) {
      tags[idx] = newName;
      await db.updateTable("expenses").set({ tags: JSON.stringify(tags) }).where("id", "=", row.id).execute();
    }
  }
}

export async function dbRemoveExpenseTagBatch(tagName: string): Promise<void> {
  const rows = await db.selectFrom("expenses")
    .select(["id", "tags"])
    .where("tags", "like", `%"${tagName}"%`)
    .execute();
  for (const row of rows) {
    const tags = safeJsonParse<string[]>(row.tags, []).filter((t) => t !== tagName);
    await db.updateTable("expenses").set({ tags: JSON.stringify(tags) }).where("id", "=", row.id).execute();
  }
}

// =============================================
// Granular: Catalogs
// =============================================

export async function dbLoadCatalogs(): Promise<{
  categories: Category[]; currencies: Currency[];
  household: HouseholdMember[]; paymentMethods: PaymentMethod[]; tags: Tag[];
}> {
  const [catRows, curRows, hmRows, pmRows, tagRows] = await Promise.all([
    db.selectFrom("categories").selectAll().orderBy("sortOrder", "asc").execute(),
    db.selectFrom("currencies").selectAll().orderBy("sortOrder", "asc").execute(),
    db.selectFrom("householdMembers").selectAll().orderBy("sortOrder", "asc").execute(),
    db.selectFrom("paymentMethods").selectAll().orderBy("sortOrder", "asc").execute(),
    db.selectFrom("tags").selectAll().orderBy("sortOrder", "asc").execute(),
  ]);
  return {
    categories: catRows as Category[],
    currencies: curRows as Currency[],
    household: hmRows as HouseholdMember[],
    paymentMethods: pmRows.map((p) => ({ ...p, enabled: !!p.enabled })) as PaymentMethod[],
    tags: tagRows.map((t) => ({ ...t, favorite: !!t.favorite })) as Tag[],
  };
}

export async function dbUpsertCategory(c: Category): Promise<void> {
  await db.insertInto("categories")
    .values({ id: c.id, name: c.name, icon: c.icon, sortOrder: c.sortOrder, i18nKey: c.i18nKey })
    .onConflict((oc) => oc.column("id").doUpdateSet({ name: c.name, icon: c.icon, sortOrder: c.sortOrder, i18nKey: c.i18nKey }))
    .execute();
  triggerSync();
}

export async function dbDeleteCategory(id: string): Promise<void> {
  await db.deleteFrom("categories").where("id", "=", id).execute();
  triggerSync();
}

export async function dbUpsertCurrency(c: Currency): Promise<void> {
  await db.insertInto("currencies")
    .values({ id: c.id, name: c.name, symbol: c.symbol, code: c.code, rate: c.rate, sortOrder: c.sortOrder, i18nKey: c.i18nKey })
    .onConflict((oc) => oc.column("id").doUpdateSet({ name: c.name, symbol: c.symbol, code: c.code, rate: c.rate, sortOrder: c.sortOrder, i18nKey: c.i18nKey }))
    .execute();
  triggerSync();
}

export async function dbDeleteCurrency(id: string): Promise<void> {
  await db.deleteFrom("currencies").where("id", "=", id).execute();
  triggerSync();
}

export async function dbUpdateCurrencyRates(updates: { id: string; rate: number }[]): Promise<void> {
  for (const u of updates) {
    await db.updateTable("currencies").set({ rate: u.rate }).where("id", "=", u.id).execute();
  }
  triggerSync();
}

export async function dbUpsertHouseholdMember(h: HouseholdMember): Promise<void> {
  await db.insertInto("householdMembers")
    .values({ id: h.id, name: h.name, email: h.email, sortOrder: h.sortOrder })
    .onConflict((oc) => oc.column("id").doUpdateSet({ name: h.name, email: h.email, sortOrder: h.sortOrder }))
    .execute();
  triggerSync();
}

export async function dbDeleteHouseholdMember(id: string): Promise<void> {
  await db.deleteFrom("householdMembers").where("id", "=", id).execute();
  triggerSync();
}

export async function dbUpsertPaymentMethod(p: PaymentMethod): Promise<void> {
  await db.insertInto("paymentMethods")
    .values({ id: p.id, name: p.name, icon: p.icon, enabled: p.enabled ? 1 : 0, sortOrder: p.sortOrder, i18nKey: p.i18nKey })
    .onConflict((oc) => oc.column("id").doUpdateSet({ name: p.name, icon: p.icon, enabled: p.enabled ? 1 : 0, sortOrder: p.sortOrder, i18nKey: p.i18nKey }))
    .execute();
  triggerSync();
}

export async function dbDeletePaymentMethod(id: string): Promise<void> {
  await db.deleteFrom("paymentMethods").where("id", "=", id).execute();
  triggerSync();
}

export async function dbUpsertTag(t: Tag): Promise<void> {
  await db.insertInto("tags")
    .values({ id: t.id, name: t.name, favorite: t.favorite ? 1 : 0, sortOrder: t.sortOrder, i18nKey: t.i18nKey })
    .onConflict((oc) => oc.column("id").doUpdateSet({ name: t.name, favorite: t.favorite ? 1 : 0, sortOrder: t.sortOrder, i18nKey: t.i18nKey }))
    .execute();
  triggerSync();
}

export async function dbDeleteTag(id: string): Promise<void> {
  await db.deleteFrom("tags").where("id", "=", id).execute();
  triggerSync();
}

// =============================================
// Catalog helpers: max sort_order
// =============================================

export async function dbMaxSortOrder(table: "categories" | "currencies" | "householdMembers" | "paymentMethods" | "tags"): Promise<number> {
  const row = await db.selectFrom(table)
    .select(sql<number>`coalesce(max(sort_order), 0)`.as("mx"))
    .executeTakeFirstOrThrow();
  return row.mx;
}

// =============================================
// Config (settings)
// =============================================

export async function getConfigValue<T>(key: string): Promise<T | null> {
  const row = await db.selectFrom("config").select("value").where("key", "=", key).executeTakeFirst();
  if (!row) return null;
  try { return JSON.parse(row.value) as T; }
  catch { return row.value as unknown as T; }
}

export async function setConfigValue(key: string, value: unknown): Promise<void> {
  const serialized = typeof value === "string" ? value : JSON.stringify(value);
  await db.insertInto("config")
    .values({ key, value: serialized })
    .onConflict((oc) => oc.column("key").doUpdateSet({ value: serialized }))
    .execute();
}

export async function deleteConfigValue(key: string): Promise<void> {
  await db.deleteFrom("config").where("key", "=", key).execute();
}

// =============================================
// Currency Rate History
// =============================================

export interface RateHistoryPoint {
  rate: number;
  recordedAt: string;
}

export async function dbSaveRateSnapshot(currencyId: string, rate: number): Promise<void> {
  const today = new Date().toISOString().split("T")[0];
  await db.insertInto("currencyRateHistory")
    .values({ currencyId, rate, recordedAt: today })
    .onConflict((oc) => oc.columns(["currencyId", "recordedAt"]).doUpdateSet({ rate }))
    .execute();
}

export async function dbGetRateHistory(currencyId: string, days = 30): Promise<RateHistoryPoint[]> {
  const since = new Date();
  since.setDate(since.getDate() - days);
  const sinceStr = since.toISOString().split("T")[0];

  return await db.selectFrom("currencyRateHistory")
    .select(["rate", "recordedAt"])
    .where("currencyId", "=", currencyId)
    .where("recordedAt", ">=", sinceStr)
    .orderBy("recordedAt", "asc")
    .execute();
}

export async function dbGetRateHistoryBatch(currencyIds: string[], days = 30): Promise<Record<string, RateHistoryPoint[]>> {
  if (currencyIds.length === 0) return {};
  const since = new Date();
  since.setDate(since.getDate() - days);
  const sinceStr = since.toISOString().split("T")[0];

  const rows = await db.selectFrom("currencyRateHistory")
    .select(["currencyId", "rate", "recordedAt"])
    .where("currencyId", "in", currencyIds)
    .where("recordedAt", ">=", sinceStr)
    .orderBy("recordedAt", "asc")
    .execute();

  const result: Record<string, RateHistoryPoint[]> = {};
  for (const row of rows) {
    (result[row.currencyId] ??= []).push({ rate: row.rate, recordedAt: row.recordedAt });
  }
  return result;
}

export async function dbPruneRateHistory(keepDays: number): Promise<number> {
  const cutoff = new Date();
  cutoff.setDate(cutoff.getDate() - keepDays);
  const cutoffStr = cutoff.toISOString().split("T")[0];

  const result = await db.deleteFrom("currencyRateHistory")
    .where("recordedAt", "<", cutoffStr)
    .executeTakeFirst();

  return Number(result.numDeletedRows ?? 0);
}

export async function dbClearRateHistory(): Promise<void> {
  await db.deleteFrom("currencyRateHistory").execute();
}

export async function dbRateHistoryCount(): Promise<number> {
  const row = await db.selectFrom("currencyRateHistory")
    .select(db.fn.countAll<number>().as("cnt"))
    .executeTakeFirstOrThrow();
  return Number(row.cnt);
}

// =============================================
// SQL Aggregations for Dashboard
// =============================================

export interface ExpenseAggregation {
  monthTotal: number;
  yearTotal: number;
  recentExpenses: Expense[];
}

export async function dbGetExpenseAggregations(monthPrefix: string, yearPrefix: string): Promise<ExpenseAggregation> {
  const [monthResult, yearResult, recentRows] = await Promise.all([
    db.selectFrom("expenses")
      .select(sql<number>`coalesce(sum(amount), 0)`.as("total"))
      .where("date", "like", `${monthPrefix}%`)
      .executeTakeFirstOrThrow(),
    db.selectFrom("expenses")
      .select(sql<number>`coalesce(sum(amount), 0)`.as("total"))
      .where("date", "like", `${yearPrefix}%`)
      .executeTakeFirstOrThrow(),
    db.selectFrom("expenses").selectAll().orderBy("date", "desc").limit(5).execute(),
  ]);

  return {
    monthTotal: monthResult.total,
    yearTotal: yearResult.total,
    recentExpenses: recentRows.map(toExpense),
  };
}

export async function dbGetExpensesForMonth(monthPrefix: string): Promise<Expense[]> {
  const rows = await db.selectFrom("expenses").selectAll()
    .where("date", "like", `${monthPrefix}%`)
    .orderBy("date", "desc")
    .execute();
  return rows.map(toExpense);
}

export async function dbGetExpensesSince(dateFrom: string): Promise<Expense[]> {
  const rows = await db.selectFrom("expenses").selectAll()
    .where("date", ">=", dateFrom)
    .orderBy("date", "desc")
    .execute();
  return rows.map(toExpense);
}

export async function dbGetExpenseTotalFiltered(filter: ExpenseFilter): Promise<number> {
  let q = db.selectFrom("expenses").select(sql<number>`coalesce(sum(amount), 0)`.as("total"));
  q = applyExpenseFilters(q, filter);
  const { total } = await q.executeTakeFirstOrThrow();
  return total;
}

// =============================================
// Dashboard widget queries
// =============================================

export interface TopExpense {
  id: string;
  name: string;
  amount: number;
  currencyId: string;
  date: string;
  categoryId: string;
}

export async function dbGetTopExpenses(monthPrefix: string, limit = 5): Promise<TopExpense[]> {
  const rows = await db.selectFrom("expenses")
    .select(["id", "name", "amount", "currencyId", "date", "categoryId"])
    .where("date", "like", `${monthPrefix}%`)
    .orderBy("amount", "desc")
    .limit(limit)
    .execute();
  return rows;
}

export interface AvgExpenseStats {
  avgAmount: number;
  count: number;
  total: number;
}

export async function dbGetAvgExpenseStats(monthPrefix: string): Promise<AvgExpenseStats> {
  const row = await db.selectFrom("expenses")
    .select([
      sql<number>`coalesce(avg(amount), 0)`.as("avgAmount"),
      sql<number>`count(*)`.as("count"),
      sql<number>`coalesce(sum(amount), 0)`.as("total"),
    ])
    .where("date", "like", `${monthPrefix}%`)
    .executeTakeFirstOrThrow();
  return { avgAmount: Number(row.avgAmount), count: Number(row.count), total: Number(row.total) };
}

export interface DayOfWeekStat {
  dayOfWeek: number;
  total: number;
  count: number;
}

export async function dbGetExpensesByDayOfWeek(monthPrefix: string): Promise<DayOfWeekStat[]> {
  const rows = await db.selectFrom("expenses")
    .select([
      sql<number>`cast(strftime('%w', date) as integer)`.as("dayOfWeek"),
      sql<number>`coalesce(sum(amount), 0)`.as("total"),
      sql<number>`count(*)`.as("count"),
    ])
    .where("date", "like", `${monthPrefix}%`)
    .groupBy(sql`strftime('%w', date)`)
    .orderBy(sql`strftime('%w', date)`)
    .execute();
  return rows.map((r) => ({ dayOfWeek: Number(r.dayOfWeek), total: Number(r.total), count: Number(r.count) }));
}

export interface MonthComparisonData {
  currentTotal: number;
  currentCount: number;
  previousTotal: number;
  previousCount: number;
}

export async function dbGetExpenseMonthComparison(currentPrefix: string, previousPrefix: string): Promise<MonthComparisonData> {
  const [cur, prev] = await Promise.all([
    db.selectFrom("expenses")
      .select([
        sql<number>`coalesce(sum(amount), 0)`.as("total"),
        sql<number>`count(*)`.as("count"),
      ])
      .where("date", "like", `${currentPrefix}%`)
      .executeTakeFirstOrThrow(),
    db.selectFrom("expenses")
      .select([
        sql<number>`coalesce(sum(amount), 0)`.as("total"),
        sql<number>`count(*)`.as("count"),
      ])
      .where("date", "like", `${previousPrefix}%`)
      .executeTakeFirstOrThrow(),
  ]);
  return {
    currentTotal: Number(cur.total),
    currentCount: Number(cur.count),
    previousTotal: Number(prev.total),
    previousCount: Number(prev.count),
  };
}

export interface TagExpenseStat {
  tag: string;
  total: number;
}

export async function dbGetExpensesByTags(monthPrefix: string): Promise<TagExpenseStat[]> {
  const rows = await db.selectFrom("expenses")
    .select(["tags", "amount"])
    .where("date", "like", `${monthPrefix}%`)
    .where("tags", "!=", "")
    .where("tags", "!=", "[]")
    .execute();

  const map: Record<string, number> = {};
  for (const row of rows) {
    let parsed: string[] = [];
    try { parsed = JSON.parse(row.tags); } catch { continue; }
    for (const tag of parsed) {
      map[tag] = (map[tag] ?? 0) + Number(row.amount);
    }
  }
  return Object.entries(map)
    .map(([tag, total]) => ({ tag, total }))
    .sort((a, b) => b.total - a.total);
}

// =============================================
// Execute raw SQL from file content
// =============================================

export async function executeSqlFile(sqlContent: string): Promise<{ statementsRun: number }> {
  const rawDb = getRawDb();
  const statements = sqlContent
    .replace(/--[^\n]*/g, "")
    .split(";")
    .map((s) => s.trim())
    .filter((s) => s.length > 0);

  let statementsRun = 0;
  for (const stmt of statements) {
    await rawDb.execute(stmt);
    statementsRun++;
  }
  return { statementsRun };
}
