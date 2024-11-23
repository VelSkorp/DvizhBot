use crate::db::db_objects::Event;
use crate::db::repository::DvizhRepository;
use anyhow::Result;
use chrono::Local;
use log::debug;
use rusqlite::params;

impl DvizhRepository {
    pub fn add_or_update_event(&self, event: Event) -> Result<()> {
        let conn = self.pool.get()?;
        conn.execute(
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

    pub fn get_upcoming_events_for_chat(&self, group_id: i64) -> Result<Vec<Event>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT group_id, title, date, location, description
            FROM Events WHERE group_id = ?1 AND date >= strftime('%d.%m.%Y', 'now')",
        )?;
        let events = stmt
            .query_map(params![group_id], |row| {
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

        debug!("db get events for chat {group_id}: {events:#?}");

        Ok(events)
    }

    pub fn get_today_events(&self) -> Result<Vec<Event>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT group_id, title, location, date, description
            FROM Events WHERE date = strftime('%d.%m.%Y', 'now')",
        )?;
        let users = stmt
            .query_map([], |row| {
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

        debug!(
            "db get events by today {}: {:#?}",
            Local::now().date_naive(),
            users
        );

        Ok(users)
    }
}
