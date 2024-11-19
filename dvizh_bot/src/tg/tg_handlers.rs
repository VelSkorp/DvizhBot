use crate::application::Application;
use crate::db::db_objects::{Chat, Event, User as DbUser};
use crate::db::repository::DvizhRepository;
use crate::tg::command_utils::{command_str_to_type, parse_command_arguments, CommandType};
use crate::tg::language_utils::translate_text;
use crate::tg::messaging::{
    edit_message_and_remove_keyboard, remove_keyboard, send_error_msg, send_keyboard_msg,
    send_keyboard_reply_msg, send_msg, send_photo_msg, send_reply_msg,
};
use crate::tg::msg_request::{create_msg_request, MsgRequest};
use crate::tg::tg_objects::User;
use crate::tg::tg_utils::{get_horoscope, parse_memes, get_chat_administrators};
use crate::validations::{validate_argument_count, validate_date_format};
use log::{debug, error};
use rand::Rng;
use serde_json::{json, Error, Value};

pub async fn handle_message(
    app: Application,
    response_results: &Vec<Value>,
    offset: &mut i64,
) -> Result<(), Box<dyn std::error::Error>> {
    for res in response_results {
        debug!("{res:?}");

        if let Some(callback_query) = res.get("callback_query") {
            // Use `create_msg_request` for "callback_query"
            if let Some(message) = callback_query.get("message") {
                if let Some(mut req) =
                    create_msg_request(&app, message, res["update_id"].as_i64().unwrap(), offset)
                        .await?
                {
                    handle_callback_query(callback_query, offset, &mut req).await?;
                }
            }
        } else if let Some(message) = res.get("message") {
            // Ensure `create_msg_request` handles the `photo` check
            if let Some(mut req) =
                create_msg_request(&app, message, res["update_id"].as_i64().unwrap(), offset)
                    .await?
            {
                if let Some(new_member) = &req.msg.as_ref().unwrap().new_chat_member {
                    handle_new_member(new_member.clone(), offset, &mut req).await?;
                    continue;
                }

                // Check if the message is a command
                if req.get_msg_text().starts_with("/") {
                    if req.get_msg_text().len() == 1 {
                        handle_command(offset, None, None, &mut req).await?;
                        continue;
                    }
                    let msg_text = &req.get_msg_text()[1..];
                    let mut args = parse_command_arguments(msg_text);
                    let command_str = args.remove(0);
                    let command = command_str.split('@').next().unwrap().trim();
                    debug!("Handle {} command", command);
                    handle_command(offset, command_str_to_type(command), Some(args), &mut req)
                        .await?;
                }
            }
        }

        // Update the offset after processing the message if it's not a command
        let update_id = res["update_id"].as_i64().unwrap();
        *offset = update_id + 1;
    }
    Ok(())
}

pub async fn handle_error(
    error: Error,
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    error!("Handle error: {error}");
    let text = req.get_translation_for("wrong").await?;
    req.set_msg_text(text);
    send_error_msg(offset, req.get_msg().unwrap().chat.id, req).await
}

async fn handle_new_member(
    member: User,
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    debug!("Handle new member: {member:#?}");
    let chat = req.get_msg().unwrap_or_default().chat;
    let dvizh_repo = DvizhRepository::new(&req.get_db_path()?)?;
    if member.is_bot && member.username == "dvizh_wroclaw_bot" {
        handle_start_command(offset, req).await?;
        let admins =
            get_chat_administrators(&req.app.client, &req.app.conf.tg_token, chat.id).await?;
        debug!("List of {} admins: {:#?}", chat.id, admins);
        for admin in admins {
            dvizh_repo.add_or_update_user(
                DbUser::new(
                    admin.username.clone(),
                    admin.first_name,
                    None,
                    admin.language_code,
                ),
                chat.id,
            )?;

            dvizh_repo.add_admin(&admin.username, chat.id)?;
        }
    } else {
        let text = req.get_translation_for("welcome").await?;
        req.set_msg_text(format!("{} {}", text, member.first_name));
        dvizh_repo.add_or_update_user(
            DbUser::new(
                member.username,
                Some(member.first_name),
                None,
                member.language_code,
            ),
            chat.id,
        )?;
        send_msg(offset, req).await?;
    }

    handle_help_command(offset, req).await
}

