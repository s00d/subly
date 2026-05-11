use redb::Database;
use tempfile::TempDir;

use crate::commands::seed::seed_get_default_data;
use crate::models::{AppConfigDoc, AppDataDoc, ExpenseDoc, PaymentRecordDto, SubscriptionDoc};

pub(crate) fn temp_db() -> Result<(TempDir, Database), crate::errors::AppError> {
    let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    let db_path = dir.path().join("test.redb");
    let db = Database::create(db_path).map_err(|e| e.to_string())?;
    Ok((dir, db))
}

pub(crate) fn base_seeded_doc() -> Result<AppDataDoc, crate::errors::AppError> {
    seed_get_default_data()
}

pub(crate) fn doc_with_restart_sensitive_fields() -> Result<AppDataDoc, crate::errors::AppError> {
    let mut doc = base_seeded_doc()?;
    let main_currency = doc
        .currencies
        .first()
        .map(|x| x.id.clone())
        .ok_or("seed currencies are empty".to_string())?;
    let category_id = doc
        .categories
        .first()
        .map(|x| x.id.clone())
        .ok_or("seed categories are empty".to_string())?;
    let payment_method_id = doc
        .payment_methods
        .first()
        .map(|x| x.id.clone())
        .ok_or("seed payment methods are empty".to_string())?;
    let payer_user_id = doc
        .household
        .first()
        .map(|x| x.id.clone())
        .ok_or("seed household is empty".to_string())?;

    doc.subscriptions.push(SubscriptionDoc {
        id: "sub-test-1".to_string(),
        updated_at: 1,
        name: "Restart smoke subscription".to_string(),
        logo: "data:image/png;base64,iVBORw0KGgo=".to_string(),
        price: 19.99,
        currency_id: main_currency.clone(),
        next_payment: "2026-05-01".to_string(),
        start_date: "2026-04-01".to_string(),
        cycle: 3,
        frequency: 1,
        inactive: false,
        category_id: category_id.clone(),
        payment_method_id: payment_method_id.clone(),
        payer_user_id: payer_user_id.clone(),
        cancellation_date: None,
        notes: "keep-notes-after-restart".to_string(),
        notify: true,
        notify_days_before: 2,
        last_notified_date: String::new(),
        auto_renew: true,
        url: "https://example.com/subscription".to_string(),
        replacement_subscription_id: None,
        created_at: "2026-04-01T10:00:00.000Z".to_string(),
        tags: vec!["tag-1".to_string(), "tag-2".to_string()],
        favorite: true,
        payment_history: vec![PaymentRecordDto {
            id: "pay-1".to_string(),
            date: "2026-04-01".to_string(),
            amount: 19.99,
            currency_id: main_currency.clone(),
            note: "first payment".to_string(),
        }],
    });

    doc.expenses.push(ExpenseDoc {
        id: "exp-test-1".to_string(),
        updated_at: 1,
        name: "Restart smoke expense".to_string(),
        amount: 7.5,
        currency_id: main_currency,
        category_id,
        tags: vec!["tag-3".to_string()],
        payment_method_id,
        payer_user_id,
        notes: "expense-notes".to_string(),
        url: "https://example.com/expense".to_string(),
        created_at: "2026-04-15T10:00:00.000Z".to_string(),
        subscription_id: String::new(),
        payment_record_id: String::new(),
    });

    Ok(doc)
}

pub(crate) fn default_config() -> AppConfigDoc {
    let mut cfg = AppConfigDoc::default();
    cfg.initialized = true;
    cfg
}

pub(crate) fn assert_core_invariants(before: &AppDataDoc, after: &AppDataDoc) {
    assert_eq!(before.subscriptions.len(), after.subscriptions.len(), "subscriptions count changed");
    assert_eq!(before.expenses.len(), after.expenses.len(), "expenses count changed");
    assert_eq!(before.categories.len(), after.categories.len(), "categories count changed");
    assert_eq!(before.currencies.len(), after.currencies.len(), "currencies count changed");

    let lhs = &before.subscriptions[0];
    let rhs = &after.subscriptions[0];
    assert_eq!(lhs.logo, rhs.logo, "subscription logo changed after restart");
    assert_eq!(lhs.url, rhs.url, "subscription url changed after restart");
    assert_eq!(lhs.notes, rhs.notes, "subscription notes changed after restart");
    assert_eq!(lhs.tags, rhs.tags, "subscription tags changed after restart");
    assert_eq!(lhs.payment_history.len(), rhs.payment_history.len(), "payment history count changed");

    let lhs_exp = &before.expenses[0];
    let rhs_exp = &after.expenses[0];
    assert_eq!(lhs_exp.url, rhs_exp.url, "expense url changed after restart");
    assert_eq!(lhs_exp.notes, rhs_exp.notes, "expense notes changed after restart");
}
