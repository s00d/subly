import type { AppData, Category, Currency, HouseholdMember, PaymentMethod, Tag, Settings } from "@/schemas/appData";
import {
  CategorySchema,
  CurrencySchema,
  HouseholdMemberSchema,
  PaymentMethodSchema,
  TagSchema,
  SettingsSchema,
  AppDataSchema,
} from "@/schemas/appData";

export function getDefaultCategories(): Category[] {
  return [
    CategorySchema.parse({ id: "cat-1", name: "No category", sortOrder: 0, i18nKey: "cat_no_category" }),
    CategorySchema.parse({ id: "cat-2", name: "Entertainment", sortOrder: 1, i18nKey: "cat_entertainment" }),
    CategorySchema.parse({ id: "cat-3", name: "Music", sortOrder: 2, i18nKey: "cat_music" }),
    CategorySchema.parse({ id: "cat-4", name: "Utilities", sortOrder: 3, i18nKey: "cat_utilities" }),
    CategorySchema.parse({ id: "cat-5", name: "Food & Beverages", sortOrder: 4, i18nKey: "cat_food_beverages" }),
    CategorySchema.parse({ id: "cat-6", name: "Health & Wellbeing", sortOrder: 5, i18nKey: "cat_health" }),
    CategorySchema.parse({ id: "cat-7", name: "Productivity", sortOrder: 6, i18nKey: "cat_productivity" }),
    CategorySchema.parse({ id: "cat-8", name: "Banking", sortOrder: 7, i18nKey: "cat_banking" }),
    CategorySchema.parse({ id: "cat-9", name: "Transport", sortOrder: 8, i18nKey: "cat_transport" }),
    CategorySchema.parse({ id: "cat-10", name: "Education", sortOrder: 9, i18nKey: "cat_education" }),
    CategorySchema.parse({ id: "cat-11", name: "Insurance", sortOrder: 10, i18nKey: "cat_insurance" }),
    CategorySchema.parse({ id: "cat-12", name: "Gaming", sortOrder: 11, i18nKey: "cat_gaming" }),
    CategorySchema.parse({ id: "cat-13", name: "News & Magazines", sortOrder: 12, i18nKey: "cat_news_magazines" }),
    CategorySchema.parse({ id: "cat-14", name: "Software", sortOrder: 13, i18nKey: "cat_software" }),
    CategorySchema.parse({ id: "cat-15", name: "Technology", sortOrder: 14, i18nKey: "cat_technology" }),
    CategorySchema.parse({ id: "cat-16", name: "Cloud Services", sortOrder: 15, i18nKey: "cat_cloud_services" }),
    CategorySchema.parse({ id: "cat-17", name: "Charity & Donations", sortOrder: 16, i18nKey: "cat_charity" }),
  ];
}

