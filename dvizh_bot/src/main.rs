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

pub use bot_config::BotConfig;
pub use std::error::Error;
pub use tg::tg_utils::MsgType;
pub use tg::tg_bot::run;
pub use application::Application;
 
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = Application::init()?;

    run(app, &MsgType::GetUpdates).await;
    Ok(())
}
