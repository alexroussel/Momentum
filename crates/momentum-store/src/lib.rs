//! SQLite persistence for Momentum.

mod categories;
mod migrations;
mod settings;

pub use categories::Category;

use rusqlite::Connection;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("migration error: {0}")]
    Migration(#[from] rusqlite_migration::Error),
    #[error("serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid data: {0}")]
    Invalid(String),
}

pub type Result<T> = std::result::Result<T, StoreError>;

/// File name of the database inside the app data directory.
pub const DB_FILE_NAME: &str = "momentum.sqlite3";

/// Owns the SQLite connection. Callers serialize access (a mutex in the app).
pub struct Store {
    conn: Connection,
}

impl Store {
    /// Opens (or creates) the database at `path` and applies pending migrations.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        Self::init(conn, true)
    }

    /// Opens a private in-memory database (tests, smoke test).
    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?, false)
    }

    fn init(mut conn: Connection, wal: bool) -> Result<Self> {
        if wal {
            conn.pragma_update(None, "journal_mode", "WAL")?;
            conn.pragma_update(None, "synchronous", "NORMAL")?;
        }
        conn.pragma_update(None, "foreign_keys", "ON")?;
        migrations::migrations().to_latest(&mut conn)?;
        let store = Self { conn };
        store.seed_defaults()?;
        Ok(store)
    }

    /// Current schema version (SQLite `user_version`).
    pub fn schema_version(&self) -> Result<i64> {
        Ok(self
            .conn
            .pragma_query_value(None, "user_version", |r| r.get(0))?)
    }

    fn seed_defaults(&self) -> Result<()> {
        if self.count_categories()? == 0 {
            self.insert_category("Focus", "#E5484D")?;
        }
        Ok(())
    }

    pub(crate) fn conn(&self) -> &Connection {
        &self.conn
    }
}

/// Current time as unix epoch milliseconds.
pub(crate) fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_valid() {
        migrations::migrations().validate().unwrap();
    }

    #[test]
    fn opens_and_seeds_default_category() {
        let store = Store::open_in_memory().unwrap();
        assert_eq!(store.schema_version().unwrap(), 1);
        let cats = store.list_categories().unwrap();
        assert_eq!(cats.len(), 1);
        assert_eq!(cats[0].title, "Focus");
    }

    #[test]
    fn reopening_a_file_database_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(DB_FILE_NAME);
        drop(Store::open(&path).unwrap());
        let store = Store::open(&path).unwrap();
        assert_eq!(store.list_categories().unwrap().len(), 1);
    }
}
