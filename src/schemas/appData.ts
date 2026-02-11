import { z } from "zod";

// =============================================
// Zod-схемы — единый источник истины для типов
// =============================================

// ---- Cycle type ----
export const CycleTypeSchema = z.union([
  z.literal(1),
  z.literal(2),
  z.literal(3),
  z.literal(4),
]);

// ---- Payment record (history entry) ----
export const PaymentRecordSchema = z.object({
  id: z.string(),
  date: z.string(),            // ISO date YYYY-MM-DD when payment was made
  amount: z.number(),          // actual amount paid
  currencyId: z.string(),
  note: z.string().default(""),
});

// ---- Subscription ----
export const SubscriptionSchema = z.object({
  id: z.string(),
  name: z.string(),
  logo: z.string().default(""),
  price: z.number().default(0),
  currencyId: z.string(),
  nextPayment: z.string(), // ISO date YYYY-MM-DD
  startDate: z.string(),
  cycle: CycleTypeSchema.default(3),
  frequency: z.number().int().min(1).default(1),
  notes: z.string().default(""),
  paymentMethodId: z.string().default(""),
  payerUserId: z.string().default(""),
  categoryId: z.string().default("cat-1"),
  notify: z.boolean().default(true),
  notifyDaysBefore: z.number().default(1),
  lastNotifiedDate: z.string().default(""),   // ISO date of last sent notification (for recurring)
  inactive: z.boolean().default(false),
  autoRenew: z.boolean().default(true),
  url: z.string().default(""),
  cancellationDate: z.nullable(z.string()).default(null),
  replacementSubscriptionId: z.nullable(z.string()).default(null),
  createdAt: z.string().default(""),
  tags: z.array(z.string()).default([]),
  favorite: z.boolean().default(false),
  paymentHistory: z.array(PaymentRecordSchema).default([]),
});

// ---- Expense (one-time / irregular spending) ----
export const ExpenseSchema = z.object({
  id: z.string(),
  name: z.string(),
  amount: z.number().default(0),
  currencyId: z.string(),
  date: z.string(),  // ISO date YYYY-MM-DD
  categoryId: z.string().default("cat-1"),
  paymentMethodId: z.string().default(""),
  payerUserId: z.string().default(""),
  tags: z.array(z.string()).default([]),
  notes: z.string().default(""),
  createdAt: z.string().default(""),
});

// ---- Category ----
export const CategorySchema = z.object({
  id: z.string(),
  name: z.string(),
  icon: z.string().default(""),
  order: z.number().int().default(0),
  i18nKey: z.string().default(""),  // if set, name is auto-translated on lang switch
});

// ---- Currency ----
export const CurrencySchema = z.object({
  id: z.string(),
  name: z.string(),
  symbol: z.string(),
  code: z.string(),
  rate: z.number().default(1),
  order: z.number().int().default(0),
  i18nKey: z.string().default(""),
});

// ---- Household member ----
export const HouseholdMemberSchema = z.object({
  id: z.string(),
  name: z.string(),
  email: z.string().default(""),
  order: z.number().int().default(0),
});

// ---- Payment method ----
export const PaymentMethodSchema = z.object({
  id: z.string(),
  name: z.string(),
  icon: z.string().default("💳"),
  enabled: z.boolean().default(true),
  order: z.number().int().default(0),
  i18nKey: z.string().default(""),
});

// ---- Tag ----
export const TagSchema = z.object({
  id: z.string(),
  name: z.string(),
  favorite: z.boolean().default(true),
  order: z.number().int().default(0),
  i18nKey: z.string().default(""),
});

// ---- Custom colors ----
export const CustomColorsSchema = z.object({
  main: z.string().default(""),
  accent: z.string().default(""),
  hover: z.string().default(""),
});

