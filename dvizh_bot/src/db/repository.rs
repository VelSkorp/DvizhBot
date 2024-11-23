use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct DvizhRepository {
    pub(super) pool: Pool<SqliteConnectionManager>,
}

impl DvizhRepository {
    pub fn new(db_path: &str) -> Result<Self> {
        let manager = SqliteConnectionManager::file(db_path);
        let pool = Pool::builder().max_size(15).build(manager)?;
        Ok(DvizhRepository { pool })
    }
}
