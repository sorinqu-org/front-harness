use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub id: String,
    pub title: String,
    pub target_url: Option<String>,
    pub macrostructure: String,
    pub color_palette: String,
    pub typography: String,
    pub user_rating: Option<u8>,
    pub lessons_learned: String,
    pub created_at: String,
}

pub struct MemoryStore {
    conn: Connection,
}

impl MemoryStore {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        let store = Self { conn };
        store.init()?;
        Ok(store)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn };
        store.init()?;
        Ok(store)
    }

    fn init(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS project_memory (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                target_url TEXT,
                macrostructure TEXT NOT NULL,
                color_palette TEXT NOT NULL,
                typography TEXT NOT NULL,
                user_rating INTEGER,
                lessons_learned TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn save_summary(&self, summary: &ProjectSummary) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO project_memory 
            (id, title, target_url, macrostructure, color_palette, typography, user_rating, lessons_learned, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                summary.id,
                summary.title,
                summary.target_url,
                summary.macrostructure,
                summary.color_palette,
                summary.typography,
                summary.user_rating,
                summary.lessons_learned,
                summary.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_summaries(&self) -> Result<Vec<ProjectSummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, target_url, macrostructure, color_palette, typography, user_rating, lessons_learned, created_at 
             FROM project_memory ORDER BY created_at DESC LIMIT 20",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ProjectSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                target_url: row.get(2)?,
                macrostructure: row.get(3)?,
                color_palette: row.get(4)?,
                typography: row.get(5)?,
                user_rating: row.get(6)?,
                lessons_learned: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn get_context_injection(&self) -> String {
        if let Ok(summaries) = self.list_summaries() {
            if summaries.is_empty() {
                return String::new();
            }
            let mut out = String::from("\n### Long-Term Memory Insights:\n");
            for s in summaries.iter().take(3) {
                out.push_str(&format!(
                    "- Project '{}' (Macrostructure: {}, Palette: {}): {}\n",
                    s.title, s.macrostructure, s.color_palette, s.lessons_learned
                ));
            }
            out
        } else {
            String::new()
        }
    }
}
