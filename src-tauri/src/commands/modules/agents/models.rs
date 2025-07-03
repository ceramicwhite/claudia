use serde::{Deserialize, Serialize};

/// Represents a CC Agent stored in the database
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Agent {
    pub id: Option<i64>,
    pub name: String,
    pub icon: String,
    pub system_prompt: String,
    pub default_task: Option<String>,
    pub model: String,
    pub sandbox_enabled: bool,
    pub enable_file_read: bool,
    pub enable_file_write: bool,
    pub enable_network: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Represents an agent execution run
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentRun {
    pub id: Option<i64>,
    pub agent_id: i64,
    pub agent_name: String,
    pub agent_icon: String,
    pub task: String,
    pub model: String,
    pub project_path: String,
    pub session_id: String, // UUID session ID from Claude Code
    pub status: String,     // 'pending', 'running', 'completed', 'failed', 'cancelled', 'scheduled', 'paused_usage_limit'
    pub pid: Option<u32>,
    pub process_started_at: Option<String>,
    pub scheduled_start_time: Option<String>, // ISO 8601 datetime string for scheduled runs
    pub created_at: String,
    pub completed_at: Option<String>,
    pub usage_limit_reset_time: Option<String>, // ISO 8601 datetime when usage limit resets
    pub auto_resume_enabled: bool,
    pub resume_count: i64,
    pub parent_run_id: Option<i64>, // ID of the original run if this is a resumed run
}

/// Represents runtime metrics calculated from JSONL
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentRunMetrics {
    pub duration_ms: Option<i64>,
    pub total_tokens: Option<i64>,
    pub cost_usd: Option<f64>,
    pub message_count: Option<i64>,
}

/// Combined agent run with real-time metrics
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentRunWithMetrics {
    #[serde(flatten)]
    pub run: AgentRun,
    pub metrics: Option<AgentRunMetrics>,
    pub output: Option<String>, // Real-time JSONL content
}

/// Agent export format
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentExport {
    pub version: u32,
    pub exported_at: String,
    pub agent: AgentData,
}

/// Agent data within export
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentData {
    pub name: String,
    pub icon: String,
    pub system_prompt: String,
    pub default_task: Option<String>,
    pub model: String,
    pub sandbox_enabled: bool,
    pub enable_file_read: bool,
    pub enable_file_write: bool,
    pub enable_network: bool,
}

/// GitHub agent file metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubAgentFile {
    pub name: String,
    pub path: String,
    pub sha: String,
    pub size: u64,
    pub download_url: String,
}

/// GitHub API response structure
#[derive(Debug, Deserialize)]
pub(crate) struct GitHubApiResponse {
    pub name: String,
    pub path: String,
    pub sha: String,
    pub size: u64,
    pub download_url: Option<String>,
    #[serde(rename = "type")]
    pub file_type: String,
}