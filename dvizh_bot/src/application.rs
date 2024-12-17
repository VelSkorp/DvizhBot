use crate::args;
use crate::bot_config;
use crate::db::repository::DvizhRepository;
use crate::LanguageCache;
use anyhow::Result;
use args::Verbose;
use clap::Parser;
use derivative::Derivative;
use env_logger;
use log::{debug, error};
use reqwest::Client;
use rust_bert::t5::T5ModelResources;
use rust_bert::pipelines::common::ModelResource;
use rust_bert::resources::RemoteResource;
use rust_bert::pipelines::translation::{Language, TranslationModel, TranslationConfig};
use std::str::FromStr;
use std::sync::Arc;
use tch::Device;
use tokio::sync::{Mutex, RwLock};
use crate::tg::tg_utils::parse_memes;

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct Application {
    pub client: Client,
    pub tg_token: String,
    pub dvizh_repo: Arc<Mutex<DvizhRepository>>,
    pub language_cache: Arc<RwLock<LanguageCache>>,
    pub meme_cache: Arc<RwLock<Vec<String>>>,
    #[derivative(Debug = "ignore")]
    pub translation_model: Arc<Mutex<TranslationModel>>,
}

impl Application {
    pub fn init() -> Result<Self> {
        let client = Client::new();
        let language_cache = Arc::new(RwLock::new(LanguageCache::new()));
        let meme_cache = Arc::new(RwLock::new(Vec::new()));
        let conf = bot_config::load_config();
        let args = args::Arguments::parse();
        let dvizh_repo = Arc::new(Mutex::new(DvizhRepository::new(&conf.db_path)?));

        let arg_line = std::env::args()
            .skip(1)
            .map(|arg| arg.to_string())
            .collect::<Vec<String>>()
            .join(" ");

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

        let translation_model = Arc::new(Mutex::new(create_translation_model()?));

        Ok(Application {
            client,
            tg_token: conf.tg_token,
            dvizh_repo,
            language_cache,
            meme_cache,
            translation_model,
        })
    }

    /// Initializes the meme cache in a background task.
    /// This method spawns a task that parses memes and appends them to the meme cache.
    pub fn init_meme_cache(&self) {
        let app = self.clone();
        tokio::spawn(async move {
            match parse_memes().await {
                Ok(mut memes) => {
                    app.meme_cache.write().await.append(&mut memes);
                }
                Err(e) => {
                    error!("Failed to parse memes: {:?}", e);
                }
            }
        });
    }
}

fn create_translation_model() -> Result<TranslationModel> {
    let model_resource = ModelResource::Torch(Box::new(
        RemoteResource::from_pretrained(T5ModelResources::T5_SMALL),
    ));
    let config_resource = RemoteResource::from_pretrained(T5ModelResources::T5_SMALL);
    let vocab_resource = RemoteResource::from_pretrained(T5ModelResources::T5_SMALL);

    // Creating a translation configuration
    let translation_config = TranslationConfig::new(
        rust_bert::pipelines::common::ModelType::T5,
        model_resource,
        config_resource,
        vocab_resource,
        None, // No SentencePiece model for T5
        vec![Language::English],
        vec![Language::Russian],
        Device::Cpu,
    );

    // Return the TranslationModel
    Ok(TranslationModel::new(translation_config)?)
}
