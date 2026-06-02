use anyhow::Result;
use rusqlite::Connection;
use std::sync::Mutex;

pub struct Db(pub Mutex<Connection>);

impl Db {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             CREATE TABLE IF NOT EXISTS definitions (
               character   TEXT NOT NULL,
               slug        TEXT NOT NULL,
               data        TEXT NOT NULL,
               created_at  DATETIME DEFAULT CURRENT_TIMESTAMP,
               PRIMARY KEY (character, slug)
             );
",
        )?;
        Ok(Db(Mutex::new(conn)))
    }

    pub fn get_definition(&self, character: &str, slug: &str) -> Option<String> {
        let conn = self.0.lock().unwrap();
        conn.query_row(
            "SELECT data FROM definitions WHERE character = ?1 AND slug = ?2",
            [character, slug],
            |row| row.get(0),
        )
        .ok()
    }

    pub fn set_definition(&self, character: &str, slug: &str, data: &str) -> Result<()> {
        let conn = self.0.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO definitions (character, slug, data) VALUES (?1, ?2, ?3)",
            [character, slug, data],
        )?;
        Ok(())
    }

    pub fn delete_definition(&self, character: &str, slug: &str) -> Result<()> {
        let conn = self.0.lock().unwrap();
        conn.execute(
            "DELETE FROM definitions WHERE character = ?1 AND slug = ?2",
            [character, slug],
        )?;
        Ok(())
    }
}
