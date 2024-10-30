use crate::tg::tg_handlers::handle_error;
use crate::tg::tg_objects::Message;
use crate::application::Application;
use crate::tg::tg_bot::MsgRequest;
use serde_json::Value;
use log::error;

#[derive(Debug)]
pub enum MsgType {
    GetMe,
    GetUpdates,
    SendMessage,
    SendPhoto,
    EditMessageReplyMarkup,
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
}

pub fn msg_type_to_str(t: &MsgType) -> &'static str 
{
    match t {
        MsgType::GetMe => "getMe",
        MsgType::GetUpdates => "getUpdates",
        MsgType::SendMessage => "sendMessage",
        MsgType::SendPhoto => "sendPhoto",
        MsgType::EditMessageReplyMarkup => "editMessageReplyMarkup",
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
        _ => None,
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
            '"' => {
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