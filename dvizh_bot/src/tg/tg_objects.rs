use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub chat: Chat,
    pub date: i64,
    pub from: User,
    pub message_id: i64,
    pub text: Option<String>,
    pub reply_markup: Option<Value>,
    pub new_chat_member: Option<User>,
}

impl Message {
    pub fn new(chat_id: i64) -> Self {
        Message {
            chat: Chat {
                id: chat_id,
                chat_type: "".to_string(),
                first_name: Some("".to_string()),
                title: Some("".to_string()),
                username: None,
            },
            date: 0,
            from: User {
                first_name: "".to_string(),
                id: chat_id,
                is_bot: false,
                language_code: Some("".to_string()),
                username: "".to_string(),
            },
            message_id: 0,
            text: Some("".to_string()),
            reply_markup: Some(json!({})),
            new_chat_member: Some(User {
                first_name: "".to_string(),
                id: chat_id,
                is_bot: false,
                language_code: Some("".to_string()),
                username: "".to_string(),
            }),
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Chat {
    pub id: i64,
    #[serde(rename = "type")]
    pub chat_type: String,
    pub first_name: Option<String>,
    pub username: Option<String>,
    pub title: Option<String>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub first_name: String,
    pub id: i64,
    pub is_bot: bool,
    pub language_code: Option<String>,
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Update {
    pub message: Message,
    pub update_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Updates {
    pub ok: bool,
    pub result: Vec<Update>,
}
