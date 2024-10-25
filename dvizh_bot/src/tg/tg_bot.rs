use crate::application::Application;
use crate::db::db_objects::{Event, User};
use crate::db::repository::DvizhRepository;
use crate::tg::tg_utils::{MsgType, msg_type_to_str};
use crate::tg::tg_handlers::handle_message;
use crate::tg::tg_objects::Message;
use chrono::{Datelike, Local, NaiveDate};
use tokio::time::{sleep, Duration};
use std::collections::HashMap;
use reqwest::Client;
use log::{debug, error};

#[derive(Debug)]
pub struct MsgRequest {
    pub app: Application,
    pub update_id: i64,
    pub method: MsgType,
    pub msg: Option<Message>,
}

impl MsgRequest {
    pub fn new(app: Application, update_id: i64, method: MsgType, msg: Option<Message>) -> Self {
        MsgRequest {app, update_id, method, msg }
    }

    pub fn get_msg_text(&self) -> String {
        return self.get_msg().unwrap_or_default().text.unwrap_or_default();
    }
    
    pub fn get_msg(&self) -> Result<Message, &'static str> {
        self.msg.as_ref().cloned().ok_or("Have no field in Message")
    }

    pub fn set_msg_text(&mut self, value: &str) {
        if let Some(msg) = self.msg.as_mut() {
            msg.text = Some(value.to_string());
        }
    }
}

pub async fn send_request(
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
    let response = send_request(&req.app.cli, &req.app.conf.tg_token, msg_type_to_str(&req.method), &params).await?;
    
    *offset = req.update_id + 1;
    debug!("Updated offset: {}", offset);
    Ok(response)
}

pub async fn send_error_msg(
    offset: &mut i64,
    chat_id : i64,
    req: &mut MsgRequest
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut params = HashMap::new();
    params.insert("chat_id", chat_id.to_string());
    params.insert("text", req.get_msg_text());

    send_msg_internal(offset, req, params).await
}

pub async fn send_msg(
    offset: &mut i64,
    req : &mut MsgRequest
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let msg = req.get_msg()?;
    let mut params = HashMap::new();
    params.insert("chat_id", msg.chat.id.to_string());
    params.insert("text", msg.text.unwrap().to_string());

    send_msg_internal(offset, req, params).await
}

pub async fn send_photo(
    photo_url: &str,
    photo_tite: &str,
    offset: &mut i64,
    req : &mut MsgRequest
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let msg = req.get_msg()?;
    let mut params = HashMap::new();
    params.insert("chat_id", msg.chat.id.to_string());
    params.insert("photo", photo_url.to_string());
    params.insert("caption", photo_tite.to_string());
    req.method = MsgType::SendPhoto;

    send_msg_internal(offset, req, params).await
}

pub async fn run(app : Application, t: &MsgType) {
    // Set the initial offset to 0
    let mut offset: i64 = 0;
    loop {
        // Set up the parameters for the getUpdates method
        let mut params = HashMap::new();
        params.insert("offset", offset.to_string());
        params.insert("timeout", "30".to_string());
    
        // Send the request and get the response
        let response = send_request(
            &app.cli, &app.conf.tg_token, 
            msg_type_to_str(t), 
            &params).await;
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

pub async fn check_and_perform_daily_operations(app : Application) {
    loop {
        // Get the current date and time
        let now = Local::now();
        let current_day: NaiveDate = now.date_naive();

        // Calculate the time until midnight (00:00 of the next day)
        let next_midnight = now.date_naive().succ_opt().unwrap().and_hms_opt(0, 0, 0).unwrap();
        let time_until_midnight = next_midnight.signed_duration_since(now.naive_local());

        // Sleep until midnight
        sleep(Duration::from_secs(time_until_midnight.num_seconds() as u64 + 60)).await;

        // After waking up, it's the new day, perform daily operations
        debug!("New day detected (00:00), performing daily operations.");

        // If the date has changed, send a message}
        if let Ok(dvizh_repo) = DvizhRepository::new(&app.conf.db_path) {
            let day = format!("{:02}.{:02}", current_day.day(), current_day.month());
            perform_happy_birthday(&app, &dvizh_repo, &day).await;
            perform_events_reminder(&app, &dvizh_repo).await;
        } else {
            error!("Failed to connect to DvizhRepository.");
        }
    }
}

async fn perform_happy_birthday(app : &Application, dvizh_repo: &DvizhRepository, birthday: &str)
{
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

async fn perform_events_reminder(app : &Application, dvizh_repo: &DvizhRepository)
{
    if let Ok(events) = dvizh_repo.get_today_events() {
        for event in events {
            reminde_events(&app, &event).await;
        }
    } else {
        error!("Failed to retrieve today's events.");
    }
}

async fn reminde_events(app : &Application, event: &Event)
{
    // Formatting the message for the user
    let mut params = HashMap::new();
    params.insert("chat_id", event.group_id.to_string());
    params.insert("text", format!(
        "📅 *Event Title*: {}\n🗓 *Date*: {}\n📍 *Location*: {}\n📖 *Description*: {}\n",
        event.title, event.date, event.location, event.description
    ));
    // Sending a message to Telegram
    if let Err(e) = send_request(
        &app.cli, &app.conf.tg_token, 
        msg_type_to_str(&MsgType::SendMessage), &params).await {
        error!("Failed to send event reminder to chat {}: {}", event.group_id, e);
    }
}

async fn send_happy_birthday(app : &Application, user: &User, chat_id : i64)
{
    // Formatting the message for the user
    let mut params: HashMap<&str, String> = HashMap::new();
    params.insert("chat_id", chat_id.to_string());
    params.insert("text", format!("Happy Birthday, {} @{}", user.first_name.clone().unwrap_or("unknown :(".to_string()), user.username));
    // Sending a message to Telegram
    if let Err(e) = send_request(
        &app.cli, &app.conf.tg_token, 
        msg_type_to_str(&MsgType::SendMessage), &params).await {
        error!("Failed to send birthday message to user {}: {}", user.username, e);
    }
}