// ---- Settings ----
export const SettingsSchema = z.object({
  darkTheme: z.union([z.literal(0), z.literal(1), z.literal(2)]).default(2),
  colorTheme: z.string().default("blue"),
  monthlyPrice: z.boolean().default(false),
  convertCurrency: z.boolean().default(false),
  hideDisabled: z.boolean().default(false),
  disabledToBottom: z.boolean().default(true),
  showOriginalPrice: z.boolean().default(false),
  showSubscriptionProgress: z.boolean().default(true),
  language: z.string().default("en"),
  mainCurrencyId: z.string().default("cur-2"),
  defaultCategoryId: z.string().default("cat-1"),
  defaultPaymentMethodId: z.string().default("pm-1"),
  budget: z.number().default(0),
  notifyDaysBefore: z.number().default(1),
  notificationTitle: z.string().default("Subly — Payment Reminder"),
  notificationBodyDueToday: z.string().default('Payment for "{name}" is due today!'),
  notificationBodyDueSoon: z.string().default('Payment for "{name}" is due in {days} day(s).'),
  notificationOverdueTitle: z.string().default("Subly — Overdue Payment"),
  notificationOverdueBody: z.string().default('"{name}" is overdue by {days} day(s). Please renew manually.'),
  notificationSchedule: z.enum(["any", "morning", "evening", "custom"]).default("any"),
  notificationCustomHour: z.number().int().min(0).max(23).default(9),
  recurringNotifications: z.boolean().default(true),
  currencyAutoUpdate: z.boolean().default(false),
  currencyUpdateTargets: z.array(z.string()).default([]),       // currency IDs to auto-update
  lastCurrencyUpdate: z.string().default(""),                   // ISO date of last rate fetch
  // Dashboard widgets config: array of { id, visible }; order = array order
  dashboardWidgets: z.array(z.object({
    id: z.string(),
    visible: z.boolean().default(true),
  })).default([]),
  // Subscription list view
  subscriptionViewMode: z.enum(["default", "compact", "expanded"]).default("default"),
  subscriptionGroupBy: z.enum(["none", "category", "payment_method"]).default("none"),
  customColors: CustomColorsSchema.default({ main: "", accent: "", hover: "" }),
});

// ---- Notification settings ----
export const NotificationSettingsSchema = z.object({
  enabled: z.boolean().default(true),
  daysBefore: z.number().default(1),
});

// ---- Stats data ----
const StatsEntrySchema = z.object({
  label: z.string(),
  value: z.number(),
});

export const StatsDataSchema = z.object({
  activeSubscriptions: z.number().default(0),
  inactiveSubscriptions: z.number().default(0),
  totalCostPerMonth: z.number().default(0),
  totalCostPerYear: z.number().default(0),
  averageSubscriptionCost: z.number().default(0),
  mostExpensive: z.nullable(
    z.object({ name: z.string(), price: z.number(), logo: z.string() }),
  ).default(null),
  amountDueThisMonth: z.number().default(0),
  totalSavingsPerMonth: z.number().default(0),
  budget: z.number().default(0),
  budgetUsed: z.nullable(z.number()).default(null),
  budgetLeft: z.nullable(z.number()).default(null),
  overBudgetAmount: z.nullable(z.number()).default(null),
  categoryCosts: z.array(StatsEntrySchema).default([]),
  memberCosts: z.array(StatsEntrySchema).default([]),
  paymentMethodCounts: z.array(StatsEntrySchema).default([]),
});

// ---- Calendar day ----
export const CalendarDaySchema = z.object({
  day: z.number(),
  isToday: z.boolean(),
  isEmpty: z.boolean(),
  subscriptions: z.array(
    z.object({
      id: z.string(),
      name: z.string(),
      price: z.number(),
      currencyId: z.string(),
    }),
  ).default([]),
});

