use log::{debug, info};
use rusqlite::{params, Connection, Result as SqliteResult};
use serde_json::Value as JsonValue;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

use super::utils::parse_usage_limit_error;

pub struct AgentDb(pub Mutex<Connection>);

/// Migrate old runs that ended with usage limit but don't have the new status
fn migrate_old_usage_limit_runs(conn: &Connection) {
    // Get all completed runs that might have ended with usage limit
    let mut stmt = match conn.prepare(
        "SELECT id, session_id, project_path FROM agent_runs 
         WHERE status IN ('completed', 'failed') 
         AND usage_limit_reset_time IS NULL"
    ) {
        Ok(stmt) => stmt,
        Err(e) => {
            debug!("Failed to prepare migration query: {}", e);
            return;
        }
    };
    
    let runs = match stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,     // id
            row.get::<_, String>(1)?,  // session_id
            row.get::<_, String>(2)?,  // project_path
        ))
    }) {
        Ok(runs) => runs.collect::<Result<Vec<_>, _>>(),
        Err(e) => {
            debug!("Failed to query runs for migration: {}", e);
            return;
        }
    };
    
    if let Ok(runs) = runs {
        for (run_id, session_id, project_path) in runs {
            // Try to read the JSONL file to check for usage limit error
            let claude_dir = dirs::home_dir()
                .and_then(|home| Some(home.join(".claude").join("projects")));
                
            if let Some(claude_dir) = claude_dir {
                let encoded_project = project_path.replace('/', "-");
                let project_dir = claude_dir.join(&encoded_project);
                let session_file = project_dir.join(format!("{}.jsonl", session_id));
                
                if session_file.exists() {
                    if let Ok(content) = std::fs::read_to_string(&session_file) {
                        // Check the last few lines for usage limit error
                        let lines: Vec<&str> = content.lines().collect();
                        for line in lines.iter().rev().take(10) {
                            if let Ok(json) = serde_json::from_str::<JsonValue>(line) {
                                // Check for error in result messages
                                if json.get("type").and_then(|t| t.as_str()) == Some("result") {
                                    if let Some(error) = json.get("error").and_then(|e| e.as_str()) {
                                        if let Some(reset_time) = parse_usage_limit_error(error) {
                                            // Update the run with the new status and reset time
                                            let _ = conn.execute(
                                                "UPDATE agent_runs 
                                                 SET status = 'paused_usage_limit', 
                                                     usage_limit_reset_time = ?1 
                                                 WHERE id = ?2",
                                                params![reset_time, run_id]
                                            );
                                            info!("Migrated run {} to paused_usage_limit status", run_id);
                                            break;
                                        }
                                    }
                                }
                                
                                // Also check assistant messages for the error
                                if json.get("type").and_then(|t| t.as_str()) == Some("assistant") {
                                    if let Some(message) = json.get("message").and_then(|m| m.as_object()) {
                                        if let Some(content) = message.get("content").and_then(|c| c.as_array()) {
                                            for item in content {
                                                if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                                                    if let Some(reset_time) = parse_usage_limit_error(text) {
                                                        let _ = conn.execute(
                                                            "UPDATE agent_runs 
                                                             SET status = 'paused_usage_limit', 
                                                                 usage_limit_reset_time = ?1 
                                                             WHERE id = ?2",
                                                            params![reset_time, run_id]
                                                        );
                                                        info!("Migrated run {} to paused_usage_limit status", run_id);
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Initialize the agents database
pub fn init_database(app: &AppHandle) -> SqliteResult<Connection> {
    let app_dir = app
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir");
    std::fs::create_dir_all(&app_dir).expect("Failed to create app data dir");

    let db_path = app_dir.join("agents.db");
    let conn = Connection::open(db_path)?;

    // Create agents table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS agents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            icon TEXT NOT NULL,
            system_prompt TEXT NOT NULL,
            default_task TEXT,
            model TEXT NOT NULL DEFAULT 'sonnet',
            sandbox_enabled BOOLEAN NOT NULL DEFAULT 1,
            enable_file_read BOOLEAN NOT NULL DEFAULT 1,
            enable_file_write BOOLEAN NOT NULL DEFAULT 1,
            enable_network BOOLEAN NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Add columns to existing table if they don't exist
    let _ = conn.execute("ALTER TABLE agents ADD COLUMN default_task TEXT", []);
    let _ = conn.execute(
        "ALTER TABLE agents ADD COLUMN model TEXT DEFAULT 'sonnet'",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE agents ADD COLUMN sandbox_profile_id INTEGER REFERENCES sandbox_profiles(id)",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE agents ADD COLUMN sandbox_enabled BOOLEAN DEFAULT 1",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE agents ADD COLUMN enable_file_read BOOLEAN DEFAULT 1",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE agents ADD COLUMN enable_file_write BOOLEAN DEFAULT 1",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE agents ADD COLUMN enable_network BOOLEAN DEFAULT 0",
        [],
    );

    // Create agent_runs table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS agent_runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
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
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            completed_at TEXT,
            FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Migrate existing agent_runs table if needed
    let _ = conn.execute("ALTER TABLE agent_runs ADD COLUMN session_id TEXT", []);
    let _ = conn.execute(
        "ALTER TABLE agent_runs ADD COLUMN status TEXT DEFAULT 'pending'",
        [],
    );
    let _ = conn.execute("ALTER TABLE agent_runs ADD COLUMN pid INTEGER", []);
    let _ = conn.execute(
        "ALTER TABLE agent_runs ADD COLUMN process_started_at TEXT",
        [],
    );

    // Add scheduled_start_time column to agent_runs table for scheduling individual runs
    let _ = conn.execute(
        "ALTER TABLE agent_runs ADD COLUMN scheduled_start_time TEXT",
        [],
    );
    
    // Add columns for usage limit tracking and resumption
    let _ = conn.execute(
        "ALTER TABLE agent_runs ADD COLUMN usage_limit_reset_time TEXT",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE agent_runs ADD COLUMN auto_resume_enabled BOOLEAN DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE agent_runs ADD COLUMN resume_count INTEGER DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE agent_runs ADD COLUMN parent_run_id INTEGER",
        [],
    );
    
    // Migrate existing data to have default values for new non-nullable columns
    let _ = conn.execute(
        "UPDATE agent_runs SET auto_resume_enabled = 0 WHERE auto_resume_enabled IS NULL",
        [],
    );
    let _ = conn.execute(
        "UPDATE agent_runs SET resume_count = 0 WHERE resume_count IS NULL",
        [],
    );
    
    // Migrate old runs that ended with usage limit but don't have the new status
    // This will help identify old runs that should show the resume button
    migrate_old_usage_limit_runs(&conn);

    // Update status for old runs that don't have a status set
    let _ = conn.execute("UPDATE agent_runs SET status = 'completed' WHERE status IS NULL AND completed_at IS NOT NULL", []);
    let _ = conn.execute("UPDATE agent_runs SET status = 'failed' WHERE status IS NULL AND completed_at IS NOT NULL AND session_id IS NULL", []);
    let _ = conn.execute(
        "UPDATE agent_runs SET status = 'pending' WHERE status IS NULL",
        [],
    );

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

    // Create sandbox profiles table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sandbox_profiles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            is_active BOOLEAN NOT NULL DEFAULT 0,
            is_default BOOLEAN NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Create sandbox rules table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sandbox_rules (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            profile_id INTEGER NOT NULL,
            operation_type TEXT NOT NULL,
            pattern_type TEXT NOT NULL,
            pattern_value TEXT NOT NULL,
            enabled BOOLEAN NOT NULL DEFAULT 1,
            platform_support TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (profile_id) REFERENCES sandbox_profiles(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Create trigger to update sandbox profile timestamp
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS update_sandbox_profile_timestamp 
         AFTER UPDATE ON sandbox_profiles 
         FOR EACH ROW
         BEGIN
             UPDATE sandbox_profiles SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
         END",
        [],
    )?;

    // Create sandbox violations table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sandbox_violations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            profile_id INTEGER,
            agent_id INTEGER,
            agent_run_id INTEGER,
            operation_type TEXT NOT NULL,
            pattern_value TEXT,
            process_name TEXT,
            pid INTEGER,
            denied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (profile_id) REFERENCES sandbox_profiles(id) ON DELETE CASCADE,
            FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE,
            FOREIGN KEY (agent_run_id) REFERENCES agent_runs(id) ON DELETE CASCADE
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
    crate::sandbox::defaults::create_default_profiles(&conn)?;

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

    Ok(conn)
}