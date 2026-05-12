import { importJsonPayloadSchema } from "@/schemas/zod/importEnvelope";

export type CycleType = 1 | 2 | 3 | 4;

/** Те же поля, что в `SubscriptionInputDto.credentials`; хранятся в secure storage, не в снапшоте данных. */
export interface SubscriptionCredentials {
  login: string;
  password: string;
  totpSecret: string;
}

/**
 * Lightweight "is each field stored?" probe returned by the list endpoint.
 * Mirrors `SubscriptionCredentialsMetaDto` on the backend. Lets the UI render
 * Reveal / Copy controls without ever decrypting the keyring entry.
 */
export interface SubscriptionCredentialsMeta {
  hasLogin: boolean;
  hasPassword: boolean;
  hasTotp: boolean;
}

export interface PaymentRecord {
  id: string;
  date: string;
  amount: number;
  currencyId: string;
  note: string;
}

export interface Subscription {
  id: string;
  name: string;
  logo: string;
  price: number;
  currencyId: string;
  nextPayment: string;
  startDate: string;
  cycle: CycleType;
  frequency: number;
  notes: string;
  paymentMethodId: string;
  payerUserId: string;
  categoryId: string;
  notify: boolean;
  notifyDaysBefore: number;
  lastNotifiedDate: string;
  inactive: boolean;
  autoRenew: boolean;
  url: string;
  cancellationDate: string | null;
  replacementSubscriptionId: string | null;
  createdAt: string;
  tags: string[];
  favorite: boolean;
  paymentHistory: PaymentRecord[];
  /** Приходит из list/upsert IPC; не часть импорта JSON-бэкапа. */
  credentials?: SubscriptionCredentials | null;
}

/** Row from list/detail IPC (`SubscriptionListItemDto`): base subscription plus computed list fields. */
export interface SubscriptionListItem extends Subscription {
  monthlyPrice: number;
  daysLeft: number;
  overdue: boolean;
  /** Non-secret bitmap: which credential fields the user has saved. */
  credentialsMeta: SubscriptionCredentialsMeta;
}

export interface Expense {
  id: string;
  name: string;
  amount: number;
  currencyId: string;
  createdAt: string;
  categoryId: string;
  paymentMethodId: string;
  payerUserId: string;
  tags: string[];
  notes: string;
  url: string;
  /** Empty string when not linked to a subscription payment. */
  subscriptionId: string;
  /** Empty string when not from a subscription payment record. */
  paymentRecordId: string;
  /** Milliseconds (`ExpenseDoc.updated_at`), present when returned from backend. */
  updatedAt?: number;
}

/** `YYYY-MM-DD` for date pickers / formatters (UTC calendar date derived from `createdAt`). */
export function expenseToIsoDate(e: Pick<Expense, "createdAt">): string {
  const s = String(e.createdAt ?? "").trim();
  if (s.length >= 10 && /^\d{4}-\d{2}-\d{2}/.test(s)) {
    return s.slice(0, 10);
  }
  const t = Date.parse(s);
  if (Number.isFinite(t)) {
    const d = new Date(t);
    const pad = (n: number) => String(n).padStart(2, "0");
    return `${d.getUTCFullYear()}-${pad(d.getUTCMonth() + 1)}-${pad(d.getUTCDate())}`;
  }
  return "2000-01-01";
}

function expenseCreatedAtFromRaw(r: Record<string, unknown>): string {
  const ca = String(r.createdAt ?? "").trim();
  if (ca) return ca;
  const y = r.dateYear;
  const m = r.dateMonth;
  const d = r.dateDay;
  if (typeof y === "number" && typeof m === "number" && typeof d === "number" && Number.isFinite(y) && Number.isFinite(m) && Number.isFinite(d)) {
    const pad = (n: number) => String(Math.trunc(n)).padStart(2, "0");
    return `${Math.trunc(y)}-${pad(m)}-${pad(d)}T12:00:00.000Z`;
  }
  const legacy = String(r.date ?? "").trim();
  const match = /^(\d{4})-(\d{2})-(\d{2})/.exec(legacy);
  if (match) {
    return `${match[1]}-${match[2]}-${match[3]}T12:00:00.000Z`;
  }
  return "2000-01-01T12:00:00.000Z";
}

export interface Category {
  id: string;
  name: string;
  icon: string;
  sortOrder: number;
  i18nKey: string;
}

export interface Currency {
  id: string;
  name: string;
  symbol: string;
  code: string;
  rate: number;
  sortOrder: number;
  i18nKey: string;
}

export interface HouseholdMember {
  id: string;
  name: string;
  email: string;
  sortOrder: number;
}

export interface PaymentMethod {
  id: string;
  name: string;
  icon: string;
  enabled: boolean;
  sortOrder: number;
  i18nKey: string;
}

