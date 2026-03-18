use crate::memory::{
    MemoryCreateProjectParams, MemoryGetContextPackParams, MemoryListSessionsParams,
    MemoryListMessagesParams, MemoryListSummariesParams, MemoryMessageInfo, MemoryProjectInfo,
    MemoryRecordMessageParams, MemorySearchItem, MemorySearchParams, MemorySearchSummariesParams,
    MemorySessionInfo, MemorySummaryInfo,
};
use rusqlite::{params, params_from_iter, types::Value, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
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

const CHUNK_SIZE: i64 = 12;
const SESSION_SUMMARY_CHUNKS: i64 = 3;
const PROJECT_SUMMARY_SESSIONS: i64 = 3;
const SUMMARY_MAX_CHARS: usize = 800;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ProjectStructSummary {
    background: Vec<String>,
    key_facts: Vec<String>,
    progress: Vec<String>,
    todo: Vec<String>,
    constraints: Vec<String>,
    updated_at: i64,
}

fn merge_unique(mut base: Vec<String>, adds: Vec<String>, max_lines: usize) -> Vec<String> {
    for line in adds {
        if !base.iter().any(|x| x == &line) && !line.trim().is_empty() {
            base.push(line);
        }
        if base.len() >= max_lines {
            break;
        }
    }
    if base.len() > max_lines {
        base.truncate(max_lines);
    }
    base
}

fn project_struct_to_text(s: &ProjectStructSummary) -> String {
    let join = |v: &Vec<String>| if v.is_empty() { "暂无".to_string() } else { v.join("\n") };
    format!(
        "【项目背景】\n{}\n\n【关键结论】\n{}\n\n【最近进展】\n{}\n\n【当前待办】\n{}\n\n【必须遵守的约束】\n{}",
        join(&s.background),
        join(&s.key_facts),
        join(&s.progress),
        join(&s.todo),
        join(&s.constraints)
    )
}

fn collapse_whitespace(input: &str) -> String {
    input
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn summarize_text(input: &str, max_chars: usize) -> String {
    let compact = collapse_whitespace(input);
    if compact.len() <= max_chars {
        compact
    } else {
        let mut s = compact[..max_chars].to_string();
        s.push_str("...");
        s
    }
}

fn extract_lines_by_keywords(input: &str, keywords: &[&str], max_lines: usize) -> String {
    let mut result: Vec<String> = Vec::new();
    for line in input.lines() {
        let lower = line.to_lowercase();
        if keywords.iter().any(|k| lower.contains(k)) {
            result.push(line.trim().to_string());
        }
        if result.len() >= max_lines {
            break;
        }
    }
    if result.is_empty() {
        "暂无".to_string()
    } else {
        result.join("\n")
    }
}

fn parse_struct_summary(content: &str) -> Option<ProjectStructSummary> {
    serde_json::from_str::<ProjectStructSummary>(content).ok()
}

fn normalize_lines_from_messages(messages: &[(String, String)]) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for (role, content) in messages {
        let trimmed = collapse_whitespace(content);
        if trimmed.is_empty() {
            continue;
        }
        let line = if trimmed.len() > 240 {
            format!("{}: {}...", role, &trimmed[..220])
        } else {
            format!("{}: {}", role, trimmed)
        };
        lines.push(line);
    }
    lines
}

fn pick_lines_with_keywords(lines: &[String], keywords: &[&str], max_lines: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in lines {
        let lower = line.to_lowercase();
        if keywords.iter().any(|k| lower.contains(k)) {
            out.push(line.clone());
            if out.len() >= max_lines {
                break;
            }
        }
    }
    out
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

        CREATE VIRTUAL TABLE IF NOT EXISTS summaries_fts USING fts5(
            summary_id,
            project_id,
            session_id,
            summary_type,
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

fn get_message_count(conn: &Connection, session_id: &str) -> Result<i64, String> {
    let mut stmt = conn
        .prepare("SELECT COUNT(1) FROM messages WHERE session_id = ?")
        .map_err(|e| format!("Prepare count failed: {}", e))?;
    let count: i64 = stmt
        .query_row(params![session_id], |row| row.get(0))
        .map_err(|e| format!("Query count failed: {}", e))?;
    Ok(count)
}

fn fetch_recent_messages(
    conn: &Connection,
    session_id: &str,
    limit: i64,
) -> Result<Vec<(String, String, i64)>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT role, content, message_order
             FROM messages
             WHERE session_id = ?
             ORDER BY created_at DESC
             LIMIT ?",
        )
        .map_err(|e| format!("Prepare fetch messages failed: {}", e))?;

    let rows = stmt
        .query_map(params![session_id, limit], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .map_err(|e| format!("Query messages failed: {}", e))?;

    let mut out: Vec<(String, String, i64)> = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| format!("Row parse failed: {}", e))?);
    }

    out.reverse();
    Ok(out)
}

