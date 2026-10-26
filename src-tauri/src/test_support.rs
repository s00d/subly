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
        price: crate::money::from_f64_legacy(19.99),
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
            amount: crate::money::from_f64_legacy(19.99),
            currency_id: main_currency.clone(),
            note: "first payment".to_string(),
        }],
    });

    doc.expenses.push(ExpenseDoc {
        id: "exp-test-1".to_string(),
        updated_at: 1,
        name: "Restart smoke expense".to_string(),
        amount: crate::money::from_f64_legacy(7.5),
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
    AppConfigDoc {
        initialized: true,
        ..Default::default()
    }
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

/// Dev helper, not a check: `SUBLY_DATA_DIR=/tmp/subly-demo cargo test --lib write_demo_db -- --ignored`
/// writes a realistic demo DB for UI work (`SUBLY_DATA_DIR=… pnpm tauri dev`). Dates are relative to today.
#[test]
#[ignore]
fn write_demo_db() {
    use chrono::{Duration, Local};
    let dir = std::env::var("SUBLY_DATA_DIR").expect("set SUBLY_DATA_DIR");
    std::fs::create_dir_all(&dir).unwrap();
    let path = std::path::Path::new(&dir).join("subly.redb");
    let _ = std::fs::remove_file(&path);
    let db = Database::create(&path).unwrap();

    let mut doc = base_seeded_doc().unwrap();
    let today = Local::now().date_naive();
    let ymd = |d: chrono::NaiveDate| d.format("%Y-%m-%d").to_string();
    let cat = |i: usize| doc.categories[i % doc.categories.len()].id.clone();
    let pm = |i: usize| doc.payment_methods[i % doc.payment_methods.len()].id.clone();
    let cur = |i: usize| doc.currencies[i % doc.currencies.len()].id.clone();
    let payer = doc.household.first().map(|h| h.id.clone()).unwrap_or_default();

    // name, price, currency idx, cycle (1 day/2 week/3 month/4 year), days until next payment, category idx, auto_renew
    let subs: &[(&str, &str, usize, u8, i64, usize, bool)] = &[
        ("Netflix", "15.49", 1, 3, 2, 1, true),
        ("Spotify Family", "16.99", 1, 3, 5, 1, true),
        ("iCloud+ 200 GB", "2.99", 1, 3, 9, 2, true),
        ("GitHub Copilot", "10.00", 1, 3, 12, 3, true),
        ("Figma Professional", "144.00", 1, 4, 47, 3, true),
        ("YouTube Premium", "13.99", 1, 3, 0, 1, true),
        ("Notion Plus", "96.00", 1, 4, 120, 3, true),
        ("1Password Families", "59.88", 1, 4, 201, 2, true),
        ("Gym membership", "45.00", 0, 3, -3, 4, false),
        ("Duolingo Super", "6.99", 1, 3, 18, 5, true),
        ("ChatGPT Plus", "20.00", 1, 3, 22, 3, true),
        ("Disney+", "9.99", 1, 3, 26, 1, true),
    ];
    for (i, (name, price, ci, cycle, days, cat_i, auto)) in subs.iter().enumerate() {
        let next = today + Duration::days(*days);
        doc.subscriptions.push(SubscriptionDoc {
            id: format!("demo-sub-{i}"),
            updated_at: 1,
            name: name.to_string(),
            logo: String::new(),
            price: price.parse().unwrap(),
            currency_id: cur(*ci),
            next_payment: ymd(next),
            start_date: ymd(today - Duration::days(400)),
            cycle: *cycle,
            frequency: 1,
            inactive: false,
            category_id: cat(*cat_i),
            payment_method_id: pm(i),
            payer_user_id: payer.clone(),
            cancellation_date: None,
            notes: String::new(),
            notify: true,
            notify_days_before: -1,
            last_notified_date: String::new(),
            auto_renew: *auto,
            url: String::new(),
            replacement_subscription_id: None,
            created_at: format!("{}T10:00:00.000Z", ymd(today - Duration::days(400))),
            tags: vec![],
            favorite: i < 2,
            payment_history: vec![],
        });
    }

    let names = [
        ("Groceries", 6), ("Coffee", 7), ("Taxi", 8), ("Lunch", 7), ("Pharmacy", 4),
        ("Books", 5), ("Cinema", 1), ("Fuel", 8), ("Restaurant", 7), ("Electronics", 3),
    ];
    for k in 0..72_i64 {
        let (name, cat_i) = names[(k as usize * 7) % names.len()];
        let amount = format!("{}.{:02}", 3 + (k * 37) % 120, (k * 13) % 100);
        let day = today - Duration::days(k * 90 / 72);
        doc.expenses.push(ExpenseDoc {
            id: format!("demo-exp-{k}"),
            updated_at: 1,
            name: name.to_string(),
            amount: amount.parse().unwrap(),
            currency_id: cur(1),
            category_id: cat(cat_i),
            tags: vec![],
            payment_method_id: pm(k as usize),
            payer_user_id: payer.clone(),
            notes: String::new(),
            url: String::new(),
            created_at: format!("{}T12:00:00.000Z", ymd(day)),
            subscription_id: String::new(),
            payment_record_id: String::new(),
        });
    }

    crate::state::save_app_data_typed(&db, &doc, &default_config()).unwrap();
    println!("demo DB: {}", path.display());
}
