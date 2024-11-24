use crate::application::Application;
use crate::db::db_objects::{Event, User};
use crate::tg::messaging::send_request;
use crate::tg::msg_type_utils::{msg_type_to_str, MsgType};
use anyhow::Result;
use chrono::{Datelike, NaiveDate, Utc};
use log::debug;
use std::collections::HashMap;

pub async fn perform_happy_birthday(
    app: &Application,
    birthday: &str,
) -> Result<()> {
    let users = app.dvizh_repo.lock().await.get_users_by_birthday(&birthday)?;
    for user in users {
        let chats = app.dvizh_repo.lock().await.get_chats_for_user(&user.username)?;
        for chat in chats {
            send_happy_birthday(&app, &user, chat).await?;
        }
    }
    Ok(())
}

pub async fn perform_events_reminder(
    app: &Application,
) -> Result<()> {
    let events = app.dvizh_repo.lock().await.get_today_events()?;
    for event in events {
        reminde_events(&app, &event).await?;
    }
    Ok(())
}

pub async fn reminde_events(app: &Application, event: &Event) -> Result<serde_json::Value> {
    let template = app
        .language_cache
        .write()
        .await
        .get_translation_for_chat(&app.dvizh_repo, event.group_id, "event_template")
        .await?;

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
    send_request(
        &app.client,
        &app.conf.tg_token,
        msg_type_to_str(&MsgType::SendMessage),
        &params,
    )
    .await
}

pub async fn send_happy_birthday(
    app: &Application,
    user: &User,
    chat_id: i64,
) -> Result<serde_json::Value> {
    let template = app
        .language_cache
        .write()
        .await
        .get_translation_for_chat(&app.dvizh_repo, chat_id, "birthday_template")
        .await?;

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
    send_request(
        &app.client,
        &app.conf.tg_token,
        msg_type_to_str(&MsgType::SendMessage),
        &params,
    )
    .await
}

pub async fn send_daily_greeting(app: &Application, key: &str) -> Result<()> {
    let chats = app.dvizh_repo.lock().await.get_all_chat_ids()?;
    for chat_id in chats {
        let message = app
            .language_cache
            .write()
            .await
            .get_translation_for_chat(&app.dvizh_repo, chat_id, key)
            .await?;
        let mut params = HashMap::new();
        params.insert("chat_id", chat_id.to_string());
        params.insert("text", message.to_string());
        send_request(
            &app.client,
            &app.conf.tg_token,
            msg_type_to_str(&MsgType::SendMessage),
            &params,
        )
        .await?;
        debug!("Sent daily greeting: {message} in {chat_id}");
    }
    Ok(())
}
