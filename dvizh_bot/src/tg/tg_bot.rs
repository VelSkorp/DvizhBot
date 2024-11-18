use crate::application::Application;
use crate::db::db_objects::{Event, User};
use crate::db::repository::DvizhRepository;
use crate::tg::tg_handlers::handle_message;
use crate::tg::msg_request::MsgRequest;
use crate::tg::msg_type_utils::{msg_type_to_str, MsgType};
use chrono::{Datelike, Local, NaiveDate, Utc};
use log::{debug, error};
use reqwest::Client;
use std::collections::HashMap;
use std::error::Error;
use tokio::time::{interval_at, Duration, Instant};

pub async fn run(app: Application, t: &MsgType) {
    debug!("Bot run");
    // Set the initial offset to 0
    let mut offset: i64 = 0;
    loop {
        // Set up the parameters for the getUpdates method
        let mut params = HashMap::new();
        params.insert("offset", offset.to_string());
        params.insert("timeout", "30".to_string());

        // Send the request and get the response
        let response =
            send_request(&app.client, &app.conf.tg_token, msg_type_to_str(t), &params).await;
        debug!("offset value - {offset}");
        // Check if there are any updates
        if let Ok(response) = response {
            if let Some(result) = response["result"].as_array() {
                if let Err(e) = handle_message(app.clone(), result, &mut offset).await {
                    error!("Error handling message: {}", e);
                };
            } else {
                error!("Message have no result {response:#?}");
            }
        } else {
            error!("Response {offset}");
        }
    }
}

pub async fn check_and_perform_daily_operations(app: Application) {
    debug!("Bot check and perform daily operations");
    // Execution time at 00:00
    let now = Local::now();
    let midnight = now.date_naive().succ_opt().unwrap().and_hms_opt(0, 0, 0);
    let time_until_midnight =
        ((midnight.unwrap_or_default() - now.naive_local()).num_seconds() + 60) as u64;

    // Running intervals
    let mut midnight_interval = interval_at(
        Instant::now() + Duration::from_secs(time_until_midnight),
        Duration::from_secs(24 * 3600),
    );

    let mut morning_interval = interval_at(
        Instant::now() + Duration::from_secs(calc_seconds_until(8, 0, 0)),
        Duration::from_secs(24 * 3600),
    );

    let mut evening_interval = interval_at(
        Instant::now() + Duration::from_secs(calc_seconds_until(22, 0, 0)),
        Duration::from_secs(24 * 3600),
    );

    loop {
        tokio::select! {
            _ = midnight_interval.tick() => {
                if let Ok(dvizh_repo) = DvizhRepository::new(&app.conf.db_path) {
                    let current_day = Local::now().date_naive();
                    let day = format!("{:02}.{:02}", current_day.day(), current_day.month());
                    debug!("Performing daily operations at midnight.");

                    perform_happy_birthday(&app, &dvizh_repo, &day).await;
                    perform_events_reminder(&app, &dvizh_repo).await;
                } else {
                    error!("Failed to connect to DvizhRepository.");
                }
            }

            _ = morning_interval.tick() => {
                if let Err(e) = send_daily_greeting(&app, "morning").await {
                    error!("Error sending morning greeting: {e:?}");
                }
            }

            _ = evening_interval.tick() => {
                if let Err(e) = send_daily_greeting(&app, "night").await {
                    error!("Error sending evening greeting: {e:?}");
                }
            }
        }
    }
}

pub async fn send_error_msg(
    offset: &mut i64,
    chat_id: i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut params = HashMap::new();
    params.insert("chat_id", chat_id.to_string());
    params.insert("text", req.get_msg_text());

    send_msg_internal(offset, req, params).await
}

pub async fn send_msg(
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let msg = req.get_msg()?;
    let mut params = HashMap::new();
    params.insert("chat_id", msg.chat.id.to_string());
    params.insert("text", msg.text.unwrap().to_string());

    send_msg_internal(offset, req, params).await
}

pub async fn send_reply_msg(
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let msg = req.get_msg()?;
    let mut params = HashMap::new();
    params.insert("chat_id", msg.chat.id.to_string());
    params.insert("text", msg.text.unwrap().to_string());
    params.insert("reply_to_message_id", msg.message_id.to_string());

    send_msg_internal(offset, req, params).await
}

pub async fn send_keyboard_msg(
    keyboard: &str,
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let msg = req.get_msg()?;
    let mut params = HashMap::new();
    params.insert("chat_id", msg.chat.id.to_string());
    params.insert("text", msg.text.unwrap().to_string());
    params.insert("reply_markup", keyboard.to_string());

    send_msg_internal(offset, req, params).await
}

