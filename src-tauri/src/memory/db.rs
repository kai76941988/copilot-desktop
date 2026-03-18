use crate::memory::MemoryRecordMessageParams;
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn get_memory_db_path(app: &AppHandle) -> Result<PathBuf, String> {
    let base_dir = app
        .path()
        .data_dir()
        .or_else(|_| app.path().config_dir())
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let product_name = app
        .config()
        .product_name
        .clone()
        .unwrap_or_else(|| "pake".to_string());

    let memory_dir = base_dir.join(product_name).join("memory");
    if !memory_dir.exists() {
        std::fs::create_dir_all(&memory_dir)
            .map_err(|e| format!("Failed to create memory dir: {}", e))?;
    }

    Ok(memory_dir.join("memory.db"))
}

fn open_connection(app: &AppHandle) -> Result<Connection, String> {
    let db_path = get_memory_db_path(app)?;
    let conn = Connection::open(db_path).map_err(|e| format!("Open DB failed: {}", e))?;
    let _ = conn.pragma_update(None, "journal_mode", &"WAL");
    let _ = conn.pragma_update(None, "synchronous", &"NORMAL");
    let _ = conn.pragma_update(None, "foreign_keys", &"ON");
    conn.busy_timeout(std::time::Duration::from_millis(2500))
        .map_err(|e| format!("DB busy timeout failed: {}", e))?;
    Ok(conn)
}

fn init_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS projects (
            project_id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            pinned INTEGER DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS sessions (
            session_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            title TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            source_url TEXT,
            archived INTEGER DEFAULT 0,
            FOREIGN KEY(project_id) REFERENCES projects(project_id)
        );

        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            project_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            source TEXT,
            message_order INTEGER,
            summary_group_id TEXT,
            tags TEXT,
            metadata_json TEXT,
            FOREIGN KEY(session_id) REFERENCES sessions(session_id),
            FOREIGN KEY(project_id) REFERENCES projects(project_id)
        );

        CREATE TABLE IF NOT EXISTS summaries (
            summary_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            session_id TEXT,
            summary_type TEXT NOT NULL,
            source_range_start INTEGER,
            source_range_end INTEGER,
            content TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id, created_at);
        CREATE INDEX IF NOT EXISTS idx_messages_project ON messages(project_id, created_at);
        CREATE INDEX IF NOT EXISTS idx_sessions_project ON sessions(project_id, updated_at);

        CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
            id,
            project_id,
            session_id,
            role,
            content,
            tokenize = 'porter'
        );
        "#,
    )
    .map_err(|e| format!("Init schema failed: {}", e))?;
    Ok(())
}

fn ensure_project(conn: &Connection, project_id: &str, ts: i64) -> Result<(), String> {
    conn.execute(
        "INSERT OR IGNORE INTO projects (project_id, name, created_at, updated_at, pinned) VALUES (?, ?, ?, ?, 0)",
        params![project_id, project_id, ts, ts],
    )
    .map_err(|e| format!("Insert project failed: {}", e))?;

    conn.execute(
        "UPDATE projects SET updated_at = ? WHERE project_id = ?",
        params![ts, project_id],
    )
    .map_err(|e| format!("Update project failed: {}", e))?;
    Ok(())
}

fn ensure_session(
    conn: &Connection,
    session_id: &str,
    project_id: &str,
    ts: i64,
    source_url: Option<&str>,
) -> Result<(), String> {
    conn.execute(
        "INSERT OR IGNORE INTO sessions (session_id, project_id, title, created_at, updated_at, source_url, archived) VALUES (?, ?, NULL, ?, ?, ?, 0)",
        params![session_id, project_id, ts, ts, source_url],
    )
    .map_err(|e| format!("Insert session failed: {}", e))?;

    conn.execute(
        "UPDATE sessions SET updated_at = ? WHERE session_id = ?",
        params![ts, session_id],
    )
    .map_err(|e| format!("Update session failed: {}", e))?;
    Ok(())
}

pub fn record_message(app: &AppHandle, params: MemoryRecordMessageParams) -> Result<(), String> {
    let conn = open_connection(app)?;
    init_schema(&conn)?;

    let MemoryRecordMessageParams {
        role,
        content,
        project_id,
        session_id,
        source,
        message_order,
        summary_group_id,
        tags,
        metadata_json,
        created_at,
    } = params;

    let ts = created_at.unwrap_or_else(now_ms);
    let project_id = project_id.unwrap_or_else(|| "default".to_string());
    let session_id = session_id.unwrap_or_else(|| format!("session_{}", ts));
    let source = source.unwrap_or_else(|| "copilot-webview".to_string());

    ensure_project(&conn, &project_id, ts)?;
    ensure_session(&conn, &session_id, &project_id, ts, None)?;

    let message_id = Uuid::new_v4().to_string();
    let message_order = message_order.unwrap_or(ts);

    conn.execute(
        "INSERT INTO messages (id, session_id, project_id, role, content, created_at, source, message_order, summary_group_id, tags, metadata_json)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            message_id,
            session_id,
            project_id,
            &role,
            &content,
            ts,
            source,
            message_order,
            summary_group_id,
            tags,
            metadata_json
        ],
    )
    .map_err(|e| format!("Insert message failed: {}", e))?;

    let _ = conn.execute(
        "INSERT INTO messages_fts (id, project_id, session_id, role, content) VALUES (?, ?, ?, ?, ?)",
        params![message_id, project_id, session_id, &role, &content],
    );

    Ok(())
}