export interface Tag {
  id: string;
  name: string;
  favorite: boolean;
  sortOrder: number;
  i18nKey: string;
}

export interface CustomColors {
  main: string;
  accent: string;
  hover: string;
}

/**
 * Full UI preferences stored in app config (`config_get` / `config_set` key `"settings"`).
 * Not the same as the compact `SettingsDoc` embedded in the Redb data snapshot — see Rust `SettingsDoc`.
 */
export interface Settings {
  darkTheme: 0 | 1 | 2;
  colorTheme: string;
  monthlyPrice: boolean;
  convertCurrency: boolean;
  hideDisabled: boolean;
  disabledToBottom: boolean;
  showOriginalPrice: boolean;
  showSubscriptionProgress: boolean;
  language: string;
  mainCurrencyId: string;
  defaultCategoryId: string;
  defaultPaymentMethodId: string;
  budget: number;
  notifyDaysBefore: number;
  notificationTitle: string;
  notificationBodyDueToday: string;
  notificationBodyDueSoon: string;
  notificationOverdueTitle: string;
  notificationOverdueBody: string;
  notificationSchedule: "any" | "morning" | "evening" | "custom";
  notificationCustomHour: number;
  recurringNotifications: boolean;
  notificationSound: boolean;
  currencyAutoUpdate: boolean;
  currencyUpdateTargets: string[];
  lastCurrencyUpdate: string;
  dashboardWidgets: Array<{ id: string; visible: boolean }>;
  subscriptionViewMode: "default" | "compact" | "expanded";
  subscriptionGroupBy: "none" | "category" | "payment_method";
  expenseViewMode: "default" | "compact" | "expanded";
  currencyViewMode: "default" | "compact" | "expanded";
  calendarViewMode: "default" | "compact" | "expanded";
  converterPresets: number[];
  rateHistoryEnabled: boolean;
  rateHistoryDays: number;
  customColors: CustomColors;
}

/** Alias: settings JSON from config store (same shape as `Settings`). */
export type ConfigSettings = Settings;

export interface NotificationSettings {
  enabled: boolean;
  daysBefore: number;
}

export interface StatsData {
  activeSubscriptions: number;
  inactiveSubscriptions: number;
  totalCostPerMonth: number;
  totalCostPerYear: number;
  averageSubscriptionCost: number;
  mostExpensive: { name: string; price: number; logo: string } | null;
  amountDueThisMonth: number;
  totalSavingsPerMonth: number;
  budget: number;
  budgetUsed: number | null;
  budgetLeft: number | null;
  overBudgetAmount: number | null;
  categoryCosts: Array<{ label: string; value: number }>;
  memberCosts: Array<{ label: string; value: number }>;
  paymentMethodCounts: Array<{ label: string; value: number }>;
}

export interface CalendarDay {
  day: number;
  isToday: boolean;
  isEmpty: boolean;
  subscriptions: Array<{ id: string; name: string; price: number; currencyId: string }>;
}

export interface AppData {
  subscriptions: Subscription[];
  expenses: Expense[];
  categories: Category[];
  currencies: Currency[];
  household: HouseholdMember[];
  paymentMethods: PaymentMethod[];
  tags: Tag[];
  settings: Settings;
  ratesApiKey: string;
  ratesProvider: string;
  fixerApiKey: string;
  fixerProvider: number;
  telegramBotToken: string;
  telegramChatId: string;
  telegramProxyUrl: string;
  telegramEnabled: boolean;
  initialized: boolean;
}

function asString(v: unknown, fallback = ""): string {
  return typeof v === "string" ? v : fallback;
}
function asNumber(v: unknown, fallback = 0): number {
  return typeof v === "number" && Number.isFinite(v) ? v : fallback;
}
function asBool(v: unknown, fallback = false): boolean {
  return typeof v === "boolean" ? v : fallback;
}
function asArray<T>(v: unknown, map: (item: unknown, index: number) => T): T[] {
  if (!Array.isArray(v)) return [];
  return v.map(map);
}

function parseSubscriptionCredentials(raw: unknown): SubscriptionCredentials | undefined {
  if (raw == null || typeof raw !== "object") return undefined;
  const c = raw as Record<string, unknown>;
  const login = asString(c.login);
  const password = asString(c.password);
  const totpSecret = asString(c.totpSecret);
  if (!login.trim() && !password && !totpSecret.trim()) return undefined;
  return { login, password, totpSecret };
}

