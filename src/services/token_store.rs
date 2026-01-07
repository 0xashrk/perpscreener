//! SQLite-backed token persistence.
//!
//! Stores the list of tokens to monitor in a local SQLite database.

use rusqlite::{Connection, Result as SqliteResult};
use std::path::Path;

const DEFAULT_DB_PATH: &str = "data/tokens.db";

/// Default tokens to seed if the database is empty.
const DEFAULT_TOKENS: &[&str] = &["BTC", "ETH", "SOL", "MON", "ZEC", "HYPE", "UNI", "PUMP"];

/// Token store backed by SQLite.
pub struct TokenStore {
    conn: Connection,
}

impl TokenStore {
    /// Opens or creates the token database at the default path.
    pub fn open() -> SqliteResult<Self> {
        Self::open_at(DEFAULT_DB_PATH)
    }

    /// Opens or creates the token database at a specific path.
    pub fn open_at<P: AsRef<Path>>(path: P) -> SqliteResult<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(path)?;
        let store = Self { conn };
        store.init_schema()?;
        store.seed_defaults_if_empty()?;
        Ok(store)
    }

    /// Creates the tokens table if it doesn't exist.
    fn init_schema(&self) -> SqliteResult<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tokens (
                ticker TEXT PRIMARY KEY NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    /// Seeds default tokens if the table is empty.
    fn seed_defaults_if_empty(&self) -> SqliteResult<()> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM tokens", [], |row| row.get(0))?;

        if count == 0 {
            tracing::info!("Token database empty, seeding with defaults");
            for ticker in DEFAULT_TOKENS {
                self.conn.execute(
                    "INSERT OR IGNORE INTO tokens (ticker) VALUES (?1)",
                    [ticker],
                )?;
            }
        }

        Ok(())
    }

    /// Returns all tokens from the database.
    pub fn get_tokens(&self) -> SqliteResult<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT ticker FROM tokens ORDER BY ticker")?;
        let rows = stmt.query_map([], |row| row.get(0))?;

        let mut tokens = Vec::new();
        for ticker in rows {
            tokens.push(ticker?);
        }
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_seeds_defaults_on_empty_db() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        let store = TokenStore::open_at(&db_path).unwrap();
        let tokens = store.get_tokens().unwrap();

        assert!(!tokens.is_empty());
        assert!(tokens.contains(&"BTC".to_string()));
        assert!(tokens.contains(&"ETH".to_string()));
    }

    #[test]
    fn test_does_not_reseed_existing_db() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        // First open seeds defaults
        {
            let store = TokenStore::open_at(&db_path).unwrap();
            let tokens = store.get_tokens().unwrap();
            assert_eq!(tokens.len(), 8);
        }

        // Manually add a token
        {
            let conn = Connection::open(&db_path).unwrap();
            conn.execute("INSERT INTO tokens (ticker) VALUES ('TEST')", [])
                .unwrap();
        }

        // Reopen - should not reseed
        {
            let store = TokenStore::open_at(&db_path).unwrap();
            let tokens = store.get_tokens().unwrap();
            assert_eq!(tokens.len(), 9);
            assert!(tokens.contains(&"TEST".to_string()));
        }
    }
}
