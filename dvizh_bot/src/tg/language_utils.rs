use crate::application::Application;
use rust_bert::pipelines::translation::Language;

pub fn language_code_to_language(code: &str) -> Language {
    match code.to_lowercase().as_str() {
        "en" => Language::English,
        "ru" => Language::Russian,
        "pl" => Language::Polish,
        _ => Language::English,
    }
}

pub async fn translate_text(
    app: &Application,
    text: &str,
    target_lang: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let lang = language_code_to_language(target_lang);
    let tanlation =
        app.translation_model
            .lock()
            .await
            .translate(&[text], Language::English, lang)?;

    Ok(tanlation.join(";"))
}
