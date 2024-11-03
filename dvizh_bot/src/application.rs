use crate::args;
use crate::bot_config;
use crate::LanguageCache;
use std::sync::Arc;
use tokio::sync::Mutex;
use bot_config::BotConfig;
use clap::Parser;
use args::Verbose;
use args::Arguments;
use log::debug;
use reqwest::Client;
use std::str::FromStr;
use std::error::Error;
use env_logger;

#[derive(Debug, Clone)]
pub struct Application {
    pub client: Client,
    pub conf: BotConfig,
    pub args: Arguments,
    pub log_level: &'static str,
    pub language_cache:  Arc<Mutex<LanguageCache>>,
    pub meme_cache:  Arc<Mutex<Vec<String>>>
}

impl Application {
    pub fn init() -> Result<Self, Box<dyn Error>> {
        let cli = Client::new();
        let language_cache = Arc::new(Mutex::new(LanguageCache::new()));
        let meme_cache = Arc::new(Mutex::new(Vec::new()));
        let conf = bot_config::load_config();
        let args = args::Arguments::parse();

        let arg_line = std::env::args().skip(1).map(|arg| arg.to_string()).collect::<Vec<String>>().join(" ");

        let log_level = match args.verbose {
            Verbose::Debug => "debug",
            Verbose::Info => "info",
            Verbose::Warn => "warn",
            Verbose::Error => "error",
        };

        env_logger::Builder::new()
            .filter_level(log::LevelFilter::from_str(log_level).unwrap())
            .init();

        debug!("Args: {}", arg_line);

        Ok(Application { client: cli, conf, args, log_level, language_cache, meme_cache })
    }
}