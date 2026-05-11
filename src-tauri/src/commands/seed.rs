use tauri::State;
use crate::AppState;
use crate::models::{AppDataDoc, CategoryDoc, CurrencyDoc, HouseholdMemberDoc, PaymentMethodDoc, SettingsDoc, TagDoc};

#[tauri::command]
pub fn seed_get_default_data() -> Result<AppDataDoc, crate::errors::AppError> {
    let categories = vec![
        CategoryDoc { id: "cat-1".to_string(), name: "No category".to_string(), icon: "".to_string(), sort_order: 0, i18n_key: "cat_no_category".to_string() },
        CategoryDoc { id: "cat-2".to_string(), name: "Entertainment".to_string(), icon: "".to_string(), sort_order: 1, i18n_key: "cat_entertainment".to_string() },
        CategoryDoc { id: "cat-3".to_string(), name: "Music".to_string(), icon: "".to_string(), sort_order: 2, i18n_key: "cat_music".to_string() },
        CategoryDoc { id: "cat-4".to_string(), name: "Utilities".to_string(), icon: "".to_string(), sort_order: 3, i18n_key: "cat_utilities".to_string() },
        CategoryDoc { id: "cat-5".to_string(), name: "Food & Beverages".to_string(), icon: "".to_string(), sort_order: 4, i18n_key: "cat_food_beverages".to_string() },
        CategoryDoc { id: "cat-6".to_string(), name: "Health & Wellbeing".to_string(), icon: "".to_string(), sort_order: 5, i18n_key: "cat_health".to_string() },
        CategoryDoc { id: "cat-7".to_string(), name: "Productivity".to_string(), icon: "".to_string(), sort_order: 6, i18n_key: "cat_productivity".to_string() },
        CategoryDoc { id: "cat-8".to_string(), name: "Banking".to_string(), icon: "".to_string(), sort_order: 7, i18n_key: "cat_banking".to_string() },
        CategoryDoc { id: "cat-9".to_string(), name: "Transport".to_string(), icon: "".to_string(), sort_order: 8, i18n_key: "cat_transport".to_string() },
        CategoryDoc { id: "cat-10".to_string(), name: "Education".to_string(), icon: "".to_string(), sort_order: 9, i18n_key: "cat_education".to_string() },
        CategoryDoc { id: "cat-11".to_string(), name: "Insurance".to_string(), icon: "".to_string(), sort_order: 10, i18n_key: "cat_insurance".to_string() },
        CategoryDoc { id: "cat-12".to_string(), name: "Gaming".to_string(), icon: "".to_string(), sort_order: 11, i18n_key: "cat_gaming".to_string() },
        CategoryDoc { id: "cat-13".to_string(), name: "News & Magazines".to_string(), icon: "".to_string(), sort_order: 12, i18n_key: "cat_news_magazines".to_string() },
        CategoryDoc { id: "cat-14".to_string(), name: "Software".to_string(), icon: "".to_string(), sort_order: 13, i18n_key: "cat_software".to_string() },
        CategoryDoc { id: "cat-15".to_string(), name: "Technology".to_string(), icon: "".to_string(), sort_order: 14, i18n_key: "cat_technology".to_string() },
        CategoryDoc { id: "cat-16".to_string(), name: "Cloud Services".to_string(), icon: "".to_string(), sort_order: 15, i18n_key: "cat_cloud_services".to_string() },
        CategoryDoc { id: "cat-17".to_string(), name: "Charity & Donations".to_string(), icon: "".to_string(), sort_order: 16, i18n_key: "cat_charity".to_string() },
    ];
    let currencies = vec![
        CurrencyDoc { id: "cur-1".to_string(), name: "Euro".to_string(), symbol: "€".to_string(), code: "EUR".to_string(), rate: 1.0, sort_order: 0, i18n_key: "cur_eur".to_string() },
        CurrencyDoc { id: "cur-2".to_string(), name: "US Dollar".to_string(), symbol: "$".to_string(), code: "USD".to_string(), rate: 1.0, sort_order: 1, i18n_key: "cur_usd".to_string() },
        CurrencyDoc { id: "cur-3".to_string(), name: "Japanese Yen".to_string(), symbol: "¥".to_string(), code: "JPY".to_string(), rate: 1.0, sort_order: 2, i18n_key: "cur_jpy".to_string() },
        CurrencyDoc { id: "cur-4".to_string(), name: "British Pound Sterling".to_string(), symbol: "£".to_string(), code: "GBP".to_string(), rate: 1.0, sort_order: 3, i18n_key: "cur_gbp".to_string() },
        CurrencyDoc { id: "cur-5".to_string(), name: "Swiss Franc".to_string(), symbol: "Fr".to_string(), code: "CHF".to_string(), rate: 1.0, sort_order: 4, i18n_key: "cur_chf".to_string() },
        CurrencyDoc { id: "cur-6".to_string(), name: "Canadian Dollar".to_string(), symbol: "CA$".to_string(), code: "CAD".to_string(), rate: 1.0, sort_order: 5, i18n_key: "cur_cad".to_string() },
        CurrencyDoc { id: "cur-7".to_string(), name: "Australian Dollar".to_string(), symbol: "A$".to_string(), code: "AUD".to_string(), rate: 1.0, sort_order: 6, i18n_key: "cur_aud".to_string() },
        CurrencyDoc { id: "cur-8".to_string(), name: "Chinese Yuan".to_string(), symbol: "¥".to_string(), code: "CNY".to_string(), rate: 1.0, sort_order: 7, i18n_key: "cur_cny".to_string() },
        CurrencyDoc { id: "cur-9".to_string(), name: "Indian Rupee".to_string(), symbol: "₹".to_string(), code: "INR".to_string(), rate: 1.0, sort_order: 8, i18n_key: "cur_inr".to_string() },
        CurrencyDoc { id: "cur-10".to_string(), name: "Russian Ruble".to_string(), symbol: "₽".to_string(), code: "RUB".to_string(), rate: 1.0, sort_order: 9, i18n_key: "cur_rub".to_string() },
        CurrencyDoc { id: "cur-11".to_string(), name: "Brazilian Real".to_string(), symbol: "R$".to_string(), code: "BRL".to_string(), rate: 1.0, sort_order: 10, i18n_key: "cur_brl".to_string() },
        CurrencyDoc { id: "cur-12".to_string(), name: "South Korean Won".to_string(), symbol: "₩".to_string(), code: "KRW".to_string(), rate: 1.0, sort_order: 11, i18n_key: "cur_krw".to_string() },
        CurrencyDoc { id: "cur-13".to_string(), name: "Mexican Peso".to_string(), symbol: "Mex$".to_string(), code: "MXN".to_string(), rate: 1.0, sort_order: 12, i18n_key: "cur_mxn".to_string() },
        CurrencyDoc { id: "cur-14".to_string(), name: "Singapore Dollar".to_string(), symbol: "S$".to_string(), code: "SGD".to_string(), rate: 1.0, sort_order: 13, i18n_key: "cur_sgd".to_string() },
        CurrencyDoc { id: "cur-15".to_string(), name: "Hong Kong Dollar".to_string(), symbol: "HK$".to_string(), code: "HKD".to_string(), rate: 1.0, sort_order: 14, i18n_key: "cur_hkd".to_string() },
        CurrencyDoc { id: "cur-16".to_string(), name: "Norwegian Krone".to_string(), symbol: "kr".to_string(), code: "NOK".to_string(), rate: 1.0, sort_order: 15, i18n_key: "cur_nok".to_string() },
        CurrencyDoc { id: "cur-17".to_string(), name: "Swedish Krona".to_string(), symbol: "kr".to_string(), code: "SEK".to_string(), rate: 1.0, sort_order: 16, i18n_key: "cur_sek".to_string() },
        CurrencyDoc { id: "cur-18".to_string(), name: "Danish Krone".to_string(), symbol: "kr".to_string(), code: "DKK".to_string(), rate: 1.0, sort_order: 17, i18n_key: "cur_dkk".to_string() },
        CurrencyDoc { id: "cur-19".to_string(), name: "New Zealand Dollar".to_string(), symbol: "NZ$".to_string(), code: "NZD".to_string(), rate: 1.0, sort_order: 18, i18n_key: "cur_nzd".to_string() },
        CurrencyDoc { id: "cur-20".to_string(), name: "Polish Zloty".to_string(), symbol: "zł".to_string(), code: "PLN".to_string(), rate: 1.0, sort_order: 19, i18n_key: "cur_pln".to_string() },
        CurrencyDoc { id: "cur-21".to_string(), name: "Turkish Lira".to_string(), symbol: "₺".to_string(), code: "TRY".to_string(), rate: 1.0, sort_order: 20, i18n_key: "cur_try".to_string() },
        CurrencyDoc { id: "cur-22".to_string(), name: "Thai Baht".to_string(), symbol: "฿".to_string(), code: "THB".to_string(), rate: 1.0, sort_order: 21, i18n_key: "cur_thb".to_string() },
        CurrencyDoc { id: "cur-23".to_string(), name: "Indonesian Rupiah".to_string(), symbol: "Rp".to_string(), code: "IDR".to_string(), rate: 1.0, sort_order: 22, i18n_key: "cur_idr".to_string() },
        CurrencyDoc { id: "cur-24".to_string(), name: "Hungarian Forint".to_string(), symbol: "Ft".to_string(), code: "HUF".to_string(), rate: 1.0, sort_order: 23, i18n_key: "cur_huf".to_string() },
        CurrencyDoc { id: "cur-25".to_string(), name: "Czech Republic Koruna".to_string(), symbol: "Kč".to_string(), code: "CZK".to_string(), rate: 1.0, sort_order: 24, i18n_key: "cur_czk".to_string() },
        CurrencyDoc { id: "cur-26".to_string(), name: "Israeli New Sheqel".to_string(), symbol: "₪".to_string(), code: "ILS".to_string(), rate: 1.0, sort_order: 25, i18n_key: "cur_ils".to_string() },
        CurrencyDoc { id: "cur-27".to_string(), name: "Philippine Peso".to_string(), symbol: "₱".to_string(), code: "PHP".to_string(), rate: 1.0, sort_order: 26, i18n_key: "cur_php".to_string() },
        CurrencyDoc { id: "cur-28".to_string(), name: "Malaysian Ringgit".to_string(), symbol: "RM".to_string(), code: "MYR".to_string(), rate: 1.0, sort_order: 27, i18n_key: "cur_myr".to_string() },
        CurrencyDoc { id: "cur-29".to_string(), name: "Romanian Leu".to_string(), symbol: "lei".to_string(), code: "RON".to_string(), rate: 1.0, sort_order: 28, i18n_key: "cur_ron".to_string() },
        CurrencyDoc { id: "cur-30".to_string(), name: "Bulgarian Lev".to_string(), symbol: "лв".to_string(), code: "BGN".to_string(), rate: 1.0, sort_order: 29, i18n_key: "cur_bgn".to_string() },
        CurrencyDoc { id: "cur-31".to_string(), name: "South African Rand".to_string(), symbol: "R".to_string(), code: "ZAR".to_string(), rate: 1.0, sort_order: 30, i18n_key: "cur_zar".to_string() },
        CurrencyDoc { id: "cur-32".to_string(), name: "Ukrainian Hryvnia".to_string(), symbol: "₴".to_string(), code: "UAH".to_string(), rate: 1.0, sort_order: 31, i18n_key: "cur_uah".to_string() },
        CurrencyDoc { id: "cur-33".to_string(), name: "Icelandic Krona".to_string(), symbol: "kr".to_string(), code: "ISK".to_string(), rate: 1.0, sort_order: 32, i18n_key: "cur_isk".to_string() },
        CurrencyDoc { id: "cur-34".to_string(), name: "New Taiwan Dollar".to_string(), symbol: "NT$".to_string(), code: "TWD".to_string(), rate: 1.0, sort_order: 33, i18n_key: "cur_twd".to_string() },
    ];
    let household = vec![
        HouseholdMemberDoc {
            id: "hm-1".to_string(),
            name: "Me".to_string(),
            email: "".to_string(),
            sort_order: 0,
        },
    ];
    let payment_methods = vec![
        PaymentMethodDoc { id: "pm-1".to_string(), name: "PayPal".to_string(), icon: "/assets/paypal.svg".to_string(), enabled: true, sort_order: 0, i18n_key: "".to_string() },
        PaymentMethodDoc { id: "pm-2".to_string(), name: "Visa".to_string(), icon: "/assets/visa.svg".to_string(), enabled: true, sort_order: 1, i18n_key: "".to_string() },
        PaymentMethodDoc { id: "pm-3".to_string(), name: "Mastercard".to_string(), icon: "/assets/mastercard.svg".to_string(), enabled: true, sort_order: 2, i18n_key: "".to_string() },
        PaymentMethodDoc { id: "pm-4".to_string(), name: "American Express".to_string(), icon: "/assets/american-express.svg".to_string(), enabled: true, sort_order: 3, i18n_key: "".to_string() },
        PaymentMethodDoc { id: "pm-5".to_string(), name: "Apple Pay".to_string(), icon: "/assets/apple-pay.svg".to_string(), enabled: true, sort_order: 4, i18n_key: "".to_string() },
        PaymentMethodDoc { id: "pm-6".to_string(), name: "Google Pay".to_string(), icon: "/assets/google-pay.svg".to_string(), enabled: true, sort_order: 5, i18n_key: "".to_string() },
        PaymentMethodDoc { id: "pm-7".to_string(), name: "Samsung Pay".to_string(), icon: "/assets/samsung-pay.svg".to_string(), enabled: true, sort_order: 6, i18n_key: "".to_string() },
        PaymentMethodDoc { id: "pm-8".to_string(), name: "Crypto".to_string(), icon: "/assets/crypto.svg".to_string(), enabled: true, sort_order: 7, i18n_key: "pm_crypto".to_string() },
        PaymentMethodDoc { id: "pm-9".to_string(), name: "Klarna".to_string(), icon: "/assets/klarna.svg".to_string(), enabled: true, sort_order: 8, i18n_key: "".to_string() },
        PaymentMethodDoc { id: "pm-10".to_string(), name: "Amazon Pay".to_string(), icon: "/assets/amazon-pay.svg".to_string(), enabled: true, sort_order: 9, i18n_key: "".to_string() },
        PaymentMethodDoc { id: "pm-11".to_string(), name: "SEPA".to_string(), icon: "/assets/sepa.svg".to_string(), enabled: true, sort_order: 10, i18n_key: "".to_string() },
        PaymentMethodDoc { id: "pm-12".to_string(), name: "Bank Transfer".to_string(), icon: "/assets/invoice.svg".to_string(), enabled: true, sort_order: 11, i18n_key: "pm_bank_transfer".to_string() },
        PaymentMethodDoc { id: "pm-13".to_string(), name: "Maestro".to_string(), icon: "/assets/maestro.svg".to_string(), enabled: true, sort_order: 12, i18n_key: "".to_string() },
        PaymentMethodDoc { id: "pm-14".to_string(), name: "Cash".to_string(), icon: "cash".to_string(), enabled: true, sort_order: 13, i18n_key: "pm_cash".to_string() },
        PaymentMethodDoc { id: "pm-15".to_string(), name: "Mir".to_string(), icon: "/assets/mir.svg".to_string(), enabled: true, sort_order: 14, i18n_key: "".to_string() },
        PaymentMethodDoc { id: "pm-16".to_string(), name: "SberPay".to_string(), icon: "/assets/sberpay.svg".to_string(), enabled: true, sort_order: 15, i18n_key: "".to_string() },
        PaymentMethodDoc { id: "pm-17".to_string(), name: "Tinkoff Pay".to_string(), icon: "/assets/tinkoff-pay.svg".to_string(), enabled: true, sort_order: 16, i18n_key: "".to_string() },
        PaymentMethodDoc { id: "pm-18".to_string(), name: "SBP".to_string(), icon: "/assets/sbp.svg".to_string(), enabled: true, sort_order: 17, i18n_key: "pm_sbp".to_string() },
        PaymentMethodDoc { id: "pm-19".to_string(), name: "YooMoney".to_string(), icon: "/assets/yoomoney.svg".to_string(), enabled: true, sort_order: 18, i18n_key: "".to_string() },
        PaymentMethodDoc { id: "pm-20".to_string(), name: "QIWI".to_string(), icon: "/assets/qiwi.svg".to_string(), enabled: true, sort_order: 19, i18n_key: "".to_string() },
        PaymentMethodDoc { id: "pm-21".to_string(), name: "UnionPay".to_string(), icon: "/assets/unionpay.svg".to_string(), enabled: true, sort_order: 20, i18n_key: "".to_string() },
        PaymentMethodDoc { id: "pm-22".to_string(), name: "WeChat Pay".to_string(), icon: "/assets/wechat-pay.svg".to_string(), enabled: true, sort_order: 21, i18n_key: "".to_string() },
        PaymentMethodDoc { id: "pm-23".to_string(), name: "Alipay".to_string(), icon: "/assets/alipay.svg".to_string(), enabled: true, sort_order: 22, i18n_key: "".to_string() },
    ];
    let tags = vec![
        TagDoc { id: "tag-1".to_string(), name: "Personal".to_string(), favorite: true, sort_order: 0, i18n_key: "tag_personal".to_string() },
        TagDoc { id: "tag-2".to_string(), name: "Work".to_string(), favorite: true, sort_order: 1, i18n_key: "tag_work".to_string() },
        TagDoc { id: "tag-3".to_string(), name: "Family".to_string(), favorite: true, sort_order: 2, i18n_key: "tag_family".to_string() },
        TagDoc { id: "tag-4".to_string(), name: "Trial".to_string(), favorite: false, sort_order: 3, i18n_key: "tag_trial".to_string() },
        TagDoc { id: "tag-5".to_string(), name: "Annual".to_string(), favorite: false, sort_order: 4, i18n_key: "tag_annual".to_string() },
        TagDoc { id: "tag-6".to_string(), name: "Monthly".to_string(), favorite: false, sort_order: 5, i18n_key: "tag_monthly".to_string() },
        TagDoc { id: "tag-7".to_string(), name: "Free Tier".to_string(), favorite: false, sort_order: 6, i18n_key: "tag_free_tier".to_string() },
        TagDoc { id: "tag-8".to_string(), name: "Can Cancel".to_string(), favorite: false, sort_order: 7, i18n_key: "tag_can_cancel".to_string() },
        TagDoc { id: "tag-9".to_string(), name: "Essential".to_string(), favorite: true, sort_order: 8, i18n_key: "tag_essential".to_string() },
        TagDoc { id: "tag-10".to_string(), name: "Optional".to_string(), favorite: false, sort_order: 9, i18n_key: "tag_optional".to_string() },
        TagDoc { id: "tag-11".to_string(), name: "Shared".to_string(), favorite: false, sort_order: 10, i18n_key: "tag_shared".to_string() },
        TagDoc { id: "tag-12".to_string(), name: "Tax Deductible".to_string(), favorite: false, sort_order: 11, i18n_key: "tag_tax_deductible".to_string() },
    ];
    Ok(AppDataDoc {
        subscriptions: Vec::new(),
        expenses: Vec::new(),
        categories,
        currencies,
        household,
        payment_methods,
        tags,
        settings: SettingsDoc {
            budget: 0.0,
            main_currency_id: "cur-2".to_string(),
            currency_update_targets: vec![],
            rate_history_days: 90,
        },
    })
}

#[tauri::command]
pub fn seed_apply_if_empty(state: State<'_, AppState>) -> Result<AppDataDoc, crate::errors::AppError> {
    let mut guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    let root = guard.doc()?;
    let needs_init = root.subscriptions.is_empty()
        && root.expenses.is_empty()
        && root.categories.is_empty()
        && root.currencies.is_empty()
        && root.household.is_empty()
        && root.payment_methods.is_empty()
        && root.tags.is_empty();
    if !needs_init {
        return Ok(root);
    }
    let seed = seed_get_default_data()?;
    guard.apply_snapshot_typed(&seed)?;
    Ok(seed)
}
