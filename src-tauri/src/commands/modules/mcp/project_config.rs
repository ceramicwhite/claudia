use super::models::MCPProjectConfig;
use log::{error, info};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Reads .mcp.json from the current project
#[tauri::command]
pub async fn mcp_read_project_config(project_path: String) -> Result<MCPProjectConfig, String> {
    info!("Reading .mcp.json from project: {}", project_path);

    let mcp_json_path = PathBuf::from(&project_path).join(".mcp.json");

    if !mcp_json_path.exists() {
        return Ok(MCPProjectConfig {
            mcp_servers: HashMap::new(),
        });
    }

    match fs::read_to_string(&mcp_json_path) {
        Ok(content) => match serde_json::from_str::<MCPProjectConfig>(&content) {
            Ok(config) => Ok(config),
            Err(e) => {
                error!("Failed to parse .mcp.json: {}", e);
                Err(format!("Failed to parse .mcp.json: {}", e))
            }
        },
        Err(e) => {
            error!("Failed to read .mcp.json: {}", e);
            Err(format!("Failed to read .mcp.json: {}", e))
        }
    }
}

/// Saves .mcp.json to the current project
#[tauri::command]
pub async fn mcp_save_project_config(
    project_path: String,
    config: MCPProjectConfig,
) -> Result<String, String> {
    info!("Saving .mcp.json to project: {}", project_path);

    let mcp_json_path = PathBuf::from(&project_path).join(".mcp.json");

    let json_content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&mcp_json_path, json_content)
        .map_err(|e| format!("Failed to write .mcp.json: {}", e))?;

    Ok("Project MCP configuration saved".to_string())
}