use crate::db::db_objects::{Chat, Event, User as DbUser};
use crate::tg::command_utils::CommandType;
use crate::tg::messaging::{
    send_keyboard_msg, send_keyboard_reply_msg, send_msg, send_photo_msg, send_reply_msg, edit_msg,
};
use crate::tg::msg_request::MsgRequest;
use crate::tg::tg_utils::parse_memes;
use crate::validations::{validate_argument_count, validate_date_format};
use anyhow::Result;
use log::debug;
use rand::Rng;
use serde_json::{json, Value};

pub async fn handle_command(
    offset: &mut i64,
    command_t: Option<CommandType>,
    command_args: Option<Vec<String>>,
    req: &mut MsgRequest,
) -> Result<serde_json::Value> {
    match command_t {
        Some(CommandType::Start) => handle_start_command(offset, req).await,
        Some(CommandType::Hello) => handle_hello_command(offset, req).await,
        Some(CommandType::Help) => handle_help_command(offset, req).await,
        Some(CommandType::SetBirthdate) => match validate_argument_count(command_args, 1) {
            Ok(mut args) => match validate_date_format(&args[0]) {
                Ok(()) => handle_set_birthdate_command(args.remove(0), offset, req).await,
                Err(error_key) => {
                    let text = req.get_translation_for(&error_key).await?;
                    req.set_msg_text(&text);
                    send_msg(offset, req).await
                }
            },
            Err(error_key) => {
                let text = req.get_translation_for(&error_key).await?;
                req.set_msg_text(&text);
                send_msg(offset, req).await
            }
        },
        Some(CommandType::SetBirthdateFor) => match validate_argument_count(command_args, 2) {
            Ok(mut args) => match validate_date_format(&args[1]) {
                Ok(()) => {
                    handle_set_birthdate_for_command(&args.remove(0), None, None, args.remove(1), offset, req)
                        .await
                }
                Err(error_key) => {
                    let text = req.get_translation_for(&error_key).await?;
                    req.set_msg_text(&text);
                    send_msg(offset, req).await
                }
            },
            Err(error_key) => {
                let text = req.get_translation_for(&error_key).await?;
                req.set_msg_text(&text);
                send_msg(offset, req).await
            }
        },
        Some(CommandType::AddEvent) => match validate_argument_count(command_args, 4) {
            Ok(args) => handle_add_event_command(args, offset, req).await,
            Err(error_key) => {
                let text = req.get_translation_for(&error_key).await?;
                req.set_msg_text(&text);
                send_msg(offset, req).await
            }
        },
        Some(CommandType::ListEvents) => handle_list_events_command(offset, req).await,
        Some(CommandType::Meme) => handle_meme_command(offset, req).await,
        Some(CommandType::Astro) => handle_astro_command(offset, req).await,
        Some(CommandType::Luck) => handle_luck_command(offset, req).await,
        Some(CommandType::Test) => match validate_argument_count(command_args, 1) {
            Ok(args) => handle_test_command(args, offset, req).await,
            Err(error_key) => {
                let text = req.get_translation_for(&error_key).await?;
                req.set_msg_text(&text);
                send_msg(offset, req).await
            }
        },
        None => Ok(serde_json::Value::Null),
    }
}

