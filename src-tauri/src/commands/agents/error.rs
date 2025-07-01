use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Lock error: {0}")]
    Lock(String),

    #[error("Agent not found: {0}")]
    AgentNotFound(i64),

    #[error("Run not found: {0}")]
    RunNotFound(i64),

    #[error("Process error: {0}")]
    Process(String),

    #[error("Binary not found: {0}")]
    BinaryNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid status: {0}")]
    InvalidStatus(String),

    #[error("Invalid model: {0}")]
    InvalidModel(String),

    #[error("Sandbox error: {0}")]
    Sandbox(String),

    #[error("Schedule error: {0}")]
    Schedule(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Other error: {0}")]
    Other(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
}

// Implement conversion to String for Tauri commands
impl From<AgentError> for String {
    fn from(err: AgentError) -> String {
        err.to_string()
    }
}

// Implement Serialize for Tauri
impl Serialize for AgentError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