fn build_chunk_summary(
    conn: &Connection,
    session_id: &str,
) -> Result<Option<(i64, i64, String)>, String> {
    let messages = fetch_recent_messages(conn, session_id, CHUNK_SIZE)?;
    if messages.len() < CHUNK_SIZE as usize {
        return Ok(None);
    }

    let range_start = messages.first().map(|m| m.2).unwrap_or(0);
    let range_end = messages.last().map(|m| m.2).unwrap_or(0);
    let mut lines: Vec<String> = Vec::new();
    for (role, content, _) in messages {
        let prefix = if role == "user" { "U" } else { "A" };
        lines.push(format!("{}: {}", prefix, content));
    }
    let joined = lines.join("\n");
    let summary = summarize_text(&joined, SUMMARY_MAX_CHARS);
    Ok(Some((range_start, range_end, summary)))
}

fn get_last_chunk_end(conn: &Connection, session_id: &str) -> Result<Option<i64>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT source_range_end
             FROM summaries
             WHERE session_id = ? AND summary_type = 'chunk'
             ORDER BY created_at DESC
             LIMIT 1",
        )
        .map_err(|e| format!("Prepare last chunk failed: {}", e))?;
    let row = stmt
        .query_row(params![session_id], |r| r.get(0))
        .optional()
        .map_err(|e| format!("Query last chunk failed: {}", e))?;
    Ok(row)
}

fn upsert_summary(
    conn: &Connection,
    project_id: &str,
    session_id: Option<&str>,
    summary_type: &str,
    content: &str,
    range_start: Option<i64>,
    range_end: Option<i64>,
) -> Result<(), String> {
    if summary_type != "chunk" {
        let _ = conn.execute(
            "DELETE FROM summaries WHERE summary_type = ? AND project_id = ? AND session_id IS ?",
            params![summary_type, project_id, session_id],
        );
        let _ = conn.execute(
            "DELETE FROM summaries_fts WHERE summary_type = ? AND project_id = ? AND session_id IS ?",
            params![summary_type, project_id, session_id],
        );
    }

    let summary_id = Uuid::new_v4().to_string();
    let ts = now_ms();
    conn.execute(
        "INSERT INTO summaries (summary_id, project_id, session_id, summary_type, source_range_start, source_range_end, content, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            summary_id,
            project_id,
            session_id,
            summary_type,
            range_start,
            range_end,
            content,
            ts
        ],
    )
    .map_err(|e| format!("Insert summary failed: {}", e))?;

    let _ = conn.execute(
        "INSERT INTO summaries_fts (summary_id, project_id, session_id, summary_type, content) VALUES (?, ?, ?, ?, ?)",
        params![summary_id, project_id, session_id, summary_type, content],
    );
    Ok(())
}

