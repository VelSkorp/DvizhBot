use headless_chrome::{Browser, LaunchOptions};
use log::debug;
use scraper::{Html, Selector};
use serde_json::Value;
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
