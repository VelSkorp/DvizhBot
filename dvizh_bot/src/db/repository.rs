use std::{fmt::Debug, sync::{Arc, Mutex}};

use chrono::Local;
use rusqlite::{params, Connection, Result};
use log::debug;
use super::db_objects::{Chat, Event, User};

#[derive(Debug)]
pub struct DvizhRepository {
    connection: Arc<Mutex<Connection>>
}

impl DvizhRepository {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(DvizhRepository {
            connection: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn add_or_update_user(&self, user: User, chat: Chat) -> Result<()> {
        self.connection.lock().unwrap().execute(
            "INSERT INTO User (username, first_name, birthdate, language_code)
                VALUES (?1, ?2, ?3, ?4)
                ON CONFLICT(username) DO UPDATE SET
                    first_name = CASE WHEN User.first_name IS NULL THEN excluded.first_name ELSE User.first_name END,
                    birthdate = CASE WHEN excluded.birthdate IS NOT NULL THEN excluded.birthdate ELSE User.birthdate END,
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
        self.connection.lock().unwrap().execute(
            "INSERT INTO Events (group_id, title, date, location, description)
                VALUES (?1, ?2, ?3, ?4, ?5)
                ON CONFLICT(group_id, title) DO UPDATE SET
                    date = CASE WHEN Events.date IS NOT NULL THEN excluded.date ELSE Events.date END,
                    location = CASE WHEN Events.location IS NOT NULL THEN excluded.location ELSE Events.location END,
                    description = CASE WHEN Events.description IS NOT NULL THEN excluded.description ELSE Events.description END",
            params![event.group_id, event.title, event.date, event.location, event.description],
        )?;
        
        debug!("db updated or added event {event:#?}");

        Ok(())
    }

    pub fn add_chat(&self, chat: Chat) -> Result<()> {
        self.connection.lock().unwrap().execute(
            "INSERT INTO Chat (id, title)
            VALUES (?1, ?2)
            ON CONFLICT(id) DO NOTHING",
            params![chat.id, chat.title]
        )?;

        debug!("db added new chat {chat:#?}");
        
        Ok(())
    }

    pub fn get_users_by_birthday(&self, birthday: &str) -> Result<Vec<User>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT username,
                first_name,
                birthdate,
                language_code
            FROM User WHERE birthdate LIKE ?1"
        )?;
        let users = stmt.query_map(params![format!("{}%", birthday)], |row| {
            Ok(User::new(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
            ))
        })?
        .map(|result| result.unwrap())
        .collect::<Vec<User>>();

        debug!("db get users by birthday {birthday}: {users:#?}");

        Ok(users)
    }

    pub fn get_chats_for_user(&self, user_id: &str) -> Result<Vec<i64>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT group_id FROM Members WHERE user_id = ?1",
        )?;
        
        let chat_ids = stmt.query_map(params![user_id], |row| row.get(0))?
            .map(|result| result.unwrap())
            .collect::<Vec<i64>>();
        
        debug!("db get chats for user {user_id}: {chat_ids:#?}");
    
        Ok(chat_ids)
    }

    pub fn get_upcoming_events_for_chat(&self, group_id: i64) -> Result<Vec<Event>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT group_id, title, date, location, description
            FROM Events WHERE group_id = ?1 AND date >= strftime('%d.%m.%Y', 'now')"
        )?;
        let events = stmt.query_map(params![group_id], |row| {
            Ok(Event::new(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?))
        })?
        .map(|result| result.unwrap())
        .collect::<Vec<Event>>();

        debug!("db get events for chat {group_id}: {events:#?}");

        Ok(events)
    }

    pub fn get_today_events(&self) -> Result<Vec<Event>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT group_id, title, location, date, description
            FROM Events WHERE date = strftime('%d.%m.%Y', 'now')"
        )?;
        let users = stmt.query_map([], |row| {
            Ok(Event::new(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })?
        .map(|result| result.unwrap())
        .collect::<Vec<Event>>();

        debug!("db get events by today {}: {:#?}", Local::now().date_naive(), users);

        Ok(users)
    }

    pub fn get_chat_language_code(&self, group_id: i64) -> Result<String> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT language_code FROM Chat WHERE id = ?1"
        )?;
        let code = stmt.query_row(params![group_id], |row| {
            row.get(0).or(Ok("en".to_string()))
        }).unwrap_or_else(|_| "en".to_string());

        debug!("db get chat language code: {}", code);

        Ok(code)
    }

    fn add_membership(&self, user_id: &str, group_id: i64) -> Result<()> {
        self.connection.lock().unwrap().execute(
            "INSERT INTO Members (group_id, user_id)
            VALUES (?1, ?2)
            ON CONFLICT(group_id, user_id) DO NOTHING",
            params![group_id, user_id]
        )?;

        debug!("db added new membership between {user_id} and {group_id}");

        Ok(())
    }
}