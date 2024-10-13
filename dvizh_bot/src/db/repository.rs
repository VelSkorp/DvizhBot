use rusqlite::{params, Connection, Result};
use log::debug;
use super::db_objects::{Chat, User};

#[derive(Debug)]
pub struct DvizhRepository {
    connection: Connection,
}

impl DvizhRepository {
    pub fn new(db_path: &String) -> Result<Self> {
        let connection = Connection::open(db_path)?;
        Ok(DvizhRepository { connection })
    }

    pub fn add_user(&self, new_user: User, chat: Chat) -> Result<()> {
        if self.user_not_exists(new_user.id)? {
            self.connection.execute(
                "INSERT INTO User (
                        id,
                        username,
                        first_name,
                        birthdate,
                        language_code
                    )
                    VALUES (
                        ?1,
                        ?2,
                        ?3,
                        ?4,
                        ?5
                    )",
                params![new_user.id, new_user.username, new_user.first_name, new_user.birthdate, new_user.language_code]
            )?;
            debug!("db added new user {new_user:#?}");
        }

        let chat_id = chat.id;

        if self.chat_not_exists(chat.id)? {
            self.add_chat(chat)?;
        }

        self.add_membership(new_user.id, chat_id)?;

        Ok(())
    }

    pub fn add_chat(&self, chat: Chat) -> Result<()> {
        self.connection.execute(
            "INSERT INTO Chat (
                    id,
                    title
                )
                VALUES (
                    ?1,
                    ?2
                )",
            params![chat.id, chat.title]
        )?;

        debug!("db added new chat {chat:#?}");

        Ok(())
    }

    pub fn add_membership(&self, user_id: i64, grop_id: i64) -> Result<()> {
        self.connection.execute(
            "INSERT INTO Members (
                        group_id,
                        user_id
                    )
                    VALUES (
                        ?1,
                        ?2
                    )",
            params![grop_id, user_id]
        )?;

        debug!("db added new membership between {user_id} and {grop_id}");

        Ok(())
    }

    pub fn update_user(&self, user: User) -> Result<()> {
        self.connection.execute(
            "UPDATE User
                SET username = ?2,
                    first_name = ?3,
                    birthdate = ?4,
                    language_code = ?5
            WHERE id = ?1",
            params![user.id, user.username, user.first_name, user.birthdate, user.language_code]
        )?;
        debug!("db updated user {user:#?}");

        Ok(())
    }

    fn chat_not_exists(&self, group_id: i64) -> Result<bool> {
        let mut stmt = self.connection.prepare("SELECT EXISTS(SELECT 1 FROM Chat WHERE id = ?1)")?;
        let exists: bool = stmt.query_row(params![group_id], |row| row.get(0))?;
        Ok(!exists)
    }

    fn user_not_exists(&self, user_id: i64) -> Result<bool> {
        let mut stmt = self.connection.prepare("SELECT EXISTS(SELECT 1 FROM User WHERE id = ?1)")?;
        let exists: bool = stmt.query_row(params![user_id], |row| row.get(0))?;
        Ok(!exists)
    }
}