use crate::db::db_objects::User;
use crate::tg::messaging::send_request;
use crate::tg::msg_type_utils::msg_type_to_str;
use crate::MsgType;
use chrono::Local;
use headless_chrome::{Browser, LaunchOptions};
use log::debug;
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

pub async fn parse_memes() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Initialize the headless browser
    let browser = Browser::new(LaunchOptions {
        headless: true,
        ..Default::default()
    })?;

    let tab = browser.new_tab()?;
    tab.navigate_to("https://admem.net/ru")?;

    tab.wait_until_navigated()?;
    tab.wait_for_element("div img[src*='storage/meme']")?;
    tab.evaluate("window.scrollTo(0, document.body.scrollHeight);", false)?;
    std::thread::sleep(Duration::from_secs(2));
    let html = tab.get_content()?;

    debug!("Parse html");

    // Parse the HTML to extract image URLs
    let document = Html::parse_document(&html);
    let meme_selector = Selector::parse("img[src*='storage/meme']").unwrap();
    let meme_urls: Vec<_> = document
        .select(&meme_selector)
        .filter_map(|element| element.value().attr("src"))
        .map(|src| format!("https://admem.net/{src}"))
        .collect();

    debug!("{meme_urls:#?}");

    // Check and return the results
    if meme_urls.is_empty() {
        Err("No memes found".into())
    } else {
        Ok(meme_urls)
    }
}

pub async fn get_horoscope(sign: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://horoscope-app-api.vercel.app/api/v1/get-horoscope/daily?sign={}&day=TODAY",
        sign
    );
    let response = client
        .get(&url)
        .header("accept", "application/json")
        .send()
        .await?
        .text()
        .await?;

    let json: Value = serde_json::from_str(&response)?;
    Ok(json["data"]["horoscope_data"]
        .to_string()
        .trim_matches('"')
        .to_string())
}

// Function for calculating the time to the next specific time in seconds
pub fn calc_seconds_until(target_hour: u32, target_minute: u32, target_second: u32) -> u64 {
    let now = Local::now();
    let target_time = now
        .date_naive()
        .and_hms_opt(target_hour, target_minute, target_second)
        .unwrap();
    let duration = if now.time() < target_time.time() {
        target_time - now.naive_local()
    } else {
        target_time + chrono::Duration::days(1) - now.naive_local()
    };
    duration.num_seconds() as u64
}

pub async fn get_chat_administrators(
    client: &Client,
    api_token: &str,
    chat_id: i64,
) -> Result<Vec<User>, Box<dyn std::error::Error>> {
    let mut params = HashMap::new();
    params.insert("chat_id", chat_id.to_string());

    let response = send_request(
        client,
        api_token,
        msg_type_to_str(&MsgType::GetChatAdministrators),
        &params,
    )
    .await?;

    if response["ok"].as_bool().unwrap_or(false) {
        let admins = response["result"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|admin| {
                let user = &admin["user"];

                Some(User {
                    username: user["username"].as_str()?.to_string(),
                    first_name: user["first_name"].as_str().map(|s| s.to_string()),
                    birthdate: None,
                    language_code: user["language_code"].as_str().map(|s| s.to_string()),
                })
            })
            .collect();
        Ok(admins)
    } else {
        Err("Failed to retrieve chat administrators".into())
    }
}
