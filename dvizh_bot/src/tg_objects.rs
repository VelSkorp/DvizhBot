use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct EditedMessage {
    edited_message: Message,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub chat: Chat,
    pub date: i64,
    pub from: User,
    pub message_id: i64,
    pub text: String,
}

impl Message {
    pub fn new(chat_id: i64, from_first_name: &str) -> Self {
        Message {
            chat: Chat {
                first_name: from_first_name.to_string(),
                id: chat_id,
                chat_type: "".to_string(),
                username: None,
            },
            date: 0,
            from: User {
                first_name: from_first_name.to_string(),
                id: chat_id,
                is_bot: false,
                language_code: "".to_string(),
                username: "".to_string(),
            },
            message_id: 0,
            text: "".to_string(),
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Chat {
    pub first_name: String,
    pub id: i64,
    #[serde(rename = "type")]
    pub chat_type: String,
    pub username: Option<String>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub first_name: String,
    pub id: i64,
    pub is_bot: bool,
    pub language_code: String,
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