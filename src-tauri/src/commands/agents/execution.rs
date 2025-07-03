use crate::sandbox::profile::ProfileBuilder;
use chrono;
use log::{debug, error, info, warn};
use rusqlite::{params, Connection};
use serde_json::Value as JsonValue;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::io::{AsyncBufReadExt, BufReader};

use super::database::AgentDb;
use super::models::{AgentRun, AgentRunWithMetrics};
use super::utils::{create_command_with_env, find_claude_binary, get_agent_run_with_metrics, parse_usage_limit_error, read_session_jsonl};
use super::crud::{get_agent, get_agent_run};

/// Execute an agent with the given parameters
#[tauri::command]
pub async fn execute_agent(
    app: AppHandle,
    agent_id: i64,
    project_path: String,
    task: String,
    model: Option<String>,
    auto_resume_enabled: Option<bool>,
    db: State<'_, AgentDb>,
    registry: State<'_, crate::process::ProcessRegistryState>,
) -> Result<i64, String> {
    info!("Executing agent {} with task: {}", agent_id, task);

    // Get the agent from database
    let agent = get_agent(db.clone(), agent_id).await?;
    let execution_model = model.unwrap_or(agent.model.clone());

    // Create a new run record
    let run_id = {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO agent_runs (agent_id, agent_name, agent_icon, task, model, project_path, session_id, auto_resume_enabled, resume_count) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![agent_id, agent.name, agent.icon, task, execution_model, project_path, "", auto_resume_enabled.unwrap_or(false), 0],
        )
        .map_err(|e| e.to_string())?;
        conn.last_insert_rowid()
    };

    // Create sandbox rules based on agent-specific permissions (no database dependency)
    let sandbox_profile = if !agent.sandbox_enabled {
        info!("🔓 Agent '{}': Sandbox DISABLED", agent.name);
        None
    } else {
        info!(
            "🔒 Agent '{}': Sandbox enabled | File Read: {} | File Write: {} | Network: {}",
            agent.name, agent.enable_file_read, agent.enable_file_write, agent.enable_network
        );

        // Create rules dynamically based on agent permissions
        let mut rules = Vec::new();

        // Add file read rules if enabled
        if agent.enable_file_read {
            // Project directory access
            rules.push(crate::sandbox::profile::SandboxRule {
                id: Some(1),
                profile_id: 0,
                operation_type: "file_read_all".to_string(),
                pattern_type: "subpath".to_string(),
                pattern_value: "{{PROJECT_PATH}}".to_string(),
                enabled: true,
                platform_support: Some(r#"["linux", "macos", "windows"]"#.to_string()),
                created_at: String::new(),
            });

            // System libraries (for language runtimes, etc.)
            rules.push(crate::sandbox::profile::SandboxRule {
                id: Some(2),
                profile_id: 0,
                operation_type: "file_read_all".to_string(),
                pattern_type: "subpath".to_string(),
                pattern_value: "/usr/lib".to_string(),
                enabled: true,
                platform_support: Some(r#"["linux", "macos"]"#.to_string()),
                created_at: String::new(),
            });

            rules.push(crate::sandbox::profile::SandboxRule {
                id: Some(3),
                profile_id: 0,
                operation_type: "file_read_all".to_string(),
                pattern_type: "subpath".to_string(),
                pattern_value: "/usr/local/lib".to_string(),
                enabled: true,
                platform_support: Some(r#"["linux", "macos"]"#.to_string()),
                created_at: String::new(),
            });

            rules.push(crate::sandbox::profile::SandboxRule {
                id: Some(4),
                profile_id: 0,
                operation_type: "file_read_all".to_string(),
                pattern_type: "subpath".to_string(),
                pattern_value: "/System/Library".to_string(),
                enabled: true,
                platform_support: Some(r#"["macos"]"#.to_string()),
                created_at: String::new(),
            });

            rules.push(crate::sandbox::profile::SandboxRule {
                id: Some(5),
                profile_id: 0,
                operation_type: "file_read_metadata".to_string(),
                pattern_type: "subpath".to_string(),
                pattern_value: "/".to_string(),
                enabled: true,
                platform_support: Some(r#"["macos"]"#.to_string()),
                created_at: String::new(),
            });
        }

        // Add network rules if enabled
        if agent.enable_network {
            rules.push(crate::sandbox::profile::SandboxRule {
                id: Some(6),
                profile_id: 0,
                operation_type: "network_outbound".to_string(),
                pattern_type: "all".to_string(),
                pattern_value: "".to_string(),
                enabled: true,
                platform_support: Some(r#"["linux", "macos"]"#.to_string()),
                created_at: String::new(),
            });
        }

        // Always add essential system paths (needed for executables to run)
        rules.push(crate::sandbox::profile::SandboxRule {
            id: Some(7),
            profile_id: 0,
            operation_type: "file_read_all".to_string(),
            pattern_type: "subpath".to_string(),
            pattern_value: "/usr/bin".to_string(),
            enabled: true,
            platform_support: Some(r#"["linux", "macos"]"#.to_string()),
            created_at: String::new(),
        });

        rules.push(crate::sandbox::profile::SandboxRule {
            id: Some(8),
            profile_id: 0,
            operation_type: "file_read_all".to_string(),
            pattern_type: "subpath".to_string(),
            pattern_value: "/opt/homebrew/bin".to_string(),
            enabled: true,
            platform_support: Some(r#"["macos"]"#.to_string()),
            created_at: String::new(),
        });

        rules.push(crate::sandbox::profile::SandboxRule {
            id: Some(9),
            profile_id: 0,
            operation_type: "file_read_all".to_string(),
            pattern_type: "subpath".to_string(),
            pattern_value: "/usr/local/bin".to_string(),
            enabled: true,
            platform_support: Some(r#"["linux", "macos"]"#.to_string()),
            created_at: String::new(),
        });

        rules.push(crate::sandbox::profile::SandboxRule {
            id: Some(10),
            profile_id: 0,
            operation_type: "file_read_all".to_string(),
            pattern_type: "subpath".to_string(),
            pattern_value: "/bin".to_string(),
            enabled: true,
            platform_support: Some(r#"["linux", "macos"]"#.to_string()),
            created_at: String::new(),
        });

        // System libraries (needed for executables to link)
        rules.push(crate::sandbox::profile::SandboxRule {
            id: Some(11),
            profile_id: 0,
            operation_type: "file_read_all".to_string(),
            pattern_type: "subpath".to_string(),
            pattern_value: "/usr/lib".to_string(),
            enabled: true,
            platform_support: Some(r#"["linux", "macos"]"#.to_string()),
            created_at: String::new(),
        });

        rules.push(crate::sandbox::profile::SandboxRule {
            id: Some(12),
            profile_id: 0,
            operation_type: "file_read_all".to_string(),
            pattern_type: "subpath".to_string(),
            pattern_value: "/System/Library".to_string(),
            enabled: true,
            platform_support: Some(r#"["macos"]"#.to_string()),
            created_at: String::new(),
        });

        // Always add system info reading (minimal requirement)
        rules.push(crate::sandbox::profile::SandboxRule {
            id: Some(13),
            profile_id: 0,
            operation_type: "system_info_read".to_string(),
            pattern_type: "all".to_string(),
            pattern_value: "".to_string(),
            enabled: true,
            platform_support: Some(r#"["linux", "macos"]"#.to_string()),
            created_at: String::new(),
        });

        Some(("Agent-specific".to_string(), rules))
    };

    // Build the command
    let mut cmd = if let Some((_profile_name, rules)) = sandbox_profile {
        info!("🧪 DEBUG: Testing Claude command first without sandbox...");
        // Quick test to see if Claude is accessible at all
        let claude_path = match find_claude_binary(&app) {
            Ok(path) => path,
            Err(e) => {
                error!("❌ Claude binary not found: {}", e);
                return Err(e);
            }
        };
        match std::process::Command::new(&claude_path)
            .arg("--version")
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    info!(
                        "✅ Claude command works: {}",
                        String::from_utf8_lossy(&output.stdout).trim()
                    );
                } else {
                    warn!("⚠️ Claude command failed with status: {}", output.status);
                    warn!("   stdout: {}", String::from_utf8_lossy(&output.stdout));
                    warn!("   stderr: {}", String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => {
                error!("❌ Claude command not found or not executable: {}", e);
                error!("   This could be why the agent is failing to start");
            }
        }

        // Test if Claude can actually start a session (this might reveal auth issues)
        info!("🧪 Testing Claude with exact same arguments as agent (without sandbox env vars)...");
        let mut test_cmd = std::process::Command::new(&claude_path);
        test_cmd
            .arg("-p")
            .arg(&task)
            .arg("--system-prompt")
            .arg(&agent.system_prompt)
            .arg("--model")
            .arg(&execution_model)
            .arg("--output-format")
            .arg("stream-json")
            .arg("--verbose")
            .arg("--dangerously-skip-permissions")
            .current_dir(&project_path);

        info!("🧪 Testing command: claude -p \"{}\" --system-prompt \"{}\" --model {} --output-format stream-json --verbose --dangerously-skip-permissions", 
              task, agent.system_prompt, execution_model);

        // Start the test process and give it 5 seconds to produce output
        match test_cmd.spawn() {
            Ok(mut child) => {
                // Wait for 5 seconds to see if it produces output
                let start = std::time::Instant::now();
                let mut output_received = false;

                while start.elapsed() < std::time::Duration::from_secs(5) {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            info!("🧪 Test process exited with status: {}", status);
                            output_received = true;
                            break;
                        }
                        Ok(None) => {
                            // Still running
                            std::thread::sleep(std::time::Duration::from_millis(100));
                        }
                        Err(e) => {
                            warn!("🧪 Error checking test process: {}", e);
                            break;
                        }
                    }
                }

                if !output_received {
                    warn!("🧪 Test process is still running after 5 seconds - this suggests Claude might be waiting for input");
                    // Kill the test process
                    let _ = child.kill();
                    let _ = child.wait();
                } else {
                    info!("🧪 Test process completed quickly - command seems to work");
                }
            }
            Err(e) => {
                error!("❌ Failed to spawn test Claude process: {}", e);
            }
        }

        info!("🧪 End of Claude test, proceeding with sandbox...");

        // Build the gaol profile using agent-specific permissions
        let project_path_buf = PathBuf::from(&project_path);

        match ProfileBuilder::new(project_path_buf.clone()) {
            Ok(builder) => {
                // Build agent-specific profile with permission filtering
                match builder.build_agent_profile(
                    rules,
                    agent.sandbox_enabled,
                    agent.enable_file_read,
                    agent.enable_file_write,
                    agent.enable_network,
                ) {
                    Ok(build_result) => {
                        // Create the enhanced sandbox executor
                        #[cfg(unix)]
                        let executor =
                            crate::sandbox::executor::SandboxExecutor::new_with_serialization(
                                build_result.profile,
                                project_path_buf.clone(),
                                build_result.serialized,
                            );

                        #[cfg(not(unix))]
                        let executor =
                            crate::sandbox::executor::SandboxExecutor::new_with_serialization(
                                (),
                                project_path_buf.clone(),
                                build_result.serialized,
                            );

                        // Prepare the sandboxed command
                        let args = vec![
                            "-p",
                            &task,
                            "--system-prompt",
                            &agent.system_prompt,
                            "--model",
                            &execution_model,
                            "--output-format",
                            "stream-json",
                            "--verbose",
                            "--dangerously-skip-permissions",
                        ];

                        let claude_path = match find_claude_binary(&app) {
                            Ok(path) => path,
                            Err(e) => {
                                error!("Failed to find claude binary: {}", e);
                                return Err(e);
                            }
                        };
                        executor.prepare_sandboxed_command(&claude_path, &args, &project_path_buf)
                    }
                    Err(e) => {
                        error!("Failed to build agent-specific sandbox profile: {}, falling back to non-sandboxed", e);
                        let claude_path = match find_claude_binary(&app) {
                            Ok(path) => path,
                            Err(e) => {
                                error!("Failed to find claude binary: {}", e);
                                return Err(e);
                            }
                        };
                        let mut cmd = create_command_with_env(&claude_path);
                        cmd.arg("-p")
                            .arg(&task)
                            .arg("--system-prompt")
                            .arg(&agent.system_prompt)
                            .arg("--model")
                            .arg(&execution_model)
                            .arg("--output-format")
                            .arg("stream-json")
                            .arg("--verbose")
                            .arg("--dangerously-skip-permissions")
                            .current_dir(&project_path)
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped());
                        cmd
                    }
                }
            }
            Err(e) => {
                error!(
                    "Failed to create ProfileBuilder: {}, falling back to non-sandboxed",
                    e
                );

                // Fall back to non-sandboxed command
                let claude_path = match find_claude_binary(&app) {
                    Ok(path) => path,
                    Err(e) => {
                        error!("Failed to find claude binary: {}", e);
                        return Err(e);
                    }
                };
                let mut cmd = create_command_with_env(&claude_path);
                cmd.arg("-p")
                    .arg(&task)
                    .arg("--system-prompt")
                    .arg(&agent.system_prompt)
                    .arg("--model")
                    .arg(&execution_model)
                    .arg("--output-format")
                    .arg("stream-json")
                    .arg("--verbose")
                    .arg("--dangerously-skip-permissions")
                    .current_dir(&project_path)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());
                cmd
            }
        }
    } else {
        // No sandbox or sandbox disabled, use regular command
        warn!(
            "🚨 Running agent '{}' WITHOUT SANDBOX - full system access!",
            agent.name
        );
        let claude_path = match find_claude_binary(&app) {
            Ok(path) => path,
            Err(e) => {
                error!("Failed to find claude binary: {}", e);
                return Err(e);
            }
        };
        let mut cmd = create_command_with_env(&claude_path);
        cmd.arg("-p")
            .arg(&task)
            .arg("--system-prompt")
            .arg(&agent.system_prompt)
            .arg("--model")
            .arg(&execution_model)
            .arg("--output-format")
            .arg("stream-json")
            .arg("--verbose")
            .arg("--dangerously-skip-permissions")
            .current_dir(&project_path)
            .stdin(Stdio::null()) // Don't pipe stdin - we have no input to send
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        cmd
    };

    // Spawn the process
    info!("🚀 Spawning Claude process...");
    let mut child = cmd.spawn().map_err(|e| {
        error!("❌ Failed to spawn Claude process: {}", e);
        format!("Failed to spawn Claude: {}", e)
    })?;

    info!("🔌 Using Stdio::null() for stdin - no input expected");

    // Get the PID and register the process
    let pid = child.id().unwrap_or(0);
    let now = chrono::Utc::now().to_rfc3339();
    info!("✅ Claude process spawned successfully with PID: {}", pid);

    // Update the database with PID and status
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE agent_runs SET status = 'running', pid = ?1, process_started_at = ?2 WHERE id = ?3",
            params![pid as i64, now, run_id],
        ).map_err(|e| e.to_string())?;
        info!("📝 Updated database with running status and PID");
    }

    // Get stdout and stderr
    let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to get stderr")?;
    info!("📡 Set up stdout/stderr readers");

    // Create readers
    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    // Shared state for collecting session ID and live output
    let session_id = std::sync::Arc::new(Mutex::new(String::new()));
    let live_output = std::sync::Arc::new(Mutex::new(String::new()));
    let start_time = std::time::Instant::now();

    // Spawn tasks to read stdout and stderr
    let app_handle = app.clone();
    let session_id_clone = session_id.clone();
    let live_output_clone = live_output.clone();
    let registry_clone = registry.0.clone();
    let first_output = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let first_output_clone = first_output.clone();

    let stdout_task = tokio::spawn(async move {
        info!("📖 Starting to read Claude stdout...");
        let mut lines = stdout_reader.lines();
        let mut line_count = 0;

        while let Ok(Some(line)) = lines.next_line().await {
            line_count += 1;

            // Log first output
            if !first_output_clone.load(std::sync::atomic::Ordering::Relaxed) {
                info!(
                    "🎉 First output received from Claude process! Line: {}",
                    line
                );
                first_output_clone.store(true, std::sync::atomic::Ordering::Relaxed);
            }

            if line_count <= 5 {
                info!("stdout[{}]: {}", line_count, line);
            } else {
                debug!("stdout[{}]: {}", line_count, line);
            }

            // Store live output in both local buffer and registry
            if let Ok(mut output) = live_output_clone.lock() {
                output.push_str(&line);
                output.push('\n');
            }

            // Also store in process registry for cross-session access
            let _ = registry_clone.append_live_output(run_id, &line);

            // Extract session ID from JSONL output
            if let Ok(json) = serde_json::from_str::<JsonValue>(&line) {
                if let Some(sid) = json.get("sessionId").and_then(|s| s.as_str()) {
                    if let Ok(mut current_session_id) = session_id_clone.lock() {
                        if current_session_id.is_empty() {
                            *current_session_id = sid.to_string();
                            info!("🔑 Extracted session ID: {}", sid);
                        }
                    }
                }
            }

            // Emit the line to the frontend with run_id for isolation
            let _ = app_handle.emit(&format!("agent-output:{}", run_id), &line);
        }

        info!(
            "📖 Finished reading Claude stdout. Total lines: {}",
            line_count
        );
    });

    let app_handle_stderr = app.clone();
    let first_error = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let first_error_clone = first_error.clone();

    let stderr_task = tokio::spawn(async move {
        info!("📖 Starting to read Claude stderr...");
        let mut lines = stderr_reader.lines();
        let mut error_count = 0;

        while let Ok(Some(line)) = lines.next_line().await {
            error_count += 1;

            // Log first error
            if !first_error_clone.load(std::sync::atomic::Ordering::Relaxed) {
                warn!("⚠️ First error output from Claude process! Line: {}", line);
                first_error_clone.store(true, std::sync::atomic::Ordering::Relaxed);
            }

            error!("stderr[{}]: {}", error_count, line);
            // Emit error lines to the frontend with run_id for isolation
            let _ = app_handle_stderr.emit(&format!("agent-error:{}", run_id), &line);
            // Also emit to the generic event for backward compatibility
            let _ = app_handle_stderr.emit("agent-error", &line);
        }

        if error_count > 0 {
            warn!(
                "📖 Finished reading Claude stderr. Total error lines: {}",
                error_count
            );
        } else {
            info!("📖 Finished reading Claude stderr. No errors.");
        }
    });

    // Register the process in the registry for live output tracking (after stdout/stderr setup)
    registry
        .0
        .register_process(
            run_id,
            agent_id,
            agent.name.clone(),
            pid,
            project_path.clone(),
            task.clone(),
            execution_model.clone(),
            child,
        )
        .map_err(|e| format!("Failed to register process: {}", e))?;
    info!("📋 Registered process in registry");

    // Create variables we need for the spawned task
    let app_dir = app
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir");
    let db_path = app_dir.join("agents.db");

    // Monitor process status and wait for completion
    tokio::spawn(async move {
        info!("🕐 Starting process monitoring...");

        // Wait for first output with timeout
        for i in 0..300 {
            // 30 seconds (300 * 100ms)
            if first_output.load(std::sync::atomic::Ordering::Relaxed) {
                info!(
                    "✅ Output detected after {}ms, continuing normal execution",
                    i * 100
                );
                break;
            }

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            // Log progress every 5 seconds
            if i > 0 && i % 50 == 0 {
                info!(
                    "⏳ Still waiting for Claude output... ({}s elapsed)",
                    i / 10
                );
            }
        }

        // Check if we timed out
        if !first_output.load(std::sync::atomic::Ordering::Relaxed) {
            warn!("⏰ TIMEOUT: No output from Claude process after 30 seconds");
            warn!("💡 This usually means:");
            warn!("   1. Claude process is waiting for user input");
            warn!("   2. Sandbox permissions are too restrictive");
            warn!("   3. Claude failed to initialize but didn't report an error");
            warn!("   4. Network connectivity issues");
            warn!("   5. Authentication issues (API key not found/invalid)");

            // Process timed out - kill it via PID
            warn!(
                "🔍 Process likely stuck waiting for input, attempting to kill PID: {}",
                pid
            );
            let kill_result = std::process::Command::new("kill")
                .arg("-TERM")
                .arg(pid.to_string())
                .output();

            match kill_result {
                Ok(output) if output.status.success() => {
                    warn!("🔍 Successfully sent TERM signal to process");
                }
                Ok(_) => {
                    warn!("🔍 Failed to kill process with TERM, trying KILL");
                    let _ = std::process::Command::new("kill")
                        .arg("-KILL")
                        .arg(pid.to_string())
                        .output();
                }
                Err(e) => {
                    warn!("🔍 Error killing process: {}", e);
                }
            }

            // Update database
            if let Ok(conn) = Connection::open(&db_path) {
                let _ = conn.execute(
                    "UPDATE agent_runs SET status = 'failed', completed_at = CURRENT_TIMESTAMP WHERE id = ?1",
                    params![run_id],
                );
            }

            let _ = app.emit(&format!("agent-complete:{}", run_id), false);
            return;
        }

        // Wait for reading tasks to complete
        info!("⏳ Waiting for stdout/stderr reading to complete...");
        let _ = stdout_task.await;
        let _ = stderr_task.await;

        let duration_ms = start_time.elapsed().as_millis() as i64;
        info!("⏱️ Process execution took {} ms", duration_ms);

        // Get the session ID that was extracted
        let extracted_session_id = if let Ok(sid) = session_id.lock() {
            sid.clone()
        } else {
            String::new()
        };

        // Wait for process completion and update status
        info!("✅ Claude process execution monitoring complete");

        // Check the output for usage limit errors
        let final_output = if let Ok(output) = live_output.lock() {
            output.clone()
        } else {
            String::new()
        };

        // Check for usage limit error
        let is_usage_limit_error = if let Some(reset_time) = parse_usage_limit_error(&final_output) {
            info!("🚫 Detected usage limit error. Reset time: {}", reset_time);
            
            // Update the run record with usage limit status - open a new connection
            if let Ok(conn) = Connection::open(&db_path) {
                // First check if auto-resume is enabled
                let auto_resume = conn.query_row(
                    "SELECT auto_resume_enabled FROM agent_runs WHERE id = ?1",
                    params![run_id],
                    |row| row.get::<_, bool>(0)
                ).unwrap_or(false);
                
                if auto_resume {
                    // Schedule the resume
                    let _ = conn.execute(
                        "UPDATE agent_runs SET session_id = ?1, status = 'paused_usage_limit', usage_limit_reset_time = ?2 WHERE id = ?3",
                        params![extracted_session_id, reset_time, run_id],
                    );
                    
                    // Create a scheduled run for the reset time
                    let _ = conn.execute(
                        "INSERT INTO agent_runs (agent_id, agent_name, agent_icon, task, model, project_path, session_id, status, scheduled_start_time, auto_resume_enabled, parent_run_id, resume_count) 
                         SELECT agent_id, agent_name, agent_icon, task, model, project_path, '', 'scheduled', ?1, 1, ?2, resume_count + 1 
                         FROM agent_runs WHERE id = ?2",
                        params![reset_time, run_id],
                    );
                } else {
                    // Just mark as paused
                    let _ = conn.execute(
                        "UPDATE agent_runs SET session_id = ?1, status = 'paused_usage_limit', usage_limit_reset_time = ?2, completed_at = CURRENT_TIMESTAMP WHERE id = ?3",
                        params![extracted_session_id, reset_time, run_id],
                    );
                }
            }
            true
        } else {
            false
        };

        if !is_usage_limit_error {
            // Update the run record with session ID and mark as completed - open a new connection
            if let Ok(conn) = Connection::open(&db_path) {
                let _ = conn.execute(
                    "UPDATE agent_runs SET session_id = ?1, status = 'completed', completed_at = CURRENT_TIMESTAMP WHERE id = ?2",
                    params![extracted_session_id, run_id],
                );
            }
        }

        // Cleanup will be handled by the cleanup_finished_processes function

        let _ = app.emit(&format!("agent-complete:{}", run_id), true);
    });

    Ok(run_id)
}

/// List all agent sessions (including completed, failed, etc.)
#[tauri::command]
pub async fn list_running_sessions(db: State<'_, AgentDb>) -> Result<Vec<AgentRun>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = conn
        .prepare(
            "SELECT id, agent_id, agent_name, agent_icon, task, model, project_path, session_id, status, pid, process_started_at, scheduled_start_time, created_at, completed_at, usage_limit_reset_time, auto_resume_enabled, resume_count, parent_run_id 
             FROM agent_runs 
             WHERE status = 'running' 
             ORDER BY created_at DESC"
        )
        .map_err(|e| e.to_string())?;

    let runs = stmt
        .query_map([], |row| {
            Ok(AgentRun {
                id: Some(row.get(0)?),
                agent_id: row.get(1)?,
                agent_name: row.get(2)?,
                agent_icon: row.get(3)?,
                task: row.get(4)?,
                model: row.get(5)?,
                project_path: row.get(6)?,
                session_id: row.get(7)?,
                status: row.get(8)?,
                pid: row.get(9)?,
                process_started_at: row.get(10)?,
                scheduled_start_time: row.get(11)?,
                created_at: row.get(12)?,
                completed_at: row.get(13)?,
                usage_limit_reset_time: row.get(14)?,
                auto_resume_enabled: row.get(15)?,
                resume_count: row.get(16)?,
                parent_run_id: row.get(17)?,
            })
        })
        .map_err(|e| e.to_string())?;

    runs.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

/// List all running sessions with metrics
#[tauri::command]
pub async fn list_running_sessions_with_metrics(db: State<'_, AgentDb>) -> Result<Vec<AgentRunWithMetrics>, String> {
    let runs = list_running_sessions(db).await?;
    
    let mut runs_with_metrics = Vec::new();
    for run in runs {
        let with_metrics = get_agent_run_with_metrics(run).await;
        runs_with_metrics.push(with_metrics);
    }
    
    Ok(runs_with_metrics)
}

/// Kill an agent session
#[tauri::command]
pub async fn kill_agent_session(
    db: State<'_, AgentDb>,
    registry: State<'_, crate::process::ProcessRegistryState>,
    run_id: i64,
) -> Result<(), String> {
    info!("Killing agent session {}", run_id);

    // Kill via process registry
    let result = registry.0.kill_process(run_id).await;
    
    if let Err(e) = result {
        // If registry kill fails, try direct kill via PID from database
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        
        let pid: Option<u32> = conn.query_row(
            "SELECT pid FROM agent_runs WHERE id = ?1",
            params![run_id],
            |row| row.get(0)
        ).ok();
        
        if let Some(pid) = pid {
            info!("Registry kill failed, attempting direct kill of PID {}", pid);
            
            // Try to kill the process directly
            #[cfg(unix)]
            {
                use std::process::Command;
                let _ = Command::new("kill").arg("-TERM").arg(pid.to_string()).output();
                // Give it a moment to terminate
                std::thread::sleep(std::time::Duration::from_millis(500));
                // Force kill if still running
                let _ = Command::new("kill").arg("-KILL").arg(pid.to_string()).output();
            }
            
            #[cfg(windows)]
            {
                use std::process::Command;
                let _ = Command::new("taskkill").arg("/F").arg("/PID").arg(pid.to_string()).output();
            }
        } else {
            return Err(format!("Failed to kill process: {}", e));
        }
    }

    // Update database status
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE agent_runs SET status = 'cancelled', completed_at = CURRENT_TIMESTAMP WHERE id = ?1",
        params![run_id],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

/// Get session status
#[tauri::command]
pub async fn get_session_status(
    db: State<'_, AgentDb>,
    run_id: i64,
) -> Result<String, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    
    let status: String = conn.query_row(
        "SELECT status FROM agent_runs WHERE id = ?1",
        params![run_id],
        |row| row.get(0)
    ).map_err(|e| format!("Failed to get session status: {}", e))?;
    
    Ok(status)
}

/// Cleanup finished processes
#[tauri::command]
pub async fn cleanup_finished_processes(db: State<'_, AgentDb>) -> Result<Vec<i64>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    
    // Find all runs marked as 'running' that have PIDs
    let mut stmt = conn
        .prepare("SELECT id, pid FROM agent_runs WHERE status = 'running' AND pid IS NOT NULL")
        .map_err(|e| e.to_string())?;

    let runs: Vec<(i64, u32)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,  // id
                row.get::<_, u32>(1)?,  // pid
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut cleaned_ids = Vec::new();

    for (run_id, pid) in runs {
        // Check if the process is still running
        let is_running = {
            #[cfg(unix)]
            {
                // On Unix, we can use kill with signal 0 to check if process exists
                use std::process::Command;
                Command::new("kill")
                    .arg("-0")
                    .arg(pid.to_string())
                    .output()
                    .map(|output| output.status.success())
                    .unwrap_or(false)
            }
            #[cfg(windows)]
            {
                // On Windows, use tasklist to check if process exists
                use std::process::Command;
                Command::new("tasklist")
                    .arg("/FI")
                    .arg(format!("PID eq {}", pid))
                    .output()
                    .map(|output| {
                        let output_str = String::from_utf8_lossy(&output.stdout);
                        output_str.contains(&pid.to_string())
                    })
                    .unwrap_or(false)
            }
        };

        if !is_running {
            // Process is not running, update status
            conn.execute(
                "UPDATE agent_runs SET status = 'completed', completed_at = CURRENT_TIMESTAMP WHERE id = ?1",
                params![run_id],
            ).map_err(|e| e.to_string())?;
            
            cleaned_ids.push(run_id);
            info!("Cleaned up finished process: run_id={}, pid={}", run_id, pid);
        }
    }

    Ok(cleaned_ids)
}

/// Get live session output from process registry
#[tauri::command]
pub async fn get_live_session_output(
    registry: State<'_, crate::process::ProcessRegistryState>,
    run_id: i64,
) -> Result<String, String> {
    registry.0.get_live_output(run_id)
}

/// Get session output (from JSONL file for completed runs, or live output for running ones)
#[tauri::command]
pub async fn get_session_output(
    db: State<'_, AgentDb>,
    registry: State<'_, crate::process::ProcessRegistryState>,
    run_id: i64,
) -> Result<String, String> {
    // First get the run details
    let run = get_agent_run(db, run_id).await?;
    
    // If the run is still running, get live output from registry
    if run.status == "running" {
        info!("Getting live output for running session {}", run_id);
        return get_live_session_output(registry, run_id).await;
    }
    
    // For completed runs, check if we have a session_id
    if run.session_id.is_empty() {
        // No session ID means the process never started properly
        // Try to get any output from the registry (might have error messages)
        if let Ok(output) = registry.0.get_live_output(run_id) {
            if !output.is_empty() {
                return Ok(output);
            }
        }
        return Err("No output available - process may have failed to start".to_string());
    }
    
    // Read from JSONL file
    info!("Reading JSONL file for completed session {} with session_id {}", run_id, run.session_id);
    read_session_jsonl(&run.session_id, &run.project_path).await
}

/// Stream session output with real-time updates
#[tauri::command]
pub async fn stream_session_output(
    app: AppHandle,
    run_id: i64,
    db: State<'_, AgentDb>,
) -> Result<(), String> {
    info!("Starting to stream output for session {}", run_id);
    
    // Get the run details
    let run = get_agent_run(db.clone(), run_id).await?;
    
    if run.session_id.is_empty() {
        return Err("Session ID not available yet".to_string());
    }
    
    // Build the path to the JSONL file
    let claude_dir = dirs::home_dir()
        .ok_or("Failed to get home directory")?
        .join(".claude")
        .join("projects");
    
    let encoded_project = run.project_path.replace('/', "-");
    let project_dir = claude_dir.join(&encoded_project);
    let session_file = project_dir.join(format!("{}.jsonl", run.session_id));
    
    info!("Streaming from file: {}", session_file.display());
    
    // Get the app data directory to use for checking run status
    let app_dir = app
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir");
    let db_path = app_dir.join("agents.db");
    
    // Spawn a task to read the file and emit updates
    tokio::spawn(async move {
        let mut last_size = 0u64;
        let mut consecutive_errors = 0;
        
        loop {
            match tokio::fs::metadata(&session_file).await {
                Ok(metadata) => {
                    let current_size = metadata.len();
                    
                    if current_size > last_size {
                        // File has grown, read the new content
                        match tokio::fs::read_to_string(&session_file).await {
                            Ok(content) => {
                                // Split into lines and skip the ones we've already sent
                                let lines: Vec<&str> = content.lines().collect();
                                let previous_line_count = if last_size == 0 { 0 } else {
                                    // Estimate line count from size (this is approximate)
                                    content[..last_size as usize].lines().count()
                                };
                                
                                // Send new lines
                                for line in lines.iter().skip(previous_line_count) {
                                    let _ = app.emit(&format!("session-output:{}", run_id), line);
                                }
                                
                                last_size = current_size;
                                consecutive_errors = 0;
                            }
                            Err(e) => {
                                error!("Failed to read session file: {}", e);
                                consecutive_errors += 1;
                            }
                        }
                    }
                }
                Err(e) => {
                    if consecutive_errors == 0 {
                        info!("Session file not found yet, waiting... {}", e);
                    }
                    consecutive_errors += 1;
                }
            }
            
            // Stop streaming if we've had too many errors
            if consecutive_errors > 30 {
                error!("Too many consecutive errors, stopping stream for session {}", run_id);
                break;
            }
            
            // Check if the run is still active by opening a new connection
            if let Ok(conn) = Connection::open(&db_path) {
                if let Ok(status) = conn.query_row(
                    "SELECT status FROM agent_runs WHERE id = ?1",
                    params![run_id],
                    |row| row.get::<_, String>(0)
                ) {
                    if !["running", "pending"].contains(&status.as_str()) {
                        info!("Session {} is no longer active (status: {}), stopping stream", run_id, status);
                        break;
                    }
                }
            }
            
            // Wait before checking again
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
        
        info!("Stopped streaming output for session {}", run_id);
    });
    
    Ok(())
}

/// Debug helper to check JSONL file status
#[tauri::command]
pub async fn debug_jsonl_status(
    db: State<'_, AgentDb>,
    run_id: i64,
) -> Result<serde_json::Value, String> {
    let run = get_agent_run(db, run_id).await?;
    
    let claude_dir = dirs::home_dir()
        .ok_or("Failed to get home directory")?
        .join(".claude")
        .join("projects");
    
    let encoded_project = run.project_path.replace('/', "-");
    let project_dir = claude_dir.join(&encoded_project);
    let session_file = project_dir.join(format!("{}.jsonl", run.session_id));
    
    let mut result = serde_json::json!({
        "run_id": run_id,
        "session_id": run.session_id,
        "parent_run_id": run.parent_run_id,
        "status": run.status,
        "jsonl_path": session_file.display().to_string(),
        "jsonl_exists": session_file.exists(),
    });
    
    if session_file.exists() {
        if let Ok(metadata) = std::fs::metadata(&session_file) {
            result["file_size"] = serde_json::json!(metadata.len());
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = modified.elapsed() {
                    result["last_modified_seconds_ago"] = serde_json::json!(duration.as_secs());
                }
            }
        }
        
        if let Ok(content) = tokio::fs::read_to_string(&session_file).await {
            let lines: Vec<&str> = content.lines().collect();
            result["line_count"] = serde_json::json!(lines.len());
            
            // Check for usage limit message
            let has_usage_limit = lines.iter().any(|line| {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                    if let Some(msg) = json.get("messageText").and_then(|m| m.as_str()) {
                        return msg.contains("I've reached my usage limit");
                    }
                }
                false
            });
            result["has_usage_limit_message"] = serde_json::json!(has_usage_limit);
            
            // Get last few lines
            if lines.len() > 0 {
                let last_lines: Vec<&str> = lines.iter().rev().take(5).rev().copied().collect();
                result["last_5_lines"] = serde_json::json!(last_lines);
            }
        }
    }
    
    Ok(result)
}

/// Get agent run with real-time metrics
#[tauri::command]
pub async fn get_agent_run_with_real_time_metrics(
    db: State<'_, AgentDb>,
    run_id: i64,
) -> Result<AgentRunWithMetrics, String> {
    let run = get_agent_run(db, run_id).await?;
    Ok(get_agent_run_with_metrics(run).await)
}

/// Resume a paused agent that hit usage limits
#[tauri::command]
pub async fn resume_agent(
    app: AppHandle,
    run_id: i64,
    db: State<'_, AgentDb>,
    registry: State<'_, crate::process::ProcessRegistryState>,
) -> Result<i64, String> {
    info!("Resuming agent from run {}", run_id);
    
    // Get the original run details
    let original_run = get_agent_run(db.clone(), run_id).await?;
    
    // Verify it's in a resumable state
    if original_run.status != "paused_usage_limit" {
        return Err(format!("Cannot resume run with status: {}", original_run.status));
    }
    
    // Create a new run that continues from the original
    let new_run_id = execute_agent(
        app,
        original_run.agent_id,
        original_run.project_path.clone(),
        format!("--continue {}", original_run.session_id), // Special task format for continuation
        Some(original_run.model.clone()),
        Some(original_run.auto_resume_enabled),
        db.clone(),
        registry,
    ).await?;
    
    // Update the new run to track its parent
    {
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE agent_runs SET parent_run_id = ?1, resume_count = ?2 WHERE id = ?3",
            params![run_id, original_run.resume_count + 1, new_run_id],
        ).map_err(|e| e.to_string())?;
    }
    
    Ok(new_run_id)
}