// ---- Root AppData ----
export const AppDataSchema = z.object({
  subscriptions: z.array(SubscriptionSchema).default([]),
  expenses: z.array(ExpenseSchema).default([]),
  categories: z.array(CategorySchema).default([]),
  currencies: z.array(CurrencySchema).default([]),
  household: z.array(HouseholdMemberSchema).default([]),
  paymentMethods: z.array(PaymentMethodSchema).default([]),
  tags: z.array(TagSchema).default([]),
  settings: SettingsSchema.default({
    darkTheme: 2,
    colorTheme: "blue",
    monthlyPrice: false,
    convertCurrency: false,
    hideDisabled: false,
    disabledToBottom: true,
    showOriginalPrice: false,
    showSubscriptionProgress: true,
    language: "en",
    mainCurrencyId: "cur-2",
    defaultCategoryId: "cat-1",
    defaultPaymentMethodId: "pm-1",
    budget: 0,
    notifyDaysBefore: 1,
    notificationTitle: "Subly — Payment Reminder",
    notificationBodyDueToday: 'Payment for "{name}" is due today!',
    notificationBodyDueSoon: 'Payment for "{name}" is due in {days} day(s).',
    notificationOverdueTitle: "Subly — Overdue Payment",
    notificationOverdueBody: '"{name}" is overdue by {days} day(s). Please renew manually.',
    notificationSchedule: "any",
    notificationCustomHour: 9,
    recurringNotifications: true,
    currencyAutoUpdate: false,
    currencyUpdateTargets: [],
    lastCurrencyUpdate: "",
    dashboardWidgets: [],
    subscriptionViewMode: "default",
    subscriptionGroupBy: "none",
    customColors: { main: "", accent: "", hover: "" },
  }),
  fixerApiKey: z.string().default(""),
  fixerProvider: z.number().default(0),
  telegramBotToken: z.string().default(""),
  telegramChatId: z.string().default(""),
  telegramEnabled: z.boolean().default(false),
  initialized: z.boolean().default(true),
});

// =============================================
// Inferred TypeScript types
// =============================================
export type CycleType = z.infer<typeof CycleTypeSchema>;
export type PaymentRecord = z.infer<typeof PaymentRecordSchema>;
export type Subscription = z.infer<typeof SubscriptionSchema>;
export type Expense = z.infer<typeof ExpenseSchema>;
export type Category = z.infer<typeof CategorySchema>;
export type Currency = z.infer<typeof CurrencySchema>;
export type HouseholdMember = z.infer<typeof HouseholdMemberSchema>;
export type PaymentMethod = z.infer<typeof PaymentMethodSchema>;
export type Tag = z.infer<typeof TagSchema>;
export type CustomColors = z.infer<typeof CustomColorsSchema>;
export type Settings = z.infer<typeof SettingsSchema>;
export type NotificationSettings = z.infer<typeof NotificationSettingsSchema>;
export type StatsData = z.infer<typeof StatsDataSchema>;
export type CalendarDay = z.infer<typeof CalendarDaySchema>;
export type AppData = z.infer<typeof AppDataSchema>;

// =============================================
// Validation helpers
// =============================================

/**
 * Parse a single subscription (fill defaults for missing fields).
 */
export function parseSubscription(raw: unknown): Subscription {
  return SubscriptionSchema.parse(raw);
}

/**
 * Parse a single tag.
 */
export function parseTag(raw: unknown): Tag {
  return TagSchema.parse(raw);
}

/**
 * Parse a single expense.
 */
export function parseExpense(raw: unknown): Expense {
  return ExpenseSchema.parse(raw);
}

/**
 * Parse a single category.
 */
export function parseCategory(raw: unknown): Category {
  return CategorySchema.parse(raw);
}

/**
 * Parse a single currency.
 */
export function parseCurrency(raw: unknown): Currency {
  return CurrencySchema.parse(raw);
}

/**
 * Parse a single household member.
 */
export function parseHouseholdMember(raw: unknown): HouseholdMember {
  return HouseholdMemberSchema.parse(raw);
}

