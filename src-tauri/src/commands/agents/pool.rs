use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use std::path::Path;
use super::error::AgentError;

pub type SqlitePool = Pool<SqliteConnectionManager>;
pub type SqlitePooledConnection = PooledConnection<SqliteConnectionManager>;

/// Create a new SQLite connection pool
pub fn create_pool<P: AsRef<Path>>(path: P) -> Result<SqlitePool, AgentError> {
    let manager = SqliteConnectionManager::file(path)
        .with_init(|c| {
            // Enable foreign keys
            c.execute_batch("PRAGMA foreign_keys = ON")?;
            Ok(())
        });
    
    Pool::builder()
        .max_size(10) // Maximum 10 connections
        .min_idle(Some(1)) // Keep at least 1 connection idle
        .build(manager)
        .map_err(|e| AgentError::Other(format!("Failed to create connection pool: {}", e)))
}

/// Initialize the database schema
pub fn init_pool_db(pool: &SqlitePool) -> Result<(), AgentError> {
    let conn = pool.get()
        .map_err(|e| AgentError::Other(format!("Failed to get connection from pool: {}", e)))?;
    
    // Create agents table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS agents (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            icon TEXT NOT NULL,
            system_prompt TEXT NOT NULL,
            default_task TEXT,
            model TEXT DEFAULT 'sonnet',
            sandbox_enabled BOOLEAN DEFAULT 1,
            enable_file_read BOOLEAN DEFAULT 1,
            enable_file_write BOOLEAN DEFAULT 1,
            enable_network BOOLEAN DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Create trigger to update the updated_at timestamp
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS update_agent_timestamp 
         AFTER UPDATE ON agents 
         FOR EACH ROW
         BEGIN
             UPDATE agents SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
         END",
        [],
    )?;

    // Create agent_runs table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS agent_runs (
            id INTEGER PRIMARY KEY,
            agent_id INTEGER NOT NULL,
            agent_name TEXT NOT NULL,
            agent_icon TEXT NOT NULL,
            task TEXT NOT NULL,
            model TEXT NOT NULL,
            project_path TEXT NOT NULL,
            session_id TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            pid INTEGER,
            process_started_at TEXT,
            scheduled_start_time TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            completed_at TEXT,
            usage_limit_reset_time TEXT,
            auto_resume_enabled BOOLEAN DEFAULT 0,
            resume_count INTEGER DEFAULT 0,
            parent_run_id INTEGER,
            FOREIGN KEY (agent_id) REFERENCES agents(id)
        )",
        [],
    )?;

    // Create indexes
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_agent_runs_agent_id ON agent_runs(agent_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_agent_runs_status ON agent_runs(status)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_agent_runs_session_id ON agent_runs(session_id)",
        [],
    )?;

    // Create jsonl_outputs table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS jsonl_outputs (
            id INTEGER PRIMARY KEY,
            run_id INTEGER NOT NULL,
            line_number INTEGER NOT NULL,
            timestamp TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (run_id) REFERENCES agent_runs(id),
            UNIQUE(run_id, line_number)
        )",
        [],
    )?;

    // Create index for efficient querying
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_jsonl_outputs_run_id ON jsonl_outputs(run_id)",
        [],
    )?;

    // Add migration for new columns if they don't exist
    let columns_exist: Result<i32, _> = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('agent_runs') WHERE name IN ('usage_limit_reset_time', 'auto_resume_enabled', 'resume_count', 'parent_run_id')",
        [],
        |row| row.get(0),
    );

    if columns_exist.unwrap_or(0) < 4 {
        // Add new columns if they don't exist
        conn.execute(
            "ALTER TABLE agent_runs ADD COLUMN usage_limit_reset_time TEXT",
            [],
        )
        .ok();
        conn.execute(
            "ALTER TABLE agent_runs ADD COLUMN auto_resume_enabled BOOLEAN DEFAULT 0",
            [],
        )
        .ok();
        conn.execute(
            "ALTER TABLE agent_runs ADD COLUMN resume_count INTEGER DEFAULT 0",
            [],
        )
        .ok();
        conn.execute(
            "ALTER TABLE agent_runs ADD COLUMN parent_run_id INTEGER",
            [],
        )
        .ok();
    }

    // Create sandbox_violations table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sandbox_violations (
            id INTEGER PRIMARY KEY,
            run_id INTEGER NOT NULL,
            denied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            operation_type TEXT NOT NULL,
            resource TEXT NOT NULL,
            reason TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (run_id) REFERENCES agent_runs(id)
        )",
        [],
    )?;

    // Create index for efficient querying
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sandbox_violations_denied_at 
         ON sandbox_violations(denied_at DESC)",
        [],
    )?;

    // Create default sandbox profiles if they don't exist
    crate::sandbox::defaults::create_default_profiles(&*conn)?;

    // Create settings table for app-wide settings
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Create trigger to update the updated_at timestamp
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS update_app_settings_timestamp 
         AFTER UPDATE ON app_settings 
         FOR EACH ROW
         BEGIN
             UPDATE app_settings SET updated_at = CURRENT_TIMESTAMP WHERE key = NEW.key;
         END",
        [],
    )?;

    Ok(())
}