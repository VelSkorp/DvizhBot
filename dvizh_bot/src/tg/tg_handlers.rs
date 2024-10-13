use crate::db::db_objects::{Chat, User as DbUser};
use crate::db::repository::DvizhRepository;
use crate::tg::tg_objects::{Message, User};
use crate::application::Application;
use crate::tg::tg_bot::{send_msg, send_error_msg, MsgRequest};
use crate::tg::tg_utils::{CommandType, command_str_to_type, find_chat_id, MsgType};
use serde_json::{Error, Value};
use log::{debug, warn, error};

pub async fn handle_message(app : Application, response_results: &Vec<Value>, offset: &mut i64) -> Result<(), Box<dyn std::error::Error>>
{
    for res in response_results {
        if res.get("message").is_some() && 
           res["message"].is_object() && 
           res["message"].as_object().and_then(|m| m.get("photo")).is_none() 
        {
            
            debug!("{res:?}");
            
            let msg_obj: Value = serde_json::from_str(res["message"].to_string().as_str()).unwrap();
            let chat_id = find_chat_id(&msg_obj);
            let msg: Option<Message> = match serde_json::from_value(msg_obj) {
                Ok(m) => m,
                Err(er) => {
                    error!("{er}");
                    handle_error(er, offset, &mut MsgRequest::new(app.clone(), res["update_id"].as_i64().unwrap(), MsgType::SendMessage, Some(Message::new(chat_id.unwrap())))).await?;
                    continue;
                }
            };
            let new_member = msg.clone().unwrap().new_chat_member;

            let mut req : MsgRequest = 
                MsgRequest::new(app.clone(), res["update_id"].as_i64().unwrap(), MsgType::SendMessage, msg);
            
            if new_member.is_some() {
                handle_new_member(new_member.unwrap(), offset, &mut req).await?;
                continue;
            }

            // Check if the message is a command
            if req.get_msg_text().starts_with("/") 
            {
                if req.get_msg_text().len() == 1 {
                    handle_command(offset, None, &mut req).await?;
                    continue;
                }
                let text = req.get_msg_text();
                let command = text[1..].split_whitespace().next().unwrap();
                debug!("Handle {command} command");
                handle_command(offset, command_str_to_type(command), &mut req).await?;
                continue;
            }
            req.set_msg_text(&"It's not a command");
            send_msg(offset, &mut req).await?;
            continue;
        }
        else {
            debug!("Unknown command {:?}", res);
            *offset = res["update_id"].as_i64().unwrap() + 1;
            continue;
        }
    }
    Ok(())
}


async fn handle_error(error: Error, offset: &mut i64, req: &mut MsgRequest) -> Result<serde_json::Value, reqwest::Error> 
{
    error!("Handle error: {error}");
    req.set_msg_text(&"Wrong command".to_string());
    send_error_msg(offset, req.get_msg().unwrap().chat.id, req).await
}

async fn handle_new_member(member: User, offset: &mut i64, req: &mut MsgRequest) -> Result<serde_json::Value, Box<dyn std::error::Error>> 
{
    debug!("Handle new member: {member:#?}");

    let chat = req.msg.as_ref().unwrap().chat.clone();
    let user_repo = DvizhRepository::new(&req.app.conf.db_path)?;

    if member.is_bot && member.first_name == "DvizhBot" {
        req.set_msg_text("Hello everyone!!! My name is Oleg, I'm a bot of our dvizh.");
        user_repo.add_chat(
            Chat::new(chat.id, chat.title)
        )?;
    }
    else {
        req.set_msg_text(&format!("Welcome {}", member.first_name));
        user_repo.add_user(
            DbUser::new(member.id, member.username, member.first_name, None, member.language_code),
            Chat::new(chat.id, chat.title)
        )?;
    }

    send_msg(offset, req).await.map_err(|e| Box::<dyn std::error::Error>::from(e))
}

async fn handle_command(offset: &mut i64, command_t : Option<CommandType>, req: &mut MsgRequest) -> Result<serde_json::Value, reqwest::Error> 
{
    match command_t {
        Some(CommandType::Hello) => handle_hello_command(offset, req).await,
        None => handle_unknown_command(offset, req).await,
    }
}

async fn handle_hello_command(offset: &mut i64, req: &mut MsgRequest) -> Result<serde_json::Value, reqwest::Error> 
{
    debug!("Hello command was called");
    req.set_msg_text(&"Hello command was called");
    send_msg(offset, req).await
}

async fn handle_unknown_command(offset: &mut i64, req: &mut MsgRequest) -> Result<serde_json::Value, reqwest::Error> 
{
    warn!("Unknown command was called");
    req.set_msg_text(&"Unknown command was called");
    send_msg(offset, req).await
}