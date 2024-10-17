use std::fmt::Debug;

use rusqlite::{params, Connection, Result};
use log::debug;
use super::db_objects::{Chat, Event, User};

#[derive(Debug)]
pub struct DvizhRepository {
    connection: Connection
}

impl DvizhRepository {
    pub fn new(db_path: &str) -> Result<Self> {
        let connection = Connection::open(db_path)?;
        Ok(DvizhRepository { connection })
    }

    pub fn add_or_update_user(&self, user: User, chat: Chat) -> Result<()> {
        self.connection.execute(
            "INSERT INTO User (username, first_name, birthdate, language_code)
                VALUES (?1, ?2, ?3, ?4)
                ON CONFLICT(username) DO UPDATE SET
                    first_name = CASE WHEN User.first_name IS NULL THEN excluded.first_name ELSE User.first_name END,
                    birthdate = excluded.birthdate,
                    language_code = CASE WHEN User.language_code IS NULL THEN excluded.language_code ELSE User.language_code END",
            params![user.username, user.first_name, user.birthdate, user.language_code],
        )?;
        
        debug!("db updated or added user {user:#?}");

        let chat_id = chat.id;
        self.add_chat(chat)?;
        self.add_membership(&user.username, chat_id)?;
        
        Ok(())
    }

    pub fn add_or_update_event(&self, event: Event) -> Result<()> {
        self.connection.execute(
            "INSERT INTO Events (group_id, title, date, description)
                VALUES (?1, ?2, ?3, ?4)
                ON CONFLICT(group_id, title) DO UPDATE SET
                    date = excluded.date,
                    description = excluded.description",
            params![event.group_id, event.title, event.date, event.description],
        )?;
        
        debug!("db updated or added event {event:#?}");

        Ok(())
    }

    pub fn add_chat(&self, chat: Chat) -> Result<()> {
        self.connection.execute(
            "INSERT INTO Chat (id, title)
            VALUES (?1, ?2)
            ON CONFLICT(id) DO NOTHING",
            params![chat.id, chat.title]
        )?;

        debug!("db added new chat {chat:#?}");
        
        Ok(())
    }

    pub fn get_users_by_birthday(&self, birthday: &str) -> Result<Vec<User>> {
        let mut stmt = self.connection.prepare(
            "SELECT username,
                first_name,
                birthdate,
                language_code
            FROM User WHERE birthdate LIKE ?1"
        )?;
        let users = stmt.query_map(params![format!("{}%", birthday)], |row| {
            Ok(User {
                username: row.get(0)?,
                first_name: row.get(1)?,
                birthdate: row.get(2)?,
                language_code: row.get(3)?,
            })
        })?
        .map(|result| result.unwrap())
        .collect::<Vec<User>>();

        debug!("db get users by birthday {birthday}: {users:#?}");

        Ok(users)
    }

    pub fn get_chats_for_user(&self, user_id: &str) -> Result<Vec<i64>> {
        let mut stmt = self.connection.prepare(
            "SELECT group_id FROM Members WHERE user_id = ?1",
        )?;
        
        let chat_ids = stmt.query_map(params![user_id], |row| row.get(0))?
            .map(|result| result.unwrap())
            .collect::<Vec<i64>>();
        
        debug!("db get chats for user {user_id}: {chat_ids:#?}");
    
        Ok(chat_ids)
    }

    pub fn get_events_for_chat(&self, group_id: i64) -> Result<Vec<Event>> {
        let mut stmt = self.connection.prepare(
            "SELECT group_id, title, date, description
            FROM Events WHERE group_id = ?1"
        )?;
        let events = stmt.query_map(params![format!("{}%", group_id)], |row| {
            Ok(Event {
                group_id: row.get(0)?,
                title: row.get(1)?,
                date: row.get(2)?,
                description: row.get(3)?,
            })
        })?
        .map(|result| result.unwrap())
        .collect::<Vec<Event>>();
        
        debug!("db get events for chat {group_id}");
    
        Ok(events)
    }

    fn add_membership(&self, user_id: &str, group_id: i64) -> Result<()> {
        self.connection.execute(
            "INSERT INTO Members (group_id, user_id)
            VALUES (?1, ?2)
            ON CONFLICT(group_id, user_id) DO NOTHING",
            params![group_id, user_id]
        )?;

        debug!("db added new membership between {user_id} and {group_id}");

        Ok(())
    }
}