fn build_session_summary(
    conn: &Connection,
    project_id: &str,
    session_id: &str,
) -> Result<String, String> {
    let mut stmt = conn
        .prepare(
            "SELECT content FROM summaries
             WHERE session_id = ? AND summary_type = 'chunk'
             ORDER BY created_at DESC
             LIMIT ?",
        )
        .map_err(|e| format!("Prepare session summary failed: {}", e))?;

    let rows = stmt
        .query_map(params![session_id, SESSION_SUMMARY_CHUNKS], |row| row.get::<_, String>(0))
        .map_err(|e| format!("Query session summaries failed: {}", e))?;

    let mut chunks: Vec<String> = Vec::new();
    for r in rows {
        chunks.push(r.map_err(|e| format!("Row parse failed: {}", e))?);
    }

    if chunks.is_empty() {
        let messages = fetch_recent_messages(conn, session_id, CHUNK_SIZE)?;
        let joined = messages
            .into_iter()
            .map(|(role, content, _)| format!("{}: {}", role, content))
            .collect::<Vec<_>>()
            .join("\n");
        return Ok(summarize_text(&joined, SUMMARY_MAX_CHARS));
    }

    let joined = chunks.into_iter().rev().collect::<Vec<_>>().join("\n");
    Ok(summarize_text(&joined, SUMMARY_MAX_CHARS))
}

fn build_project_summary(conn: &Connection, project_id: &str) -> Result<String, String> {
    let mut stmt = conn
        .prepare(
            "SELECT content FROM summaries
             WHERE project_id = ? AND summary_type = 'session'
             ORDER BY created_at DESC
             LIMIT ?",
        )
        .map_err(|e| format!("Prepare project summary failed: {}", e))?;

    let rows = stmt
        .query_map(params![project_id, PROJECT_SUMMARY_SESSIONS], |row| {
            row.get::<_, String>(0)
        })
        .map_err(|e| format!("Query project summaries failed: {}", e))?;

    let mut items: Vec<String> = Vec::new();
    for r in rows {
        items.push(r.map_err(|e| format!("Row parse failed: {}", e))?);
    }

    if items.is_empty() {
        return Ok("暂无项目摘要".to_string());
    }

    let joined = items.into_iter().rev().collect::<Vec<_>>().join("\n");
    Ok(summarize_text(&joined, SUMMARY_MAX_CHARS))
}

fn build_project_struct_summary(
    conn: &Connection,
    project_id: &str,
    session_id: &str,
) -> Result<ProjectStructSummary, String> {
    let existing = get_latest_summary(conn, project_id, None, "project_struct")?;
    let mut summary = existing
        .as_deref()
        .and_then(parse_struct_summary)
        .unwrap_or_default();

    let recent = fetch_recent_messages(conn, session_id, 20)?
        .into_iter()
        .map(|(role, content, _)| (role, content))
        .collect::<Vec<_>>();

    if summary.background.is_empty() {
        if let Some(first_user) = recent.iter().find(|(r, _)| r == "user") {
            summary
                .background
                .push(summarize_text(&first_user.1, 200));
        }
    }

    let lines = normalize_lines_from_messages(&recent);
    let todo_lines = pick_lines_with_keywords(
        &lines,
        &[
            "todo",
            "待办",
            "下一步",
            "next",
            "计划",
            "需要",
            "to do",
        ],
        6,
    );
    let constraint_lines = pick_lines_with_keywords(
        &lines,
        &["必须", "不要", "禁止", "only", "must", "constraint", "限制"],
        6,
    );
    let fact_lines = pick_lines_with_keywords(
        &lines,
        &["结论", "决定", "确认", "confirmed", "final", "关键", "事实", "因此"],
        6,
    );
    let progress_lines = pick_lines_with_keywords(
        &lines,
        &["完成", "done", "进展", "progress", "已完成", "当前", "working"],
        6,
    );

    summary.todo = merge_unique(summary.todo, todo_lines, 8);
    summary.constraints = merge_unique(summary.constraints, constraint_lines, 8);
    summary.key_facts = merge_unique(summary.key_facts, fact_lines, 8);
    summary.progress = merge_unique(summary.progress, progress_lines, 8);
    summary.updated_at = now_ms();

    Ok(summary)
}

