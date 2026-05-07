use chrono::{Local, NaiveDate, Timelike};
use tauri::State;
use tauri::Emitter;
use tauri_plugin_notification::{NotificationExt, Schedule};
use crate::AppState;
use crate::state::EntityTable;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationScheduleItemDto {
    pub id: String,
    pub name: String,
    pub notify_date: String,
    pub next_payment: String,
    #[serde(default)]
    pub price: f64,
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
    pub price: f64,
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
struct LocalNotificationEventPayloadDto {
    title: String,
    body: String,
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
const FALLBACK_SCHEDULE_TITLE: &str = "Subly - {name}";
const FALLBACK_SCHEDULE_BODY: &str = "Payment date: {due_date}";

struct NotificationTemplateContext<'a> {
    name: &'a str,
    days: i64,
    price: f64,
    currency_id: &'a str,
    due_date: &'a str,
}

fn notification_template_price_string(price: f64) -> String {
    let rounded = (price * 100.0).round() / 100.0;
    if rounded.fract().abs() < f64::EPSILON {
        format!("{}", rounded as i64)
    } else {
        format!("{:.2}", rounded)
    }
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
    tauri_plugin_http::reqwest::Client::new()
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
        "custom" => {
            let target = settings.notification_custom_hour as i32;
            let now_minutes = (hour as i32) * 60 + now.minute() as i32;
            let target_minutes = target * 60;
            let delta = (now_minutes - target_minutes).abs();
            let min_delta = delta.min(1440 - delta);
            min_delta <= 30
        }
        _ => true,
    }
}

