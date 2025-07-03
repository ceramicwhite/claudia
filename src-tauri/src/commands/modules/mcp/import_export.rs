use super::{
    models::{AddServerResult, ImportResult, ImportServerResult},
    utils::execute_claude_mcp_command,
};
use dirs;
use log::{error, info};
use std::fs;
use tauri::AppHandle;

/// Adds an MCP server from JSON configuration
#[tauri::command]
pub async fn mcp_add_json(
    app: AppHandle,
    name: String,
    json_config: String,
    scope: String,
) -> Result<AddServerResult, String> {
    info!(
        "Adding MCP server from JSON: {} with scope: {}",
        name, scope
    );

    // Build command args
    let mut cmd_args = vec!["add-json", &name, &json_config];

    // Add scope flag
    let scope_flag = "-s";
    cmd_args.push(scope_flag);
    cmd_args.push(&scope);

    match execute_claude_mcp_command(&app, cmd_args) {
        Ok(output) => {
            info!("Successfully added MCP server from JSON: {}", name);
            Ok(AddServerResult {
                success: true,
                message: output.trim().to_string(),
                server_name: Some(name),
            })
        }
        Err(e) => {
            error!("Failed to add MCP server from JSON: {}", e);
            Ok(AddServerResult {
                success: false,
                message: e.to_string(),
                server_name: None,
            })
        }
    }
}

/// Imports MCP servers from Claude Desktop
#[tauri::command]
pub async fn mcp_add_from_claude_desktop(
    app: AppHandle,
    scope: String,
) -> Result<ImportResult, String> {
    info!(
        "Importing MCP servers from Claude Desktop with scope: {}",
        scope
    );

    // Get Claude Desktop config path based on platform
    let config_path = if cfg!(target_os = "macos") {
        dirs::home_dir()
            .ok_or_else(|| "Could not find home directory".to_string())?
            .join("Library")
            .join("Application Support")
            .join("Claude")
            .join("claude_desktop_config.json")
    } else if cfg!(target_os = "linux") {
        // For WSL/Linux, check common locations
        dirs::config_dir()
            .ok_or_else(|| "Could not find config directory".to_string())?
            .join("Claude")
            .join("claude_desktop_config.json")
    } else {
        return Err(
            "Import from Claude Desktop is only supported on macOS and Linux/WSL".to_string(),
        );
    };

    // Check if config file exists
    if !config_path.exists() {
        return Err(
            "Claude Desktop configuration not found. Make sure Claude Desktop is installed."
                .to_string(),
        );
    }

    // Read and parse the config file
    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read Claude Desktop config: {}", e))?;

    let config: serde_json::Value = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse Claude Desktop config: {}", e))?;

    // Extract MCP servers
    let mcp_servers = config
        .get("mcpServers")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "No MCP servers found in Claude Desktop config".to_string())?;

    let mut imported_count = 0;
    let mut failed_count = 0;
    let mut server_results = Vec::new();

    // Import each server using add-json
    for (name, server_config) in mcp_servers {
        info!("Importing server: {}", name);

        // Convert Claude Desktop format to add-json format
        let mut json_config = serde_json::Map::new();

        // All Claude Desktop servers are stdio type
        json_config.insert(
            "type".to_string(),
            serde_json::Value::String("stdio".to_string()),
        );

        // Add command
        if let Some(command) = server_config.get("command").and_then(|v| v.as_str()) {
            json_config.insert(
                "command".to_string(),
                serde_json::Value::String(command.to_string()),
            );
        } else {
            failed_count += 1;
            server_results.push(ImportServerResult {
                name: name.clone(),
                success: false,
                error: Some("Missing command field".to_string()),
            });
            continue;
        }

        // Add args if present
        if let Some(args) = server_config.get("args").and_then(|v| v.as_array()) {
            json_config.insert("args".to_string(), args.clone().into());
        } else {
            json_config.insert("args".to_string(), serde_json::Value::Array(vec![]));
        }

        // Add env if present
        if let Some(env) = server_config.get("env").and_then(|v| v.as_object()) {
            json_config.insert("env".to_string(), env.clone().into());
        } else {
            json_config.insert(
                "env".to_string(),
                serde_json::Value::Object(serde_json::Map::new()),
            );
        }

        // Convert to JSON string
        let json_str = serde_json::to_string(&json_config)
            .map_err(|e| format!("Failed to serialize config for {}: {}", name, e))?;

        // Call add-json command
        match crate::commands::mcp::mcp_add_json(app.clone(), name.clone(), json_str, scope.clone()).await {
            Ok(result) => {
                if result.success {
                    imported_count += 1;
                    server_results.push(ImportServerResult {
                        name: name.clone(),
                        success: true,
                        error: None,
                    });
                    info!("Successfully imported server: {}", name);
                } else {
                    failed_count += 1;
                    let error_msg = result.message.clone();
                    server_results.push(ImportServerResult {
                        name: name.clone(),
                        success: false,
                        error: Some(result.message),
                    });
                    error!("Failed to import server {}: {}", name, error_msg);
                }
            }
            Err(e) => {
                failed_count += 1;
                let error_msg = e.clone();
                server_results.push(ImportServerResult {
                    name: name.clone(),
                    success: false,
                    error: Some(e),
                });
                error!("Error importing server {}: {}", name, error_msg);
            }
        }
    }

    info!(
        "Import complete: {} imported, {} failed",
        imported_count, failed_count
    );

    Ok(ImportResult {
        imported_count,
        failed_count,
        servers: server_results,
    })
}