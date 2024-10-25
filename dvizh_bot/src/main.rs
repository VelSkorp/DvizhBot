mod bot_config;
mod args;
mod tg { 
    pub mod tg_bot;
    pub mod tg_objects;
    pub mod tg_handlers;
    pub mod tg_utils;
}
mod db { 
    pub mod db_objects;
    pub mod repository;
}
mod application;
mod errors;
mod translations{
    pub mod language_cache;
}

pub use bot_config::BotConfig;
use tg::tg_bot::check_and_perform_daily_operations;
pub use std::error::Error;
pub use tg::tg_utils::MsgType;
pub use tg::tg_bot::run;
pub use application::Application;
pub use translations::language_cache::LanguageCache;
 
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = Application::init()?;

    tokio::spawn(check_and_perform_daily_operations(app.clone()));

    run(app, &MsgType::GetUpdates).await;
    Ok(())
}
