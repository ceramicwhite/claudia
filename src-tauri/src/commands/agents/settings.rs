use rusqlite::params;
use tauri::State;

use super::database::AgentDb;

/// Get the stored Claude binary path from settings
#[tauri::command]
pub async fn get_claude_binary_path(db: State<'_, AgentDb>) -> Result<Option<String>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    match conn.query_row(
        "SELECT value FROM app_settings WHERE key = 'claude_binary_path'",
        [],
        |row| row.get::<_, String>(0),
    ) {
        Ok(path) => Ok(Some(path)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Failed to get Claude binary path: {}", e)),
    }
}

/// Set the Claude binary path in settings
#[tauri::command]
pub async fn set_claude_binary_path(db: State<'_, AgentDb>, path: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Validate that the path exists and is executable
    let path_buf = std::path::PathBuf::from(&path);
    if !path_buf.exists() {
        return Err(format!("File does not exist: {}", path));
    }

    // Check if it's executable (on Unix systems)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&path_buf)
            .map_err(|e| format!("Failed to read file metadata: {}", e))?;
        let permissions = metadata.permissions();
        if permissions.mode() & 0o111 == 0 {
            return Err(format!("File is not executable: {}", path));
        }
    }

    // Insert or update the setting
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES ('claude_binary_path', ?1)
         ON CONFLICT(key) DO UPDATE SET value = ?1",
        params![path],
    )
    .map_err(|e| format!("Failed to save Claude binary path: {}", e))?;

    Ok(())
}

/// List all available Claude installations on the system
#[tauri::command]
pub async fn list_claude_installations(
) -> Result<Vec<crate::claude_binary::ClaudeInstallation>, String> {
    let installations = crate::claude_binary::discover_claude_installations();

    if installations.is_empty() {
        return Err("No Claude Code installations found on the system".to_string());
    }

    Ok(installations)
}