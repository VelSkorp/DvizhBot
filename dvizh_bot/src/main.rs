mod args;
mod bot_config;
mod tg {
    pub mod callback_queries;
    pub mod command_utils;
    pub mod commands;
    pub mod events;
    pub mod language_utils;
    pub mod message_handler;
    pub mod messaging;
    pub mod msg_request;
    pub mod msg_type_utils;
    pub mod tg_bot;
    pub mod tg_objects;
    pub mod tg_utils;
}
mod db {
    pub mod chats;
    pub mod db_objects;
    pub mod events;
    pub mod repository;
    pub mod users;
}
mod application;
mod errors;
mod translations {
    pub mod language_cache;
    pub mod translation_value;
}
mod spam;
mod validations;

use anyhow::Result;
pub use application::Application;
pub use bot_config::BotConfig;
pub use std::error::Error;
pub use tg::msg_type_utils::MsgType;
use tg::tg_bot::check_and_perform_daily_operations;
pub use tg::tg_bot::run;
pub use translations::language_cache::LanguageCache;

#[tokio::main]
async fn main() -> Result<()> {
    let app = Application::init()?;
    app.init_meme_cache();

    tokio::spawn(check_and_perform_daily_operations(app.clone()));

    run(app, MsgType::GetUpdates).await?;
    Ok(())
}
