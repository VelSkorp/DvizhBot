use config::{Config, File};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct BotConfig {
    pub tg_token: String,
    pub db_path: String,
    pub ip_address: String,
}

pub fn load_config() -> BotConfig {
    let builder = Config::builder();
    let settings = builder.add_source(File::with_name("config"))
        .build()
        .expect("Failed to load the configuration");

        settings.try_deserialize::<BotConfig>().expect("Failed to convert the configuration")
}