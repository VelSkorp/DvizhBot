use crate::db::db_objects::User;
use crate::tg::messaging::send_request;
use crate::tg::msg_type_utils::msg_type_to_str;
use crate::MsgType;
use anyhow::Result;
use chrono::Local;
use headless_chrome::{Browser, LaunchOptions};
use log::debug;
use log::error;
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;
use std::env::temp_dir;
use std::time::Duration;

pub async fn parse_memes() -> Result<Vec<String>> {
    if let Err(e) = std::fs::remove_dir_all(temp_dir()) {
        error!("Failed to remove temp directory: {:?}", e);
    }

    // Initialize the headless browser
    let browser = Browser::new(LaunchOptions {
        headless: true,
        window_size: Some((800, 600)),
        enable_gpu: false,
        sandbox: false,
        ..Default::default()
    })?;

    let tab = browser.new_tab()?;
    tab.navigate_to("https://admem.net/ru")?;

    tab.wait_until_navigated()?;
    tab.wait_for_element("div img[src*='storage/meme']")?;
    tab.evaluate("window.scrollTo(0, document.body.scrollHeight);", false)?;
    tab.wait_for_element_with_custom_timeout(
        "div img[src*='storage/meme']",
        Duration::from_secs(10),
    )?;

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
    Ok(meme_urls)
}

pub async fn get_horoscope(sign: &str) -> Result<String> {
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
) -> Result<Vec<User>> {
    let mut params = HashMap::new();
    params.insert("chat_id", chat_id.to_string());

    let response = send_request(
        client,
        api_token,
        msg_type_to_str(&MsgType::GetChatAdministrators),
        params,
    )
    .await?;

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
}