fn emit_local_notification_event(app: &tauri::AppHandle, title: &str, body: &str) {
    let payload = LocalNotificationEventPayloadDto {
        title: title.to_string(),
        body: body.to_string(),
    };
    let _ = app.emit(
        "notifications:local",
        payload,
    );
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

fn build_schedule_preview(state: &State<'_, AppState>) -> Result<Vec<NotificationScheduleItemDto>, String> {
    let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    let settings: NotificationSettingsDto = guard.settings_typed()?;
    let subscriptions: Vec<crate::models::SubscriptionDoc> =
        guard.table_list_typed(EntityTable::Subscriptions)?;
    let default_days_before = settings.notify_days_before;
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
        if let Ok(payment_date) = NaiveDate::parse_from_str(&sub.next_payment, "%Y-%m-%d") {
            let notify_date = payment_date - chrono::Duration::days(notify_days_before);
            let notify_date_str = notify_date.format("%Y-%m-%d").to_string();
            let notification_id = notification_id_for(&sub.id, &notify_date_str);
            out.push(NotificationScheduleItemDto {
                id: sub.id.clone(),
                name,
                notify_date: notify_date_str,
                next_payment: sub.next_payment.clone(),
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
) -> Result<NotificationsRescheduleResultDto, String> {
    let schedule = build_schedule_preview(state)?;
    let settings: NotificationSettingsDto = {
        let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
        guard.settings_typed()?
    };
    let previous_ids = {
        let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
        guard.redb_get("notifications.scheduled.ids")?
    };
    if let Some(raw_ids) = previous_ids {
        #[cfg(any(target_os = "android", target_os = "ios"))]
        {
            let ids: Vec<i32> = serde_json::from_str(&raw_ids)
                .map_err(|e| format!("invalid notifications.scheduled.ids payload: {}", e))?;
            app.notification()
                .cancel(ids)
                .map_err(|e| format!("failed to cancel scheduled notifications: {}", e))?;
        }
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            let _ = &raw_ids;
        }
    }

    let today = Local::now().date_naive();
    let mut scheduled_ids: Vec<i32> = Vec::new();
    for item in &schedule {
        let payment_date = NaiveDate::parse_from_str(&item.next_payment, "%Y-%m-%d").ok();
        let notify_date_parsed = NaiveDate::parse_from_str(&item.notify_date, "%Y-%m-%d").ok();
        let days_to_payment = match (payment_date, notify_date_parsed) {
            (Some(p), Some(n)) => (p - n).num_days(),
            _ => 0,
        };
        let tpl_ctx = NotificationTemplateContext {
            name: item.name.as_str(),
            days: days_to_payment,
            price: item.price,
            currency_id: item.currency_id.as_str(),
            due_date: item.next_payment.as_str(),
        };
        let title_tpl = pick_notification_template(&settings.notification_title, FALLBACK_SCHEDULE_TITLE);
        let body_tpl = pick_notification_template(&settings.notification_body_due_soon, FALLBACK_SCHEDULE_BODY);
        let title = render_notification_placeholders(title_tpl, &tpl_ctx);
        let body = render_notification_placeholders(body_tpl, &tpl_ctx);
        let schedule_hour = item.schedule_hour;
        let notify_date = match NaiveDate::parse_from_str(&item.notify_date, "%Y-%m-%d") {
            Ok(v) => v,
            Err(_) => continue,
        };
        if notify_date < today {
            continue;
        }
        let dt = notify_date
            .and_hms_opt(schedule_hour as u32, 0, 0)
            .ok_or("invalid schedule datetime".to_string())?;
        let ts = dt.and_utc().timestamp();
        let date = time::OffsetDateTime::from_unix_timestamp(ts).map_err(|e| e.to_string())?;
        let _ = app
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
            .map_err(|e| format!("failed to schedule notification {}: {}", item.notification_id, e))?;
        scheduled_ids.push(item.notification_id);
    }

    {
        let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
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

#[tauri::command]
pub fn notifications_cancel_all_scheduled(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<NotificationsCancelResultDto, String> {
    let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
    if let Some(raw_ids) = guard.redb_get("notifications.scheduled.ids")? {
        #[cfg(any(target_os = "android", target_os = "ios"))]
        {
            let ids: Vec<i32> = serde_json::from_str(&raw_ids)
                .map_err(|e| format!("invalid notifications.scheduled.ids payload: {}", e))?;
            app.notification()
                .cancel(ids)
                .map_err(|e| format!("failed to cancel scheduled notifications: {}", e))?;
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
) -> Result<NotificationsRescheduleResultDto, String> {
    notifications_reschedule_with_state(&app, &state)
}

async fn notifications_run_check(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<NotificationsRunCheckResultDto, String> {
    let today = Local::now().date_naive();
    let today_str = today.format("%Y-%m-%d").to_string();
    let (sent_count, alerts, telegram_jobs) = {
        let mut guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
        let settings: NotificationSettingsDto = guard.settings_typed()?;
        let config: NotificationConfigDto = guard.config_typed()?;
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
        let telegram_enabled = config.telegram_enabled;
        let telegram_token = config.telegram_bot_token;
        let telegram_chat_id = config.telegram_chat_id;
        let telegram_proxy = config.telegram_proxy_url;
        let mut telegram_jobs: Vec<String> = Vec::new();

        for sub in &mut subscriptions {
                if sub.inactive || !sub.notify {
                    continue;
                }
                let next_payment = match NaiveDate::parse_from_str(&sub.next_payment, "%Y-%m-%d") {
                    Ok(d) => d,
                    Err(_) => continue,
                };
                let diff_days = (next_payment - today).num_days();
                let days_before = if sub.notify_days_before < 0 {
                    notify_days_before
                } else {
                    sub.notify_days_before
                };
                let notified_today = sub.last_notified_date == today_str;
                let notified_for_current_payment =
                    !sub.last_notified_date.is_empty() && sub.last_notified_date >= sub.next_payment;
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
                        id: format!("{id}-{alert_type}"),
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
                        should_send_push && !notified_for_current_payment
                    };
                    if should_send_for_sub {
                        let tpl_ctx = NotificationTemplateContext {
                            name: &name,
                            days: diff_days,
                            price,
                            currency_id: &currency_id,
                            due_date: &sub.next_payment,
                        };
                        let title = render_notification_placeholders(title_due_tpl, &tpl_ctx);
                        let body_tpl = if diff_days == 0 {
                            body_today_tpl
                        } else {
                            body_soon_tpl
                        };
                        let body = render_notification_placeholders(body_tpl, &tpl_ctx);
                        emit_local_notification_event(&app, &title, &body);
                        if telegram_enabled {
                            telegram_jobs.push(body.clone());
                        }
                        sent_count += 1;
                        sub.last_notified_date = today_str.clone();
                    }
                }
                if !sub.auto_renew && diff_days < 0 {
                    alerts.push(InAppAlertDto {
                        id: format!("{id}-overdue"),
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
                        should_send_push && !notified_for_current_payment
                    };
                    if should_send_overdue {
                        let tpl_ctx = NotificationTemplateContext {
                            name: &name,
                            days: diff_days.abs(),
                            price,
                            currency_id: &currency_id,
                            due_date: &sub.next_payment,
                        };
                        let title = render_notification_placeholders(title_overdue_tpl, &tpl_ctx);
                        let body = render_notification_placeholders(body_overdue_tpl, &tpl_ctx);
                        emit_local_notification_event(&app, &title, &body);
                        if telegram_enabled {
                            telegram_jobs.push(body.clone());
                        }
                        sent_count += 1;
                        sub.last_notified_date = today_str.clone();
                    }
                }
        }
        for sub in &subscriptions {
            guard.table_upsert_typed(EntityTable::Subscriptions, sub, &sub.id)?;
        }
        let telegram_context = if telegram_enabled {
            Some((telegram_token, telegram_chat_id, telegram_proxy))
        } else {
            None
        };
        (sent_count, alerts, (telegram_context, telegram_jobs))
    };
    if let (Some((token, chat_id, proxy)), jobs) = telegram_jobs {
        for message in jobs {
            let _ = send_telegram_message(&token, &chat_id, &message, proxy.as_deref()).await;
        }
    }
    Ok(NotificationsRunCheckResultDto {
        sent_count,
        alerts,
    })
}

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
) -> Result<NotificationsDispatchResultDto, String> {
    let do_system = show_system.unwrap_or(true);
    let _ = play_sound;
    let _ = force_sound;
    let do_telegram = send_telegram.unwrap_or(false);

    let system = if do_system {
        emit_local_notification_event(
            &app,
            title.as_deref().unwrap_or("Subly"),
            body.as_deref().unwrap_or(""),
        );
        true
    } else {
        false
    };

    let telegram_ok = if do_telegram {
        let fallback = {
            let guard = state.lock().map_err(|_| "state lock poisoned".to_string())?;
            let data: NotificationConfigDto = guard.config_typed()?;
            (
                data.telegram_bot_token,
                data.telegram_chat_id,
                data.telegram_proxy_url,
            )
        };
        let token = telegram_bot_token.unwrap_or(fallback.0);
        let chat = telegram_chat_id.unwrap_or(fallback.1);
        let proxy = telegram_proxy_base_url.or(fallback.2);
        let text = body.as_deref().unwrap_or("Subly notification");
        send_telegram_message(&token, &chat, text, proxy.as_deref()).await
    } else {
        false
    };

    Ok(NotificationsDispatchResultDto {
        system,
        sound: false,
        telegram: telegram_ok,
    })
}

#[tauri::command]
pub async fn notifications_event(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    event: NotificationsEventKind,
    payload: Option<NotificationsEventPayloadDto>,
) -> Result<NotificationsEventResponseDto, String> {
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