export function parseSubscription(raw: unknown): Subscription {
  const r = (raw ?? {}) as Record<string, unknown>;
  const credentials = parseSubscriptionCredentials(r.credentials);
  return {
    id: asString(r.id),
    name: asString(r.name),
    logo: asString(r.logo),
    price: asNumber(r.price, 0),
    currencyId: asString(r.currencyId),
    nextPayment: asString(r.nextPayment),
    startDate: asString(r.startDate),
    cycle: ([1, 2, 3, 4].includes(Number(r.cycle)) ? Number(r.cycle) : 3) as CycleType,
    frequency: Math.max(1, Math.trunc(asNumber(r.frequency, 1))),
    notes: asString(r.notes),
    paymentMethodId: asString(r.paymentMethodId),
    payerUserId: asString(r.payerUserId),
    categoryId: asString(r.categoryId, "cat-1"),
    notify: asBool(r.notify, true),
    notifyDaysBefore: asNumber(r.notifyDaysBefore, 1),
    lastNotifiedDate: asString(r.lastNotifiedDate),
    inactive: asBool(r.inactive, false),
    autoRenew: asBool(r.autoRenew, true),
    url: asString(r.url),
    cancellationDate: r.cancellationDate == null ? null : asString(r.cancellationDate),
    replacementSubscriptionId: r.replacementSubscriptionId == null ? null : asString(r.replacementSubscriptionId),
    createdAt: asString(r.createdAt),
    tags: asArray<string>(r.tags, (item) => asString(item)).filter(Boolean),
    favorite: asBool(r.favorite, false),
    paymentHistory: asArray<PaymentRecord>(r.paymentHistory, (item) => {
      const p = (item ?? {}) as Record<string, unknown>;
      return {
        id: asString(p.id),
        date: asString(p.date),
        amount: asNumber(p.amount),
        currencyId: asString(p.currencyId),
        note: asString(p.note),
      };
    }),
    ...(credentials ? { credentials } : {}),
  };
}

function parseCredentialsMeta(raw: unknown): SubscriptionCredentialsMeta {
  const r = (raw ?? {}) as Record<string, unknown>;
  return {
    hasLogin: asBool(r.hasLogin, false),
    hasPassword: asBool(r.hasPassword, false),
    hasTotp: asBool(r.hasTotp, false),
  };
}

/** Строка списка подписок из `list_subscriptions_page`. */
export function parseSubscriptionListItem(raw: unknown): SubscriptionListItem {
  const r = (raw ?? {}) as Record<string, unknown>;
  const base = parseSubscription(raw);
  return {
    ...base,
    monthlyPrice: asNumber(r.monthlyPrice, 0),
    daysLeft: asNumber(r.daysLeft, 0),
    overdue: asBool(r.overdue, false),
    credentialsMeta: parseCredentialsMeta(r.credentialsMeta),
  };
}

export function parseExpense(raw: unknown): Expense {
  const r = (raw ?? {}) as Record<string, unknown>;
  const createdAt = expenseCreatedAtFromRaw(r);
  const updatedRaw = r.updatedAt;
  const updatedAt =
    typeof updatedRaw === "number" && Number.isFinite(updatedRaw)
      ? updatedRaw
      : typeof updatedRaw === "string" && updatedRaw.trim() !== "" && Number.isFinite(Number(updatedRaw))
        ? Number(updatedRaw)
        : undefined;
  return {
    id: asString(r.id),
    name: asString(r.name),
    amount: asNumber(r.amount),
    currencyId: asString(r.currencyId),
    categoryId: asString(r.categoryId, "cat-1"),
    paymentMethodId: asString(r.paymentMethodId),
    payerUserId: asString(r.payerUserId),
    tags: asArray<string>(r.tags, (item) => asString(item)).filter(Boolean),
    notes: asString(r.notes),
    url: asString(r.url),
    createdAt,
    subscriptionId: asString(r.subscriptionId),
    paymentRecordId: asString(r.paymentRecordId),
    ...(updatedAt !== undefined ? { updatedAt } : {}),
  };
}

/**
 * Payload for `subscriptions_upsert` / loose imports: camelCase like Rust `SubscriptionInputDto`;
 * extra JSON fields are ignored by serde on the backend.
 * Uses a loose object type so form numeric fields (e.g. `cycle`) stay compatible with IPC JSON.
 */
export type SubscriptionUpsertPayload = Record<string, unknown>;

/**
 * Payload for `expenses_upsert`: camelCase like Rust `ExpenseInputDto`; extra keys ignored server-side.
 */
export type ExpenseUpsertPayload = Partial<Expense> & Record<string, unknown>;

/**
 * Structural checks aligned with Rust `validate_import_payload` (required catalogs, main currency, currency codes).
 * Does not fully deserialize rows — backend still applies `parse_import_payload_json` + validation on import.
 */
export function validateImportData(raw: unknown): AppData | null {
  const parsed = importJsonPayloadSchema.safeParse(raw);
  if (!parsed.success) return null;
  return raw as AppData;
}
