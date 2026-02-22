CREATE TABLE IF NOT EXISTS categories (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    icon TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL DEFAULT 0,
    i18n_key TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS currencies (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    symbol TEXT NOT NULL,
    code TEXT NOT NULL,
    rate REAL NOT NULL DEFAULT 1.0,
    sort_order INTEGER NOT NULL DEFAULT 0,
    i18n_key TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS household_members (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    email TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS payment_methods (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    icon TEXT NOT NULL DEFAULT '',
    enabled INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    i18n_key TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS tags (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    favorite INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    i18n_key TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS subscriptions (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    logo TEXT NOT NULL DEFAULT '',
    price REAL NOT NULL DEFAULT 0,
    currency_id TEXT NOT NULL,
    next_payment TEXT NOT NULL,
    start_date TEXT NOT NULL,
    cycle INTEGER NOT NULL DEFAULT 3,
    frequency INTEGER NOT NULL DEFAULT 1,
    notes TEXT NOT NULL DEFAULT '',
    payment_method_id TEXT NOT NULL DEFAULT '',
    payer_user_id TEXT NOT NULL DEFAULT '',
    category_id TEXT NOT NULL DEFAULT 'cat-1',
    notify INTEGER NOT NULL DEFAULT 1,
    notify_days_before INTEGER NOT NULL DEFAULT 1,
    last_notified_date TEXT NOT NULL DEFAULT '',
    inactive INTEGER NOT NULL DEFAULT 0,
    auto_renew INTEGER NOT NULL DEFAULT 1,
    url TEXT NOT NULL DEFAULT '',
    cancellation_date TEXT,
    replacement_subscription_id TEXT,
    created_at TEXT NOT NULL DEFAULT '',
    favorite INTEGER NOT NULL DEFAULT 0,
    tags TEXT NOT NULL DEFAULT '[]'
);

CREATE TABLE IF NOT EXISTS payment_records (
    id TEXT PRIMARY KEY NOT NULL,
    subscription_id TEXT NOT NULL,
    date TEXT NOT NULL,
    amount REAL NOT NULL,
    currency_id TEXT NOT NULL,
    note TEXT NOT NULL DEFAULT '',
    FOREIGN KEY (subscription_id) REFERENCES subscriptions(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS expenses (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    amount REAL NOT NULL DEFAULT 0,
    currency_id TEXT NOT NULL,
    date TEXT NOT NULL,
    category_id TEXT NOT NULL DEFAULT 'cat-1',
    payment_method_id TEXT NOT NULL DEFAULT '',
    payer_user_id TEXT NOT NULL DEFAULT '',
    tags TEXT NOT NULL DEFAULT '[]',
    notes TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT '',
    subscription_id TEXT NOT NULL DEFAULT '',
    payment_record_id TEXT NOT NULL DEFAULT '',
    url TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS config (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

-- Subscriptions indexes
CREATE INDEX IF NOT EXISTS idx_sub_category ON subscriptions(category_id);
CREATE INDEX IF NOT EXISTS idx_sub_currency ON subscriptions(currency_id);
CREATE INDEX IF NOT EXISTS idx_sub_next_payment ON subscriptions(next_payment);
CREATE INDEX IF NOT EXISTS idx_sub_inactive ON subscriptions(inactive);
CREATE INDEX IF NOT EXISTS idx_sub_favorite ON subscriptions(favorite);
CREATE INDEX IF NOT EXISTS idx_sub_payment_method ON subscriptions(payment_method_id);
CREATE INDEX IF NOT EXISTS idx_sub_replacement ON subscriptions(replacement_subscription_id);
CREATE INDEX IF NOT EXISTS idx_sub_active_next ON subscriptions(inactive, next_payment);
CREATE INDEX IF NOT EXISTS idx_sub_name ON subscriptions(name);
CREATE INDEX IF NOT EXISTS idx_sub_price ON subscriptions(price);

-- Payment records indexes
CREATE INDEX IF NOT EXISTS idx_pr_subscription ON payment_records(subscription_id);
CREATE INDEX IF NOT EXISTS idx_pr_date ON payment_records(date);

-- Expenses indexes
CREATE INDEX IF NOT EXISTS idx_exp_date ON expenses(date);
CREATE INDEX IF NOT EXISTS idx_exp_category ON expenses(category_id);
CREATE INDEX IF NOT EXISTS idx_exp_subscription ON expenses(subscription_id);
CREATE INDEX IF NOT EXISTS idx_exp_payment_record ON expenses(payment_record_id);
CREATE INDEX IF NOT EXISTS idx_exp_payment_method ON expenses(payment_method_id);
CREATE INDEX IF NOT EXISTS idx_exp_amount ON expenses(amount);
CREATE INDEX IF NOT EXISTS idx_exp_sub_pr ON expenses(subscription_id, payment_record_id);

-- Catalogs indexes
CREATE INDEX IF NOT EXISTS idx_cat_sort ON categories(sort_order);
CREATE INDEX IF NOT EXISTS idx_cur_sort ON currencies(sort_order);
CREATE INDEX IF NOT EXISTS idx_hm_sort ON household_members(sort_order);
CREATE INDEX IF NOT EXISTS idx_pm_sort ON payment_methods(sort_order);
CREATE INDEX IF NOT EXISTS idx_tag_sort ON tags(sort_order);
