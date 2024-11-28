use crate::tg::msg_request::MsgRequest;
use crate::tg::msg_type_utils::{msg_type_to_str, MsgType};
use anyhow::Result;
use log::debug;
use reqwest::Client;
use std::collections::HashMap;

pub async fn send_error_msg(
    offset: &mut i64,
    chat_id: i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value> {
    let mut params = HashMap::new();
    params.insert("chat_id", chat_id.to_string());
    params.insert("text", req.get_msg_text());

    send_msg_internal(offset, req, params).await
}

pub async fn send_msg(offset: &mut i64, req: &mut MsgRequest) -> Result<serde_json::Value> {
    let msg = req.get_msg();
    let mut params = HashMap::new();
    params.insert("chat_id", msg.chat.id.to_string());
    params.insert("text", req.get_msg_text());

    send_msg_internal(offset, req, params).await
}

pub async fn edit_msg(offset: &mut i64, req: &mut MsgRequest) -> Result<serde_json::Value> {
    let msg = req.get_msg();
    let mut params = HashMap::new();
    params.insert("chat_id", msg.chat.id.to_string());
    params.insert("message_id", msg.message_id.to_string());
    params.insert("text", req.get_msg_text().to_string());
    req.method = MsgType::EditMessageText;

    send_msg_internal(offset, req, params).await
}

pub async fn send_reply_msg(offset: &mut i64, req: &mut MsgRequest) -> Result<serde_json::Value> {
    let msg = req.get_msg();
    let mut params = HashMap::new();
    params.insert("chat_id", msg.chat.id.to_string());
    params.insert("text", req.get_msg_text());
    params.insert("reply_to_message_id", msg.message_id.to_string());

    send_msg_internal(offset, req, params).await
}

pub async fn send_keyboard_msg(
    keyboard: &str,
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value> {
    let msg = req.get_msg();
    let mut params = HashMap::new();
    params.insert("chat_id", msg.chat.id.to_string());
    params.insert("text", req.get_msg_text());
    params.insert("reply_markup", keyboard.to_string());

    send_msg_internal(offset, req, params).await
}

pub async fn send_keyboard_reply_msg(
    keyboard: &str,
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value> {
    let msg = req.get_msg();
    let mut params = HashMap::new();
    params.insert("chat_id", msg.chat.id.to_string());
    params.insert("text", req.get_msg_text());
    params.insert("reply_to_message_id", msg.message_id.to_string());
    params.insert("reply_markup", keyboard.to_string());

    send_msg_internal(offset, req, params).await
}

pub async fn send_photo_msg(
    photo_url: &str,
    photo_tite: &str,
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value> {
    let msg = req.get_msg();
    let mut params = HashMap::new();
    params.insert("chat_id", msg.chat.id.to_string());
    params.insert("photo", photo_url.to_string());
    params.insert("caption", photo_tite.to_string());
    req.method = MsgType::SendPhoto;

    send_msg_internal(offset, req, params).await
}

pub async fn edit_msg_and_remove_keyboard(
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value> {
    let msg = req.get_msg();
    let mut params = HashMap::new();
    params.insert("chat_id", msg.chat.id.to_string());
    params.insert("message_id", msg.message_id.to_string());
    params.insert("text", req.get_msg_text().to_string());
    params.insert("reply_markup", "{}".to_string());
    req.method = MsgType::EditMessageText;

    send_msg_internal(offset, req, params).await
}

pub async fn remove_keyboard(offset: &mut i64, req: &mut MsgRequest) -> Result<serde_json::Value> {
    let msg = req.get_msg();
    let mut params = HashMap::new();
    params.insert("chat_id", msg.chat.id.to_string());
    params.insert("message_id", msg.message_id.to_string());
    params.insert("reply_markup", "{}".to_string());
    req.method = MsgType::EditMessageReplyMarkup;

    send_msg_internal(offset, req, params).await
}

pub async fn send_request(
    client: &Client,
    api_token: &str,
    method: &str,
    params: HashMap<&str, String>,
) -> Result<serde_json::Value> {
    let url = format!("https://api.telegram.org/bot{}/{}", api_token, method);

    let response = client.get(&url).query(&params).send().await?;
    Ok(response.json().await?)
}

async fn send_msg_internal(
    offset: &mut i64,
    req: &mut MsgRequest,
    params: HashMap<&str, String>,
) -> Result<serde_json::Value> {
    debug!("Send message: {:?}", params);
    let response = send_request(
        &req.app.client,
        &req.app.tg_token,
        msg_type_to_str(&req.method),
        params,
    )
    .await?;

    if let Some(new_message_id) = response["result"]["message_id"].as_i64() {
        if let Some(ref mut message) = req.msg {
            message.message_id = new_message_id;
        }
    }

    *offset = req.update_id + 1;
    debug!("Updated offset: {}", offset);
    Ok(response)
}
