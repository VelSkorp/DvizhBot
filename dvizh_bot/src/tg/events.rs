use crate::application::Application;
use crate::db::db_objects::{Event, User};
use crate::db::repository::DvizhRepository;
use crate::tg::messaging::send_request;
use crate::tg::msg_type_utils::{msg_type_to_str, MsgType};
use chrono::{Datelike, NaiveDate, Utc};
use log::{debug, error};
use std::collections::HashMap;

pub async fn perform_happy_birthday(
    app: &Application,
    dvizh_repo: &DvizhRepository,
    birthday: &str,
) {
    if let Ok(users) = dvizh_repo.get_users_by_birthday(&birthday) {
        for user in users {
            if let Ok(chats) = dvizh_repo.get_chats_for_user(&user.username) {
                for chat in chats {
                    send_happy_birthday(&app, &user, chat).await;
                }
            } else {
                error!("Failed to get chats for user: {user:#?}");
            }
        }
    } else {
        error!("Failed to retrieve users with birthday: {birthday}");
    }
}

pub async fn perform_events_reminder(app: &Application, dvizh_repo: &DvizhRepository) {
    if let Ok(events) = dvizh_repo.get_today_events() {
        for event in events {
            reminde_events(&app, &event).await;
        }
    } else {
        error!("Failed to retrieve today's events.");
    }
}

pub async fn reminde_events(app: &Application, event: &Event) {
    let template = app.language_cache.lock().await
        .get_translation_for_chat(&app.conf.db_path, event.group_id, "event_template")
        .unwrap_or("📅 *Event Title*: {title}\n🗓 *Date*: {date}\n📍 *Location*: {location}\n📖 *Description*: {description}\n".to_string());

    let message = template
        .replace("{title}", &event.title)
        .replace("{date}", &event.date)
        .replace("{location}", &event.location)
        .replace("{description}", &event.description);

    // Formatting the message for the user
    let mut params = HashMap::new();
    params.insert("chat_id", event.group_id.to_string());
    params.insert("text", message);

    // Sending a message to Telegram
    if let Err(e) = send_request(
        &app.client,
        &app.conf.tg_token,
        msg_type_to_str(&MsgType::SendMessage),
        &params,
    )
    .await
    {
        error!(
            "Failed to send event reminder to chat {}: {}",
            event.group_id, e
        );
    }
}

pub async fn send_happy_birthday(app: &Application, user: &User, chat_id: i64) {
    let template = app.language_cache.lock().await
        .get_translation_for_chat(&app.conf.db_path, chat_id, "birthday_template")
        .unwrap_or("Happy Birthday to {first_name} (@{username}) 🎉 You've turned {age} years old! May this year be filled with joy, success, and happy moments! 🥳".to_string());

    let birth_date = NaiveDate::parse_from_str(&user.birthdate.clone().unwrap(), "%d.%m.%Y")
        .ok()
        .unwrap_or_default();
    let today = Utc::now().date_naive();
    let age = today.year() - birth_date.year();

    let message = template
        .replace(
            "{first_name}",
            &user.first_name.clone().unwrap_or("unknown :(".to_string()),
        )
        .replace("{username}", &user.username)
        .replace("{age}", &age.to_string());

    // Formatting the message for the user
    let mut params: HashMap<&str, String> = HashMap::new();
    params.insert("chat_id", chat_id.to_string());
    params.insert("text", message);

    // Sending a message to Telegram
    if let Err(e) = send_request(
        &app.client,
        &app.conf.tg_token,
        msg_type_to_str(&MsgType::SendMessage),
        &params,
    )
    .await
    {
        error!(
            "Failed to send birthday message to user {}: {}",
            user.username, e
        );
    }
}

pub async fn send_daily_greeting(
    app: &Application,
    key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(dvizh_repo) = DvizhRepository::new(&app.conf.db_path) {
        if let Ok(chats) = dvizh_repo.get_all_chat_ids() {
            for chat_id in chats {
                let message = app.language_cache.lock().await.get_translation_for_chat(
                    &app.conf.db_path,
                    chat_id,
                    key,
                )?;
                let mut params = HashMap::new();
                params.insert("chat_id", chat_id.to_string());
                params.insert("text", message.to_string());

                if let Err(e) = send_request(
                    &app.client,
                    &app.conf.tg_token,
                    msg_type_to_str(&MsgType::SendMessage),
                    &params,
                )
                .await
                {
                    error!("Failed to send daily greeting to chat {}: {}", chat_id, e);
                }
                debug!("Sent daily greeting: {message} in {chat_id}");
            }
        } else {
            error!("Failed to retrieve chat IDs for daily greetings.");
        }
    }
    Ok(())
}
