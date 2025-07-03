use anyhow::{Context, Result};
use log::info;
use std::process::Command;
use tauri::AppHandle;

/// Helper function to create a std::process::Command with proper environment variables
/// This ensures commands like Claude can find Node.js and other dependencies
pub fn create_command_with_env(program: &str) -> Command {
    crate::claude_binary::create_command_with_env(program)
}

/// Finds the full path to the claude binary
/// This is necessary because macOS apps have a limited PATH environment
pub fn find_claude_binary(app_handle: &AppHandle) -> Result<String> {
    crate::claude_binary::find_claude_binary(app_handle).map_err(|e| anyhow::anyhow!(e))
}

/// Executes a claude mcp command
pub fn execute_claude_mcp_command(app_handle: &AppHandle, args: Vec<&str>) -> Result<String> {
    info!("Executing claude mcp command with args: {:?}", args);

    let claude_path = find_claude_binary(app_handle)?;
    let mut cmd = create_command_with_env(&claude_path);
    cmd.arg("mcp");
    for arg in args {
        cmd.arg(arg);
    }

    let output = cmd.output().context("Failed to execute claude command")?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(anyhow::anyhow!("Command failed: {}", stderr))
    }
}