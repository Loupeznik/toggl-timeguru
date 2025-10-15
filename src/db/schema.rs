use anyhow::Result;
use rusqlite::Connection;

pub fn init_database(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS time_entries (
            id INTEGER PRIMARY KEY,
            workspace_id INTEGER NOT NULL,
            project_id INTEGER,
            task_id INTEGER,
            billable INTEGER NOT NULL,
            start TEXT NOT NULL,
            stop TEXT,
            duration INTEGER NOT NULL,
            description TEXT,
            tags TEXT,
            tag_ids TEXT,
            user_id INTEGER NOT NULL,
            at TEXT NOT NULL,
            synced_at TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_time_entries_start ON time_entries(start)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_time_entries_project_id ON time_entries(project_id)",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY,
            workspace_id INTEGER NOT NULL,
            client_id INTEGER,
            name TEXT NOT NULL,
            is_private INTEGER NOT NULL,
            active INTEGER NOT NULL,
            at TEXT NOT NULL,
            created_at TEXT NOT NULL,
            color TEXT NOT NULL,
            billable INTEGER,
            synced_at TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS sync_metadata (
            resource_type TEXT PRIMARY KEY,
            last_sync TEXT NOT NULL,
            last_entry_id INTEGER
        )",
        [],
    )?;

    Ok(())
}
