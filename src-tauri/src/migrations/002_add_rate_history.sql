CREATE TABLE IF NOT EXISTS currency_rate_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    currency_id TEXT NOT NULL,
    rate REAL NOT NULL,
    recorded_at TEXT NOT NULL DEFAULT (date('now')),
    UNIQUE(currency_id, recorded_at)
);

CREATE INDEX IF NOT EXISTS idx_rate_history_currency ON currency_rate_history(currency_id);
CREATE INDEX IF NOT EXISTS idx_rate_history_date ON currency_rate_history(recorded_at);
