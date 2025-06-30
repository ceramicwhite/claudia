use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

// ===== Newtype Wrappers for Type Safety =====

/// Type-safe wrapper for Agent ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AgentId(i64);

impl AgentId {
    /// Create a new AgentId with validation
    pub fn new(id: i64) -> Result<Self, String> {
        if id <= 0 {
            return Err("Agent ID must be positive".to_string());
        }
        Ok(Self(id))
    }
    
    /// Get the inner value
    pub fn inner(&self) -> i64 {
        self.0
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for AgentId {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s.parse::<i64>()
            .map_err(|_| "Invalid agent ID format".to_string())?;
        Self::new(id)
    }
}

/// Type-safe wrapper for Run ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RunId(i64);

impl RunId {
    /// Create a new RunId with validation
    pub fn new(id: i64) -> Result<Self, String> {
        if id <= 0 {
            return Err("Run ID must be positive".to_string());
        }
        Ok(Self(id))
    }
    
    /// Get the inner value
    pub fn inner(&self) -> i64 {
        self.0
    }
}

impl fmt::Display for RunId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for RunId {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s.parse::<i64>()
            .map_err(|_| "Invalid run ID format".to_string())?;
        Self::new(id)
    }
}

/// Type-safe wrapper for Session ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(String);

impl SessionId {
    /// Create a new SessionId with validation
    pub fn new(id: String) -> Result<Self, String> {
        if id.is_empty() {
            return Err("Session ID cannot be empty".to_string());
        }
        // Validate UUID format
        if !Self::is_valid_uuid(&id) {
            return Err("Invalid session ID format (must be UUID)".to_string());
        }
        Ok(Self(id))
    }
    
    /// Generate a new random SessionId
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
    
    /// Get the inner value
    pub fn inner(&self) -> &str {
        &self.0
    }
    
    /// Check if string is valid UUID
    fn is_valid_uuid(s: &str) -> bool {
        uuid::Uuid::parse_str(s).is_ok()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for SessionId {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_string())
    }
}

/// Agent run status enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Scheduled,
    PausedUsageLimit,
}

impl fmt::Display for RunStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunStatus::Pending => write!(f, "pending"),
            RunStatus::Running => write!(f, "running"),
            RunStatus::Completed => write!(f, "completed"),
            RunStatus::Failed => write!(f, "failed"),
            RunStatus::Cancelled => write!(f, "cancelled"),
            RunStatus::Scheduled => write!(f, "scheduled"),
            RunStatus::PausedUsageLimit => write!(f, "paused_usage_limit"),
        }
    }
}

impl RunStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "running" => RunStatus::Running,
            "completed" => RunStatus::Completed,
            "failed" => RunStatus::Failed,
            "cancelled" => RunStatus::Cancelled,
            "scheduled" => RunStatus::Scheduled,
            "paused_usage_limit" => RunStatus::PausedUsageLimit,
            _ => RunStatus::Pending,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            RunStatus::Completed | RunStatus::Failed | RunStatus::Cancelled
        )
    }

    pub fn is_active(&self) -> bool {
        matches!(self, RunStatus::Running)
    }
}

/// Model type enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelType {
    Opus3,
    Sonnet3,
    Opus4,
    Sonnet4,
    Sonnet,  // Default/alias
    Opus,    // Default/alias
}

impl fmt::Display for ModelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelType::Opus3 => write!(f, "opus-3"),
            ModelType::Sonnet3 => write!(f, "sonnet-3"),
            ModelType::Opus4 => write!(f, "opus-4"),
            ModelType::Sonnet4 => write!(f, "sonnet-4"),
            ModelType::Sonnet => write!(f, "sonnet"),
            ModelType::Opus => write!(f, "opus"),
        }
    }
}

impl ModelType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "opus-3" | "opus3" => ModelType::Opus3,
            "sonnet-3" | "sonnet3" => ModelType::Sonnet3,
            "opus-4" | "opus4" => ModelType::Opus4,
            "sonnet-4" | "sonnet4" => ModelType::Sonnet4,
            "opus" => ModelType::Opus,
            _ => ModelType::Sonnet, // Default
        }
    }

    pub fn get_pricing(&self) -> (f64, f64, f64, f64) {
        use crate::commands::agents::constants::*;
        
        match self {
            ModelType::Opus3 | ModelType::Opus => {
                (OPUS_3_INPUT_PRICE, OPUS_3_OUTPUT_PRICE, OPUS_3_CACHE_WRITE_PRICE, OPUS_3_CACHE_READ_PRICE)
            }
            ModelType::Sonnet3 => {
                (SONNET_3_INPUT_PRICE, SONNET_3_OUTPUT_PRICE, SONNET_3_CACHE_WRITE_PRICE, SONNET_3_CACHE_READ_PRICE)
            }
            ModelType::Opus4 => {
                (OPUS_4_INPUT_PRICE, OPUS_4_OUTPUT_PRICE, OPUS_4_CACHE_WRITE_PRICE, OPUS_4_CACHE_READ_PRICE)
            }
            ModelType::Sonnet4 | ModelType::Sonnet => {
                (SONNET_4_INPUT_PRICE, SONNET_4_OUTPUT_PRICE, SONNET_4_CACHE_WRITE_PRICE, SONNET_4_CACHE_READ_PRICE)
            }
        }
    }
}

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
    pub status: String,     // TODO: Convert to RunStatus enum in phase 2
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

