use crate::application::Application;
use crate::db::db_objects::User as DbUser;
use crate::tg::callback_queries::handle_callback_query;
use crate::tg::command_utils::{command_str_to_type, parse_command_arguments};
use crate::tg::commands::{handle_command, handle_help_command, handle_start_command};
use crate::tg::messaging::{send_error_msg, send_msg};
use crate::tg::msg_request::{create_msg_request, MsgRequest};
use crate::tg::tg_objects::User;
use crate::tg::tg_utils::get_chat_administrators;
use anyhow::Result;
use log::{debug, error};
use serde_json::{Error, Value};

pub async fn handle_message(
    app: Application,
    response_results: &Vec<Value>,
    offset: &mut i64,
) -> Result<()> {
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
) -> Result<serde_json::Value> {
    error!("Handle error: {error}");
    let text = req.get_translation_for("wrong").await?;
    req.set_msg_text(&text);
    send_error_msg(offset, req.get_msg().chat.id, req).await
}

async fn handle_new_member(
    member: User,
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<serde_json::Value> {
    debug!("Handle new member: {member:#?}");
    let chat = req.get_msg().chat;
    if member.is_bot && member.username == "dvizh_wroclaw_bot" {
        handle_start_command(offset, req).await?;
        let admins =
            get_chat_administrators(&req.app.client, &req.app.conf.tg_token, chat.id).await?;
        debug!("List of {} admins: {:#?}", chat.id, admins);
        for admin in admins {
            req.get_dvizh_repo().await.add_or_update_user(
                DbUser::new(
                    admin.username.clone(),
                    admin.first_name,
                    None,
                    admin.language_code,
                ),
                chat.id,
            )?;

            req.get_dvizh_repo()
                .await
                .add_admin(&admin.username, chat.id)?;
        }
    } else {
        let text = &req.get_translation_for("welcome").await?;
        req.set_msg_text(&format!("{} {}", text, member.first_name));
        req.get_dvizh_repo().await.add_or_update_user(
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
