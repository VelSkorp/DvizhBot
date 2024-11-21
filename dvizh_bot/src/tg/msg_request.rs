use crate::application::Application;
use crate::tg::message_handler::handle_error;
use crate::tg::msg_type_utils::MsgType;
use crate::tg::tg_objects::Message;
use log::error;
use serde_json::Value;

#[derive(Debug)]
pub struct MsgRequest {
    pub app: Application,
    pub update_id: i64,
    pub method: MsgType,
    pub msg: Option<Message>,
}

impl MsgRequest {
    pub fn new(app: Application, update_id: i64, method: MsgType, msg: Option<Message>) -> Self {
        MsgRequest {
            app,
            update_id,
            method,
            msg,
        }
    }

    pub fn get_msg_text(&self) -> String {
        self.get_msg().unwrap_or_default().text.unwrap_or_default()
    }

    pub async fn get_translation_for(
        &mut self,
        key: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self
            .app
            .language_cache
            .lock()
            .await
            .get_translation_for_chat(
                &self.app.conf.db_path,
                self.get_msg().unwrap().chat.id,
                key,
            )?)
    }

    pub async fn update_group_language_code(
        &mut self,
        group_id: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(self
            .app
            .language_cache
            .lock()
            .await
            .update_group_language_code_cache(&self.app.conf.db_path, group_id)?)
    }

    pub fn get_db_path(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self.app.conf.db_path.clone())
    }

    pub fn get_msg(&self) -> Result<Message, &'static str> {
        self.msg.as_ref().cloned().ok_or("Have no field in Message")
    }

    pub fn set_msg_text(&mut self, value: String) {
        if let Some(msg) = self.msg.as_mut() {
            msg.text = Some(value);
        }
    }
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
    let req = MsgRequest::new(app.clone(), update_id, MsgType::SendMessage, msg);

    Ok(Some(req))
}

fn find_chat_id(json: &Value) -> Option<i64> {
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
