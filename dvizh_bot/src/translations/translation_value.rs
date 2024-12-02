use serde::Deserialize;
use anyhow::anyhow;
use anyhow::Result;

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum TranslationValue {
    Text(String),
    Array(Vec<String>),
}

impl TranslationValue {
    pub fn expect_text(self) -> Result<String> {
        if let TranslationValue::Text(text) = self {
            Ok(text)
        } else {
            Err(anyhow!("Expected a text translation but found an array."))
        }
    }

    pub fn expect_array(self) -> Result<Vec<String>> {
        if let TranslationValue::Array(array) = self {
            Ok(array)
        } else {
            Err(anyhow!("Expected an array translation but found text."))
        }
    }
}