fn maybe_update_summaries(
    conn: &Connection,
    project_id: &str,
    session_id: &str,
) -> Result<(), String> {
    let count = get_message_count(conn, session_id)?;
    if count == 0 {
        return Ok(());
    }

    let should_make_chunk = count % CHUNK_SIZE == 0;
    if should_make_chunk {
        let last_chunk_end = get_last_chunk_end(conn, session_id)?;
        if let Some((range_start, range_end, summary)) = build_chunk_summary(conn, session_id)? {
            if last_chunk_end.unwrap_or(-1) != range_end {
                upsert_summary(
                    conn,
                    project_id,
                    Some(session_id),
                    "chunk",
                    &summary,
                    Some(range_start),
                    Some(range_end),
                )?;
            }
        }
    }

    if should_make_chunk || count == 1 {
        let session_summary = build_session_summary(conn, project_id, session_id)?;
        upsert_summary(
            conn,
            project_id,
            Some(session_id),
            "session",
            &session_summary,
            None,
            None,
        )?;
        if let Ok(struct_summary) = build_project_struct_summary(conn, project_id, session_id) {
            if let Ok(json) = serde_json::to_string(&struct_summary) {
                let _ = upsert_summary(
                    conn,
                    project_id,
                    None,
                    "project_struct",
                    &json,
                    None,
                    None,
                );
            }
            let project_text = project_struct_to_text(&struct_summary);
            upsert_summary(
                conn,
                project_id,
                None,
                "project",
                &project_text,
                None,
                None,
            )?;
        } else {
            let project_summary = build_project_summary(conn, project_id)?;
            upsert_summary(
                conn,
                project_id,
                None,
                "project",
                &project_summary,
                None,
                None,
            )?;
        }
    }

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

    let _ = maybe_update_summaries(&conn, &project_id, &session_id);

    Ok(())
}

pub fn create_project(
    app: &AppHandle,
    params: MemoryCreateProjectParams,
) -> Result<String, String> {
    let conn = open_connection(app)?;
    init_schema(&conn)?;

    let project_id = Uuid::new_v4().to_string();
    let ts = now_ms();
    conn.execute(
        "INSERT INTO projects (project_id, name, description, created_at, updated_at, pinned) VALUES (?, ?, ?, ?, ?, 0)",
        params![project_id, params.name, params.description, ts, ts],
    )
    .map_err(|e| format!("Create project failed: {}", e))?;

    Ok(project_id)
}

pub fn list_projects(app: &AppHandle) -> Result<Vec<MemoryProjectInfo>, String> {
    let conn = open_connection(app)?;
    init_schema(&conn)?;

    let mut stmt = conn
        .prepare(
            "SELECT project_id, name, description, updated_at, pinned
             FROM projects
             ORDER BY pinned DESC, updated_at DESC
             LIMIT 200",
        )
        .map_err(|e| format!("Prepare list projects failed: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(MemoryProjectInfo {
                project_id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                updated_at: row.get(3)?,
                pinned: row.get(4)?,
            })
        })
        .map_err(|e| format!("Query projects failed: {}", e))?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| format!("Row parse failed: {}", e))?);
    }
    Ok(out)
}

pub fn list_sessions(
    app: &AppHandle,
    params: MemoryListSessionsParams,
) -> Result<Vec<MemorySessionInfo>, String> {
    let conn = open_connection(app)?;
    init_schema(&conn)?;

    let mut stmt = conn
        .prepare(
            "SELECT session_id, project_id, title, updated_at, source_url, archived
             FROM sessions
             WHERE project_id = ?
             ORDER BY updated_at DESC
             LIMIT 200",
        )
        .map_err(|e| format!("Prepare list sessions failed: {}", e))?;

    let rows = stmt
        .query_map(params![params.project_id], |row| {
            Ok(MemorySessionInfo {
                session_id: row.get(0)?,
                project_id: row.get(1)?,
                title: row.get(2)?,
                updated_at: row.get(3)?,
                source_url: row.get(4)?,
                archived: row.get(5)?,
            })
        })
        .map_err(|e| format!("Query sessions failed: {}", e))?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| format!("Row parse failed: {}", e))?);
    }
    Ok(out)
}