async fn handle_command(
    offset: &mut i64,
    command_t: Option<CommandType>,
    command_args: Option<Vec<String>>,
    req: &mut MsgRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    match command_t {
        Some(CommandType::Start) => handle_start_command(offset, req).await,
        Some(CommandType::Hello) => handle_hello_command(offset, req).await,
        Some(CommandType::Help) => handle_help_command(offset, req).await,
        Some(CommandType::SetBirthdate) => match validate_argument_count(&command_args, 1) {
            Ok(args) => match validate_date_format(&args[0]) {
                Ok(_) => handle_set_birthdate_command(&args[0], offset, req).await,
                Err(error_key) => {
                    let text = req.get_translation_for(&error_key).await?;
                    req.set_msg_text(text);
                    send_msg(offset, req).await
                }
            },
            Err(error_key) => {
                let text = req.get_translation_for(&error_key).await?;
                req.set_msg_text(text);
                send_msg(offset, req).await
            }
        },
        Some(CommandType::SetBirthdateFor) => match validate_argument_count(&command_args, 2) {
            Ok(args) => match validate_date_format(&args[1]) {
                Ok(_) => {
                    handle_set_birthdate_for_command(&args[0], None, None, &args[1], offset, req)
                        .await
                }
                Err(error_key) => {
                    let text = req.get_translation_for(&error_key).await?;
                    req.set_msg_text(text);
                    send_msg(offset, req).await
                }
            },
            Err(error_key) => {
                let text = req.get_translation_for(&error_key).await?;
                req.set_msg_text(text);
                send_msg(offset, req).await
            }
        },
        Some(CommandType::AddEvent) => match validate_argument_count(&command_args, 4) {
            Ok(args) => handle_add_event_command(args, offset, req).await,
            Err(error_key) => {
                let text = req.get_translation_for(&error_key).await?;
                req.set_msg_text(text);
                send_msg(offset, req).await
            }
        },
        Some(CommandType::ListEvents) => handle_list_events_command(offset, req).await,
        Some(CommandType::Meme) => handle_meme_command(offset, req).await,
        Some(CommandType::Astro) => handle_astro_command(offset, req).await,
        Some(CommandType::Luck) => handle_luck_command(offset, req).await,
        Some(CommandType::Test) => match validate_argument_count(&command_args, 1) {
            Ok(args) => handle_test_command(args, offset, req).await,
            Err(error_key) => {
                let text = req.get_translation_for(&error_key).await?;
                req.set_msg_text(text);
                send_msg(offset, req).await
            }
        },
        None => Ok(serde_json::Value::Null),
    }
}

async fn handle_hello_command(
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    debug!("Hello command was called");
    let text = req.get_translation_for("hello").await?;
    req.set_msg_text(text);
    send_msg(offset, req).await
}

async fn handle_start_command(
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    debug!("Start command was called");
    let chat = req.get_msg().unwrap_or_default().chat;
    let user = req.get_msg().unwrap_or_default().from;
    let dvizh_repo = DvizhRepository::new(&req.get_db_path()?)?;
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

    let text = req.get_translation_for("hello").await?;
    req.set_msg_text(text);
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

async fn handle_help_command(
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    debug!("Help command was called");
    let text = req.get_translation_for("help").await?;
    req.set_msg_text(text);
    send_msg(offset, req).await
}

async fn handle_set_birthdate_command(
    date: &str,
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    debug!("SetBirthdate command was called with {date}");
    let user = req.get_msg().unwrap_or_default().from;
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
    date: &str,
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    debug!("SetBirthdateFor command was called with {date}");
    let usr = username.replace("@", "");
    let chat_id = req.get_msg().unwrap_or_default().chat.id;
    let dvizh_repo = DvizhRepository::new(&req.get_db_path()?)?;
    dvizh_repo.add_or_update_user(
        DbUser::new(usr, first_name, Some(date.to_string()), language_code),
        chat_id,
    )?;
    let text = req.get_translation_for("remeber_birthday").await?;
    req.set_msg_text(format!("{} {}", text, date));
    send_msg(offset, req).await
}

async fn handle_add_event_command(
    args: &Vec<String>,
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    debug!("AddEvent command was called");
    let chat = req.get_msg().unwrap_or_default().chat;
    let user = req.get_msg().unwrap_or_default().from;
    let dvizh_repo = DvizhRepository::new(&req.get_db_path()?)?;

    if dvizh_repo.is_not_admin(&user.username, chat.id)? {
        let text = req.get_translation_for("error_not_admin").await?;
        req.set_msg_text(text);
        return send_msg(offset, req).await;
    }

    dvizh_repo.add_or_update_event(Event::new(
        chat.id,
        args[0].to_string(),
        args[1].to_string(),
        args[2].to_string(),
        args[3].to_string(),
    ))?;
    let text = req.get_translation_for("remeber_event").await?;
    req.set_msg_text(format!("{} {}", text, args[0]));
    send_msg(offset, req).await
}

async fn handle_list_events_command(
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    debug!("ListEvents command was called");
    let chat = req.get_msg().unwrap_or_default().chat;
    let dvizh_repo = DvizhRepository::new(&req.get_db_path()?)?;
    let events = dvizh_repo.get_upcoming_events_for_chat(chat.id)?;

    if events.is_empty() {
        let text = req.get_translation_for("no_upcoming_event").await?;
        req.set_msg_text(text);
        return send_msg(offset, req).await;
    }

    let text = req.get_translation_for("upcoming_event").await?;
    req.set_msg_text(text);
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

        req.set_msg_text(message);
        send_msg(offset, req).await?;
    }

    Ok(serde_json::Value::Null)
}

