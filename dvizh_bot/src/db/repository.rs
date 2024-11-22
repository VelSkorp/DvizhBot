use std::fmt::Debug;
use r2d2::{Pool, Error};
use r2d2_sqlite::SqliteConnectionManager;

#[derive(Debug, Clone)]
pub struct DvizhRepository {
    pub(super) pool: Pool<SqliteConnectionManager>,
}

impl DvizhRepository {
    pub fn new(db_path: &str) -> Result<Self, Error> {
        let manager = SqliteConnectionManager::file(db_path);
        let pool = Pool::builder()
            .max_size(15)
            .build(manager)?;
        Ok(DvizhRepository { pool })
    }
}
