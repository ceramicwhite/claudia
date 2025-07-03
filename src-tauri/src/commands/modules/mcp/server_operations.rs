use super::{
    models::ServerStatus,
    utils::{create_command_with_env, execute_claude_mcp_command, find_claude_binary},
};
use log::{error, info};
use std::collections::HashMap;
use tauri::AppHandle;

/// Starts Claude Code as an MCP server
#[tauri::command]
pub async fn mcp_serve(app: AppHandle) -> Result<String, String> {
    info!("Starting Claude Code as MCP server");

    // Start the server in a separate process
    let claude_path = match find_claude_binary(&app) {
        Ok(path) => path,
        Err(e) => {
            error!("Failed to find claude binary: {}", e);
            return Err(e.to_string());
        }
    };

    let mut cmd = create_command_with_env(&claude_path);
    cmd.arg("mcp").arg("serve");

    match cmd.spawn() {
        Ok(_) => {
            info!("Successfully started Claude Code MCP server");
            Ok("Claude Code MCP server started".to_string())
        }
        Err(e) => {
            error!("Failed to start MCP server: {}", e);
            Err(e.to_string())
        }
    }
}

/// Tests connection to an MCP server
#[tauri::command]
pub async fn mcp_test_connection(app: AppHandle, name: String) -> Result<String, String> {
    info!("Testing connection to MCP server: {}", name);

    // For now, we'll use the get command to test if the server exists
    match execute_claude_mcp_command(&app, vec!["get", &name]) {
        Ok(_) => Ok(format!("Connection to {} successful", name)),
        Err(e) => Err(e.to_string()),
    }
}

/// Resets project-scoped server approval choices
#[tauri::command]
pub async fn mcp_reset_project_choices(app: AppHandle) -> Result<String, String> {
    info!("Resetting MCP project choices");

    match execute_claude_mcp_command(&app, vec!["reset-project-choices"]) {
        Ok(output) => {
            info!("Successfully reset MCP project choices");
            Ok(output.trim().to_string())
        }
        Err(e) => {
            error!("Failed to reset project choices: {}", e);
            Err(e.to_string())
        }
    }
}

/// Gets the status of MCP servers
#[tauri::command]
pub async fn mcp_get_server_status() -> Result<HashMap<String, ServerStatus>, String> {
    info!("Getting MCP server status");

    // TODO: Implement actual status checking
    // For now, return empty status
    Ok(HashMap::new())
}