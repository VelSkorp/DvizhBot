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
use rust_bert::marian::{
    MarianConfigResources, MarianModelResources, MarianSourceLanguages, MarianSpmResources,
    MarianTargetLanguages, MarianVocabResources,
};
use rust_bert::pipelines::common::{ModelResource, ModelType};
use rust_bert::pipelines::translation::{TranslationConfig, TranslationModel};
use rust_bert::resources::RemoteResource;
use tch::Device;
use std::str::FromStr;
use std::sync::Arc;
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

        let translation_model = Arc::new(Mutex::new(
            create_translation_model()?
        ));

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
    let model_resource = ModelResource::Torch(Box::new(RemoteResource::from_pretrained(
        MarianModelResources::ENGLISH2RUSSIAN,
    )));
    let config_resource = RemoteResource::from_pretrained(MarianConfigResources::ENGLISH2RUSSIAN);
    let vocab_resource = RemoteResource::from_pretrained(MarianVocabResources::ENGLISH2RUSSIAN);
    let spm_resource = RemoteResource::from_pretrained(MarianSpmResources::ENGLISH2RUSSIAN);

    let source_languages = MarianSourceLanguages::ENGLISH2RUSSIAN;
    let target_languages = MarianTargetLanguages::ENGLISH2RUSSIAN;

    let translation_config = TranslationConfig::new(
        ModelType::Marian,
        model_resource,
        config_resource,
        vocab_resource,
        Some(spm_resource),
        source_languages,
        target_languages,
        Device::cuda_if_available(),
    );

    Ok(TranslationModel::new(translation_config)?)
}
