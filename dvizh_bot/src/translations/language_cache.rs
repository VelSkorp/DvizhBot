use crate::db::repository::DvizhRepository;
use anyhow::Result;
use log::debug;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use crate::translations::translation_value::TranslationValue;

#[derive(Debug)]
pub struct LanguageCache {
    chat_language_cache: RwLock<HashMap<i64, String>>,
    translation_cache: RwLock<HashMap<String, HashMap<String, TranslationValue>>>,
}

impl LanguageCache {
    pub fn new() -> Self {
        LanguageCache {
            chat_language_cache: RwLock::new(HashMap::new()),
            translation_cache: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get_translation_for_chat(
        &mut self,
        dvizh_repo: &Arc<Mutex<DvizhRepository>>,
        group_id: i64,
        key: &str,
    ) -> Result<TranslationValue> {
        debug!("Get translation for {key}");

        // Acquire read lock on chat_language_cache
        let lang_code = {
            let cache = self.chat_language_cache.read().await;
            cache.get(&group_id).cloned()
        };

        let lang_code = match lang_code {
            Some(code) => code,
            None => {
                self.update_group_language_code_cache(dvizh_repo, group_id)
                    .await?
            }
        };

        // Acquire read lock on translation_cache
        let translation_value = {
            let cache = self.translation_cache.read().await;
            if let Some(translations) = cache.get(&lang_code) {
                translations.get(key).cloned()
            } else {
                None
            }
        };

        let translation = match translation_value {
            Some(value) => value,
            None => {
                // Load translations and update cache
                let translations = self.load_translations_for_language(&lang_code)?;
                let mut cache = self.translation_cache.write().await;
                cache.insert(lang_code.clone(), translations.clone());
    
                translations.get(key).cloned().unwrap_or_else(|| {
                    // If key is not found after loading, return default Text
                    TranslationValue::Text(key.to_string())
                })
            }
        };

        Ok(translation)
    }

    pub async fn update_group_language_code_cache(
        &mut self,
        dvizh_repo: &Arc<Mutex<DvizhRepository>>,
        group_id: i64,
    ) -> Result<String> {
        debug!("Load {group_id} group language code cache");

        // Lock dvizh_repo and fetch lang_code
        let dvizh_repo_guard = dvizh_repo.lock().await;
        let code = dvizh_repo_guard.get_chat_language_code(group_id)?;

        // Update chat_language_cache
        let mut cache = self.chat_language_cache.write().await;
        cache.insert(group_id, code.clone());
        Ok(code)
    }

    fn load_translations_for_language(&self, lang_code: &str) -> Result<HashMap<String, TranslationValue>> {
        debug!("Load {lang_code} translation cahce");
        let file_path = format!("src/translations/{lang_code}.json");
        let data = std::fs::read_to_string(&file_path)?;
        Ok(serde_json::from_str(&data)?)
    }
}
