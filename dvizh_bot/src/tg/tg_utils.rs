use serde_json::Value;

#[derive(Debug)]
pub enum MsgType {
    GetMe,
    GetUpdates,
    SendMessage,
    SendPhoto,
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