export function getDefaultCurrencies(): Currency[] {
  return [
    CurrencySchema.parse({ id: "cur-1", name: "Euro", symbol: "\u20AC", code: "EUR", rate: 1, sortOrder: 0, i18nKey: "cur_eur" }),
    CurrencySchema.parse({ id: "cur-2", name: "US Dollar", symbol: "$", code: "USD", rate: 1, sortOrder: 1, i18nKey: "cur_usd" }),
    CurrencySchema.parse({ id: "cur-3", name: "Japanese Yen", symbol: "\u00A5", code: "JPY", rate: 1, sortOrder: 2, i18nKey: "cur_jpy" }),
    CurrencySchema.parse({ id: "cur-4", name: "British Pound Sterling", symbol: "\u00A3", code: "GBP", rate: 1, sortOrder: 3, i18nKey: "cur_gbp" }),
    CurrencySchema.parse({ id: "cur-5", name: "Swiss Franc", symbol: "Fr", code: "CHF", rate: 1, sortOrder: 4, i18nKey: "cur_chf" }),
    CurrencySchema.parse({ id: "cur-6", name: "Canadian Dollar", symbol: "$", code: "CAD", rate: 1, sortOrder: 5, i18nKey: "cur_cad" }),
    CurrencySchema.parse({ id: "cur-7", name: "Australian Dollar", symbol: "$", code: "AUD", rate: 1, sortOrder: 6, i18nKey: "cur_aud" }),
    CurrencySchema.parse({ id: "cur-8", name: "Chinese Yuan", symbol: "\u00A5", code: "CNY", rate: 1, sortOrder: 7, i18nKey: "cur_cny" }),
    CurrencySchema.parse({ id: "cur-9", name: "Indian Rupee", symbol: "\u20B9", code: "INR", rate: 1, sortOrder: 8, i18nKey: "cur_inr" }),
    CurrencySchema.parse({ id: "cur-10", name: "Russian Ruble", symbol: "\u20BD", code: "RUB", rate: 1, sortOrder: 9, i18nKey: "cur_rub" }),
    CurrencySchema.parse({ id: "cur-11", name: "Brazilian Real", symbol: "R$", code: "BRL", rate: 1, sortOrder: 10, i18nKey: "cur_brl" }),
    CurrencySchema.parse({ id: "cur-12", name: "South Korean Won", symbol: "\u20A9", code: "KRW", rate: 1, sortOrder: 11, i18nKey: "cur_krw" }),
    CurrencySchema.parse({ id: "cur-13", name: "Mexican Peso", symbol: "Mex$", code: "MXN", rate: 1, sortOrder: 12, i18nKey: "cur_mxn" }),
    CurrencySchema.parse({ id: "cur-14", name: "Singapore Dollar", symbol: "S$", code: "SGD", rate: 1, sortOrder: 13, i18nKey: "cur_sgd" }),
    CurrencySchema.parse({ id: "cur-15", name: "Hong Kong Dollar", symbol: "HK$", code: "HKD", rate: 1, sortOrder: 14, i18nKey: "cur_hkd" }),
    CurrencySchema.parse({ id: "cur-16", name: "Norwegian Krone", symbol: "kr", code: "NOK", rate: 1, sortOrder: 15, i18nKey: "cur_nok" }),
    CurrencySchema.parse({ id: "cur-17", name: "Swedish Krona", symbol: "kr", code: "SEK", rate: 1, sortOrder: 16, i18nKey: "cur_sek" }),
    CurrencySchema.parse({ id: "cur-18", name: "Danish Krone", symbol: "kr", code: "DKK", rate: 1, sortOrder: 17, i18nKey: "cur_dkk" }),
    CurrencySchema.parse({ id: "cur-19", name: "New Zealand Dollar", symbol: "NZ$", code: "NZD", rate: 1, sortOrder: 18, i18nKey: "cur_nzd" }),
    CurrencySchema.parse({ id: "cur-20", name: "Polish Zloty", symbol: "z\u0142", code: "PLN", rate: 1, sortOrder: 19, i18nKey: "cur_pln" }),
    CurrencySchema.parse({ id: "cur-21", name: "Turkish Lira", symbol: "\u20BA", code: "TRY", rate: 1, sortOrder: 20, i18nKey: "cur_try" }),
    CurrencySchema.parse({ id: "cur-22", name: "Thai Baht", symbol: "\u0E3F", code: "THB", rate: 1, sortOrder: 21, i18nKey: "cur_thb" }),
    CurrencySchema.parse({ id: "cur-23", name: "Indonesian Rupiah", symbol: "Rp", code: "IDR", rate: 1, sortOrder: 22, i18nKey: "cur_idr" }),
    CurrencySchema.parse({ id: "cur-24", name: "Hungarian Forint", symbol: "Ft", code: "HUF", rate: 1, sortOrder: 23, i18nKey: "cur_huf" }),
    CurrencySchema.parse({ id: "cur-25", name: "Czech Republic Koruna", symbol: "K\u010D", code: "CZK", rate: 1, sortOrder: 24, i18nKey: "cur_czk" }),
    CurrencySchema.parse({ id: "cur-26", name: "Israeli New Sheqel", symbol: "\u20AA", code: "ILS", rate: 1, sortOrder: 25, i18nKey: "cur_ils" }),
    CurrencySchema.parse({ id: "cur-27", name: "Philippine Peso", symbol: "\u20B1", code: "PHP", rate: 1, sortOrder: 26, i18nKey: "cur_php" }),
    CurrencySchema.parse({ id: "cur-28", name: "Malaysian Ringgit", symbol: "RM", code: "MYR", rate: 1, sortOrder: 27, i18nKey: "cur_myr" }),
    CurrencySchema.parse({ id: "cur-29", name: "Romanian Leu", symbol: "lei", code: "RON", rate: 1, sortOrder: 28, i18nKey: "cur_ron" }),
    CurrencySchema.parse({ id: "cur-30", name: "Bulgarian Lev", symbol: "\u043B\u0432", code: "BGN", rate: 1, sortOrder: 29, i18nKey: "cur_bgn" }),
    CurrencySchema.parse({ id: "cur-31", name: "South African Rand", symbol: "R", code: "ZAR", rate: 1, sortOrder: 30, i18nKey: "cur_zar" }),
    CurrencySchema.parse({ id: "cur-32", name: "Ukrainian Hryvnia", symbol: "\u20B4", code: "UAH", rate: 1, sortOrder: 31, i18nKey: "cur_uah" }),
    CurrencySchema.parse({ id: "cur-33", name: "Icelandic Kr\u00F3na", symbol: "kr", code: "ISK", rate: 1, sortOrder: 32, i18nKey: "cur_isk" }),
    CurrencySchema.parse({ id: "cur-34", name: "New Taiwan Dollar", symbol: "NT$", code: "TWD", rate: 1, sortOrder: 33, i18nKey: "cur_twd" }),
  ];
}

