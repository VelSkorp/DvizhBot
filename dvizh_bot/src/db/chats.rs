use crate::db::repository::DvizhRepository;
use crate::db::db_objects::Chat;
use log::debug;
use rusqlite::params;
use std::error::Error;

impl DvizhRepository {
    pub fn add_chat(&self, chat: Chat) -> Result<(), Box<dyn Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO Chat (id, title, language_code)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(id) DO UPDATE SET
                title = CASE WHEN Chat.title IS NOT NULL THEN excluded.title ELSE Chat.title END,
                language_code = Chat.language_code",
            params![chat.id, chat.title, chat.language_code],
        )?;

        debug!("db added new chat {chat:#?}");

        Ok(())
    }

    pub fn update_chat_language(&self, chat_id: i64, new_language: String) -> Result<(), Box<dyn Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE Chat SET language_code = ?1 WHERE id = ?2",
            params![new_language, chat_id],
        )?;
        Ok(())
    }

    pub fn get_all_chat_ids(&self) -> Result<Vec<i64>, Box<dyn Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT id FROM Chat")?;

        let chat_ids = stmt
            .query_map([], |row| row.get(0))?
            .map(|result| result.unwrap())
            .collect::<Vec<i64>>();

        debug!("db get all chat ids: {chat_ids:#?}");

        Ok(chat_ids)
    }

    pub fn get_chat_language_code(&self, group_id: i64) -> Result<String, Box<dyn Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT language_code FROM Chat WHERE id = ?1")?;
        let code = stmt
            .query_row(params![group_id], |row| row.get(0).or(Ok("en".to_string())))
            .unwrap_or_else(|_| "en".to_string());

        debug!("db get chat language code: {}", code);

        Ok(code)
    }
}
