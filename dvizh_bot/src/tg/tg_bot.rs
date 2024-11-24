use crate::application::Application;
use crate::tg::events::{perform_events_reminder, perform_happy_birthday, send_daily_greeting};
use crate::tg::message_handler::handle_message;
use crate::tg::messaging::send_request;
use crate::tg::msg_type_utils::{msg_type_to_str, MsgType};
use crate::tg::tg_utils::calc_seconds_until;
use anyhow::Result;
use chrono::{Datelike, Local};
use log::{debug, error};
use std::collections::HashMap;
use tokio::time::{interval_at, Duration, Instant};

pub async fn run(app: Application, t: MsgType) -> Result<()> {
    debug!("Bot run");
    // Set the initial offset to 0
    let mut offset: i64 = 0;
    loop {
        // Set up the parameters for the getUpdates method
        let mut params = HashMap::new();
        params.insert("offset", offset.to_string());
        params.insert("timeout", "30".to_string());

        // Send the request and get the response
        let response =
            send_request(&app.client, &app.conf.tg_token, msg_type_to_str(&t), &params).await;
        debug!("offset value - {offset}");
        // Check if there are any updates
        if let Ok(response) = response {
            if let Some(result) = response["result"].as_array() {
                if let Err(e) = handle_message(app.clone(), result, &mut offset).await {
                    error!("Error handling message: {}", e);
                };
            } else {
                error!("Message have no result {response:#?}");
            }
        } else {
            error!("Response {offset}");
        }
    }
}

pub async fn check_and_perform_daily_operations(app: Application) -> Result<()> {
    debug!("Bot check and perform daily operations");
    // Execution time at 00:00
    let now = Local::now();
    let midnight = now.date_naive().succ_opt().unwrap().and_hms_opt(0, 0, 0);
    let time_until_midnight =
        ((midnight.unwrap_or_default() - now.naive_local()).num_seconds() + 60) as u64;

    // Running intervals
    let mut midnight_interval = interval_at(
        Instant::now() + Duration::from_secs(time_until_midnight),
        Duration::from_secs(24 * 3600),
    );

    let mut morning_interval = interval_at(
        Instant::now() + Duration::from_secs(calc_seconds_until(8, 0, 0)),
        Duration::from_secs(24 * 3600),
    );

    let mut evening_interval = interval_at(
        Instant::now() + Duration::from_secs(calc_seconds_until(22, 0, 0)),
        Duration::from_secs(24 * 3600),
    );

    loop {
        tokio::select! {
            _ = midnight_interval.tick() => {
                debug!("Performing daily operations at midnight.");
                let current_day = Local::now().date_naive();
                let day = format!("{:02}.{:02}", current_day.day(), current_day.month());

                perform_happy_birthday(&app, &day).await?;
                perform_events_reminder(&app).await?;
            }

            _ = morning_interval.tick() => {
                send_daily_greeting(&app, "morning").await?
            }

            _ = evening_interval.tick() => {
                send_daily_greeting(&app, "night").await?
            }
        }
    }
}
