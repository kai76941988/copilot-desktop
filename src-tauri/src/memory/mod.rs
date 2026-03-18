pub mod db;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct MemoryRecordMessageParams {
    pub role: String,
    pub content: String,
    pub project_id: Option<String>,
    pub session_id: Option<String>,
    pub source: Option<String>,
    pub message_order: Option<i64>,
    pub summary_group_id: Option<String>,
    pub tags: Option<String>,
    pub metadata_json: Option<String>,
    pub created_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct MemoryCreateProjectParams {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MemoryListSessionsParams {
    pub project_id: String,
}

#[derive(Debug, Deserialize)]
pub struct MemoryGetContextPackParams {
    pub project_id: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MemorySearchParams {
    pub query: String,
    pub project_id: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct MemorySearchSummariesParams {
    pub query: String,
    pub project_id: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct MemoryListSummariesParams {
    pub project_id: String,
    pub session_id: Option<String>,
    pub summary_type: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct MemoryListMessagesParams {
    pub session_id: String,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct MemoryContinueParams {
    pub project_id: Option<String>,
    pub session_id: Option<String>,
    pub open_new: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct MemorySetProjectParams {
    pub project_id: String,
}

#[derive(Debug, Serialize)]
pub struct MemoryProjectInfo {
    pub project_id: String,
    pub name: String,
    pub description: Option<String>,
    pub updated_at: i64,
    pub pinned: i64,
}

#[derive(Debug, Serialize)]
pub struct MemorySessionInfo {
    pub session_id: String,
    pub project_id: String,
    pub title: Option<String>,
    pub updated_at: i64,
    pub source_url: Option<String>,
    pub archived: i64,
}

#[derive(Debug, Serialize)]
pub struct MemorySearchItem {
    pub id: String,
    pub project_id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub created_at: i64,
}

#[derive(Debug, Serialize)]
pub struct MemoryMessageInfo {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub created_at: i64,
    pub message_order: i64,
}

#[derive(Debug, Serialize)]
pub struct MemorySummaryInfo {
    pub summary_id: String,
    pub project_id: String,
    pub session_id: Option<String>,
    pub summary_type: String,
    pub source_range_start: Option<i64>,
    pub source_range_end: Option<i64>,
    pub content: String,
    pub created_at: i64,
}