pub async fn handle_start_command(
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value> {
    debug!("Start command was called");
    let chat = req.get_msg().chat.clone();
    let user = req.get_msg().from.clone();

    {
        let dvizh_repo = req.get_dvizh_repo().await;
        let title = chat.title.unwrap_or(chat.first_name.unwrap_or_default());
        dvizh_repo.add_chat(Chat::new(chat.id, title, "en".to_string()))?;
        if chat.chat_type == "private" {
            dvizh_repo.add_or_update_user(
                DbUser::new(
                    user.username.clone(),
                    Some(user.first_name),
                    None,
                    user.language_code,
                ),
                chat.id,
            )?;
            dvizh_repo.add_admin(&user.username, chat.id)?;
        }
    }

    let text = req.get_translation_for("hello").await?;
    req.set_msg_text(&text);
    let keyboard = json!({
        "inline_keyboard": [
            [
                { "text": "English", "callback_data": "lang_en" },
                { "text": "Русский", "callback_data": "lang_ru" },
                { "text": "Polski", "callback_data": "lang_pl" }
            ]
        ]
    })
    .to_string();

    send_keyboard_msg(&keyboard, offset, req).await
}

pub async fn handle_help_command(
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value> {
    debug!("Help command was called");
    let text = req.get_translation_for("help").await?;
    req.set_msg_text(&text);
    send_msg(offset, req).await
}

async fn handle_hello_command(offset: &mut i64, req: &mut MsgRequest) -> Result<serde_json::Value> {
    debug!("Hello command was called");
    let text = req.get_translation_for("hello").await?;
    req.set_msg_text(&text);
    send_msg(offset, req).await
}

async fn handle_set_birthdate_command(
    date: String,
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value> {
    debug!("SetBirthdate command was called with {date}");
    let user = req.get_msg().from.clone();
    handle_set_birthdate_for_command(
        &user.username,
        Some(user.first_name),
        user.language_code,
        date,
        offset,
        req,
    )
    .await
}

async fn handle_set_birthdate_for_command(
    username: &str,
    first_name: Option<String>,
    language_code: Option<String>,
    date: String,
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value> {
    debug!("SetBirthdateFor command was called with {date}");
    let usr = username.replace("@", "");
    let chat_id = req.get_msg().chat.id;
    req.get_dvizh_repo().await.add_or_update_user(
        DbUser::new(usr, first_name, Some(date.to_string()), language_code),
        chat_id,
    )?;
    let text = req.get_translation_for("remeber_birthday").await?;
    req.set_msg_text(&format!("{} {}", text, date));
    send_msg(offset, req).await
}

async fn handle_add_event_command(
    args: Vec<String>,
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value> {
    debug!("AddEvent command was called");
    let chat_id = req.get_msg().chat.id;
    let user = req.get_msg().from.username.clone();

    if req
        .get_dvizh_repo()
        .await
        .is_not_admin(&user, chat_id)?
    {
        let text = req.get_translation_for("error_not_admin").await?;
        req.set_msg_text(&text);
        return send_msg(offset, req).await;
    }

    req.get_dvizh_repo().await.add_or_update_event(Event::new(
        chat_id,
        args[0].to_string(),
        args[1].to_string(),
        args[2].to_string(),
        args[3].to_string(),
    ))?;
    let text = req.get_translation_for("remeber_event").await?;
    req.set_msg_text(&format!("{} {}", text, args[0]));
    send_msg(offset, req).await
}

async fn handle_list_events_command(
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value> {
    debug!("ListEvents command was called");
    let chat_id = req.get_msg().chat.id;
    let events = &req
        .get_dvizh_repo()
        .await
        .get_upcoming_events_for_chat(chat_id)?;

    if events.is_empty() {
        let text = req.get_translation_for("no_upcoming_event").await?;
        req.set_msg_text(&text);
        return send_msg(offset, req).await;
    }

    let text = req.get_translation_for("upcoming_event").await?;
    req.set_msg_text(&text);
    send_msg(offset, req).await?;

    // Retrieve the entire event template from translation
    let template = req.get_translation_for("event_template").await?;

    // Send each event using the template
    for event in events {
        let message = template
            .replace("{title}", &event.title)
            .replace("{date}", &event.date)
            .replace("{location}", &event.location)
            .replace("{description}", &event.description);

        req.set_msg_text(&message);
        send_msg(offset, req).await?;
    }

    Ok(serde_json::Value::Null)
}

async fn handle_meme_command(offset: &mut i64, req: &mut MsgRequest) -> Result<serde_json::Value> {
    debug!("Meme command was called");
    let mut mem_cnt = req.app.meme_cache.read().await.len();
    if mem_cnt <= 2 {
        debug!("get and load meme chache");
        let mut memes = parse_memes().await?;
        req.app.meme_cache.write().await.append(&mut memes);
        mem_cnt = req.app.meme_cache.read().await.len();
    }
    debug!("Mem count: {mem_cnt}");
    let random_index = rand::thread_rng().gen_range(0..mem_cnt);
    let mem_url = req.app.meme_cache.write().await.remove(random_index);
    send_photo_msg(&mem_url, "", offset, req).await
}

async fn handle_astro_command(offset: &mut i64, req: &mut MsgRequest) -> Result<serde_json::Value> {
    debug!("Astro command was called");

    let text = req.get_translation_for("astro").await?;
    req.set_msg_text(&text);
    let keyboard = json!({
        "inline_keyboard": [
            [{ "text": "Aries", "callback_data": "zodiac_aries" }, { "text": "Taurus", "callback_data": "zodiac_taurus" }],
            [{ "text": "Gemini", "callback_data": "zodiac_gemini" }, { "text": "Cancer", "callback_data": "zodiac_cancer" }],
            [{ "text": "Leo", "callback_data": "zodiac_leo" }, { "text": "Virgo", "callback_data": "zodiac_virgo" }],
            [{ "text": "Libra", "callback_data": "zodiac_libra" }, { "text": "Scorpio", "callback_data": "zodiac_scorpio" }],
            [{ "text": "Sagittarius", "callback_data": "zodiac_sagittarius" }, { "text": "Capricorn", "callback_data": "zodiac_capricorn" }],
            [{ "text": "Aquarius", "callback_data": "zodiac_aquarius" }, { "text": "Pisces", "callback_data": "zodiac_pisces" }]
        ]
    }).to_string();

    send_keyboard_reply_msg(&keyboard, offset, req).await
}

async fn handle_luck_command(offset: &mut i64, req: &mut MsgRequest) -> Result<serde_json::Value> {
    debug!("Luck command was called");
    let text = req.get_translation_for("luck").await?;
    req.set_msg_text(&text);
    send_reply_msg(offset, req).await
}

async fn handle_test_command(
    text: Vec<String>,
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value> {
    debug!("Test command was called");
    
    let text = req.get_translation_for("thinking").await?;
    req.set_msg_text(&text);
    send_reply_msg(offset, req).await?;

    let client = reqwest::Client::new();
    let response = client
        .get("https://v2.jokeapi.dev/joke/Any?type=single")
        .header("accept", "application/json")
        .send()
        .await?
        .text()
        .await?;

    let json: Value = serde_json::from_str(&response)?;
    let output = json["joke"]
        .to_string()
        .trim_matches('"')
        .to_string();

    req.set_msg_text(&output);
    edit_msg(offset, req).await
}
