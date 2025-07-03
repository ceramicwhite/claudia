use chrono;
use rusqlite::params;
use tauri::State;

use super::database::AgentDb;
use super::models::{Agent, AgentExport};

/// Export an agent to JSON format
#[tauri::command]
pub async fn export_agent(db: State<'_, AgentDb>, id: i64) -> Result<String, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Fetch the agent
    let agent = conn
        .query_row(
            "SELECT name, icon, system_prompt, default_task, model, sandbox_enabled, enable_file_read, enable_file_write, enable_network FROM agents WHERE id = ?1",
            params![id],
            |row| {
                Ok(serde_json::json!({
                    "name": row.get::<_, String>(0)?,
                    "icon": row.get::<_, String>(1)?,
                    "system_prompt": row.get::<_, String>(2)?,
                    "default_task": row.get::<_, Option<String>>(3)?,
                    "model": row.get::<_, String>(4)?,
                    "sandbox_enabled": row.get::<_, bool>(5)?,
                    "enable_file_read": row.get::<_, bool>(6)?,
                    "enable_file_write": row.get::<_, bool>(7)?,
                    "enable_network": row.get::<_, bool>(8)?
                }))
            },
        )
        .map_err(|e| format!("Failed to fetch agent: {}", e))?;

    // Create the export wrapper
    let export_data = serde_json::json!({
        "version": 1,
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "agent": agent
    });

    // Convert to pretty JSON string
    serde_json::to_string_pretty(&export_data)
        .map_err(|e| format!("Failed to serialize agent: {}", e))
}

/// Export agent to file with native dialog
#[tauri::command]
pub async fn export_agent_to_file(
    db: State<'_, AgentDb>,
    id: i64,
    file_path: String,
) -> Result<(), String> {
    // Get the JSON data
    let json_data = export_agent(db, id).await?;

    // Write to file
    std::fs::write(&file_path, json_data).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

/// Import an agent from JSON data
#[tauri::command]
pub async fn import_agent(db: State<'_, AgentDb>, json_data: String) -> Result<Agent, String> {
    // Parse the JSON data
    let export_data: AgentExport =
        serde_json::from_str(&json_data).map_err(|e| format!("Invalid JSON format: {}", e))?;

    // Validate version
    if export_data.version != 1 {
        return Err(format!(
            "Unsupported export version: {}. This version of the app only supports version 1.",
            export_data.version
        ));
    }

    let agent_data = export_data.agent;
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Check if an agent with the same name already exists
    let existing_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM agents WHERE name = ?1",
            params![agent_data.name],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    // If agent with same name exists, append a suffix
    let final_name = if existing_count > 0 {
        format!("{} (Imported)", agent_data.name)
    } else {
        agent_data.name
    };

    // Create the agent
    conn.execute(
        "INSERT INTO agents (name, icon, system_prompt, default_task, model, sandbox_enabled, enable_file_read, enable_file_write, enable_network) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            final_name,
            agent_data.icon,
            agent_data.system_prompt,
            agent_data.default_task,
            agent_data.model,
            agent_data.sandbox_enabled,
            agent_data.enable_file_read,
            agent_data.enable_file_write,
            agent_data.enable_network
        ],
    )
    .map_err(|e| format!("Failed to create agent: {}", e))?;

    let id = conn.last_insert_rowid();

    // Fetch the created agent
    let agent = conn
        .query_row(
            "SELECT id, name, icon, system_prompt, default_task, model, sandbox_enabled, enable_file_read, enable_file_write, enable_network, created_at, updated_at FROM agents WHERE id = ?1",
            params![id],
            |row| {
                Ok(Agent {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    icon: row.get(2)?,
                    system_prompt: row.get(3)?,
                    default_task: row.get(4)?,
                    model: row.get(5)?,
                    sandbox_enabled: row.get(6)?,
                    enable_file_read: row.get(7)?,
                    enable_file_write: row.get(8)?,
                    enable_network: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            },
        )
        .map_err(|e| format!("Failed to fetch created agent: {}", e))?;

    Ok(agent)
}

/// Import agent from file
#[tauri::command]
pub async fn import_agent_from_file(
    db: State<'_, AgentDb>,
    file_path: String,
) -> Result<Agent, String> {
    // Read the file
    let json_data =
        std::fs::read_to_string(&file_path).map_err(|e| format!("Failed to read file: {}", e))?;

    // Import the agent
    import_agent(db, json_data).await
}