async fn handle_meme_command(
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    debug!("Meme command was called");
    let mut mem_cnt = req.app.meme_cache.lock().await.len();
    if mem_cnt <= 2 {
        debug!("get and load meme chache");
        let mut memes = parse_memes().await?;
        req.app.meme_cache.lock().await.append(&mut memes);
        mem_cnt = req.app.meme_cache.lock().await.len();
    }
    debug!("Mem count: {mem_cnt}");
    let random_index = rand::thread_rng().gen_range(0..mem_cnt);
    let mem_url = req.app.meme_cache.lock().await.remove(random_index);
    send_photo_msg(&mem_url, "", offset, req).await
}

async fn handle_astro_command(
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    debug!("Astro command was called");

    let text = req.get_translation_for("astro").await?;
    req.set_msg_text(text);
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

async fn handle_luck_command(
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    debug!("Luck command was called");
    let text = req.get_translation_for("luck").await?;
    req.set_msg_text(text);
    send_reply_msg(offset, req).await
}

async fn handle_test_command(
    text: &Vec<String>,
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    debug!("Test command was called");

    req.set_msg_text(text.join(";"));
    send_reply_msg(offset, req).await
}

async fn handle_callback_query(
    callback_query: &serde_json::Value,
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Handle callback query");

    let callback_data = callback_query["data"].as_str().unwrap_or_default();
    let chat_id = callback_query["message"]["chat"]["id"].as_i64().unwrap();
    let dvizh_repo = DvizhRepository::new(&req.get_db_path()?)?;

    if callback_data.starts_with("lang_") {
        let new_language = match callback_data {
            "lang_en" => "en",
            "lang_ru" => "ru",
            "lang_pl" => "pl",
            _ => "en",
        };

        dvizh_repo.update_chat_language(chat_id, new_language.to_string())?;
        req.update_group_language_code(chat_id).await?;
        remove_keyboard(offset, req).await?;
    } else if callback_data.starts_with("zodiac_") {
        let zodiac_sign = match callback_data {
            "zodiac_aries" => "Aries",
            "zodiac_taurus" => "Taurus",
            "zodiac_gemini" => "Gemini",
            "zodiac_cancer" => "Cancer",
            "zodiac_leo" => "Leo",
            "zodiac_virgo" => "Virgo",
            "zodiac_libra" => "Libra",
            "zodiac_scorpio" => "Scorpio",
            "zodiac_sagittarius" => "Sagittarius",
            "zodiac_capricorn" => "Capricorn",
            "zodiac_aquarius" => "Aquarius",
            "zodiac_pisces" => "Pisces",
            _ => "Unnown",
        };

        let text = req.get_translation_for("thinking").await?;
        req.set_msg_text(text);
        edit_message_and_remove_keyboard(offset, req).await?;

        let mut message = format!(
            "{} your horoscope for today: {}",
            zodiac_sign,
            get_horoscope(zodiac_sign).await?
        );
        let lang_code = dvizh_repo.get_chat_language_code(chat_id)?;
        if lang_code != "en" {
            message = translate_text(&req.app, &message, &lang_code).await?;
        }
        req.set_msg_text(message);
        edit_message_and_remove_keyboard(offset, req).await?;
    }
    Ok(())
}
