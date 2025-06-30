use super::*;
use super::error::AgentError;
use super::helpers;
use super::repository::*;
use crate::process_registry::ProcessRegistry;
use crate::sandbox::profile::ProfileBuilder;
use log::{debug, error, info, warn};
use rusqlite::{params, Connection};
use serde_json::Value as JsonValue;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};

/// Execute an agent (internal function used by commands and service)
pub async fn execute_agent(
    app: AppHandle,
    conn: &Connection,
    registry: Arc<Mutex<ProcessRegistry>>,
    agent_id: i64,
    task: Option<String>,
    project_path: Option<String>,
    session_id: Option<String>,
    run_id: Option<i64>,
    resume_from_line: Option<i64>,
) -> Result<AgentRun, AgentError> {
    info!("Executing agent {} with task: {:?}", agent_id, task);

    // Get the agent from database
    let repo = SqliteAgentRepository::new(conn);
    let agent = repo.find_agent_by_id(agent_id)?;
    let execution_model = agent.model.clone();

    // Use provided values or defaults
    let task = task.unwrap_or_else(|| agent.default_task.clone().unwrap_or_default());
    let project_path = project_path.unwrap_or_else(|| {
        app.path()
            .home_dir()
            .unwrap()
            .to_string_lossy()
            .to_string()
    });

    // Create or use existing run
    let run = if let Some(run_id) = run_id {
        repo.find_run_by_id(run_id)?
    } else {
        let new_run = NewAgentRun {
            agent_id,
            agent_name: agent.name.clone(),
            agent_icon: agent.icon.clone(),
            task: task.clone(),
            model: execution_model.clone(),
            project_path: project_path.clone(),
            status: Some("pending"),
            scheduled_start_time: None,
            parent_run_id: None,
        };
        repo.create_run(new_run)?
    };

    let run_id = run.id.ok_or_else(|| AgentError::Other("Run ID not found".to_string()))?;
    let session_id = session_id.unwrap_or_else(|| run.session_id.clone());

    // Get app data directory for output storage
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AgentError::Other(e.to_string()))?;

    // Build the command
    let claude_path = find_claude_binary(&app).map_err(|e| AgentError::BinaryNotFound(e))?;
    
    // Create base command arguments
    let mut args = vec![
        "-p".to_string(),
        task.clone(),
        "--system-prompt".to_string(),
        agent.system_prompt.clone(),
        "--model".to_string(),
        execution_model.clone(),
        "--output-format".to_string(),
        "stream-json".to_string(),
        "--verbose".to_string(),
    ];

    // Add resume arguments if resuming
    if let Some(from_line) = resume_from_line {
        args.push("--resume-from".to_string());
        args.push(from_line.to_string());
    }

    // Create sandbox rules based on agent-specific permissions
    let mut cmd = if agent.sandbox_enabled {
        info!(
            "🔒 Agent '{}': Sandbox enabled | File Read: {} | File Write: {} | Network: {}",
            agent.name, agent.enable_file_read, agent.enable_file_write, agent.enable_network
        );

        // Build sandbox rules dynamically
        let rules = helpers::build_sandbox_rules(&agent, &project_path);

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

                        // Add skip permissions flag for sandboxed execution
                        args.push("--dangerously-skip-permissions".to_string());

                        // Prepare the sandboxed command
                        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                        executor.prepare_sandboxed_command(&claude_path, &args_refs, &project_path_buf)
                    }
                    Err(e) => {
                        error!("Failed to build agent-specific sandbox profile: {}, falling back to non-sandboxed", e);
                        let mut cmd = helpers::create_command_with_env(&claude_path);
                        for arg in &args {
                            cmd.arg(arg);
                        }
                        cmd.current_dir(&project_path);
                        cmd
                    }
                }
            }
            Err(e) => {
                error!("Failed to initialize sandbox profile builder: {}, falling back to non-sandboxed", e);
                let mut cmd = helpers::create_command_with_env(&claude_path);
                for arg in &args {
                    cmd.arg(arg);
                }
                cmd.current_dir(&project_path);
                cmd
            }
        }
    } else {
        info!("⚠️  Agent '{}': Running without sandbox", agent.name);
        let mut cmd = helpers::create_command_with_env(&claude_path);
        for arg in &args {
            cmd.arg(arg);
        }
        cmd.current_dir(&project_path);
        cmd
    };

    // Set up the process for streaming
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    // Spawn the process
    let mut child = cmd.spawn().map_err(|e| {
        error!("Failed to spawn claude process: {}", e);
        AgentError::Process(format!("Failed to spawn claude process: {}", e))
    })?;

    let pid = child.id();
    let process_started_at = helpers::now_iso8601();

    info!("Started agent process with PID: {:?}", pid);

    // Update run status to running
    repo.update_run_status(run_id, "running", pid, Some(process_started_at.clone()))?;

    // Register the process
    {
        let mut reg = registry
            .lock()
            .map_err(|e| AgentError::Lock(e.to_string()))?;
        reg.register_process(session_id.clone(), child);
    }

    // Create channels for output streaming
    let app_handle = app.clone();
    let app_data_dir_clone = app_data_dir.clone();
    let session_id_clone = session_id.clone();
    // Store run_id for use in the spawned task
    let run_id_clone = run_id;

    // Spawn task to stream output
    tokio::spawn(async move {
        let mut line_number = resume_from_line.unwrap_or(0);
        let mut output_buffer = Vec::new();
        let mut has_usage_limit_error = false;
        let mut reset_time = None;

        // Get child process from registry
        let mut child = {
            let mut reg = registry.lock().unwrap();
            reg.take_child(&session_id_clone)
        };

        if let Some(mut child) = child {
            // Get stdout and stderr
            let stdout = child.stdout.take();
            let stderr = child.stderr.take();

            // Spawn tasks to read both streams
            let (stdout_lines, stderr_lines) = tokio::join!(
                async {
                    if let Some(stdout) = stdout {
                        let reader = BufReader::new(stdout);
                        let mut lines = reader.lines();
                        let mut stdout_lines = Vec::new();
                        while let Ok(Some(line)) = lines.next_line().await {
                            stdout_lines.push(line);
                        }
                        stdout_lines
                    } else {
                        Vec::new()
                    }
                },
                async {
                    if let Some(stderr) = stderr {
                        let reader = BufReader::new(stderr);
                        let mut lines = reader.lines();
                        let mut stderr_lines = Vec::new();
                        while let Ok(Some(line)) = lines.next_line().await {
                            stderr_lines.push(line);
                        }
                        stderr_lines
                    } else {
                        Vec::new()
                    }
                }
            );

            // Process stdout lines
            for line in stdout_lines {
                line_number += 1;
                output_buffer.push(line.clone());

                // Emit event
                app_handle
                    .emit(&format!("agent-output-{}", session_id_clone), &line)
                    .ok();

                // We'll store in database later via a different mechanism
                // For now, just collect the output

                // Persist to file
                persist_agent_output_line(&app_data_dir_clone, run_id_clone, line_number, &line)
                    .await
                    .ok();

                // Check for usage limit in output
                if let Ok(json) = serde_json::from_str::<JsonValue>(&line) {
                    if let Some(error) = json.get("error").and_then(|e| e.as_str()) {
                        if error.contains("Usage limit exceeded") || error.contains("rate limit") {
                            has_usage_limit_error = true;
                            reset_time = helpers::parse_usage_limit_error(error);
                        }
                    }
                }
            }

            // Process stderr lines for errors
            for line in stderr_lines {
                warn!("Agent stderr: {}", line);
                
                // Check for usage limit in stderr
                if line.contains("Usage limit exceeded") || line.contains("rate limit") {
                    has_usage_limit_error = true;
                    reset_time = reset_time.or_else(|| helpers::parse_usage_limit_error(&line));
                }
            }

            // Wait for process to complete
            let exit_status = child.wait().await;

            // Update process registry
            {
                let mut reg = registry.lock().unwrap();
                reg.unregister_process(&session_id_clone);
            }

            // Determine final status
            let final_status = if has_usage_limit_error {
                "paused_usage_limit"
            } else if exit_status.is_ok() {
                "completed"
            } else {
                "failed"
            };

            // We'll update the database status later
            // For now, just emit the completion event

            // Emit completion event
            app_handle
                .emit(
                    &format!("agent-complete-{}", session_id_clone),
                    final_status,
                )
                .ok();
        }
    });

    // Return the updated run
    repo.find_run_by_id(run_id)
}

