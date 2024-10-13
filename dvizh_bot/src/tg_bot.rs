use crate::application::Application;
use crate::tg_utils::{MsgType, msg_type_to_str};
use crate::tg_handlers::handle_message;
use crate::tg_objects::Message;
use std::collections::HashMap;
use reqwest::Client;
use serde_json::Value;
use log::{debug, error};

#[derive(Debug)]
pub struct MsgRequest{
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
) -> Result<serde_json::Value, reqwest::Error> 
{
    let mut url = String::new();
    url.push_str("https://api.telegram.org/bot");
    url.push_str(api_token);
    url.push_str("/");
    url.push_str(method);

    let response = client.get(&url).query(params).send().await?;
    let json: Value = response.json().await?;
    Ok(json)
}

pub async fn send_error_msg(
    offset: &mut i64,
    chat_id : i64,
    req: &mut MsgRequest
) -> Result<serde_json::Value, reqwest::Error> 
{
    let mut params: HashMap<&str, String> = HashMap::new();
    params.insert("chat_id", chat_id.to_string());
    params.insert("text", format!("{}", req.get_msg_text().to_string()));
    debug!("Send message: {:?}", params);
    let _response = send_request(&req.app.cli, &req.app.conf.tg_token, msg_type_to_str(&req.method), &params).await?;
    
    *offset = req.update_id + 1;
    debug!("Add offset1: {}", offset);
    Ok(_response)
}

pub async fn send_msg(
    offset: &mut i64,
    req : &mut MsgRequest
) -> Result<serde_json::Value, reqwest::Error> 
{
    let msg = req.get_msg().unwrap_or_default();
    let mut params: HashMap<&str, String> = HashMap::new();
    params.insert("chat_id", msg.chat.id.to_string());
    params.insert("text", msg.text.unwrap().to_string());
    debug!("Send message: {:?}", params);
    let _response = send_request(&req.app.cli, &req.app.conf.tg_token, msg_type_to_str(&req.method), &params).await?;

    *offset = req.update_id + 1;
    debug!("Add offset2: {}", offset);
    Ok(_response)
}

pub async fn run(app : Application, t: &MsgType)
{
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
            if let Some(response_res) = response["result"].as_array() {
                let _ = match handle_message(app.clone(), response_res, &mut offset).await {
                    Ok(_) => Ok(()),
                    Err(e) => { error!("{}", e); Err(e) },
                };
            }
            else {
                error!("Message have no result {response:#?}");
            }
        }
        else {
            error!("Response {offset}");
        }
    }
}