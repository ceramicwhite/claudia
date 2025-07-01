#![allow(dead_code)]

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
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AgentRun {
    pub id: Option<i64>,
    pub agent_id: i64,
    pub agent_name: String,
    pub agent_icon: String,
    pub task: String,
    pub model: String,
    pub project_path: String,
    pub session_id: String, // UUID session ID from Claude Code
    pub status: String,     // Stored as String for database compatibility, use RunStatus::from_str() for conversion
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

impl AgentRun {
    /// Get the status as a RunStatus enum
    pub fn status(&self) -> RunStatus {
        RunStatus::from_str(&self.status)
    }
    
    /// Check if the run is in a terminal state
    pub fn is_terminal(&self) -> bool {
        self.status().is_terminal()
    }
    
    /// Check if the run is currently active
    pub fn is_active(&self) -> bool {
        self.status().is_active()
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use crate::claude_binary::ClaudeInstallation;

    // ===== Helper Functions =====
    
    fn assert_serialization<T>(value: &T, expected_json: &str)
    where
        T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
    {
        // Test serialization
        let json = serde_json::to_string(value).unwrap();
        assert_eq!(json, expected_json);
        
        // Test deserialization
        let deserialized: T = serde_json::from_str(&json).unwrap();
        assert_eq!(&deserialized, value);
    }

    // ===== AgentId Tests =====
    
    #[test]
    fn test_agent_id_valid_creation() {
        let id = AgentId::new(1).unwrap();
        assert_eq!(id.inner(), 1);
        
        let id = AgentId::new(i64::MAX).unwrap();
        assert_eq!(id.inner(), i64::MAX);
    }
    
    #[test]
    fn test_agent_id_invalid_creation() {
        assert_eq!(
            AgentId::new(0).unwrap_err(),
            "Agent ID must be positive"
        );
        
        assert_eq!(
            AgentId::new(-1).unwrap_err(),
            "Agent ID must be positive"
        );
        
        assert_eq!(
            AgentId::new(i64::MIN).unwrap_err(),
            "Agent ID must be positive"
        );
    }
    
    #[test]
    fn test_agent_id_display() {
        let id = AgentId::new(42).unwrap();
        assert_eq!(format!("{}", id), "42");
    }
    
    #[test]
    fn test_agent_id_from_str() {
        assert_eq!(AgentId::from_str("1").unwrap().inner(), 1);
        assert_eq!(AgentId::from_str("999").unwrap().inner(), 999);
        
        assert_eq!(
            AgentId::from_str("0").unwrap_err(),
            "Agent ID must be positive"
        );
        
        assert_eq!(
            AgentId::from_str("-1").unwrap_err(),
            "Agent ID must be positive"
        );
        
        assert_eq!(
            AgentId::from_str("abc").unwrap_err(),
            "Invalid agent ID format"
        );
        
        assert_eq!(
            AgentId::from_str("").unwrap_err(),
            "Invalid agent ID format"
        );
        
        assert_eq!(
            AgentId::from_str("  ").unwrap_err(),
            "Invalid agent ID format"
        );
    }
    
    #[test]
    fn test_agent_id_serialization() {
        let id = AgentId::new(42).unwrap();
        assert_serialization(&id, "42");
    }
    
    #[test]
    fn test_agent_id_traits() {
        let id1 = AgentId::new(1).unwrap();
        let id2 = AgentId::new(1).unwrap();
        let id3 = AgentId::new(2).unwrap();
        
        // Test PartialEq
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        
        // Test Clone
        let cloned = id1.clone();
        assert_eq!(id1, cloned);
        
        // Test Debug
        assert_eq!(format!("{:?}", id1), "AgentId(1)");
        
        // Test Hash
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(id1);
        set.insert(id2);
        set.insert(id3);
        assert_eq!(set.len(), 2); // id1 and id2 are the same
    }

    // ===== RunId Tests =====
    
    #[test]
    fn test_run_id_valid_creation() {
        let id = RunId::new(1).unwrap();
        assert_eq!(id.inner(), 1);
        
        let id = RunId::new(i64::MAX).unwrap();
        assert_eq!(id.inner(), i64::MAX);
    }
    
    #[test]
    fn test_run_id_invalid_creation() {
        assert_eq!(
            RunId::new(0).unwrap_err(),
            "Run ID must be positive"
        );
        
        assert_eq!(
            RunId::new(-1).unwrap_err(),
            "Run ID must be positive"
        );
    }
    
    #[test]
    fn test_run_id_display() {
        let id = RunId::new(42).unwrap();
        assert_eq!(format!("{}", id), "42");
    }
    
    #[test]
    fn test_run_id_from_str() {
        assert_eq!(RunId::from_str("1").unwrap().inner(), 1);
        assert_eq!(RunId::from_str("999").unwrap().inner(), 999);
        
        assert_eq!(
            RunId::from_str("0").unwrap_err(),
            "Run ID must be positive"
        );
        
        assert_eq!(
            RunId::from_str("abc").unwrap_err(),
            "Invalid run ID format"
        );
    }
    
    #[test]
    fn test_run_id_serialization() {
        let id = RunId::new(42).unwrap();
        assert_serialization(&id, "42");
    }

    // ===== SessionId Tests =====
    
    #[test]
    fn test_session_id_valid_creation() {
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        let id = SessionId::new(valid_uuid.to_string()).unwrap();
        assert_eq!(id.inner(), valid_uuid);
    }
    
    #[test]
    fn test_session_id_invalid_creation() {
        assert_eq!(
            SessionId::new("".to_string()).unwrap_err(),
            "Session ID cannot be empty"
        );
        
        assert_eq!(
            SessionId::new("not-a-uuid".to_string()).unwrap_err(),
            "Invalid session ID format (must be UUID)"
        );
        
        assert_eq!(
            SessionId::new("123".to_string()).unwrap_err(),
            "Invalid session ID format (must be UUID)"
        );
        
        assert_eq!(
            SessionId::new("550e8400-e29b-41d4-a716".to_string()).unwrap_err(),
            "Invalid session ID format (must be UUID)"
        );
    }
    
    #[test]
    fn test_session_id_generate() {
        let id1 = SessionId::generate();
        let id2 = SessionId::generate();
        
        // Should generate valid UUIDs
        assert!(SessionId::is_valid_uuid(id1.inner()));
        assert!(SessionId::is_valid_uuid(id2.inner()));
        
        // Should generate unique IDs
        assert_ne!(id1, id2);
    }
    
    #[test]
    fn test_session_id_display() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        let id = SessionId::new(uuid.to_string()).unwrap();
        assert_eq!(format!("{}", id), uuid);
    }
    
    #[test]
    fn test_session_id_from_str() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        let id = SessionId::from_str(uuid).unwrap();
        assert_eq!(id.inner(), uuid);
        
        assert!(SessionId::from_str("").is_err());
        assert!(SessionId::from_str("not-a-uuid").is_err());
    }
    
    #[test]
    fn test_session_id_serialization() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        let id = SessionId::new(uuid.to_string()).unwrap();
        assert_serialization(&id, &format!("\"{}\"", uuid));
    }
    
    #[test]
    fn test_session_id_traits() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        let id1 = SessionId::new(uuid.to_string()).unwrap();
        let id2 = SessionId::new(uuid.to_string()).unwrap();
        let id3 = SessionId::generate();
        
        // Test PartialEq
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        
        // Test Clone
        let cloned = id1.clone();
        assert_eq!(id1, cloned);
        
        // Test Debug
        assert!(format!("{:?}", id1).contains("SessionId"));
        assert!(format!("{:?}", id1).contains(uuid));
    }

    // ===== RunStatus Tests =====
    
    #[test]
    fn test_run_status_display() {
        assert_eq!(format!("{}", RunStatus::Pending), "pending");
        assert_eq!(format!("{}", RunStatus::Running), "running");
        assert_eq!(format!("{}", RunStatus::Completed), "completed");
        assert_eq!(format!("{}", RunStatus::Failed), "failed");
        assert_eq!(format!("{}", RunStatus::Cancelled), "cancelled");
        assert_eq!(format!("{}", RunStatus::Scheduled), "scheduled");
        assert_eq!(format!("{}", RunStatus::PausedUsageLimit), "paused_usage_limit");
    }
    
    #[test]
    fn test_run_status_from_str() {
        assert_eq!(RunStatus::from_str("running"), RunStatus::Running);
        assert_eq!(RunStatus::from_str("completed"), RunStatus::Completed);
        assert_eq!(RunStatus::from_str("failed"), RunStatus::Failed);
        assert_eq!(RunStatus::from_str("cancelled"), RunStatus::Cancelled);
        assert_eq!(RunStatus::from_str("scheduled"), RunStatus::Scheduled);
        assert_eq!(RunStatus::from_str("paused_usage_limit"), RunStatus::PausedUsageLimit);
        assert_eq!(RunStatus::from_str("pending"), RunStatus::Pending);
        assert_eq!(RunStatus::from_str("unknown"), RunStatus::Pending); // Default
    }
    
    #[test]
    fn test_run_status_is_terminal() {
        assert!(!RunStatus::Pending.is_terminal());
        assert!(!RunStatus::Running.is_terminal());
        assert!(RunStatus::Completed.is_terminal());
        assert!(RunStatus::Failed.is_terminal());
        assert!(RunStatus::Cancelled.is_terminal());
        assert!(!RunStatus::Scheduled.is_terminal());
        assert!(!RunStatus::PausedUsageLimit.is_terminal());
    }
    
    #[test]
    fn test_run_status_is_active() {
        assert!(!RunStatus::Pending.is_active());
        assert!(RunStatus::Running.is_active());
        assert!(!RunStatus::Completed.is_active());
        assert!(!RunStatus::Failed.is_active());
        assert!(!RunStatus::Cancelled.is_active());
        assert!(!RunStatus::Scheduled.is_active());
        assert!(!RunStatus::PausedUsageLimit.is_active());
    }
    
    #[test]
    fn test_run_status_serialization() {
        assert_serialization(&RunStatus::Running, "\"running\"");
        assert_serialization(&RunStatus::PausedUsageLimit, "\"paused_usage_limit\"");
    }

    // ===== ModelType Tests =====
    
    #[test]
    fn test_model_type_display() {
        assert_eq!(format!("{}", ModelType::Opus3), "opus-3");
        assert_eq!(format!("{}", ModelType::Sonnet3), "sonnet-3");
        assert_eq!(format!("{}", ModelType::Opus4), "opus-4");
        assert_eq!(format!("{}", ModelType::Sonnet4), "sonnet-4");
        assert_eq!(format!("{}", ModelType::Sonnet), "sonnet");
        assert_eq!(format!("{}", ModelType::Opus), "opus");
    }
    
    #[test]
    fn test_model_type_from_str() {
        assert_eq!(ModelType::from_str("opus-3"), ModelType::Opus3);
        assert_eq!(ModelType::from_str("opus3"), ModelType::Opus3);
        assert_eq!(ModelType::from_str("OPUS-3"), ModelType::Opus3);
        assert_eq!(ModelType::from_str("sonnet-3"), ModelType::Sonnet3);
        assert_eq!(ModelType::from_str("sonnet3"), ModelType::Sonnet3);
        assert_eq!(ModelType::from_str("opus-4"), ModelType::Opus4);
        assert_eq!(ModelType::from_str("opus4"), ModelType::Opus4);
        assert_eq!(ModelType::from_str("sonnet-4"), ModelType::Sonnet4);
        assert_eq!(ModelType::from_str("sonnet4"), ModelType::Sonnet4);
        assert_eq!(ModelType::from_str("opus"), ModelType::Opus);
        assert_eq!(ModelType::from_str("sonnet"), ModelType::Sonnet);
        assert_eq!(ModelType::from_str("unknown"), ModelType::Sonnet); // Default
    }
    
    #[test]
    fn test_model_type_get_pricing() {
        // Just verify the method returns a tuple - actual pricing values are tested elsewhere
        let (input, output, cache_write, cache_read) = ModelType::Opus3.get_pricing();
        assert!(input > 0.0);
        assert!(output > 0.0);
        assert!(cache_write > 0.0);
        assert!(cache_read > 0.0);
    }
    
    #[test]
    fn test_model_type_serialization() {
        assert_serialization(&ModelType::Opus3, "\"opus3\"");
        assert_serialization(&ModelType::Sonnet4, "\"sonnet4\"");
    }

    // ===== AgentCreateBuilder Tests =====
    
    #[test]
    fn test_agent_create_builder_valid() {
        let agent = AgentCreateBuilder::new()
            .name("Test Agent")
            .icon("🤖")
            .system_prompt("You are a helpful assistant")
            .build()
            .unwrap();
        
        assert_eq!(agent.name, "Test Agent");
        assert_eq!(agent.icon, "🤖");
        assert_eq!(agent.system_prompt, "You are a helpful assistant");
        assert_eq!(agent.model, "sonnet"); // Default model
        assert!(agent.sandbox_enabled); // Default from constants
    }
    
    #[test]
    fn test_agent_create_builder_with_all_fields() {
        let agent = AgentCreateBuilder::new()
            .name("Test Agent")
            .icon("🤖")
            .system_prompt("You are a helpful assistant")
            .default_task("Help with coding")
            .model(ModelType::Opus4)
            .sandbox_enabled(false)
            .enable_file_read(false)
            .enable_file_write(false)
            .enable_network(false)
            .build()
            .unwrap();
        
        assert_eq!(agent.name, "Test Agent");
        assert_eq!(agent.icon, "🤖");
        assert_eq!(agent.system_prompt, "You are a helpful assistant");
        assert_eq!(agent.default_task, Some("Help with coding".to_string()));
        assert_eq!(agent.model, "opus-4");
        assert!(!agent.sandbox_enabled);
        assert!(!agent.enable_file_read);
        assert!(!agent.enable_file_write);
        assert!(!agent.enable_network);
    }
    
    #[test]
    fn test_agent_create_builder_missing_name() {
        let result = AgentCreateBuilder::new()
            .icon("🤖")
            .system_prompt("You are a helpful assistant")
            .build();
        
        assert_eq!(result.unwrap_err(), "Agent name is required");
    }
    
    #[test]
    fn test_agent_create_builder_missing_icon() {
        let result = AgentCreateBuilder::new()
            .name("Test Agent")
            .system_prompt("You are a helpful assistant")
            .build();
        
        assert_eq!(result.unwrap_err(), "Agent icon is required");
    }
    
    #[test]
    fn test_agent_create_builder_missing_system_prompt() {
        let result = AgentCreateBuilder::new()
            .name("Test Agent")
            .icon("🤖")
            .build();
        
        assert_eq!(result.unwrap_err(), "System prompt is required");
    }
    
    #[test]
    fn test_agent_create_builder_empty_name() {
        let result = AgentCreateBuilder::new()
            .name("")
            .icon("🤖")
            .system_prompt("You are a helpful assistant")
            .build();
        
        assert_eq!(result.unwrap_err(), "Agent name is required");
    }
    
    #[test]
    fn test_agent_create_builder_whitespace_name() {
        let result = AgentCreateBuilder::new()
            .name("   ")
            .icon("🤖")
            .system_prompt("You are a helpful assistant")
            .build();
        
        assert_eq!(result.unwrap_err(), "Agent name cannot be empty or whitespace");
    }
    
    #[test]
    fn test_agent_create_builder_whitespace_system_prompt() {
        let result = AgentCreateBuilder::new()
            .name("Test Agent")
            .icon("🤖")
            .system_prompt("   ")
            .build();
        
        assert_eq!(result.unwrap_err(), "System prompt cannot be empty or whitespace");
    }
    
    #[test]
    fn test_agent_create_builder_empty_optional_fields() {
        let agent = AgentCreateBuilder::new()
            .name("Test Agent")
            .icon("🤖")
            .system_prompt("You are a helpful assistant")
            .default_task("") // Empty optional field should be ignored
            .build()
            .unwrap();
        
        assert_eq!(agent.default_task, None);
    }

    // ===== AgentCreate Tests =====
    
    #[test]
    fn test_agent_create_validate() {
        let valid_agent = AgentCreate {
            name: "Test Agent".to_string(),
            icon: "🤖".to_string(),
            system_prompt: "You are a helpful assistant".to_string(),
            default_task: None,
            model: "opus-4".to_string(),
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: true,
            enable_network: true,
        };
        
        assert!(valid_agent.validate().is_ok());
    }
    
    #[test]
    fn test_agent_create_validate_empty_name() {
        let agent = AgentCreate {
            name: "".to_string(),
            icon: "🤖".to_string(),
            system_prompt: "You are a helpful assistant".to_string(),
            default_task: None,
            model: "opus-4".to_string(),
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: true,
            enable_network: true,
        };
        
        assert_eq!(agent.validate().unwrap_err(), "Agent name cannot be empty");
    }
    
    #[test]
    fn test_agent_create_validate_whitespace_name() {
        let agent = AgentCreate {
            name: "   ".to_string(),
            icon: "🤖".to_string(),
            system_prompt: "You are a helpful assistant".to_string(),
            default_task: None,
            model: "opus-4".to_string(),
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: true,
            enable_network: true,
        };
        
        assert_eq!(agent.validate().unwrap_err(), "Agent name cannot be empty");
    }
    
    #[test]
    fn test_agent_create_validate_empty_system_prompt() {
        let agent = AgentCreate {
            name: "Test Agent".to_string(),
            icon: "🤖".to_string(),
            system_prompt: "".to_string(),
            default_task: None,
            model: "opus-4".to_string(),
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: true,
            enable_network: true,
        };
        
        assert_eq!(agent.validate().unwrap_err(), "System prompt cannot be empty");
    }
    
    #[test]
    fn test_agent_create_builder_method() {
        let builder = AgentCreate::builder();
        assert!(builder.name.is_none());
        assert!(builder.icon.is_none());
        assert!(builder.system_prompt.is_none());
    }

    // ===== Struct Tests =====
    
    #[test]
    fn test_agent_struct_creation() {
        let agent = Agent {
            id: Some(1),
            name: "Test Agent".to_string(),
            icon: "🤖".to_string(),
            system_prompt: "You are a helpful assistant".to_string(),
            default_task: Some("Help with coding".to_string()),
            model: "opus-4".to_string(),
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: false,
            enable_network: true,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };
        
        assert_eq!(agent.id, Some(1));
        assert_eq!(agent.name, "Test Agent");
        assert!(agent.sandbox_enabled);
    }
    
    #[test]
    fn test_agent_struct_clone() {
        let agent = Agent {
            id: Some(1),
            name: "Test Agent".to_string(),
            icon: "🤖".to_string(),
            system_prompt: "You are a helpful assistant".to_string(),
            default_task: None,
            model: "opus-4".to_string(),
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: true,
            enable_network: true,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };
        
        let cloned = agent.clone();
        assert_eq!(agent.id, cloned.id);
        assert_eq!(agent.name, cloned.name);
    }
    
    #[test]
    fn test_agent_run_struct_creation() {
        let run = AgentRun {
            id: Some(1),
            agent_id: 1,
            agent_name: "Test Agent".to_string(),
            agent_icon: "🤖".to_string(),
            task: "Test task".to_string(),
            model: "opus-4".to_string(),
            project_path: "/tmp/test".to_string(),
            session_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            status: "running".to_string(),
            pid: Some(12345),
            process_started_at: Some("2024-01-01T00:00:00Z".to_string()),
            scheduled_start_time: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            completed_at: None,
            usage_limit_reset_time: None,
            auto_resume_enabled: false,
            resume_count: 0,
            parent_run_id: None,
        };
        
        assert_eq!(run.agent_id, 1);
        assert_eq!(run.status, "running");
        assert_eq!(run.pid, Some(12345));
    }
    
    #[test]
    fn test_agent_run_with_metrics() {
        let run = AgentRun {
            id: Some(1),
            agent_id: 1,
            agent_name: "Test Agent".to_string(),
            agent_icon: "🤖".to_string(),
            task: "Test task".to_string(),
            model: "opus-4".to_string(),
            project_path: "/tmp/test".to_string(),
            session_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            status: "completed".to_string(),
            pid: None,
            process_started_at: None,
            scheduled_start_time: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            completed_at: Some("2024-01-01T01:00:00Z".to_string()),
            usage_limit_reset_time: None,
            auto_resume_enabled: false,
            resume_count: 0,
            parent_run_id: None,
        };
        
        let metrics = AgentRunMetrics {
            duration_ms: Some(3600000),
            total_tokens: Some(1000),
            cost_usd: Some(0.05),
            message_count: Some(10),
        };
        
        let run_with_metrics = AgentRunWithMetrics {
            run: run.clone(),
            metrics: Some(metrics),
            output: Some("Sample output".to_string()),
        };
        
        assert_eq!(run_with_metrics.run.status, "completed");
        assert_eq!(run_with_metrics.metrics.as_ref().unwrap().total_tokens, Some(1000));
    }
    
    #[test]
    fn test_sandbox_violation_struct() {
        let violation = SandboxViolation {
            id: Some(1),
            run_id: 1,
            denied_at: "2024-01-01T00:00:00Z".to_string(),
            operation_type: "file_read".to_string(),
            resource: "/etc/passwd".to_string(),
            reason: "Access denied".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };
        
        assert_eq!(violation.operation_type, "file_read");
        assert_eq!(violation.resource, "/etc/passwd");
    }
    
    #[test]
    fn test_app_setting_struct() {
        let setting = AppSetting {
            key: "theme".to_string(),
            value: "dark".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };
        
        assert_eq!(setting.key, "theme");
        assert_eq!(setting.value, "dark");
        
        let cloned = setting.clone();
        assert_eq!(setting.key, cloned.key);
    }
    
    #[test]
    fn test_agent_export_struct() {
        let agent_data = AgentData {
            name: "Test Agent".to_string(),
            icon: "🤖".to_string(),
            system_prompt: "You are a helpful assistant".to_string(),
            default_task: None,
            model: "opus-4".to_string(),
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: true,
            enable_network: true,
        };
        
        let export = AgentExport {
            version: 1,
            exported_at: "2024-01-01T00:00:00Z".to_string(),
            agent: agent_data,
        };
        
        assert_eq!(export.version, 1);
        assert_eq!(export.agent.name, "Test Agent");
    }
    
    #[test]
    fn test_jsonl_message_struct() {
        let message = JsonlMessage {
            r#type: "response".to_string(),
            timestamp: Some("2024-01-01T00:00:00Z".to_string()),
            message: Some("Hello, world!".to_string()),
            model: Some("opus-4".to_string()),
            input_tokens: Some(100),
            output_tokens: Some(50),
            cache_creation_input_tokens: Some(10),
            cache_read_input_tokens: Some(5),
            cost: Some(0.01),
            total_cost: Some(0.05),
        };
        
        assert_eq!(message.r#type, "response");
        assert_eq!(message.input_tokens, Some(100));
    }
    
    #[test]
    fn test_github_agent_file_struct() {
        let file = GitHubAgentFile {
            name: "test.txt".to_string(),
            path: "path/to/test.txt".to_string(),
            sha: "abc123".to_string(),
            size: 1024,
            url: "https://api.github.com/...".to_string(),
            html_url: "https://github.com/...".to_string(),
            git_url: "https://github.com/...".to_string(),
            download_url: "https://raw.githubusercontent.com/...".to_string(),
            file_type: "file".to_string(),
        };
        
        assert_eq!(file.name, "test.txt");
        assert_eq!(file.size, 1024);
    }
    
    #[test]
    fn test_claude_installation_struct() {
        let installation = ClaudeInstallation {
            path: "/usr/local/bin/claude".to_string(),
            version: Some("1.0.0".to_string()),
            source: "homebrew".to_string(),
        };
        
        assert_eq!(installation.path, "/usr/local/bin/claude");
        assert_eq!(installation.version, Some("1.0.0".to_string()));
    }

    // ===== Edge Cases and Additional Tests =====
    
    #[test]
    fn test_model_type_case_insensitive() {
        assert_eq!(ModelType::from_str("OPUS-3"), ModelType::Opus3);
        assert_eq!(ModelType::from_str("SoNnEt-4"), ModelType::Sonnet4);
        assert_eq!(ModelType::from_str("OpUs"), ModelType::Opus);
    }
    
    #[test]
    fn test_agent_create_builder_chaining() {
        let agent = AgentCreate::builder()
            .name("Test")
            .name("Test Agent") // Should override
            .icon("👍")
            .icon("🤖") // Should override
            .system_prompt("First")
            .system_prompt("You are a helpful assistant") // Should override
            .build()
            .unwrap();
        
        assert_eq!(agent.name, "Test Agent");
        assert_eq!(agent.icon, "🤖");
        assert_eq!(agent.system_prompt, "You are a helpful assistant");
    }
    
    #[test]
    fn test_session_id_uuid_edge_cases() {
        // Test uppercase UUID
        let uppercase = "550E8400-E29B-41D4-A716-446655440000";
        assert!(SessionId::new(uppercase.to_string()).is_ok());
        
        // Test lowercase UUID
        let lowercase = "550e8400-e29b-41d4-a716-446655440000";
        assert!(SessionId::new(lowercase.to_string()).is_ok());
        
        // Test UUID without hyphens (uuid crate accepts this format)
        let no_hyphens = "550e8400e29b41d4a716446655440000";
        assert!(SessionId::new(no_hyphens.to_string()).is_ok());
        
        // Test invalid UUID formats
        let too_short = "550e8400-e29b-41d4";
        assert!(SessionId::new(too_short.to_string()).is_err());
        
        let invalid_chars = "550e8400-e29b-41d4-a716-44665544000g";
        assert!(SessionId::new(invalid_chars.to_string()).is_err());
        
        let wrong_format = "550e8400e29b41d4a716446655440000123"; // Too long
        assert!(SessionId::new(wrong_format.to_string()).is_err());
    }
}