/// Stream session output from database
pub async fn stream_session_output(
    app: AppHandle,
    conn: &Connection,
    session_id: String,
) -> Result<(), AgentError> {
    let repo = SqliteAgentRepository::new(conn);
    
    // Find the run by session ID
    if let Some(run) = repo.find_run_by_session_id(&session_id)? {
        if let Some(run_id) = run.id {
            // Get all output lines
            let output = repo.get_jsonl_output(run_id)?;
            
            // Emit each line
            for line in output.lines() {
                app.emit(&format!("agent-output-{}", session_id), line)
                    .map_err(|e| AgentError::Other(e.to_string()))?;
            }
            
            // Emit completion event
            app.emit(&format!("agent-complete-{}", session_id), &run.status)
                .map_err(|e| AgentError::Other(e.to_string()))?;
        }
    }
    
    Ok(())
}

/// Helper function to persist agent output to file
async fn persist_agent_output_line(
    app_data_dir: &std::path::Path,
    run_id: i64,
    line_number: i64,
    content: &str,
) -> Result<(), AgentError> {
    let output_dir = app_data_dir.join("agent_outputs");
    tokio::fs::create_dir_all(&output_dir).await?;
    
    let output_file = output_dir.join(format!("{}.jsonl", run_id));
    
    // Append to the file
    use tokio::fs::OpenOptions;
    use tokio::io::AsyncWriteExt;
    
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&output_file)
        .await?;
    
    file.write_all(content.as_bytes()).await?;
    file.write_all(b"\n").await?;
    
    Ok(())
}

/// Finds the full path to the claude binary
fn find_claude_binary(app_handle: &AppHandle) -> Result<String, String> {
    crate::claude_binary::find_claude_binary(app_handle)
}