use anyhow::Result;
use rusqlite::{params, Connection};
use serde_json::Value;
use std::path::Path;

pub struct SessionStore {
    conn: Connection,
}

impl SessionStore {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        let store = Self { conn };
        store.init_tables()?;
        Ok(store)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn };
        store.init_tables()?;
        Ok(store)
    }

    fn init_tables(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                created_at TEXT NOT NULL,
                goal TEXT NOT NULL,
                target_url TEXT,
                status TEXT NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS checkpoints (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                step_index INTEGER NOT NULL,
                phase TEXT NOT NULL,
                data TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(id)
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS event_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                payload TEXT NOT NULL,
                timestamp TEXT NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    pub fn create_session(&self, id: &str, goal: &str, target_url: Option<&str>) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO sessions (id, created_at, goal, target_url, status) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, now, goal, target_url, "RUNNING"],
        )?;
        Ok(())
    }

    pub fn save_checkpoint(&self, session_id: &str, step_index: usize, phase: &str, data: &Value) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let json_str = serde_json::to_string(data)?;
        self.conn.execute(
            "INSERT INTO checkpoints (session_id, step_index, phase, data, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![session_id, step_index as i64, phase, json_str, now],
        )?;
        Ok(())
    }

    pub fn log_event(&self, session_id: &str, event_type: &str, payload: &Value) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let payload_str = serde_json::to_string(payload)?;
        self.conn.execute(
            "INSERT INTO event_logs (session_id, event_type, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![session_id, event_type, payload_str, now],
        )?;
        Ok(())
    }
}
