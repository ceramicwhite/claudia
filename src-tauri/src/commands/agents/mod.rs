pub mod commands;
pub mod constants;
pub mod error;
pub mod execute;
pub mod helpers;
pub mod pool;
pub mod repository;
pub mod service;
pub mod types;

// Re-export main functionality
pub use commands::*;
pub use error::AgentError;
pub use types::*;

// Specifically re-export newtype wrappers and builder for convenience
pub use types::{AgentId, RunId, SessionId, AgentCreate, AgentCreateBuilder};

use anyhow::Result;
use log::info;
use pool::{SqlitePool, create_pool, init_pool_db};
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};

pub struct AgentDb(pub Arc<SqlitePool>);

/// Finds the full path to the claude binary
/// This is necessary because macOS apps have a limited PATH environment
fn find_claude_binary(app_handle: &AppHandle) -> Result<String, String> {
    crate::claude_binary::find_claude_binary(app_handle)
}

/// Initialize the agents database
pub fn init_db(data_dir: std::path::PathBuf) -> Result<SqlitePool> {
    let db_path = data_dir.join("agents.db");
    let pool = create_pool(db_path)?;
    init_pool_db(&pool)?;
    Ok(pool)
}