/// GitHub agent file information
#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubAgentFile {
    pub name: String,
    pub path: String,
    pub sha: String,
    pub size: u64,
    pub url: String,
    pub html_url: String,
    pub git_url: String,
    pub download_url: String,
    #[serde(rename = "type")]
    pub file_type: String,
}

/// Claude installation info
#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeInstallation {
    pub path: String,
    pub version: Option<String>,
    pub source: String,
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

/// JSONL message types
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonlMessage {
    pub r#type: String,
    pub timestamp: Option<String>,
    pub message: Option<String>,
    pub model: Option<String>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cache_creation_input_tokens: Option<i64>,
    pub cache_read_input_tokens: Option<i64>,
    pub cost: Option<f64>,
    pub total_cost: Option<f64>,
}

/// Sandbox violation record
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SandboxViolation {
    pub id: Option<i64>,
    pub run_id: i64,
    pub denied_at: String,
    pub operation_type: String,
    pub resource: String,
    pub reason: String,
    pub created_at: String,
}

/// App settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSetting {
    pub key: String,
    pub value: String,
    pub created_at: String,
    pub updated_at: String,
}

// ===== Builder Pattern for AgentCreate =====

/// Builder for creating new agents with compile-time validation
#[derive(Debug, Default)]
pub struct AgentCreateBuilder {
    name: Option<String>,
    icon: Option<String>,
    system_prompt: Option<String>,
    default_task: Option<String>,
    model: Option<ModelType>,
    sandbox_enabled: bool,
    enable_file_read: bool,
    enable_file_write: bool,
    enable_network: bool,
}

impl AgentCreateBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            sandbox_enabled: crate::commands::agents::constants::DEFAULT_SANDBOX_ENABLED,
            enable_file_read: crate::commands::agents::constants::DEFAULT_FILE_READ_ENABLED,
            enable_file_write: crate::commands::agents::constants::DEFAULT_FILE_WRITE_ENABLED,
            enable_network: crate::commands::agents::constants::DEFAULT_NETWORK_ENABLED,
            ..Default::default()
        }
    }
    
    /// Set the agent name (required)
    pub fn name(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        if !name.is_empty() {
            self.name = Some(name);
        }
        self
    }
    
    /// Set the agent icon (required)
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        let icon = icon.into();
        if !icon.is_empty() {
            self.icon = Some(icon);
        }
        self
    }
    
    /// Set the system prompt (required)
    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        let prompt = prompt.into();
        if !prompt.is_empty() {
            self.system_prompt = Some(prompt);
        }
        self
    }
    
    /// Set the default task (optional)
    pub fn default_task(mut self, task: impl Into<String>) -> Self {
        let task = task.into();
        if !task.is_empty() {
            self.default_task = Some(task);
        }
        self
    }
    
    /// Set the model type
    pub fn model(mut self, model: ModelType) -> Self {
        self.model = Some(model);
        self
    }
    
    /// Enable or disable sandbox
    pub fn sandbox_enabled(mut self, enabled: bool) -> Self {
        self.sandbox_enabled = enabled;
        self
    }
    
    /// Enable or disable file read permissions
    pub fn enable_file_read(mut self, enabled: bool) -> Self {
        self.enable_file_read = enabled;
        self
    }
    
    /// Enable or disable file write permissions
    pub fn enable_file_write(mut self, enabled: bool) -> Self {
        self.enable_file_write = enabled;
        self
    }
    
    /// Enable or disable network permissions
    pub fn enable_network(mut self, enabled: bool) -> Self {
        self.enable_network = enabled;
        self
    }
    
    /// Build the AgentCreate struct
    pub fn build(self) -> Result<AgentCreate, String> {
        let name = self.name.ok_or("Agent name is required")?;
        let icon = self.icon.ok_or("Agent icon is required")?;
        let system_prompt = self.system_prompt.ok_or("System prompt is required")?;
        
        // Additional validation
        if name.trim().is_empty() {
            return Err("Agent name cannot be empty or whitespace".to_string());
        }
        
        if system_prompt.trim().is_empty() {
            return Err("System prompt cannot be empty or whitespace".to_string());
        }
        
        Ok(AgentCreate {
            name,
            icon,
            system_prompt,
            default_task: self.default_task,
            model: self.model.unwrap_or(ModelType::Sonnet).to_string(),
            sandbox_enabled: self.sandbox_enabled,
            enable_file_read: self.enable_file_read,
            enable_file_write: self.enable_file_write,
            enable_network: self.enable_network,
        })
    }
}

/// Validated agent creation struct
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentCreate {
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

impl AgentCreate {
    /// Create a new builder for AgentCreate
    pub fn builder() -> AgentCreateBuilder {
        AgentCreateBuilder::new()
    }
    
    /// Validate all fields
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Agent name cannot be empty".to_string());
        }
        
        if self.system_prompt.trim().is_empty() {
            return Err("System prompt cannot be empty".to_string());
        }
        
        // Validate model
        ModelType::from_str(&self.model);
        
        Ok(())
    }
}