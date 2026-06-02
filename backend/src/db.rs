use anyhow::Result;
use rusqlite::Connection;
use std::sync::Mutex;

pub struct Db(pub Mutex<Connection>);

impl Db {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             CREATE TABLE IF NOT EXISTS senses (
               character   TEXT NOT NULL,
               pinyin      TEXT NOT NULL,
               sense_label TEXT NOT NULL,
               slug        TEXT NOT NULL,
               created_at  DATETIME DEFAULT CURRENT_TIMESTAMP,
               PRIMARY KEY (character, slug)
             );
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
}