fn get_latest_summary(
    conn: &Connection,
    project_id: &str,
    session_id: Option<&str>,
    summary_type: &str,
) -> Result<Option<String>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT content FROM summaries
             WHERE project_id = ?
             AND summary_type = ?
             AND session_id IS ?
             ORDER BY created_at DESC
             LIMIT 1",
        )
        .map_err(|e| format!("Prepare latest summary failed: {}", e))?;

    let row = stmt
        .query_row(params![project_id, summary_type, session_id], |r| r.get(0))
        .optional()
        .map_err(|e| format!("Query latest summary failed: {}", e))?;
    Ok(row)
}

fn build_context_pack_text(
    project_summary: Option<String>,
    session_summary: Option<String>,
    recent_messages: Vec<String>,
) -> String {
    let background = project_summary.clone().unwrap_or_else(|| "暂无".to_string());
    let key_facts = project_summary.clone().unwrap_or_else(|| "暂无".to_string());
    let progress = session_summary.unwrap_or_else(|| "暂无".to_string());
    let recent = if recent_messages.is_empty() {
        "暂无".to_string()
    } else {
        recent_messages.join("\n")
    };

    let todo = extract_lines_by_keywords(&recent, &["todo", "待办", "next", "计划"], 5);
    let constraints = extract_lines_by_keywords(&recent, &["must", "禁止", "不要", "约束"], 5);

    format!(
        "【项目背景】\n{background}\n\n【关键结论】\n{key_facts}\n\n【最近进展】\n{progress}\n\n【最近对话摘录】\n{recent}\n\n【当前待办】\n{todo}\n\n【必须遵守的约束】\n{constraints}\n\n【请你基于以上上下文继续，不要重复询问已经确认的信息】"
    )
}

fn build_context_pack_from_struct(
    summary: &ProjectStructSummary,
    session_summary: Option<String>,
    recent_messages: Vec<String>,
) -> String {
    let join = |v: &Vec<String>| if v.is_empty() { "暂无".to_string() } else { v.join("\n") };
    let background = join(&summary.background);
    let key_facts = join(&summary.key_facts);
    let progress = if let Some(s) = session_summary {
        if s.trim().is_empty() { join(&summary.progress) } else { s }
    } else {
        join(&summary.progress)
    };
    let recent = if recent_messages.is_empty() {
        "暂无".to_string()
    } else {
        recent_messages.join("\n")
    };
    let todo = join(&summary.todo);
    let constraints = join(&summary.constraints);

    format!(
        "【项目背景】\n{background}\n\n【关键结论】\n{key_facts}\n\n【最近进展】\n{progress}\n\n【最近对话摘录】\n{recent}\n\n【当前待办】\n{todo}\n\n【必须遵守的约束】\n{constraints}\n\n【请你基于以上上下文继续，不要重复询问已经确认的信息】"
    )
}