export function getDefaultPaymentMethods(): PaymentMethod[] {
  return [
    PaymentMethodSchema.parse({ id: "pm-1", name: "PayPal", icon: "/assets/paypal.svg", enabled: true, sortOrder: 0 }),
    PaymentMethodSchema.parse({ id: "pm-2", name: "Visa", icon: "/assets/visa.svg", enabled: true, sortOrder: 1 }),
    PaymentMethodSchema.parse({ id: "pm-3", name: "Mastercard", icon: "/assets/mastercard.svg", enabled: true, sortOrder: 2 }),
    PaymentMethodSchema.parse({ id: "pm-4", name: "American Express", icon: "/assets/american-express.svg", enabled: true, sortOrder: 3 }),
    PaymentMethodSchema.parse({ id: "pm-5", name: "Apple Pay", icon: "/assets/apple-pay.svg", enabled: true, sortOrder: 4 }),
    PaymentMethodSchema.parse({ id: "pm-6", name: "Google Pay", icon: "/assets/google-pay.svg", enabled: true, sortOrder: 5 }),
    PaymentMethodSchema.parse({ id: "pm-7", name: "Samsung Pay", icon: "/assets/samsung-pay.svg", enabled: true, sortOrder: 6 }),
    PaymentMethodSchema.parse({ id: "pm-8", name: "Crypto", icon: "/assets/crypto.svg", enabled: true, sortOrder: 7, i18nKey: "pm_crypto" }),
    PaymentMethodSchema.parse({ id: "pm-9", name: "Klarna", icon: "/assets/klarna.svg", enabled: true, sortOrder: 8 }),
    PaymentMethodSchema.parse({ id: "pm-10", name: "Amazon Pay", icon: "/assets/amazon-pay.svg", enabled: true, sortOrder: 9 }),
    PaymentMethodSchema.parse({ id: "pm-11", name: "SEPA", icon: "/assets/sepa.svg", enabled: true, sortOrder: 10 }),
    PaymentMethodSchema.parse({ id: "pm-12", name: "Bank Transfer", icon: "/assets/invoice.svg", enabled: true, sortOrder: 11, i18nKey: "pm_bank_transfer" }),
    PaymentMethodSchema.parse({ id: "pm-13", name: "Maestro", icon: "/assets/maestro.svg", enabled: true, sortOrder: 12 }),
    PaymentMethodSchema.parse({ id: "pm-14", name: "Cash", icon: "💵", enabled: true, sortOrder: 13, i18nKey: "pm_cash" }),
    PaymentMethodSchema.parse({ id: "pm-15", name: "Mir", icon: "/assets/mir.svg", enabled: true, sortOrder: 14 }),
    PaymentMethodSchema.parse({ id: "pm-16", name: "SberPay", icon: "/assets/sberpay.svg", enabled: true, sortOrder: 15 }),
    PaymentMethodSchema.parse({ id: "pm-17", name: "Tinkoff Pay", icon: "/assets/tinkoff-pay.svg", enabled: true, sortOrder: 16 }),
    PaymentMethodSchema.parse({ id: "pm-18", name: "СБП", icon: "/assets/sbp.svg", enabled: true, sortOrder: 17, i18nKey: "pm_sbp" }),
    PaymentMethodSchema.parse({ id: "pm-19", name: "ЮMoney", icon: "/assets/yoomoney.svg", enabled: true, sortOrder: 18 }),
    PaymentMethodSchema.parse({ id: "pm-20", name: "QIWI", icon: "/assets/qiwi.svg", enabled: true, sortOrder: 19 }),
    PaymentMethodSchema.parse({ id: "pm-21", name: "UnionPay", icon: "/assets/unionpay.svg", enabled: true, sortOrder: 20 }),
    PaymentMethodSchema.parse({ id: "pm-22", name: "WeChat Pay", icon: "/assets/wechat-pay.svg", enabled: true, sortOrder: 21 }),
    PaymentMethodSchema.parse({ id: "pm-23", name: "Alipay", icon: "/assets/alipay.svg", enabled: true, sortOrder: 22 }),
  ];
}

