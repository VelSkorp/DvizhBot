use rusqlite::{params, Connection, Result};
use log::debug;
use super::db_objects::{Chat, User};

#[derive(Debug)]
pub struct DvizhRepository {
    connection: Connection
}

impl DvizhRepository {
    pub fn new(db_path: &str) -> Result<Self> {
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

    fn add_membership(&self, user_id: i64, grop_id: i64) -> Result<()> {
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

    pub fn get_users_by_birthday(&self, birthday: &str) -> Result<Vec<User>> {
        let mut stmt = self.connection.prepare(
            "SELECT id,
                username,
                first_name,
                birthdate,
                language_code
            FROM User WHERE birthdate LIKE ?1"
        )?;
        let users = stmt.query_map(params![format!("{}%", birthday)], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                first_name: row.get(2)?,
                birthdate: row.get(3)?,
                language_code: row.get(4)?,
            })
        })?
        .map(|result| result.unwrap())
        .collect::<Vec<User>>();

        debug!("db get users by birthday {birthday}: {users:#?}");

        Ok(users)
    }

    pub fn get_chats_for_user(&self, user_id: &i64) -> Result<Vec<i64>> {
        let mut stmt = self.connection.prepare(
            "SELECT group_id FROM Members WHERE user_id = ?1",
        )?;
        
        let chat_ids = stmt.query_map(params![user_id], |row| row.get(0))?
            .map(|result| result.unwrap())
            .collect::<Vec<i64>>();
        
        debug!("db get chats for user {user_id}: {chat_ids:#?}");
    
        Ok(chat_ids)
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