pub fn get_context_pack(
    app: &AppHandle,
    params: MemoryGetContextPackParams,
) -> Result<String, String> {
    let conn = open_connection(app)?;
    init_schema(&conn)?;

    let project_id = params.project_id.unwrap_or_else(|| "default".to_string());
    let session_id = params.session_id;

    let project_summary = get_latest_summary(&conn, &project_id, None, "project")?;
    let project_struct = get_latest_summary(&conn, &project_id, None, "project_struct")?;
    let session_summary = if let Some(ref sid) = session_id {
        get_latest_summary(&conn, &project_id, Some(sid), "session")?
    } else {
        None
    };

    let mut recent_messages: Vec<String> = Vec::new();
    if let Some(sid) = session_id.as_deref() {
        let msgs = fetch_recent_messages(&conn, sid, 8)?;
        recent_messages = msgs
            .into_iter()
            .map(|(role, content, _)| format!("{}: {}", role, content))
            .collect();
    }

    if recent_messages.is_empty() {
        let mut stmt = conn
            .prepare(
                "SELECT role, content FROM messages
                 WHERE project_id = ?
                 ORDER BY created_at DESC
                 LIMIT 8",
            )
            .map_err(|e| format!("Prepare project messages failed: {}", e))?;
        let rows = stmt
            .query_map(params![project_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| format!("Query project messages failed: {}", e))?;

        for r in rows {
            let (role, content) = r.map_err(|e| format!("Row parse failed: {}", e))?;
            recent_messages.push(format!("{}: {}", role, content));
        }
    }

    if let Some(struct_json) = project_struct {
        if let Some(struct_summary) = parse_struct_summary(&struct_json) {
            return Ok(build_context_pack_from_struct(
                &struct_summary,
                session_summary,
                recent_messages,
            ));
        }
    }

    Ok(build_context_pack_text(
        project_summary,
        session_summary,
        recent_messages,
    ))
}

pub fn search_messages(
    app: &AppHandle,
    params: MemorySearchParams,
) -> Result<Vec<MemorySearchItem>, String> {
    let conn = open_connection(app)?;
    init_schema(&conn)?;

    let query = params.query.trim().to_string();
    if query.is_empty() {
        return Ok(Vec::new());
    }

    let limit = params.limit.unwrap_or(50).min(200);
    let tokens: Vec<String> = query
        .split_whitespace()
        .map(|t| format!("{}*", t.replace('"', "")))
        .collect();
    let fts_query = if tokens.is_empty() {
        query.clone()
    } else {
        tokens.join(" ")
    };

    let mut out: Vec<MemorySearchItem> = Vec::new();
    if let Some(project_id) = params.project_id {
        let mut stmt = conn
            .prepare(
                "SELECT m.id, m.project_id, m.session_id, m.role, m.content, m.created_at
                 FROM messages_fts f
                 JOIN messages m ON m.id = f.id
                 WHERE f MATCH ? AND m.project_id = ?
                 ORDER BY m.created_at DESC
                 LIMIT ?",
            )
            .map_err(|e| format!("Prepare search failed: {}", e))?;
        let rows = stmt
            .query_map(params![fts_query, project_id, limit], |row| {
                Ok(MemorySearchItem {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    session_id: row.get(2)?,
                    role: row.get(3)?,
                    content: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|e| format!("Query search failed: {}", e))?;
        for r in rows {
            out.push(r.map_err(|e| format!("Row parse failed: {}", e))?);
        }
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT m.id, m.project_id, m.session_id, m.role, m.content, m.created_at
                 FROM messages_fts f
                 JOIN messages m ON m.id = f.id
                 WHERE f MATCH ?
                 ORDER BY m.created_at DESC
                 LIMIT ?",
            )
            .map_err(|e| format!("Prepare search failed: {}", e))?;
        let rows = stmt
            .query_map(params![fts_query, limit], |row| {
                Ok(MemorySearchItem {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    session_id: row.get(2)?,
                    role: row.get(3)?,
                    content: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|e| format!("Query search failed: {}", e))?;
        for r in rows {
            out.push(r.map_err(|e| format!("Row parse failed: {}", e))?);
        }
    }

    Ok(out)
}

pub fn search_summaries(
    app: &AppHandle,
    params: MemorySearchSummariesParams,
) -> Result<Vec<MemorySummaryInfo>, String> {
    let conn = open_connection(app)?;
    init_schema(&conn)?;

    let query = params.query.trim().to_string();
    if query.is_empty() {
        return Ok(Vec::new());
    }

    let limit = params.limit.unwrap_or(50).min(200);
    let tokens: Vec<String> = query
        .split_whitespace()
        .map(|t| format!("{}*", t.replace('"', "")))
        .collect();
    let fts_query = if tokens.is_empty() {
        query.clone()
    } else {
        tokens.join(" ")
    };

    let mut out: Vec<MemorySummaryInfo> = Vec::new();
    if let Some(project_id) = params.project_id {
        let mut stmt = conn
            .prepare(
                "SELECT s.summary_id, s.project_id, s.session_id, s.summary_type,
                        s.source_range_start, s.source_range_end, s.content, s.created_at
                 FROM summaries_fts f
                 JOIN summaries s ON s.summary_id = f.summary_id
                 WHERE f MATCH ? AND s.project_id = ?
                 ORDER BY s.created_at DESC
                 LIMIT ?",
            )
            .map_err(|e| format!("Prepare search summaries failed: {}", e))?;
        let rows = stmt
            .query_map(params![fts_query, project_id, limit], |row| {
                Ok(MemorySummaryInfo {
                    summary_id: row.get(0)?,
                    project_id: row.get(1)?,
                    session_id: row.get(2)?,
                    summary_type: row.get(3)?,
                    source_range_start: row.get(4)?,
                    source_range_end: row.get(5)?,
                    content: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .map_err(|e| format!("Query search summaries failed: {}", e))?;
        for r in rows {
            out.push(r.map_err(|e| format!("Row parse failed: {}", e))?);
        }
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT s.summary_id, s.project_id, s.session_id, s.summary_type,
                        s.source_range_start, s.source_range_end, s.content, s.created_at
                 FROM summaries_fts f
                 JOIN summaries s ON s.summary_id = f.summary_id
                 WHERE f MATCH ?
                 ORDER BY s.created_at DESC
                 LIMIT ?",
            )
            .map_err(|e| format!("Prepare search summaries failed: {}", e))?;
        let rows = stmt
            .query_map(params![fts_query, limit], |row| {
                Ok(MemorySummaryInfo {
                    summary_id: row.get(0)?,
                    project_id: row.get(1)?,
                    session_id: row.get(2)?,
                    summary_type: row.get(3)?,
                    source_range_start: row.get(4)?,
                    source_range_end: row.get(5)?,
                    content: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .map_err(|e| format!("Query search summaries failed: {}", e))?;
        for r in rows {
            out.push(r.map_err(|e| format!("Row parse failed: {}", e))?);
        }
    }

    Ok(out)
}

pub fn list_summaries(
    app: &AppHandle,
    params: MemoryListSummariesParams,
) -> Result<Vec<MemorySummaryInfo>, String> {
    let conn = open_connection(app)?;
    init_schema(&conn)?;

    let mut sql = String::from(
        "SELECT summary_id, project_id, session_id, summary_type, source_range_start, source_range_end, content, created_at
         FROM summaries WHERE project_id = ?",
    );
    let mut values: Vec<Value> = vec![Value::from(params.project_id.clone())];

    if let Some(ref session_id) = params.session_id {
        sql.push_str(" AND session_id = ?");
        values.push(Value::from(session_id.clone()));
    }
    if let Some(ref summary_type) = params.summary_type {
        sql.push_str(" AND summary_type = ?");
        values.push(Value::from(summary_type.clone()));
    }

    let limit = params.limit.unwrap_or(50).min(200);
    sql.push_str(" ORDER BY created_at DESC LIMIT ?");
    values.push(Value::from(limit));

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Prepare list summaries failed: {}", e))?;
    let rows = stmt
        .query_map(params_from_iter(values), |row| {
            Ok(MemorySummaryInfo {
                summary_id: row.get(0)?,
                project_id: row.get(1)?,
                session_id: row.get(2)?,
                summary_type: row.get(3)?,
                source_range_start: row.get(4)?,
                source_range_end: row.get(5)?,
                content: row.get(6)?,
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("Query summaries failed: {}", e))?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| format!("Row parse failed: {}", e))?);
    }
    Ok(out)
}

pub fn list_messages(
    app: &AppHandle,
    params: MemoryListMessagesParams,
) -> Result<Vec<MemoryMessageInfo>, String> {
    let conn = open_connection(app)?;
    init_schema(&conn)?;

    let limit = params.limit.unwrap_or(50).min(200);
    let offset = params.offset.unwrap_or(0).max(0);

    let mut stmt = conn
        .prepare(
            "SELECT id, session_id, role, content, created_at, message_order
             FROM messages
             WHERE session_id = ?
             ORDER BY created_at DESC
             LIMIT ? OFFSET ?",
        )
        .map_err(|e| format!("Prepare list messages failed: {}", e))?;

    let rows = stmt
        .query_map(params![params.session_id, limit, offset], |row| {
            Ok(MemoryMessageInfo {
                id: row.get(0)?,
                session_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
                message_order: row.get(5)?,
            })
        })
        .map_err(|e| format!("Query messages failed: {}", e))?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| format!("Row parse failed: {}", e))?);
    }
    out.reverse();
    Ok(out)
}
