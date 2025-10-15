use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use std::path::PathBuf;

use super::schema::init_database;
use crate::toggl::models::{Project, TimeEntry};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: Option<PathBuf>) -> Result<Self> {
        let path = db_path.unwrap_or_else(|| {
            let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("toggl-timeguru");
            std::fs::create_dir_all(&path).ok();
            path.push("timeguru.db");
            path
        });

        let conn = Connection::open(&path)
            .with_context(|| format!("Failed to open database at {:?}", path))?;

        init_database(&conn)?;

        Ok(Self { conn })
    }

    pub fn save_time_entries(&self, entries: &[TimeEntry]) -> Result<usize> {
        let mut count = 0;
        let now = Utc::now().to_rfc3339();

        for entry in entries {
            let tags_json = entry
                .tags
                .as_ref()
                .map(|t| serde_json::to_string(t).unwrap_or_default());
            let tag_ids_json = entry
                .tag_ids
                .as_ref()
                .map(|t| serde_json::to_string(t).unwrap_or_default());

            self.conn.execute(
                "INSERT OR REPLACE INTO time_entries
                (id, workspace_id, project_id, task_id, billable, start, stop, duration,
                 description, tags, tag_ids, user_id, at, synced_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                rusqlite::params![
                    entry.id,
                    entry.workspace_id,
                    entry.project_id,
                    entry.task_id,
                    entry.billable as i32,
                    entry.start.to_rfc3339(),
                    entry.stop.as_ref().map(|s| s.to_rfc3339()),
                    entry.duration,
                    entry.description,
                    tags_json,
                    tag_ids_json,
                    entry.user_id,
                    entry.at.to_rfc3339(),
                    &now,
                ],
            )?;
            count += 1;
        }

        Ok(count)
    }

    pub fn get_time_entries(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<TimeEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, workspace_id, project_id, task_id, billable, start, stop, duration,
                    description, tags, tag_ids, user_id, at
             FROM time_entries
             WHERE start >= ?1 AND start <= ?2
             ORDER BY start DESC",
        )?;

        let entries = stmt.query_map(
            rusqlite::params![start_date.to_rfc3339(), end_date.to_rfc3339()],
            |row| {
                let tags_str: Option<String> = row.get(9)?;
                let tags = tags_str.and_then(|s| serde_json::from_str(&s).ok());

                let tag_ids_str: Option<String> = row.get(10)?;
                let tag_ids = tag_ids_str.and_then(|s| serde_json::from_str(&s).ok());

                Ok(TimeEntry {
                    id: row.get(0)?,
                    workspace_id: row.get(1)?,
                    project_id: row.get(2)?,
                    task_id: row.get(3)?,
                    billable: row.get::<_, i32>(4)? != 0,
                    start: row.get::<_, String>(5)?.parse().unwrap(),
                    stop: row
                        .get::<_, Option<String>>(6)?
                        .and_then(|s| s.parse().ok()),
                    duration: row.get(7)?,
                    description: row.get(8)?,
                    tags,
                    tag_ids,
                    duronly: false,
                    at: row.get::<_, String>(12)?.parse().unwrap(),
                    server_deleted_at: None,
                    user_id: row.get(11)?,
                    uid: None,
                    wid: None,
                    pid: None,
                })
            },
        )?;

        entries
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to parse time entries from database")
    }

    #[allow(dead_code)]
    pub fn save_projects(&self, projects: &[Project]) -> Result<usize> {
        let mut count = 0;
        let now = Utc::now().to_rfc3339();

        for project in projects {
            self.conn.execute(
                "INSERT OR REPLACE INTO projects
                (id, workspace_id, client_id, name, is_private, active, at, created_at, color, billable, synced_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                rusqlite::params![
                    project.id,
                    project.workspace_id,
                    project.client_id,
                    project.name,
                    project.is_private as i32,
                    project.active as i32,
                    project.at.to_rfc3339(),
                    project.created_at.to_rfc3339(),
                    project.color,
                    project.billable.map(|b| b as i32),
                    &now,
                ],
            )?;
            count += 1;
        }

        Ok(count)
    }

    pub fn update_sync_metadata(
        &self,
        resource_type: &str,
        last_entry_id: Option<i64>,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT OR REPLACE INTO sync_metadata (resource_type, last_sync, last_entry_id)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![resource_type, now, last_entry_id],
        )?;

        Ok(())
    }
}
