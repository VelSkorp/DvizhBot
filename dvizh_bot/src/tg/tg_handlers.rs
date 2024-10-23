use crate::db::db_objects::{Chat, Event, User as DbUser};
use crate::db::repository::DvizhRepository;
use crate::tg::tg_objects::{Message, User};
use crate::application::Application;
use crate::tg::tg_bot::{send_msg, send_photo, send_error_msg, MsgRequest};
use crate::tg::tg_utils::{CommandType, command_str_to_type, find_chat_id, MsgType};
use reqwest::Client;
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
                let msg_text = &req.get_msg_text()[1..];
                let mut text = parse_command_arguments(msg_text);
                let command_str = text.remove(0);
                let command = command_str.split('@').next().unwrap().trim();
                debug!("Handle {} command", command);
                handle_command(offset, command_str_to_type(command), Some(text), &mut req).await?;
                continue;
            }

            // Update the offset after processing the message if it's not a command
            let update_id = res["update_id"].as_i64().unwrap();
            *offset = update_id + 1;
        }
    }
    Ok(())
}

fn parse_command_arguments(msg_text: &str) -> Vec<String> {
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

async fn handle_error(error: Error, offset: &mut i64, req: &mut MsgRequest) -> Result<serde_json::Value, Box<dyn std::error::Error>> 
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

    if member.is_bot && member.username == "dvizh_wroclaw_bot" {
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

    send_msg(offset, req).await?;
    handle_help_command(offset, req).await
}

async fn handle_command(offset: &mut i64, command_t: Option<CommandType>, command_args: Option<Vec<String>>, req: &mut MsgRequest) -> Result<serde_json::Value, Box<dyn std::error::Error>> 
{
    match command_t {
        Some(CommandType::Start) => handle_hello_command(offset, req).await,
        Some(CommandType::Hello) => handle_hello_command(offset, req).await,
        Some(CommandType::Help) => handle_help_command(offset, req).await,
        Some(CommandType::SetBirthdate) => {
            if command_args.as_ref().map_or(true, |args| args.is_empty()) {
                req.set_msg_text("Sorry, I can't remember your birthday. You didn't specify it.");
                send_msg(offset, req).await
            }
            else {
                handle_set_birthdate_command(&command_args.unwrap()[0], offset, req).await
            }
        },
        Some(CommandType::SetBirthdateFor) => {
            let empty_vec = &vec![];
            let args= command_args.as_ref().unwrap_or(empty_vec);
            if args.is_empty() || args.len() < 2 {
                req.set_msg_text("Sorry, I can't remember his/her birthday. You didn't specify it.");
                send_msg(offset, req).await
            }
            else {
                handle_set_birthdate_for_command(&args[0],None, None, &args[1], offset, req).await
            }
        },
        Some(CommandType::AddEvent) => {
            let empty_vec = &vec![];
            let args= command_args.as_ref().unwrap_or(empty_vec);
            if args.is_empty() || args.len() < 4 {
                req.set_msg_text("Sorry, I can't remember this event. You didn't specify it.");
                send_msg(offset, req).await
            }
            else {
                handle_add_event_command(args, offset, req).await
            }
        },
        Some(CommandType::ListEvents) => handle_list_events_command(offset, req).await,
        Some(CommandType::Meme) => handle_meme_command(offset, req).await,
        None => handle_unknown_command(offset, req).await,
    }
}

async fn handle_hello_command(offset: &mut i64, req: &mut MsgRequest) -> Result<serde_json::Value, Box<dyn std::error::Error>>
{
    debug!("Hello/Start command was called");
    let chat = req.get_msg().unwrap_or_default().chat;
    let user = req.get_msg().unwrap_or_default().from;
    let dvizh_repo = DvizhRepository::new(&req.app.conf.db_path)?;dvizh_repo.add_or_update_user(
        DbUser::new(user.username, Some(user.first_name), None, user.language_code),
        Chat::new(chat.id, chat.first_name.unwrap_or_default())
    )?;
    req.set_msg_text("Hello, I'm a bot of Dvizh Wrocław🔥");
    send_msg(offset, req).await
}

async fn handle_help_command(offset: &mut i64, req: &mut MsgRequest) -> Result<serde_json::Value, Box<dyn std::error::Error>>
{
    debug!("Help command was called");
    req.set_msg_text(r#"
    *Help Menu*:
    
    - /hello: Say hello to the bot.
    - /help: Show this help menu.
    - /setbirthday "[date]": Set your birthdate. (Format: YYYY-MM-DD)
    - /setbirthdayfor "[@username]" "[date]": Set birthdate for another user. (Format: YYYY-MM-DD)
    - /addevent "[title]" "[date]" "[location]" "[description]": Add a new event to the group. (Date format: YYYY-MM-DD)
    - /listevents: List all events for this group.
    
    "#);
    send_msg(offset, req).await
}

async fn handle_set_birthdate_command(date: &str, offset: &mut i64, req: &mut MsgRequest) -> Result<serde_json::Value, Box<dyn std::error::Error>>
{
    debug!("SetBirthdate command was called with {date}");
    let user = req.get_msg().unwrap_or_default().from;
    handle_set_birthdate_for_command(&user.username, Some(user.first_name), user.language_code, date, offset, req).await
}

async fn handle_set_birthdate_for_command(username: &str, first_name: Option<String>, language_code: Option<String>, date: &str, offset: &mut i64, req: &mut MsgRequest) -> Result<serde_json::Value, Box<dyn std::error::Error>>
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
    send_msg(offset, req).await
}

async fn handle_add_event_command(args: &Vec<String>, offset: &mut i64, req: &mut MsgRequest) -> Result<serde_json::Value, Box<dyn std::error::Error>>
{
    debug!("AddEvent command was called");
    let chat = req.get_msg().unwrap_or_default().chat;
    let dvizh_repo = DvizhRepository::new(&req.app.conf.db_path)?;

    dvizh_repo.add_or_update_event(
        Event::new(
            chat.id,
            args[0].to_string(),
            args[1].to_string(),
            args[2].to_string(),
            args[3].to_string()
        )
    )?;
    req.set_msg_text(&format!("I memorized this {} event", args[0]));
    send_msg(offset, req).await
}

async fn handle_list_events_command(offset: &mut i64, req: &mut MsgRequest) -> Result<serde_json::Value, Box<dyn std::error::Error>>
{
    debug!("ListEvents command was called");
    let chat = req.get_msg().unwrap_or_default().chat;
    let dvizh_repo = DvizhRepository::new(&req.app.conf.db_path)?;
    let events = dvizh_repo.get_upcoming_events_for_chat(chat.id)?;

    if events.len() < 1 {
        req.set_msg_text("There is no upcoming events");
        return send_msg(offset, req).await;
    }

    req.set_msg_text("Upcoming events:");
    send_msg(offset, req).await?;

    let mut i = 0;
    while i < events.len() - 1 {
        req.set_msg_text(&format!(
            "📅 *Event Title*: {}\n🗓 *Date*: {}\n📍 *Location*: {}\n📖 *Description*: {}\n",
            events[i].title, events[i].date, events[i].location, events[i].description
        ));
        send_msg(offset, req).await?;
        i += 1;
    }

    req.set_msg_text(&format!(
        "📅 *Event Title*: {}\n🗓 *Date*: {}\n📍 *Location*: {}\n📖 *Description*: {}\n",
        events[i].title, events[i].date, events[i].location, events[i].description
    ));
    send_msg(offset, req).await
}

async fn handle_meme_command(offset: &mut i64, req: &mut MsgRequest) -> Result<serde_json::Value, Box<dyn std::error::Error>>
{
    debug!("Meme command was called");

    // Fetch random meme from Meme API (which pulls memes from Reddit)
    let meme_api_url = "https://meme-api.com/gimme/russian_memes_only";
    let client = Client::new();
    let response = client.get(meme_api_url).send().await?;
    let body: Value = response.json().await?;
    
    // Extract the meme's image URL
    let meme_url = body["url"].as_str().unwrap();
    let meme_title = body["title"].as_str().unwrap();

    send_photo(meme_url, meme_title, offset, req).await
}

async fn handle_unknown_command(offset: &mut i64, req: &mut MsgRequest) -> Result<serde_json::Value, Box<dyn std::error::Error>>
{
    warn!("Unknown command was called");
    req.set_msg_text("Unknown command was called");
    send_msg(offset, req).await
}