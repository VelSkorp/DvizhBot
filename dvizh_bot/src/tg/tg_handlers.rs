use crate::db::db_objects::{Chat, Event, User as DbUser};
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
                    handle_command(offset, None, None, &mut req).await?;
                    continue;
                }
                let msg_text = req.get_msg_text();
                let text = msg_text[1..].split_whitespace().collect::<Vec<&str>>();
                let command = text[0].split('@').next().unwrap();
                debug!("Handle {} command", command);
                handle_command(offset, command_str_to_type(command), Some(text[1..].to_vec()), &mut req).await?;
                continue;
            }

            // Update the offset after processing the message if it's not a command
            let update_id = res["update_id"].as_i64().unwrap();
            *offset = update_id + 1;
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

    let chat = req.get_msg().unwrap_or_default().chat;
    let dvizh_repo = DvizhRepository::new(&req.app.conf.db_path)?;

    if member.is_bot && member.first_name == "DvizhBot" {
        req.set_msg_text("Hello, I'm a bot of Dvizh Wrocław🔥");
        dvizh_repo.add_chat(
            Chat::new(chat.id, chat.title.unwrap_or_default())
        )?;
    }
    else {
        req.set_msg_text(&format!("Welcome {}", member.first_name));
        dvizh_repo.add_or_update_user(
            DbUser::new(member.username, Some(member.first_name), None, member.language_code),
            Chat::new(chat.id, chat.title.unwrap_or_default())
        )?;
    }

    send_msg(offset, req).await.map_err(|e| Box::<dyn std::error::Error>::from(e))
}

async fn handle_command(offset: &mut i64, command_t: Option<CommandType>, command_args: Option<Vec<&str>>, req: &mut MsgRequest) -> Result<serde_json::Value, Box<dyn std::error::Error>> 
{
    match command_t {
        Some(CommandType::Hello) => handle_hello_command(req),
        Some(CommandType::SetBirthdate) => {
            if command_args.as_ref().map_or(true, |args| args.is_empty()) {
                req.set_msg_text("Sorry, I can't remember your birthday. You didn't specify it.");
            }
            else {
                handle_set_birthdate_command(command_args.unwrap()[0], req)?
            }
        },
        Some(CommandType::SetBirthdateFor) => {
            let empty_vec = &vec![];
            let args= command_args.as_ref().unwrap_or(empty_vec);
            if args.is_empty() || args.len() < 2 {
                req.set_msg_text("Sorry, I can't remember his/her birthday. You didn't specify it.");
            }
            else {
                handle_set_birthdate_for_command(args[0],None, None, args[1], req)?
            }
        },
        Some(CommandType::AddEvent) => {
            let empty_vec = &vec![];
            let args= command_args.as_ref().unwrap_or(empty_vec);
            if args.is_empty() || args.len() < 2 {
                req.set_msg_text("Sorry, I can't remember this event. You didn't specify it.");
            }
            else {
                handle_add_event_command(args, req)?
            }
        },
        Some(CommandType::ListEvents) => handle_hello_command(req),
        None => handle_unknown_command(req),
    }
    send_msg(offset, req).await.map_err(|e| Box::<dyn std::error::Error>::from(e))
}

fn handle_hello_command(req: &mut MsgRequest) 
{
    debug!("Hello command was called");
    req.set_msg_text("Hello, I'm a bot of Dvizh Wrocław🔥");
}

fn handle_add_event_command(args: &Vec<&str>, req: &mut MsgRequest) -> Result<(), Box<dyn std::error::Error>>
{
    debug!("AddEvent command was called");
    let chat = req.get_msg().unwrap_or_default().chat;
    let dvizh_repo = DvizhRepository::new(&req.app.conf.db_path)?;
    dvizh_repo.add_or_update_event(
        Event::new(
            chat.id,
            args[0].to_string(), 
            args[1].to_string(), 
            Some(args[2].to_string()), 
        )
    )?;
    req.set_msg_text("Hello, I'm a bot of Dvizh Wrocław🔥");
    Ok(())
}

fn handle_set_birthdate_command(date: &str, req: &mut MsgRequest) -> Result<(), Box<dyn std::error::Error>> 
{
    debug!("SetBirthdate command was called with {date}");
    let user = req.get_msg().unwrap_or_default().from;
    handle_set_birthdate_for_command(&user.username, Some(user.first_name), user.language_code, date, req)
}

fn handle_set_birthdate_for_command(username: &str, first_name: Option<String>, language_code: Option<String>, date: &str, req: &mut MsgRequest) -> Result<(), Box<dyn std::error::Error>> 
{
    debug!("SetBirthdateFor command was called with {date}");
    let usr = username.replace("@", "");
    let chat = req.get_msg().unwrap_or_default().chat;
    let dvizh_repo = DvizhRepository::new(&req.app.conf.db_path)?;
    dvizh_repo.add_or_update_user(
        DbUser::new(
            usr, 
            first_name,
            Some(date.to_string()), 
            language_code),
        Chat::new(chat.id, chat.title.unwrap_or_default())
    )?;
    req.set_msg_text(&format!("I memorized this day {date}"));
    Ok(())
}

fn handle_unknown_command(req: &mut MsgRequest)
{
    warn!("Unknown command was called");
    req.set_msg_text("Unknown command was called");
}