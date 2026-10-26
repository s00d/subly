use chrono::{Local, NaiveDate, Timelike};
#[cfg(any(target_os = "android", target_os = "ios"))]
use chrono::{LocalResult, TimeZone};
use tauri::State;
use tauri::Emitter;
use tauri_plugin_notification::NotificationExt;
#[cfg(any(target_os = "android", target_os = "ios"))]
use tauri_plugin_notification::Schedule;
use crate::models::AppConfigDoc;
use crate::AppState;
use crate::state::{AppStateInner, EntityTable};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationScheduleItemDto {
    pub id: String,
    pub name: String,
    pub notify_date: String,
    pub next_payment: String,
    #[serde(default, with = "crate::money::serde_ipc")]
    pub price: rust_decimal::Decimal,
    #[serde(default)]
    pub currency_id: String,
    pub notification_id: i32,
    pub schedule_hour: u8,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationsRescheduleResultDto {
    pub ok: bool,
    pub scheduled_count: usize,
    pub scheduled: Vec<NotificationScheduleItemDto>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationsCancelResultDto {
    pub ok: bool,
    pub cancelled: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InAppAlertDto {
    pub id: String,
    pub subscription_id: String,
    pub subscription_name: String,
    pub r#type: String,
    pub days_until: i64,
    #[serde(with = "crate::money::serde_ipc")]
    pub price: rust_decimal::Decimal,
    pub currency_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationsRunCheckResultDto {
    pub sent_count: u32,
    pub alerts: Vec<InAppAlertDto>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationsDispatchResultDto {
    pub system: bool,
    pub sound: bool,
    pub telegram: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NotificationsEventPayloadDto {
    pub title: Option<String>,
    pub body: Option<String>,
    pub show_system: Option<bool>,
    pub play_sound: Option<bool>,
    pub force_sound: Option<bool>,
    pub send_telegram: Option<bool>,
    pub telegram_bot_token: Option<String>,
    pub telegram_chat_id: Option<String>,
    pub telegram_proxy_base_url: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum NotificationsEventDataDto {
    RunCheck(NotificationsRunCheckResultDto),
    Dispatch(NotificationsDispatchResultDto),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationsEventResponseDto {
    pub ok: bool,
    pub event: NotificationsEventKind,
    pub data: NotificationsEventDataDto,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationsEventKind {
    RunCheck,
    Dispatch,
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct NotificationSettingsDto {
    #[serde(default = "default_schedule_any")]
    notification_schedule: String,
    #[serde(default = "default_notification_hour")]
    notification_custom_hour: i64,
    #[serde(default = "default_notify_days_before")]
    notify_days_before: i64,
    #[serde(default = "default_recurring_true")]
    recurring_notifications: bool,
    #[serde(default = "default_due_title")]
    notification_title: String,
    #[serde(default = "default_overdue_title")]
    notification_overdue_title: String,
    #[serde(default)]
    notification_body_due_today: String,
    #[serde(default)]
    notification_body_due_soon: String,
    #[serde(default)]
    notification_overdue_body: String,
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct NotificationConfigDto {
    #[serde(default)]
    telegram_enabled: bool,
    #[serde(default)]
    telegram_bot_token: String,
    #[serde(default)]
    telegram_chat_id: String,
    #[serde(default)]
    telegram_proxy_url: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct LocalNotificationItemDto {
    title: String,
    body: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct LocalNotificationEventPayloadDto {
    title: String,
    body: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    items: Vec<LocalNotificationItemDto>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct TelegramSendMessagePayloadDto {
    chat_id: String,
    text: String,
}

fn default_schedule_any() -> String { "any".to_string() }
fn default_notification_hour() -> i64 { 9 }
fn default_notify_days_before() -> i64 { 3 }
fn default_recurring_true() -> bool { true }
fn default_due_title() -> String { "Subly - {name}".to_string() }
fn default_overdue_title() -> String { "Subly - Overdue: {name}".to_string() }

const FALLBACK_BODY_TODAY: &str = "Payment for \"{name}\" is due today.";
const FALLBACK_BODY_SOON: &str = "Payment for \"{name}\" is due in {days} day(s).";
const FALLBACK_BODY_OVERDUE: &str = "\"{name}\" is overdue by {days} day(s).";
const FALLBACK_TITLE_DUE: &str = "Subly - {name}";
const FALLBACK_TITLE_OVERDUE: &str = "Subly - Overdue: {name}";
#[cfg(any(target_os = "android", target_os = "ios"))]
const FALLBACK_SCHEDULE_TITLE: &str = "Subly - {name}";
#[cfg(any(target_os = "android", target_os = "ios"))]
const FALLBACK_SCHEDULE_BODY: &str = "Payment date: {due_date}";
const CONFIG_SETTINGS_KEY: &str = "config:settings";
/// Same account as `setSecureValue("telegramBotToken")` in `TelegramSection.vue`.
const SECURE_TELEGRAM_BOT_TOKEN: &str = "secure_storage.telegramBotToken";

fn unwrap_config_settings_value(raw: &str) -> Result<serde_json::Value, crate::errors::AppError> {
    match serde_json::from_str::<serde_json::Value>(raw) {
        Ok(serde_json::Value::String(inner)) => match serde_json::from_str::<serde_json::Value>(&inner) {
            Ok(v) => Ok(v),
            Err(_) => Ok(serde_json::Value::String(inner)),
        },
        Ok(v) => Ok(v),
        Err(e) => Err(crate::errors::AppError::Message(e.to_string())),
    }
}

/// UI notification prefs live in KV `config:settings` (JSON), not in Redb `settings_v2` (`SettingsDoc`).
fn load_notification_settings(
    guard: &AppStateInner,
) -> Result<NotificationSettingsDto, crate::errors::AppError> {
    let Some(raw) = guard.redb_get(CONFIG_SETTINGS_KEY)? else {
        return Ok(NotificationSettingsDto::default());
    };
    let value = unwrap_config_settings_value(&raw)?;
    serde_json::from_value(value).map_err(|e| crate::errors::AppError::Message(e.to_string()))
}

fn load_notification_config(
    guard: &AppStateInner,
) -> Result<NotificationConfigDto, crate::errors::AppError> {
    let cfg: AppConfigDoc = guard.config_typed()?;
    // The settings UI stores the token in the OS keyring; `AppConfigDoc` only has it on legacy installs.
    let telegram_bot_token = crate::keyring_store::get_for_background(SECURE_TELEGRAM_BOT_TOKEN)
        .ok()
        .flatten()
        .filter(|t| !t.trim().is_empty())
        .unwrap_or(cfg.telegram_bot_token);
    Ok(NotificationConfigDto {
        telegram_enabled: cfg.telegram_enabled,
        telegram_bot_token,
        telegram_chat_id: cfg.telegram_chat_id,
        telegram_proxy_url: if cfg.telegram_proxy_url.trim().is_empty() {
            None
        } else {
            Some(cfg.telegram_proxy_url)
        },
    })
}

struct NotificationTemplateContext<'a> {
    name: &'a str,
    days: i64,
    price: rust_decimal::Decimal,
    currency_id: &'a str,
    due_date: &'a str,
}

fn notification_template_price_string(price: rust_decimal::Decimal) -> String {
    crate::money::to_ipc_string(&price)
}

fn render_notification_placeholders(template: &str, ctx: &NotificationTemplateContext<'_>) -> String {
    template
        .replace("{name}", ctx.name)
        .replace("{days}", &ctx.days.to_string())
        .replace("{price}", &notification_template_price_string(ctx.price))
        .replace("{currency}", ctx.currency_id)
        .replace("{due_date}", ctx.due_date)
}

fn pick_notification_template<'a>(custom: &'a str, fallback: &'static str) -> &'a str {
    let t = custom.trim();
    if t.is_empty() { fallback } else { t }
}

async fn send_telegram_message(bot_token: &str, chat_id: &str, text: &str, proxy_base: Option<&str>) -> bool {
    if bot_token.is_empty() || chat_id.is_empty() {
        return false;
    }
    let base = proxy_base.unwrap_or("https://api.telegram.org");
    let url = format!("{}/bot{}/sendMessage", base.trim_end_matches('/'), bot_token);
    let payload = TelegramSendMessagePayloadDto {
        chat_id: chat_id.to_string(),
        text: text.to_string(),
    };
    crate::commands::sync::providers::http_client()
        .post(url)
        .json(&payload)
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

fn notification_schedule_allows_now(settings: &NotificationSettingsDto) -> bool {
    let schedule = settings.notification_schedule.as_str();
    if schedule == "any" {
        return true;
    }
    let now = Local::now();
    let hour = now.hour();
    match schedule {
        "morning" => (7..12).contains(&hour),
        "evening" => (17..22).contains(&hour),
        // From the chosen hour until midnight: a ±30 min window was missed whenever the check
        // did not run in it (sleeping laptop, throttled timers), dropping the reminder for the day.
        "custom" => i64::from(hour) >= settings.notification_custom_hour.clamp(0, 23),
        _ => true,
    }
}

/// Payment date reminders target. Auto-renewing subscriptions are not advanced in storage, so a
/// past `next_payment` is projected forward to the next cycle (on the fly, nothing is persisted).
fn effective_next_payment(sub: &crate::models::SubscriptionDoc, today: NaiveDate) -> Option<NaiveDate> {
    let stored = NaiveDate::parse_from_str(&sub.next_payment, "%Y-%m-%d").ok()?;
    if sub.auto_renew && stored < today {
        return crate::commands::subscriptions::first_cycle_date_on_or_after(
            stored,
            u64::from(sub.cycle),
            u64::from(sub.frequency),
            today,
        );
    }
    Some(stored)
}

/// Non-recurring mode marker for the overdue reminder; distinct from the upcoming marker
/// (the due date itself) so an "upcoming" push does not suppress the later "overdue" one.
fn overdue_marker(due_date: &str) -> String {
    format!("{due_date}:overdue")
}

fn emit_local_notification_digest(app: &tauri::AppHandle, items: Vec<LocalNotificationItemDto>) {
    if items.is_empty() {
        return;
    }
    let (title, body) = if items.len() == 1 {
        (items[0].title.clone(), items[0].body.clone())
    } else {
        (
            "Subly".to_string(),
            format!("{} payments need attention", items.len()),
        )
    };
    let payload = LocalNotificationEventPayloadDto {
        title,
        body,
        items,
    };
    let _ = app.emit("notifications:local", payload);
}

fn show_os_notification_immediate(app: &tauri::AppHandle, title: &str, body: &str) -> bool {
    match app
        .notification()
        .builder()
        .title(title)
        .body(body)
        .show()
    {
        Ok(()) => true,
        Err(e) => {
            eprintln!("[subly][notifications] OS show failed: {}", e);
            false
        }
    }
}

fn notification_id_for(subscription_id: &str, notify_date: &str) -> i32 {
    let mut hash: u32 = 2166136261;
    for b in format!("{subscription_id}:{notify_date}").as_bytes() {
        hash ^= u32::from(*b);
        hash = hash.wrapping_mul(16777619);
    }
    (hash & 0x7fff_ffff) as i32
}

fn schedule_hour_for_settings(settings: &NotificationSettingsDto) -> u8 {
    let schedule = settings.notification_schedule.as_str();
    match schedule {
        "morning" => 9,
        "evening" => 18,
        "custom" => settings.notification_custom_hour.clamp(0, 23) as u8,
        _ => 9,
    }
}

fn notified_for_current_payment(last_notified_date: &str, next_payment: &str) -> bool {
    !last_notified_date.is_empty() && last_notified_date == next_payment
}

fn build_schedule_preview(state: &State<'_, AppState>) -> Result<Vec<NotificationScheduleItemDto>, crate::errors::AppError> {
    let guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    let settings: NotificationSettingsDto = load_notification_settings(&guard)?;
    let subscriptions: Vec<crate::models::SubscriptionDoc> =
        guard.table_list_typed(EntityTable::Subscriptions)?;
    let default_days_before = settings.notify_days_before;
    let today = Local::now().date_naive();
    let mut out = Vec::new();
    let schedule_hour = schedule_hour_for_settings(&settings);
    for sub in subscriptions {
        if sub.inactive {
            continue;
        }
        if !sub.notify {
            continue;
        }
        let name = if sub.name.is_empty() {
            "Subscription".to_string()
        } else {
            sub.name.clone()
        };
        let notify_days_before = if sub.notify_days_before < 0 {
            default_days_before
        } else {
            sub.notify_days_before
        };
        if let Some(payment_date) = effective_next_payment(&sub, today) {
            let notify_date = payment_date - chrono::Duration::days(notify_days_before.clamp(0, 365));
            let notify_date_str = notify_date.format("%Y-%m-%d").to_string();
            let notification_id = notification_id_for(&sub.id, &notify_date_str);
            out.push(NotificationScheduleItemDto {
                id: sub.id.clone(),
                name,
                notify_date: notify_date_str,
                next_payment: payment_date.format("%Y-%m-%d").to_string(),
                price: sub.price,
                currency_id: sub.currency_id.clone(),
                notification_id,
                schedule_hour,
            });
        }
    }
    Ok(out)
}

pub(crate) fn notifications_reschedule_with_state(
    app: &tauri::AppHandle,
    state: &State<'_, AppState>,
) -> Result<NotificationsRescheduleResultDto, crate::errors::AppError> {
    let schedule = build_schedule_preview(state)?;
    let settings: NotificationSettingsDto = {
        let guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
        load_notification_settings(&guard)?
    };
    let previous_ids = {
        let guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
        guard.redb_get("notifications.scheduled.ids")?
    };
    if let Some(raw_ids) = previous_ids {
        #[cfg(any(target_os = "android", target_os = "ios"))]
        {
            let ids: Vec<i32> = serde_json::from_str(&raw_ids)
                .map_err(|e| format!("invalid notifications.scheduled.ids payload: {}", e))?;
            if let Err(e) = app.notification().cancel(ids) {
                eprintln!("[subly][notifications] cancel failed: {}", e);
            }
        }
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            let _ = &raw_ids;
        }
    }

    #[allow(unused_mut)] // mutated only on android/ios schedule path
    let mut scheduled_ids: Vec<i32> = Vec::new();
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        // Wall-clock `hour` on `day` in the user's timezone (not UTC). Using `and_utc()` on naive
        // values caused wrong instants and iOS `pastScheduledTime` when local hour had already passed.
        let future_instant = |day: NaiveDate, hour: u8| -> Option<time::OffsetDateTime> {
            let naive = day.and_hms_opt(u32::from(hour), 0, 0)?;
            let local_dt = match Local.from_local_datetime(&naive) {
                LocalResult::Single(dt) => dt,
                LocalResult::Ambiguous(earliest, _) => earliest,
                LocalResult::None => return None,
            };
            let date = time::OffsetDateTime::from_unix_timestamp(local_dt.timestamp()).ok()?;
            (date > time::OffsetDateTime::now_utc()).then_some(date)
        };
        for item in &schedule {
            let (Ok(payment_date), Ok(notify_date)) = (
                NaiveDate::parse_from_str(&item.next_payment, "%Y-%m-%d"),
                NaiveDate::parse_from_str(&item.notify_date, "%Y-%m-%d"),
            ) else {
                continue;
            };
            // Reminder time already passed (app opened inside the window): fall back to the
            // payment day so the user still gets one system notification for this cycle.
            let (date, days_to_payment) = match future_instant(notify_date, item.schedule_hour) {
                Some(d) => (d, (payment_date - notify_date).num_days()),
                None => match future_instant(payment_date, item.schedule_hour) {
                    Some(d) => (d, 0),
                    None => continue,
                },
            };
            let tpl_ctx = NotificationTemplateContext {
                name: item.name.as_str(),
                days: days_to_payment,
                price: item.price,
                currency_id: item.currency_id.as_str(),
                due_date: item.next_payment.as_str(),
            };
            let title_tpl = pick_notification_template(&settings.notification_title, FALLBACK_SCHEDULE_TITLE);
            let body_tpl = if days_to_payment == 0 {
                pick_notification_template(&settings.notification_body_due_today, FALLBACK_SCHEDULE_BODY)
            } else {
                pick_notification_template(&settings.notification_body_due_soon, FALLBACK_SCHEDULE_BODY)
            };
            let title = render_notification_placeholders(title_tpl, &tpl_ctx);
            let body = render_notification_placeholders(body_tpl, &tpl_ctx);
            match app
                .notification()
                .builder()
                .id(item.notification_id)
                .title(title)
                .body(body)
                .schedule(Schedule::At {
                    date,
                    repeating: false,
                    allow_while_idle: true,
                })
                .show()
            {
                Ok(()) => scheduled_ids.push(item.notification_id),
                Err(e) => {
                    eprintln!(
                        "[subly][notifications] failed to schedule {}: {}",
                        item.notification_id, e
                    );
                }
            }
        }
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let _ = &settings;
        let _ = app;
    }

    {
        let guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
        guard.redb_set(
            "notifications.scheduled.preview",
            &serde_json::to_string(&schedule).map_err(|e| e.to_string())?,
        )?;
        guard.redb_set(
            "notifications.scheduled.ids",
            &serde_json::to_string(&scheduled_ids).map_err(|e| e.to_string())?,
        )?;
    }
    Ok(NotificationsRescheduleResultDto {
        ok: true,
        scheduled_count: scheduled_ids.len(),
        scheduled: schedule,
    })
}

/// Desktop has no OS-scheduled reminders, so checks must run while the app is alive. The webview's
/// `setInterval` is throttled or suspended while the window is hidden in the tray, so the periodic
/// check runs here instead; results reach the UI via `notifications:alerts`.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(crate) fn ensure_desktop_notification_scheduler_started(app: tauri::AppHandle) {
    use tauri::Manager;
    static STARTED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    STARTED.get_or_init(|| {
        tauri::async_runtime::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(15 * 60)).await;
                let state = app.state::<AppState>();
                match notifications_run_check(app.clone(), state).await {
                    Ok(result) => {
                        let _ = app.emit("notifications:alerts", result.alerts);
                    }
                    Err(e) => eprintln!("[subly][notifications] scheduled check failed: {}", e),
                }
            }
        });
    });
}

/// Re-plan OS reminders after a bulk data replace (import, reset, sync pull). Best effort.
#[cfg(any(target_os = "android", target_os = "ios"))]
pub(crate) fn reschedule_from_app(app: &tauri::AppHandle) {
    use tauri::Manager;
    let state = app.state::<AppState>();
    if let Err(e) = notifications_reschedule_with_state(app, &state) {
        eprintln!("[subly][notifications] reschedule after data replace failed: {}", e);
    }
}

#[tauri::command]
pub fn notifications_cancel_all_scheduled(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<NotificationsCancelResultDto, crate::errors::AppError> {
    let guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
    if let Some(raw_ids) = guard.redb_get("notifications.scheduled.ids")? {
        #[cfg(any(target_os = "android", target_os = "ios"))]
        {
            let ids: Vec<i32> = serde_json::from_str(&raw_ids)
                .map_err(|e| format!("invalid notifications.scheduled.ids payload: {}", e))?;
            if let Err(e) = app.notification().cancel(ids) {
                eprintln!("[subly][notifications] cancel-all failed: {}", e);
            }
        }
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            let _ = &app;
            let _ = &raw_ids;
        }
    }
    guard.redb_delete("notifications.scheduled.preview")?;
    guard.redb_delete("notifications.scheduled.ids")?;
    Ok(NotificationsCancelResultDto {
        ok: true,
        cancelled: true,
    })
}

#[tauri::command]
pub fn notifications_reschedule_all(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<NotificationsRescheduleResultDto, crate::errors::AppError> {
    notifications_reschedule_with_state(&app, &state)
}

async fn notifications_run_check(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<NotificationsRunCheckResultDto, crate::errors::AppError> {
    let today = Local::now().date_naive();
    let today_str = today.format("%Y-%m-%d").to_string();
    let (sent_count, alerts, telegram_jobs, digest_items) = {
        let mut guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
        let settings: NotificationSettingsDto = load_notification_settings(&guard)?;
        let config: NotificationConfigDto = load_notification_config(&guard)?;
        let mut subscriptions: Vec<crate::models::SubscriptionDoc> =
            guard.table_list_typed(EntityTable::Subscriptions)?;
        let within_schedule = notification_schedule_allows_now(&settings);
        let recurring = settings.recurring_notifications;
        let notify_days_before = settings.notify_days_before;
        let title_due_tpl = pick_notification_template(&settings.notification_title, FALLBACK_TITLE_DUE);
        let title_overdue_tpl =
            pick_notification_template(&settings.notification_overdue_title, FALLBACK_TITLE_OVERDUE);
        let body_today_tpl =
            pick_notification_template(&settings.notification_body_due_today, FALLBACK_BODY_TODAY);
        let body_soon_tpl =
            pick_notification_template(&settings.notification_body_due_soon, FALLBACK_BODY_SOON);
        let body_overdue_tpl =
            pick_notification_template(&settings.notification_overdue_body, FALLBACK_BODY_OVERDUE);

        let mut sent_count = 0_u32;
        let mut alerts: Vec<InAppAlertDto> = Vec::new();
        let mut digest_items: Vec<LocalNotificationItemDto> = Vec::new();
        let mut dirty_ids: Vec<String> = Vec::new();
        let telegram_enabled = config.telegram_enabled;
        let telegram_token = config.telegram_bot_token;
        let telegram_chat_id = config.telegram_chat_id;
        let telegram_proxy = config.telegram_proxy_url;
        let mut telegram_jobs: Vec<String> = Vec::new();

        for sub in &mut subscriptions {
            if sub.inactive || !sub.notify {
                continue;
            }
            let Some(next_payment) = effective_next_payment(sub, today) else {
                continue;
            };
            let due_str = next_payment.format("%Y-%m-%d").to_string();
            let diff_days = (next_payment - today).num_days();
            let days_before = if sub.notify_days_before < 0 {
                notify_days_before
            } else {
                sub.notify_days_before
            };
            let notified_today = sub.last_notified_date == today_str;
            let cycle_notified = notified_for_current_payment(&sub.last_notified_date, &due_str);
            let overdue_notified = sub.last_notified_date == overdue_marker(&due_str);
            let should_send_push = within_schedule && !notified_today;
            let id = sub.id.clone();
            let name = if sub.name.is_empty() {
                "Subscription".to_string()
            } else {
                sub.name.clone()
            };
            let price = sub.price;
            let currency_id = sub.currency_id.clone();

            if diff_days >= 0 && diff_days <= days_before {
                let alert_type = if diff_days == 0 { "due_today" } else { "upcoming" };
                alerts.push(InAppAlertDto {
                    id: format!("{id}-{alert_type}-{due_str}"),
                    subscription_id: id.clone(),
                    subscription_name: name.clone(),
                    r#type: alert_type.to_string(),
                    days_until: diff_days,
                    price,
                    currency_id: currency_id.clone(),
                });
                let should_send_for_sub = if recurring {
                    should_send_push
                } else {
                    should_send_push && !cycle_notified
                };
                if should_send_for_sub {
                    let tpl_ctx = NotificationTemplateContext {
                        name: &name,
                        days: diff_days,
                        price,
                        currency_id: &currency_id,
                        due_date: &due_str,
                    };
                    let title = render_notification_placeholders(title_due_tpl, &tpl_ctx);
                    let body_tpl = if diff_days == 0 {
                        body_today_tpl
                    } else {
                        body_soon_tpl
                    };
                    let body = render_notification_placeholders(body_tpl, &tpl_ctx);
                    digest_items.push(LocalNotificationItemDto {
                        title: title.clone(),
                        body: body.clone(),
                    });
                    if telegram_enabled {
                        telegram_jobs.push(body);
                    }
                    sent_count += 1;
                    let next_last = if recurring {
                        today_str.clone()
                    } else {
                        due_str.clone()
                    };
                    if sub.last_notified_date != next_last {
                        sub.last_notified_date = next_last;
                        dirty_ids.push(id.clone());
                    }
                }
            }
            if !sub.auto_renew && diff_days < 0 {
                alerts.push(InAppAlertDto {
                    id: format!("{id}-overdue-{due_str}"),
                    subscription_id: id.clone(),
                    subscription_name: name.clone(),
                    r#type: "overdue".to_string(),
                    days_until: diff_days,
                    price,
                    currency_id: currency_id.clone(),
                });
                let should_send_overdue = if recurring {
                    should_send_push
                } else {
                    should_send_push && !overdue_notified
                };
                if should_send_overdue {
                    let tpl_ctx = NotificationTemplateContext {
                        name: &name,
                        days: diff_days.abs(),
                        price,
                        currency_id: &currency_id,
                        due_date: &due_str,
                    };
                    let title = render_notification_placeholders(title_overdue_tpl, &tpl_ctx);
                    let body = render_notification_placeholders(body_overdue_tpl, &tpl_ctx);
                    digest_items.push(LocalNotificationItemDto {
                        title: title.clone(),
                        body: body.clone(),
                    });
                    if telegram_enabled {
                        telegram_jobs.push(body);
                    }
                    sent_count += 1;
                    let next_last = if recurring {
                        today_str.clone()
                    } else {
                        overdue_marker(&due_str)
                    };
                    if sub.last_notified_date != next_last {
                        sub.last_notified_date = next_last;
                        if !dirty_ids.iter().any(|d| d == &id) {
                            dirty_ids.push(id.clone());
                        }
                    }
                }
            }
        }
        for sub in &subscriptions {
            if dirty_ids.iter().any(|d| d == &sub.id) {
                guard.table_upsert_typed(EntityTable::Subscriptions, sub, &sub.id)?;
            }
        }
        let telegram_context = if telegram_enabled {
            Some((telegram_token, telegram_chat_id, telegram_proxy))
        } else {
            None
        };
        (sent_count, alerts, (telegram_context, telegram_jobs), digest_items)
    };

    if !digest_items.is_empty() {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            let (title, body) = if digest_items.len() == 1 {
                (
                    digest_items[0].title.clone(),
                    digest_items[0].body.clone(),
                )
            } else {
                (
                    "Subly".to_string(),
                    format!("{} payments need attention", digest_items.len()),
                )
            };
            let _ = show_os_notification_immediate(&app, &title, &body);
        }
        emit_local_notification_digest(&app, digest_items);
    }

    if let (Some((token, chat_id, proxy)), jobs) = telegram_jobs {
        for message in jobs {
            let _ = send_telegram_message(&token, &chat_id, &message, proxy.as_deref()).await;
        }
    }
    // OS alarms are one-shot per cycle; without this the next cycle is only scheduled after the
    // user happens to edit a subscription or notification settings.
    #[cfg(any(target_os = "android", target_os = "ios"))]
    if let Err(e) = notifications_reschedule_with_state(&app, &state) {
        eprintln!("[subly][notifications] reschedule after check failed: {}", e);
    }
    Ok(NotificationsRunCheckResultDto {
        sent_count,
        alerts,
    })
}

#[allow(clippy::too_many_arguments)] // one arg per optional IPC payload field
async fn notifications_dispatch(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    title: Option<String>,
    body: Option<String>,
    show_system: Option<bool>,
    play_sound: Option<bool>,
    force_sound: Option<bool>,
    send_telegram: Option<bool>,
    telegram_bot_token: Option<String>,
    telegram_chat_id: Option<String>,
    telegram_proxy_base_url: Option<String>,
) -> Result<NotificationsDispatchResultDto, crate::errors::AppError> {
    let do_system = show_system.unwrap_or(true);
    let do_sound = play_sound.unwrap_or(false) || force_sound.unwrap_or(false);
    let do_telegram = send_telegram.unwrap_or(false);
    let title_s = title.unwrap_or_else(|| "Subly".to_string());
    let body_s = body.unwrap_or_default();

    let system = if do_system {
        let os_ok = show_os_notification_immediate(&app, &title_s, &body_s);
        emit_local_notification_digest(
            &app,
            vec![LocalNotificationItemDto {
                title: title_s.clone(),
                body: body_s.clone(),
            }],
        );
        os_ok
    } else if do_sound {
        emit_local_notification_digest(
            &app,
            vec![LocalNotificationItemDto {
                title: title_s.clone(),
                body: body_s.clone(),
            }],
        );
        false
    } else {
        false
    };

    let telegram_ok = if do_telegram {
        let fallback = {
            let guard = state.lock().map_err(|_| crate::errors::AppError::StateLockPoisoned)?;
            let data: NotificationConfigDto = load_notification_config(&guard)?;
            (
                data.telegram_bot_token,
                data.telegram_chat_id,
                data.telegram_proxy_url,
            )
        };
        let token = telegram_bot_token.unwrap_or(fallback.0);
        let chat = telegram_chat_id.unwrap_or(fallback.1);
        let proxy = telegram_proxy_base_url.or(fallback.2);
        let text = if body_s.is_empty() {
            "Subly notification"
        } else {
            body_s.as_str()
        };
        send_telegram_message(&token, &chat, text, proxy.as_deref()).await
    } else {
        false
    };

    Ok(NotificationsDispatchResultDto {
        system,
        sound: do_sound,
        telegram: telegram_ok,
    })
}

#[tauri::command]
pub async fn notifications_event(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    event: NotificationsEventKind,
    payload: Option<NotificationsEventPayloadDto>,
) -> Result<NotificationsEventResponseDto, crate::errors::AppError> {
    let p = payload.unwrap_or_default();
    let data = match event {
        NotificationsEventKind::RunCheck => {
            NotificationsEventDataDto::RunCheck(notifications_run_check(app, state).await?)
        }
        NotificationsEventKind::Dispatch => NotificationsEventDataDto::Dispatch(notifications_dispatch(
            app,
            state,
            p.title,
            p.body,
            p.show_system,
            p.play_sound,
            p.force_sound,
            p.send_telegram,
            p.telegram_bot_token,
            p.telegram_chat_id,
            p.telegram_proxy_base_url,
        )
        .await?),
    };

    Ok(NotificationsEventResponseDto {
        ok: true,
        event,
        data,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        effective_next_payment, notified_for_current_payment, notification_schedule_allows_now, overdue_marker,
        NotificationSettingsDto,
    };
    use chrono::NaiveDate;

    fn ymd(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn monthly_sub(next_payment: &str, auto_renew: bool) -> crate::models::SubscriptionDoc {
        let mut sub = crate::test_support::doc_with_restart_sensitive_fields().unwrap().subscriptions[0].clone();
        sub.next_payment = next_payment.into();
        sub.cycle = 3;
        sub.frequency = 1;
        sub.auto_renew = auto_renew;
        sub
    }

    /// Stale auto-renew date must project to the upcoming cycle, otherwise "N days before"
    /// reminders never fire again after the first missed renewal.
    #[test]
    fn auto_renew_past_date_projects_to_next_cycle() {
        let sub = monthly_sub("2026-01-30", true);
        assert_eq!(effective_next_payment(&sub, ymd("2026-10-24")), Some(ymd("2026-10-30")));
        // Steps are taken from the anchor, so the 30th survives February.
        assert_eq!(effective_next_payment(&sub, ymd("2026-03-01")), Some(ymd("2026-03-30")));
        // Due today stays today.
        assert_eq!(effective_next_payment(&sub, ymd("2026-01-30")), Some(ymd("2026-01-30")));
    }

    #[test]
    fn manual_renew_past_date_stays_overdue() {
        let sub = monthly_sub("2026-01-30", false);
        assert_eq!(effective_next_payment(&sub, ymd("2026-10-24")), Some(ymd("2026-01-30")));
    }

    #[test]
    fn invalid_cycle_does_not_loop() {
        let mut sub = monthly_sub("2026-01-30", true);
        sub.cycle = 9;
        assert_eq!(effective_next_payment(&sub, ymd("2026-10-24")), None);
    }

    /// Non-recurring mode: the "upcoming" marker must not suppress the later "overdue" reminder.
    #[test]
    fn overdue_marker_is_distinct_from_upcoming_marker() {
        let due = "2026-10-20";
        assert!(notified_for_current_payment(due, due));
        assert_ne!(overdue_marker(due), due);
        assert!(!notified_for_current_payment(&overdue_marker(due), due));
    }

    #[test]
    fn custom_schedule_hour_zero_always_allows() {
        let s = NotificationSettingsDto {
            notification_schedule: "custom".into(),
            notification_custom_hour: 0,
            ..Default::default()
        };
        assert!(notification_schedule_allows_now(&s));
    }

    #[test]
    fn recurring_false_marks_cycle_by_next_payment() {
        assert!(!notified_for_current_payment("", "2099-01-15"));
        assert!(!notified_for_current_payment("2099-01-01", "2099-01-15"));
        assert!(notified_for_current_payment("2099-01-15", "2099-01-15"));
    }

    #[test]
    fn schedule_any_always_allows() {
        let s = NotificationSettingsDto {
            notification_schedule: "any".into(),
            ..Default::default()
        };
        assert!(notification_schedule_allows_now(&s));
    }
}