/**
 * Parse a single payment method.
 */
export function parsePaymentMethod(raw: unknown): PaymentMethod {
  return PaymentMethodSchema.parse(raw);
}

/**
 * Parse settings object.
 */
export function parseSettings(raw: unknown): Settings {
  return SettingsSchema.parse(raw);
}

/**
 * Validate and sanitize full AppData from storage.
 * Returns clean data with all missing fields filled with defaults.
 * If data is completely invalid, returns null.
 */
export function validateAppData(raw: unknown): AppData | null {
  const result = AppDataSchema.safeParse(raw);
  if (result.success) {
    return result.data;
  }
  console.warn("AppData validation failed, attempting partial recovery:", result.error.issues);

  // Try to recover: parse what we can with defaults for the rest
  if (raw && typeof raw === "object") {
    const obj = raw as Record<string, unknown>;
    try {
      return {
        subscriptions: safeParseArray(obj.subscriptions, SubscriptionSchema, []),
        expenses: safeParseArray(obj.expenses, ExpenseSchema, []),
        categories: safeParseArray(obj.categories, CategorySchema, []),
        currencies: safeParseArray(obj.currencies, CurrencySchema, []),
        household: safeParseArray(obj.household, HouseholdMemberSchema, []),
        paymentMethods: safeParseArray(obj.paymentMethods, PaymentMethodSchema, []),
        tags: migrateTags(obj.tags),
        settings: safeParseObj(obj.settings, SettingsSchema),
        fixerApiKey: typeof obj.fixerApiKey === "string" ? obj.fixerApiKey : "",
        fixerProvider: typeof obj.fixerProvider === "number" ? obj.fixerProvider : 0,
        telegramBotToken: typeof obj.telegramBotToken === "string" ? obj.telegramBotToken : "",
        telegramChatId: typeof obj.telegramChatId === "string" ? obj.telegramChatId : "",
        telegramEnabled: typeof obj.telegramEnabled === "boolean" ? obj.telegramEnabled : false,
        initialized: true,
      };
    } catch (e) {
      console.warn("Partial recovery failed:", e);
    }
  }
  return null;
}

/**
 * Validate imported JSON data. Returns validated data or null if unrecoverable.
 */
export function validateImportData(raw: unknown): AppData | null {
  if (!raw || typeof raw !== "object") return null;
  const obj = raw as Record<string, unknown>;
  // Must have at least subscriptions to be a valid import
  if (!Array.isArray(obj.subscriptions)) return null;
  return validateAppData(raw);
}

// ---- Internal helpers ----

function safeParseArray<T>(
  raw: unknown,
  schema: z.ZodType<T>,
  fallback: T[],
): T[] {
  if (!Array.isArray(raw)) return fallback;
  const result: T[] = [];
  for (const item of raw) {
    const parsed = schema.safeParse(item);
    if (parsed.success) {
      result.push(parsed.data);
    } else {
      console.warn("Skipping invalid array item:", parsed.error.issues);
    }
  }
  return result.length > 0 ? result : fallback;
}

function safeParseObj<T>(
  raw: unknown,
  schema: z.ZodType<T>,
): T {
  const result = schema.safeParse(raw ?? {});
  if (result.success) return result.data;
  // Return default by parsing empty object
  return schema.parse({});
}

/** Migrate tags from old string[] format to new Tag[] format */
function migrateTags(raw: unknown): Tag[] {
  if (!Array.isArray(raw)) return [];
  const result: Tag[] = [];
  for (let i = 0; i < raw.length; i++) {
    const item = raw[i];
    if (typeof item === "string") {
      // Old format: plain string => convert to Tag object
      result.push(TagSchema.parse({ id: `tag-migrated-${i}`, name: item, order: i, favorite: true }));
    } else {
      const parsed = TagSchema.safeParse(item);
      if (parsed.success) result.push(parsed.data);
    }
  }
  return result;
}
