use std::process::Stdio;
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use uuid;

use crate::commands::modules::claude::state::ClaudeProcessState;
use crate::commands::modules::claude::utils::{create_command_with_env, find_claude_binary, get_claude_settings_sync};

/// Execute a new interactive Claude Code session with streaming output
#[tauri::command]
pub async fn execute_claude_code(
    app: AppHandle,
    project_path: String,
    prompt: String,
    model: String,
) -> Result<(), String> {
    log::info!(
        "Starting new Claude Code session in: {} with model: {}",
        project_path,
        model
    );

    // Check if sandboxing should be used
    let use_sandbox = should_use_sandbox(&app)?;

    let mut cmd = if use_sandbox {
        create_sandboxed_claude_command(&app, &project_path)?
    } else {
        let claude_path = find_claude_binary(&app)?;
        create_command_with_env(&claude_path)
    };

    cmd.arg("-p")
        .arg(&prompt)
        .arg("--model")
        .arg(&model)
        .arg("--output-format")
        .arg("stream-json")
        .arg("--verbose")
        .arg("--dangerously-skip-permissions")
        .current_dir(&project_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    spawn_claude_process(app, cmd).await
}

/// Continue an existing Claude Code conversation with streaming output
#[tauri::command]
pub async fn continue_claude_code(
    app: AppHandle,
    project_path: String,
    prompt: String,
    model: String,
) -> Result<(), String> {
    log::info!(
        "Continuing Claude Code conversation in: {} with model: {}",
        project_path,
        model
    );

    // Check if sandboxing should be used
    let use_sandbox = should_use_sandbox(&app)?;

    let mut cmd = if use_sandbox {
        create_sandboxed_claude_command(&app, &project_path)?
    } else {
        let claude_path = find_claude_binary(&app)?;
        create_command_with_env(&claude_path)
    };

    cmd.arg("-c") // Continue flag
        .arg("-p")
        .arg(&prompt)
        .arg("--model")
        .arg(&model)
        .arg("--output-format")
        .arg("stream-json")
        .arg("--verbose")
        .arg("--dangerously-skip-permissions")
        .current_dir(&project_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    spawn_claude_process(app, cmd).await
}

/// Resume an existing Claude Code session by ID with streaming output
#[tauri::command]
pub async fn resume_claude_code(
    app: AppHandle,
    project_path: String,
    session_id: String,
    prompt: String,
    model: String,
) -> Result<(), String> {
    log::info!(
        "Resuming Claude Code session: {} in: {} with model: {}",
        session_id,
        project_path,
        model
    );

    // Check if sandboxing should be used
    let use_sandbox = should_use_sandbox(&app)?;

    let mut cmd = if use_sandbox {
        create_sandboxed_claude_command(&app, &project_path)?
    } else {
        let claude_path = find_claude_binary(&app)?;
        create_command_with_env(&claude_path)
    };

    cmd.arg("--resume")
        .arg(&session_id)
        .arg("-p")
        .arg(&prompt)
        .arg("--model")
        .arg(&model)
        .arg("--output-format")
        .arg("stream-json")
        .arg("--verbose")
        .arg("--dangerously-skip-permissions")
        .current_dir(&project_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    spawn_claude_process(app, cmd).await
}

/// Cancel the currently running Claude Code execution
#[tauri::command]
pub async fn cancel_claude_execution(
    app: AppHandle,
    session_id: Option<String>,
) -> Result<(), String> {
    log::info!(
        "Cancelling Claude Code execution for session: {:?}",
        session_id
    );

    let claude_state = app.state::<ClaudeProcessState>();
    let mut current_process = claude_state.current_process.lock().await;

    if let Some(mut child) = current_process.take() {
        // Try to get the PID before killing
        let pid = child.id();
        log::info!("Attempting to kill Claude process with PID: {:?}", pid);

        // Kill the process
        match child.kill().await {
            Ok(_) => {
                log::info!("Successfully killed Claude process");

                // If we have a session ID, emit session-specific events
                if let Some(sid) = session_id {
                    let _ = app.emit(&format!("claude-cancelled:{}", sid), true);
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    let _ = app.emit(&format!("claude-complete:{}", sid), false);
                }

                // Also emit generic events for backward compatibility
                let _ = app.emit("claude-cancelled", true);
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                let _ = app.emit("claude-complete", false);
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to kill Claude process: {}", e);
                Err(format!("Failed to kill Claude process: {}", e))
            }
        }
    } else {
        log::warn!("No active Claude process to cancel");
        Ok(())
    }
}

/// Helper function to check if sandboxing should be used based on settings
fn should_use_sandbox(app: &AppHandle) -> Result<bool, String> {
    // First check if sandboxing is even available on this platform
    if !crate::sandbox::platform::is_sandboxing_available() {
        log::info!("Sandboxing not available on this platform");
        return Ok(false);
    }

    // Check if a setting exists to enable/disable sandboxing
    let settings = get_claude_settings_sync(app)?;

    // Check for a sandboxing setting in the settings
    if let Some(sandbox_enabled) = settings
        .data
        .get("sandboxEnabled")
        .and_then(|v| v.as_bool())
    {
        return Ok(sandbox_enabled);
    }

    // Default to true (sandboxing enabled) on supported platforms
    Ok(true)
}

/// Helper function to create a sandboxed Claude command
fn create_sandboxed_claude_command(app: &AppHandle, project_path: &str) -> Result<Command, String> {
    use crate::sandbox::{executor::create_sandboxed_command, profile::ProfileBuilder};
    use std::path::PathBuf;

    // Get the database connection
    let conn = {
        let app_data_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {}", e))?;
        let db_path = app_data_dir.join("agents.db");
        rusqlite::Connection::open(&db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?
    };

    // Query for the default active sandbox profile
    let profile_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM sandbox_profiles WHERE is_default = 1 AND is_active = 1",
            [],
            |row| row.get(0),
        )
        .ok();

    match profile_id {
        Some(profile_id) => {
            log::info!(
                "Using default sandbox profile: {} (id: {})",
                profile_id,
                profile_id
            );

            // Get all rules for this profile
            let mut stmt = conn
                .prepare(
                    "SELECT operation_type, pattern_type, pattern_value, enabled, platform_support 
                 FROM sandbox_rules WHERE profile_id = ?1 AND enabled = 1",
                )
                .map_err(|e| e.to_string())?;

            let rules = stmt
                .query_map(rusqlite::params![profile_id], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, bool>(3)?,
                        row.get::<_, Option<String>>(4)?,
                    ))
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            log::info!("Building sandbox profile with {} rules", rules.len());

            // Build the gaol profile
            let project_path_buf = PathBuf::from(project_path);

            match ProfileBuilder::new(project_path_buf.clone()) {
                Ok(builder) => {
                    // Convert database rules to SandboxRule structs
                    let mut sandbox_rules = Vec::new();

                    for (idx, (op_type, pattern_type, pattern_value, enabled, platform_support)) in
                        rules.into_iter().enumerate()
                    {
                        // Check if this rule applies to the current platform
                        if let Some(platforms_json) = &platform_support {
                            if let Ok(platforms) =
                                serde_json::from_str::<Vec<String>>(platforms_json)
                            {
                                let current_platform = if cfg!(target_os = "linux") {
                                    "linux"
                                } else if cfg!(target_os = "macos") {
                                    "macos"
                                } else if cfg!(target_os = "freebsd") {
                                    "freebsd"
                                } else {
                                    "unsupported"
                                };

                                if !platforms.contains(&current_platform.to_string()) {
                                    continue;
                                }
                            }
                        }

                        // Create SandboxRule struct
                        let rule = crate::sandbox::profile::SandboxRule {
                            id: Some(idx as i64),
                            profile_id: 0,
                            operation_type: op_type,
                            pattern_type,
                            pattern_value,
                            enabled,
                            platform_support,
                            created_at: String::new(),
                        };

                        sandbox_rules.push(rule);
                    }

                    // Try to build the profile
                    match builder.build_profile(sandbox_rules) {
                        Ok(profile) => {
                            log::info!("Successfully built sandbox profile '{}'", profile_id);

                            // Use the helper function to create sandboxed command
                            let claude_path = find_claude_binary(app)?;
                            #[cfg(unix)]
                            return Ok(create_sandboxed_command(
                                &claude_path,
                                &[],
                                &project_path_buf,
                                profile,
                                project_path_buf.clone(),
                            ));

                            #[cfg(not(unix))]
                            {
                                log::warn!(
                                    "Sandboxing not supported on Windows, using regular command"
                                );
                                Ok(create_command_with_env(&claude_path))
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to build sandbox profile: {}, falling back to non-sandboxed", e);
                            let claude_path = find_claude_binary(app)?;
                            Ok(create_command_with_env(&claude_path))
                        }
                    }
                }
                Err(e) => {
                    log::error!(
                        "Failed to create ProfileBuilder: {}, falling back to non-sandboxed",
                        e
                    );
                    let claude_path = find_claude_binary(app)?;
                    Ok(create_command_with_env(&claude_path))
                }
            }
        }
        None => {
            log::info!("No default active sandbox profile found: proceeding without sandbox");
            let claude_path = find_claude_binary(app)?;
            Ok(create_command_with_env(&claude_path))
        }
    }
}

/// Helper function to spawn Claude process and handle streaming
async fn spawn_claude_process(app: AppHandle, mut cmd: Command) -> Result<(), String> {
    // Generate a unique session ID for this Claude Code session
    let session_id = format!(
        "claude-{}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        uuid::Uuid::new_v4().to_string()
    );

    // Spawn the process
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn Claude: {}", e))?;

    // Get stdout and stderr
    let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to get stderr")?;

    // Get the child PID for logging
    let pid = child.id();
    log::info!(
        "Spawned Claude process with PID: {:?} and session ID: {}",
        pid,
        session_id
    );

    // Create readers
    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    // Store the child process in the global state (for backward compatibility)
    let claude_state = app.state::<ClaudeProcessState>();
    {
        let mut current_process = claude_state.current_process.lock().await;
        // If there's already a process running, kill it first
        if let Some(mut existing_child) = current_process.take() {
            log::warn!("Killing existing Claude process before starting new one");
            let _ = existing_child.kill().await;
        }
        *current_process = Some(child);
    }

    // Spawn tasks to read stdout and stderr
    let app_handle = app.clone();
    let session_id_clone = session_id.clone();
    let stdout_task = tokio::spawn(async move {
        let mut lines = stdout_reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            log::debug!("Claude stdout: {}", line);
            // Emit the line to the frontend with session isolation
            let _ = app_handle.emit(&format!("claude-output:{}", session_id_clone), &line);
            // Also emit to the generic event for backward compatibility
            let _ = app_handle.emit("claude-output", &line);
        }
    });

    let app_handle_stderr = app.clone();
    let session_id_clone2 = session_id.clone();
    let stderr_task = tokio::spawn(async move {
        let mut lines = stderr_reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            log::error!("Claude stderr: {}", line);
            // Emit error lines to the frontend with session isolation
            let _ = app_handle_stderr.emit(&format!("claude-error:{}", session_id_clone2), &line);
            // Also emit to the generic event for backward compatibility
            let _ = app_handle_stderr.emit("claude-error", &line);
        }
    });

    // Wait for the process to complete
    let app_handle_wait = app.clone();
    let claude_state_wait = claude_state.current_process.clone();
    let session_id_clone3 = session_id.clone();
    tokio::spawn(async move {
        let _ = stdout_task.await;
        let _ = stderr_task.await;

        // Get the child from the state to wait on it
        let mut current_process = claude_state_wait.lock().await;
        if let Some(mut child) = current_process.take() {
            match child.wait().await {
                Ok(status) => {
                    log::info!("Claude process exited with status: {}", status);
                    // Add a small delay to ensure all messages are processed
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    let _ = app_handle_wait.emit(
                        &format!("claude-complete:{}", session_id_clone3),
                        status.success(),
                    );
                    // Also emit to the generic event for backward compatibility
                    let _ = app_handle_wait.emit("claude-complete", status.success());
                }
                Err(e) => {
                    log::error!("Failed to wait for Claude process: {}", e);
                    // Add a small delay to ensure all messages are processed
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    let _ = app_handle_wait
                        .emit(&format!("claude-complete:{}", session_id_clone3), false);
                    // Also emit to the generic event for backward compatibility
                    let _ = app_handle_wait.emit("claude-complete", false);
                }
            }
        }

        // Clear the process from state
        *current_process = None;
    });

    // Return the session ID to the frontend
    let _ = app.emit(
        &format!("claude-session-started:{}", session_id),
        session_id.clone(),
    );

    Ok(())
}