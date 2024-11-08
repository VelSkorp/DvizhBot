use crate::tg::tg_handlers::handle_error;
use crate::tg::tg_objects::Message;
use crate::application::Application;
use crate::tg::tg_bot::MsgRequest;
use chrono::NaiveDate;
use serde_json::Value;
use log::{debug, error};
use scraper::{Html, Selector};
use std::time::Duration;
use headless_chrome::{Browser, LaunchOptions};

#[derive(Debug)]
pub enum MsgType {
    GetMe,
    GetUpdates,
    SendMessage,
    SendPhoto,
    EditMessageText,
    EditMessageReplyMarkup,
    GetChatAdministrators,
}

pub enum CommandType {
    Start,
    Hello,
    Help,
    SetBirthdate,
    SetBirthdateFor,
    AddEvent,
    ListEvents,
    Meme,
    Astro,
    Luck,
}

pub fn msg_type_to_str(t: &MsgType) -> &'static str 
{
    match t {
        MsgType::GetMe => "getMe",
        MsgType::GetUpdates => "getUpdates",
        MsgType::SendMessage => "sendMessage",
        MsgType::SendPhoto => "sendPhoto",
        MsgType::EditMessageText => "editMessageText",
        MsgType::EditMessageReplyMarkup => "editMessageReplyMarkup",
        MsgType::GetChatAdministrators => "getChatAdministrators",
    }
}

pub fn command_str_to_type(t: &str) -> Option<CommandType> {
    match t.to_lowercase().as_str() {
        "start" => Some(CommandType::Start),
        "hello" => Some(CommandType::Hello),
        "help" => Some(CommandType::Help),
        "setbirthday" => Some(CommandType::SetBirthdate),
        "setbirthdayfor" => Some(CommandType::SetBirthdateFor),
        "addevent" => Some(CommandType::AddEvent),
        "listevents" => Some(CommandType::ListEvents),
        "meme" => Some(CommandType::Meme),
        "astro" => Some(CommandType::Astro),
        "luck" => Some(CommandType::Luck),
        _ => None
    }
}

pub fn find_chat_id(json: &Value) -> Option<i64> {
    match json {
        Value::Object(map) => {
            if let Some(Value::Object(chat)) = map.get("chat") {
                if let Some(Value::Number(id)) = chat.get("id") {
                    return Some(id.as_i64().unwrap());
                }
            }

            for value in map.values() {
                if let Some(id) = find_chat_id(value) {
                    return Some(id);
                }
            }

            None
        }
        Value::Array(array) => {
            for value in array {
                if let Some(id) = find_chat_id(value) {
                    return Some(id);
                }
            }

            None
        }
        _ => None,
    }
}

pub fn parse_command_arguments(msg_text: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_quotes = false;

    for c in msg_text.chars() {
        match c {
            '"' | '“' | '”' | '[' | ']' => {
                in_quotes = !in_quotes;
            }
            ' ' if !in_quotes => {
                if !current_arg.is_empty() {
                    args.push(current_arg.trim().to_string());
                    current_arg.clear();
                }
            }
            _ => {
                current_arg.push(c);
            }
        }
    }
    if !current_arg.is_empty() {
        args.push(current_arg.trim().to_string());
    }

    args
}

pub async fn parse_memes() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Initialize the headless browser
    let browser = Browser::new(LaunchOptions{
        headless: true,
        ..Default::default()
    })?;

    let tab = browser.new_tab()?;
    tab.navigate_to("https://admem.net/ru")?;

    tab.wait_until_navigated()?;
    tab.wait_for_element("div img[src*='storage/meme']")?;
    tab.evaluate(
        "window.scrollTo(0, document.body.scrollHeight);",
        false,
    )?;
    std::thread::sleep(Duration::from_secs(2));
    let html = tab.get_content()?;

    debug!("Parse html");

    // Parse the HTML to extract image URLs
    let document = Html::parse_document(&html);
    let meme_selector = Selector::parse("img[src*='storage/meme']").unwrap();
    let meme_urls: Vec<_> = document
        .select(&meme_selector)
        .filter_map(|element| element.value().attr("src"))
        .map(|src| format!("https://admem.net/{src}"))
        .collect();

    debug!("{meme_urls:#?}");

    // Check and return the results
    if meme_urls.is_empty() {
        Err("No memes found".into())
    } else {
        Ok(meme_urls)
    }
}

pub async fn get_horoscope(sign: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = format!("https://horoscope-app-api.vercel.app/api/v1/get-horoscope/daily?sign={}&day=TODAY", sign);
    debug!("{url}");
    let response = client
        .get(&url)
        .header("accept", "application/json")
        .send()
        .await?
        .text()
        .await?;

    let json: Value = serde_json::from_str(&response)?;
    Ok(json["data"]["horoscope_data"].to_string().trim_matches('"').to_string())
}

/// Validates that `command_args` has at least `required_count` arguments.
pub fn validate_argument_count(command_args: &Option<Vec<String>>, required_count: usize) -> Result<&Vec<String>, String> {
    let args = command_args.as_ref().ok_or_else(|| "error_missing_arguments".to_string())?;
    if args.len() < required_count || args.len() > required_count {
        return Err("error_insufficient_arguments".to_string());
    }
    Ok(args)
}

/// Validates that `date_str` matches the `DD.MM.YYYY` format.
pub fn validate_date_format(date_str: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(date_str, "%d.%m.%Y")
        .map_err(|_| "error_invalid_date".to_string())
}

pub async fn create_msg_request(
    app: &Application,
    message: &Value,
    update_id: i64,
    offset: &mut i64,
) -> Result<Option<MsgRequest>, Box<dyn std::error::Error>> {
    // Check if "message" is an object and does not contain "photo"
    if !message.is_object() || message.as_object().and_then(|m| m.get("photo")).is_some() {
        return Ok(None); // Return `None` if message is invalid or contains a photo
    }

    // Parse `msg_obj` from `res` and retrieve `chat_id`
    let chat_id = find_chat_id(&message);

    // Attempt to convert `msg_obj` to a `Message` object
    let msg: Option<Message> = match serde_json::from_value(message.clone()) {
        Ok(m) => Some(m),
        Err(er) => {
            error!("{er}");
            handle_error(
                er,
                offset,
                &mut MsgRequest::new(
                    app.clone(),
                    update_id,
                    MsgType::SendMessage,
                    Some(Message::new(chat_id.unwrap())),
                ),
            )
            .await?;
            return Ok(None); // Return `None` if an error occurs
        }
    };

    // Create and return `MsgRequest` on success
    let req = MsgRequest::new(
        app.clone(),
        update_id,
        MsgType::SendMessage,
        msg,
    );

    Ok(Some(req))
}