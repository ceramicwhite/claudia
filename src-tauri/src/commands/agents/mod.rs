pub mod commands;
pub mod constants;
pub mod error;
pub mod execute;
pub mod helpers;
pub mod pool;
pub mod repository;
pub mod service;
pub mod types;

#[cfg(test)]
mod error_tests;

#[cfg(test)]
mod repository_test;

#[cfg(test)]
mod service_tests;

#[cfg(test)]
mod pool_tests;

// Re-export main functionality
pub use commands::*;
pub use types::*;

// Specifically re-export newtype wrappers and builder for convenience

use anyhow::Result;
use pool::{SqlitePool, create_pool, init_pool_db};
use std::sync::Arc;

pub struct AgentDb(pub Arc<SqlitePool>);


/// Initialize the agents database
pub fn init_db(data_dir: std::path::PathBuf) -> Result<SqlitePool> {
    let db_path = data_dir.join("agents.db");
    let pool = create_pool(db_path)?;
    init_pool_db(&pool)?;
    Ok(pool)
}

