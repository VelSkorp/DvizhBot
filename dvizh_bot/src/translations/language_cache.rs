use crate::db::repository::DvizhRepository;
use std::collections::HashMap;
use rusqlite::Result;
use log::debug;

#[derive(Debug, Clone)]
pub struct LanguageCache {
    chat_language_cache: HashMap<i64, String>,
    translation_cache: HashMap<String, HashMap<String, String>>
}

impl LanguageCache {
    pub fn new() -> Self {
        LanguageCache {
            chat_language_cache: HashMap::new(),
            translation_cache: HashMap::new(),
        }
    }

    pub fn get_translation_for_chat(&mut self, db_path: &str, group_id: i64, key: &str) -> Result<String> {
        if !self.chat_language_cache.contains_key(&group_id) {
            debug!("Load {group_id} group language code cache");
            let dvizh_repo = DvizhRepository::new(db_path)?;
            let lang_code = dvizh_repo.get_chat_language_code(group_id)?;
            self.chat_language_cache.insert(group_id, lang_code.clone());
        }

        let lang_code = self.chat_language_cache.get(&group_id).cloned().unwrap_or_default();

        if !self.translation_cache.contains_key(&lang_code) {
            debug!("Load {lang_code} translation cahce");
            let translations = self.load_translations_for_language(&lang_code)?;
            self.translation_cache.insert(lang_code.clone(), translations);
        }

        debug!("Get translation for {key}");
        let translation = self.translation_cache
            .get(&lang_code)
            .and_then(|translations| translations.get(key))
            .cloned()
            .unwrap_or_else(|| key.to_string());

        Ok(translation)
    }

    fn load_translations_for_language(&self, lang_code: &str) -> Result<HashMap<String, String>> {
        let file_path = format!("src/translations/{lang_code}.json");
        let data = std::fs::read_to_string(&file_path).expect("Unable to read translation file");
        let translations: HashMap<String, String> = serde_json::from_str(&data)
            .expect("Error parsing translation file");
        Ok(translations)
    }
}
