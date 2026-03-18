pub mod db;

use serde::Deserialize;

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
