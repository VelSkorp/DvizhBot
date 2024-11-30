use crate::args;
use crate::bot_config;
use crate::db::repository::DvizhRepository;
use crate::LanguageCache;
use anyhow::Result;
use args::Verbose;
use clap::Parser;
use derivative::Derivative;
use env_logger;
use log::debug;
use reqwest::Client;
use rust_bert::m2m_100::{
    M2M100ConfigResources, M2M100ModelResources, M2M100SourceLanguages, M2M100TargetLanguages,
    M2M100VocabResources,
};
use rust_bert::pipelines::common::ModelResource;
use rust_bert::resources::RemoteResource;
use rust_bert::pipelines::translation::{TranslationModel, TranslationConfig};
use std::str::FromStr;
use std::sync::Arc;
use tch::Device;
use tokio::sync::{Mutex, RwLock};

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
}

fn create_translation_model() -> Result<TranslationModel> {
    // M2M100 Resource Loading
    let model_resource = ModelResource::Torch(Box::new(
        RemoteResource::from_pretrained(M2M100ModelResources::M2M100_418M),
    ));
    let config_resource = RemoteResource::from_pretrained(M2M100ConfigResources::M2M100_418M);
    let vocab_resource = RemoteResource::from_pretrained(M2M100VocabResources::M2M100_418M);

    // Defining supported languages
    let source_languages = M2M100SourceLanguages::M2M100_418M;
    let target_languages = M2M100TargetLanguages::M2M100_418M;

    // Creating a translation configuration
    let translation_config = TranslationConfig::new(
        rust_bert::pipelines::common::ModelType::M2M100,
        model_resource,
        config_resource,
        vocab_resource,
        None, // SentencePiece model is optional for M2M100
        source_languages,
        target_languages,
        Device::Cpu, // or Device::cuda_if_available() if GPU is present
    );

    // Return the TranslationModel
    Ok(TranslationModel::new(translation_config)?)
}
