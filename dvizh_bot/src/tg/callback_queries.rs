use crate::tg::language_utils::translate_text;
use crate::tg::messaging::{edit_msg_and_remove_keyboard, remove_keyboard};
use crate::tg::msg_request::MsgRequest;
use crate::tg::tg_utils::get_horoscope;
use anyhow::Result;
use log::debug;

pub async fn handle_callback_query(
    callback_query: &serde_json::Value,
    offset: &mut i64,
    req: &mut MsgRequest,
) -> Result<()> {
    debug!("Handle callback query");

    let callback_data = callback_query["data"].as_str().unwrap_or_default();
    let chat_id = callback_query["message"]["chat"]["id"].as_i64().unwrap();

    if callback_data.starts_with("lang_") {
        let new_language = match callback_data {
            "lang_en" => "en",
            "lang_ru" => "ru",
            "lang_pl" => "pl",
            _ => "en",
        };

        req.get_dvizh_repo()
            .await
            .update_chat_language(chat_id, new_language.to_string())?;
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
        req.set_msg_text(&text.expect_text()?);
        edit_msg_and_remove_keyboard(offset, req).await?;

        let mut message = format!(
            "{} your horoscope for today: {}",
            zodiac_sign,
            get_horoscope(zodiac_sign).await?
        );
        let lang_code = req.get_dvizh_repo().await.get_chat_language_code(chat_id)?;
        if lang_code != "en" {
            message = translate_text(&req.app, &message, &lang_code).await?;
        }
        req.set_msg_text(&message);
        edit_msg_and_remove_keyboard(offset, req).await?;
    }
    Ok(())
}