export function getDefaultHousehold(): HouseholdMember[] {
  return [HouseholdMemberSchema.parse({ id: "hm-1", name: "Me", email: "", sortOrder: 0 })];
}

export function getDefaultSettings(): Settings {
  return SettingsSchema.parse({
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
    customColors: { main: "", accent: "", hover: "" },
  });
}

export function getDefaultTags(): Tag[] {
  return [
    TagSchema.parse({ id: "tag-1", name: "Personal", sortOrder: 0, favorite: true, i18nKey: "tag_personal" }),
    TagSchema.parse({ id: "tag-2", name: "Work", sortOrder: 1, favorite: true, i18nKey: "tag_work" }),
    TagSchema.parse({ id: "tag-3", name: "Family", sortOrder: 2, favorite: true, i18nKey: "tag_family" }),
    TagSchema.parse({ id: "tag-4", name: "Trial", sortOrder: 3, favorite: false, i18nKey: "tag_trial" }),
    TagSchema.parse({ id: "tag-5", name: "Annual", sortOrder: 4, favorite: false, i18nKey: "tag_annual" }),
    TagSchema.parse({ id: "tag-6", name: "Monthly", sortOrder: 5, favorite: false, i18nKey: "tag_monthly" }),
    TagSchema.parse({ id: "tag-7", name: "Free Tier", sortOrder: 6, favorite: false, i18nKey: "tag_free_tier" }),
    TagSchema.parse({ id: "tag-8", name: "Can Cancel", sortOrder: 7, favorite: false, i18nKey: "tag_can_cancel" }),
    TagSchema.parse({ id: "tag-9", name: "Essential", sortOrder: 8, favorite: true, i18nKey: "tag_essential" }),
    TagSchema.parse({ id: "tag-10", name: "Optional", sortOrder: 9, favorite: false, i18nKey: "tag_optional" }),
    TagSchema.parse({ id: "tag-11", name: "Shared", sortOrder: 10, favorite: false, i18nKey: "tag_shared" }),
    TagSchema.parse({ id: "tag-12", name: "Tax Deductible", sortOrder: 11, favorite: false, i18nKey: "tag_tax_deductible" }),
  ];
}

export function getDefaultData(): AppData {
  return AppDataSchema.parse({
    subscriptions: [],
    categories: getDefaultCategories(),
    currencies: getDefaultCurrencies(),
    household: getDefaultHousehold(),
    paymentMethods: getDefaultPaymentMethods(),
    tags: getDefaultTags(),
    settings: getDefaultSettings(),
    fixerApiKey: "",
    fixerProvider: 0,
    initialized: true,
  });
}