pub async fn send_keyboard_reply_msg(
    keyboard: &str,
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let msg = req.get_msg()?;
    let mut params = HashMap::new();
    params.insert("chat_id", msg.chat.id.to_string());
    params.insert("text", msg.text.unwrap().to_string());
    params.insert("reply_to_message_id", msg.message_id.to_string());
    params.insert("reply_markup", keyboard.to_string());

    send_msg_internal(offset, req, params).await
}

pub async fn send_photo_msg(
    photo_url: &str,
    photo_tite: &str,
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let msg = req.get_msg()?;
    let mut params = HashMap::new();
    params.insert("chat_id", msg.chat.id.to_string());
    params.insert("photo", photo_url.to_string());
    params.insert("caption", photo_tite.to_string());
    req.method = MsgType::SendPhoto;

    send_msg_internal(offset, req, params).await
}

pub async fn edit_message_and_remove_keyboard(
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let msg = req.get_msg()?;
    let mut params = HashMap::new();
    params.insert("chat_id", msg.chat.id.to_string());
    params.insert("message_id", msg.message_id.to_string());
    params.insert("text", msg.text.unwrap().to_string());
    params.insert("reply_markup", "{}".to_string());
    req.method = MsgType::EditMessageText;

    send_msg_internal(offset, req, params).await
}

pub async fn remove_keyboard(
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let msg = req.get_msg()?;
    let mut params = HashMap::new();
    params.insert("chat_id", msg.chat.id.to_string());
    params.insert("message_id", msg.message_id.to_string());
    params.insert("reply_markup", "{}".to_string());
    req.method = MsgType::EditMessageReplyMarkup;

    send_msg_internal(offset, req, params).await
}

pub async fn get_chat_administrators(
    client: &Client,
    api_token: &str,
    chat_id: i64,
) -> Result<Vec<User>, Box<dyn Error>> {
    let mut params = HashMap::new();
    params.insert("chat_id", chat_id.to_string());

    let response = send_request(
        client,
        api_token,
        msg_type_to_str(&MsgType::GetChatAdministrators),
        &params,
    )
    .await?;

    if response["ok"].as_bool().unwrap_or(false) {
        let admins = response["result"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|admin| {
                let user = &admin["user"];

                Some(User {
                    username: user["username"].as_str()?.to_string(),
                    first_name: user["first_name"].as_str().map(|s| s.to_string()),
                    birthdate: None,
                    language_code: user["language_code"].as_str().map(|s| s.to_string()),
                })
            })
            .collect();
        Ok(admins)
    } else {
        Err("Failed to retrieve chat administrators".into())
    }
}

async fn send_request(
    client: &Client,
    api_token: &str,
    method: &str,
    params: &HashMap<&str, String>,
) -> Result<serde_json::Value, reqwest::Error> {
    let url = format!("https://api.telegram.org/bot{}/{}", api_token, method);

    let response = client.get(&url).query(params).send().await?;
    Ok(response.json().await?)
}

async fn send_msg_internal(
    offset: &mut i64,
    req: &mut MsgRequest,
    params: HashMap<&str, String>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    debug!("Send message: {:?}", params);
    let response = send_request(
        &req.app.client,
        &req.app.conf.tg_token,
        msg_type_to_str(&req.method),
        &params,
    )
    .await?;

    *offset = req.update_id + 1;
    debug!("Updated offset: {}", offset);
    Ok(response)
}

// Function for calculating the time to the next specific time in seconds
fn calc_seconds_until(target_hour: u32, target_minute: u32, target_second: u32) -> u64 {
    let now = Local::now();
    let target_time = now
        .date_naive()
        .and_hms_opt(target_hour, target_minute, target_second)
        .unwrap();
    let duration = if now.time() < target_time.time() {
        target_time - now.naive_local()
    } else {
        target_time + chrono::Duration::days(1) - now.naive_local()
    };
    duration.num_seconds() as u64
}

async fn perform_happy_birthday(app: &Application, dvizh_repo: &DvizhRepository, birthday: &str) {
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

async fn perform_events_reminder(app: &Application, dvizh_repo: &DvizhRepository) {
    if let Ok(events) = dvizh_repo.get_today_events() {
        for event in events {
            reminde_events(&app, &event).await;
        }
    } else {
        error!("Failed to retrieve today's events.");
    }
}

async fn reminde_events(app: &Application, event: &Event) {
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

async fn send_happy_birthday(app: &Application, user: &User, chat_id: i64) {
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

async fn send_daily_greeting(
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
