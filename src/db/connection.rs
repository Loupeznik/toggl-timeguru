use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

use super::schema::init_database;
use crate::toggl::models::{Project, TimeEntry};

pub struct Database {
    conn: Mutex<Connection>,
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

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn save_time_entries(&self, entries: &[TimeEntry]) -> Result<usize> {
        let mut count = 0;
        let now = Utc::now().to_rfc3339();
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock database: {}", e))?;

        for entry in entries {
            let tags_json = entry
                .tags
                .as_ref()
                .map(|t| serde_json::to_string(t).unwrap_or_default());
            let tag_ids_json = entry
                .tag_ids
                .as_ref()
                .map(|t| serde_json::to_string(t).unwrap_or_default());

            conn.execute(
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
        user_id: Option<i64>,
    ) -> Result<Vec<TimeEntry>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock database: {}", e))?;

        let query = if user_id.is_some() {
            "SELECT id, workspace_id, project_id, task_id, billable, start, stop, duration,
                    description, tags, tag_ids, user_id, at
             FROM time_entries
             WHERE start >= ?1 AND start <= ?2 AND user_id = ?3
             ORDER BY start DESC"
        } else {
            "SELECT id, workspace_id, project_id, task_id, billable, start, stop, duration,
                    description, tags, tag_ids, user_id, at
             FROM time_entries
             WHERE start >= ?1 AND start <= ?2
             ORDER BY start DESC"
        };

        let mut stmt = conn.prepare(query)?;

        let row_mapper = |row: &rusqlite::Row| {
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
        };

        let entries = if let Some(uid) = user_id {
            stmt.query_map(
                rusqlite::params![start_date.to_rfc3339(), end_date.to_rfc3339(), uid],
                row_mapper,
            )?
        } else {
            stmt.query_map(
                rusqlite::params![start_date.to_rfc3339(), end_date.to_rfc3339()],
                row_mapper,
            )?
        };

        entries
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to parse time entries from database")
    }

    pub fn save_projects(&self, projects: &[Project]) -> Result<usize> {
        let mut count = 0;
        let now = Utc::now().to_rfc3339();
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock database: {}", e))?;

        for project in projects {
            conn.execute(
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

    pub fn get_projects(&self) -> Result<Vec<Project>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock database: {}", e))?;

        let mut stmt = conn.prepare(
            "SELECT id, workspace_id, client_id, name, is_private, active, at, created_at, color, billable
             FROM projects
             WHERE active = 1
             ORDER BY name ASC",
        )?;

        let projects = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                workspace_id: row.get(1)?,
                client_id: row.get(2)?,
                name: row.get(3)?,
                is_private: row.get::<_, i32>(4)? != 0,
                active: row.get::<_, i32>(5)? != 0,
                at: row.get::<_, String>(6)?.parse().unwrap(),
                created_at: row.get::<_, String>(7)?.parse().unwrap(),
                color: row.get(8)?,
                billable: row.get::<_, Option<i32>>(9)?.map(|b| b != 0),
                template: None,
                auto_estimates: None,
                estimated_hours: None,
                rate: None,
                currency: None,
            })
        })?;

        projects
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to parse projects from database")
    }

    pub fn update_sync_metadata(
        &self,
        resource_type: &str,
        last_entry_id: Option<i64>,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock database: {}", e))?;

        conn.execute(
            "INSERT OR REPLACE INTO sync_metadata (resource_type, last_sync, last_entry_id)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![resource_type, now, last_entry_id],
        )?;

        Ok(())
    }

    /// Updates the project associated with a specific time entry.
    ///
    /// # Parameters
    /// - `entry_id`: The ID of the time entry to update.
    /// - `project_id`: The new project ID to associate with the time entry. Use `None` to remove the association.
    ///
    /// # Returns
    /// Returns `Ok(())` if the update was successful, or an error otherwise.
    ///
    /// # Side Effects
    /// This method updates both the `project_id` and the `synced_at` timestamp for the specified time entry.
    pub fn update_time_entry_project(&self, entry_id: i64, project_id: Option<i64>) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock database: {}", e))?;

        conn.execute(
            "UPDATE time_entries SET project_id = ?1, synced_at = ?2 WHERE id = ?3",
            rusqlite::params![project_id, now, entry_id],
        )?;

        Ok(())
    }

    /// Updates the description of a specific time entry.
    ///
    /// # Parameters
    /// - `entry_id`: The ID of the time entry to update.
    /// - `description`: The new description for the time entry.
    ///
    /// # Returns
    /// Returns `Ok(())` if the update was successful, or an error otherwise.
    ///
    /// # Side Effects
    /// This method updates both the `description` and the `synced_at` timestamp for the specified time entry.
    pub fn update_time_entry_description(&self, entry_id: i64, description: String) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock database: {}", e))?;

        conn.execute(
            "UPDATE time_entries SET description = ?1, synced_at = ?2 WHERE id = ?3",
            rusqlite::params![description, now, entry_id],
        )?;

        Ok(())
    }
}
