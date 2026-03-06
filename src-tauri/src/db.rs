// Phase 2: SQLite database layer — init_db, upsert_app, get_all_apps, increment_launch_count

use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Tauri managed state wrapper for the shared SQLite connection.
/// Arc<Mutex<>> allows the connection to be cloned and sent to indexer threads (Phase 3).
pub struct DbState(pub Arc<Mutex<Connection>>);

/// A single app record in the apps table.
/// id is the normalized exe path (natural unique key — no hash crate needed).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRecord {
    pub id: String,
    pub name: String,
    pub path: String,
    pub icon_path: Option<String>,
    pub source: String,
    pub last_launched: Option<i64>, // Unix timestamp seconds; NULL if never launched
    pub launch_count: i64,
}

/// Runs schema DDL on an existing connection.
/// Separated from init_db() so unit tests can use Connection::open_in_memory().
pub fn init_db_connection(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS apps (
            id             TEXT PRIMARY KEY,
            name           TEXT NOT NULL,
            path           TEXT NOT NULL,
            icon_path      TEXT,
            source         TEXT NOT NULL,
            last_launched  INTEGER,
            launch_count   INTEGER NOT NULL DEFAULT 0
        );
    ",
    )?;
    Ok(())
}

/// Opens (or creates) the SQLite file at db_path, runs schema DDL, returns the Connection.
/// On corrupted file: delete and recreate (silent reset per CONTEXT.md decision).
/// Returns Result<Connection> — caller wraps in Arc<Mutex<>> and stores as managed state.
pub fn init_db(db_path: &Path) -> Result<Connection> {
    // Silent reset on corruption: attempt open, if DDL fails delete and retry once.
    match try_init_db(db_path) {
        Ok(conn) => Ok(conn),
        Err(_) => {
            // Corrupted — delete and recreate (launch history lost, app starts normally)
            let _ = std::fs::remove_file(db_path);
            try_init_db(db_path)
        }
    }
}

fn try_init_db(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    init_db_connection(&conn)?;
    Ok(conn)
}

/// Inserts or updates an app record.
/// Uses ON CONFLICT DO UPDATE to update only discovery fields (name, path, icon_path, source)
/// while preserving existing launch_count and last_launched values.
/// Per CONTEXT.md: returns Result<()> — callers decide how to handle errors.
pub fn upsert_app(conn: &Connection, app: &AppRecord) -> Result<()> {
    conn.execute(
        "INSERT INTO apps (id, name, path, icon_path, source, last_launched, launch_count)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(id) DO UPDATE SET
             name         = excluded.name,
             path         = excluded.path,
             icon_path    = excluded.icon_path,
             source       = excluded.source",
        rusqlite::params![
            app.id,
            app.name,
            app.path,
            app.icon_path,
            app.source,
            app.last_launched,
            app.launch_count,
        ],
    )?;
    Ok(())
}

/// Returns all app records as a Vec<AppRecord>.
/// Called by Phase 4 search engine to build the nucleo search index.
pub fn get_all_apps(conn: &Connection) -> Result<Vec<AppRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, path, icon_path, source, last_launched, launch_count
         FROM apps",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(AppRecord {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            icon_path: row.get(3)?,
            source: row.get(4)?,
            last_launched: row.get(5)?,
            launch_count: row.get(6)?,
        })
    })?;
    rows.collect()
}

/// Increments launch_count by 1 and sets last_launched to current Unix timestamp for the app with id.
/// Uses std::time (no chrono crate) per RESEARCH.md note.
pub fn increment_launch_count(conn: &Connection, id: &str) -> Result<()> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    conn.execute(
        "UPDATE apps SET
             launch_count  = launch_count + 1,
             last_launched = ?1
         WHERE id = ?2",
        rusqlite::params![now, id],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_db_connection(&conn).unwrap();
        conn
    }

    fn sample_app(id: &str) -> AppRecord {
        AppRecord {
            id: id.to_string(),
            name: "Test App".to_string(),
            path: format!("C:\\apps\\{}.exe", id),
            icon_path: None,
            source: "start_menu".to_string(),
            last_launched: None,
            launch_count: 0,
        }
    }

    #[test]
    fn test_schema_init() {
        let conn = setup();
        // Verify the table exists and accepts a basic insert
        let app = sample_app("schema_test");
        upsert_app(&conn, &app).unwrap();
        let all = get_all_apps(&conn).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, "schema_test");
    }

    #[test]
    fn test_upsert_insert() {
        let conn = setup();
        let app = sample_app("chrome");
        upsert_app(&conn, &app).unwrap();
        let all = get_all_apps(&conn).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "Test App");
    }

    #[test]
    fn test_upsert_update_preserves_launch_count() {
        let conn = setup();
        let app = sample_app("notepad");
        upsert_app(&conn, &app).unwrap();
        // Simulate a launch so count > 0
        increment_launch_count(&conn, "notepad").unwrap();
        let before = get_all_apps(&conn).unwrap();
        assert_eq!(before[0].launch_count, 1);

        // Re-upsert (as indexer would on re-index) with updated name
        let updated = AppRecord {
            name: "Notepad Updated".to_string(),
            ..sample_app("notepad")
        };
        upsert_app(&conn, &updated).unwrap();

        let after = get_all_apps(&conn).unwrap();
        assert_eq!(after.len(), 1, "upsert must not duplicate the row");
        assert_eq!(after[0].name, "Notepad Updated");
        assert_eq!(after[0].launch_count, 1, "launch_count must be preserved by upsert");
    }

    #[test]
    fn test_get_all_apps_empty() {
        let conn = setup();
        let all = get_all_apps(&conn).unwrap();
        assert!(all.is_empty());
    }

    #[test]
    fn test_increment_launch_count() {
        let conn = setup();
        let app = sample_app("firefox");
        upsert_app(&conn, &app).unwrap();
        increment_launch_count(&conn, "firefox").unwrap();
        let all = get_all_apps(&conn).unwrap();
        assert_eq!(all[0].launch_count, 1);
        assert!(all[0].last_launched.is_some());
        assert!(all[0].last_launched.unwrap() > 0);
    }
}
