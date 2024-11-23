use crate::db::db_objects::User;
use crate::db::repository::DvizhRepository;
use anyhow::Result;
use log::debug;
use rusqlite::{params, Transaction};

impl DvizhRepository {
    pub fn add_or_update_user(&self, user: User, chat_id: i64) -> Result<()> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;

        tx.execute(
            "INSERT INTO User (username, first_name, birthdate, language_code)
                VALUES (?1, ?2, ?3, ?4)
                ON CONFLICT(username) DO UPDATE SET
                    first_name = CASE WHEN User.first_name IS NULL THEN excluded.first_name ELSE User.first_name END,
                    birthdate = CASE WHEN excluded.birthdate IS NOT NULL THEN excluded.birthdate ELSE User.birthdate END,
                    language_code = CASE WHEN User.language_code IS NULL THEN excluded.language_code ELSE User.language_code END",
            params![user.username, user.first_name, user.birthdate, user.language_code],
        )?;

        debug!("db updated or added user {user:#?}");

        self.add_membership_tx(&tx, &user.username, chat_id)?;

        tx.commit()?;

        Ok(())
    }

    pub fn get_users_by_birthday(&self, birthday: &str) -> Result<Vec<User>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT username,
                first_name,
                birthdate,
                language_code
            FROM User WHERE birthdate LIKE ?1",
        )?;
        let users = stmt
            .query_map(params![format!("{}%", birthday)], |row| {
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
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT group_id FROM Members WHERE user_id = ?1")?;

        let chat_ids = stmt
            .query_map(params![user_id], |row| row.get(0))?
            .map(|result| result.unwrap())
            .collect::<Vec<i64>>();

        debug!("db get chats for user {user_id}: {chat_ids:#?}");

        Ok(chat_ids)
    }

    pub fn add_admin(&self, user_id: &str, group_id: i64) -> Result<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO Admins (group_id, user_id)
            VALUES (?1, ?2)
            ON CONFLICT(group_id, user_id) DO NOTHING",
            params![group_id, user_id],
        )?;

        debug!("db added new admin {user_id} for {group_id}");

        Ok(())
    }

    pub fn is_not_admin(&self, user_id: &str, group_id: i64) -> Result<bool> {
        let conn = self.pool.get()?;
        let mut stmt =
            conn.prepare("SELECT 1 FROM Admins WHERE group_id = ? AND user_id = ? LIMIT 1")?;

        debug!("Checking if user {user_id} is NOT an admin in group {group_id}");

        Ok(!stmt.exists(params![group_id, user_id])?)
    }

    fn add_membership_tx(&self, tx: &Transaction, user_id: &str, group_id: i64) -> Result<()> {
        tx.execute(
            "INSERT INTO Members (group_id, user_id)
            VALUES (?1, ?2)
            ON CONFLICT(group_id, user_id) DO NOTHING",
            params![group_id, user_id],
        )?;

        debug!("db added new membership between {user_id} and {group_id}");

        Ok(())
    }
}
