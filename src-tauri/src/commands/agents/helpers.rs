use super::constants::*;
use super::error::AgentError;
use super::types::*;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;

/// Calculate total cost from token counts based on model
pub fn calculate_cost(
    model: &str,
    input_tokens: i64,
    output_tokens: i64,
    cache_creation_tokens: i64,
    cache_read_tokens: i64,
) -> f64 {
    let model_type = ModelType::from_str(model);
    let (input_price, output_price, cache_write_price, cache_read_price) = model_type.get_pricing();

    let input_cost = (input_tokens as f64 / TOKENS_PER_MILLION) * input_price;
    let output_cost = (output_tokens as f64 / TOKENS_PER_MILLION) * output_price;
    let cache_write_cost = (cache_creation_tokens as f64 / TOKENS_PER_MILLION) * cache_write_price;
    let cache_read_cost = (cache_read_tokens as f64 / TOKENS_PER_MILLION) * cache_read_price;

    input_cost + output_cost + cache_write_cost + cache_read_cost
}

/// Parse ISO 8601 datetime string to DateTime<Utc>
pub fn parse_datetime(datetime_str: &str) -> Result<DateTime<Utc>, AgentError> {
    DateTime::parse_from_rfc3339(datetime_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| AgentError::Parse(format!("Invalid datetime format: {}", e)))
}

/// Get current UTC time as ISO 8601 string
pub fn now_iso8601() -> String {
    Utc::now().to_rfc3339()
}

/// Extract session ID from JSONL message
pub fn extract_session_id(json: &JsonValue) -> Option<String> {
    json.get("session_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Extract PID from JSONL message
pub fn extract_pid(json: &JsonValue) -> Option<u32> {
    json.get("pid").and_then(|v| v.as_u64()).map(|p| p as u32)
}

/// Check if a process is still running
#[cfg(unix)]
pub fn is_process_running(pid: u32) -> bool {
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;

    // Send signal 0 to check if process exists
    match kill(Pid::from_raw(pid as i32), Signal::SIGCONT) {
        Ok(()) => true,
        Err(_) => false,
    }
}

#[cfg(windows)]
pub fn is_process_running(pid: u32) -> bool {
    use std::process::Command;

    // Use tasklist command on Windows
    let output = Command::new("tasklist")
        .args(&["/FI", &format!("PID eq {}", pid)])
        .output();

    match output {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            output_str.contains(&pid.to_string())
        }
        Err(_) => false,
    }
}

/// Kill a process by PID
#[cfg(unix)]
pub fn kill_process(pid: u32) -> Result<(), AgentError> {
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;
    use std::thread;
    use std::time::Duration;

    let nix_pid = Pid::from_raw(pid as i32);

    // First try SIGTERM for graceful shutdown
    kill(nix_pid, Signal::SIGTERM).map_err(|e| AgentError::Process(e.to_string()))?;

    // Wait a bit for process to terminate
    thread::sleep(Duration::from_millis(1000));

    // Check if process is still running
    if is_process_running(pid) {
        // Force kill with SIGKILL
        kill(nix_pid, Signal::SIGKILL).map_err(|e| AgentError::Process(e.to_string()))?;
    }

    Ok(())
}

#[cfg(windows)]
pub fn kill_process(pid: u32) -> Result<(), AgentError> {
    use std::process::Command;

    Command::new("taskkill")
        .args(&["/F", "/PID", &pid.to_string()])
        .output()
        .map_err(|e| AgentError::Process(e.to_string()))?;

    Ok(())
}

/// Build agent sandbox rules based on permissions
pub fn build_sandbox_rules(agent: &Agent, _project_path: &str) -> Vec<crate::sandbox::profile::SandboxRule> {
    let mut rules = Vec::new();
    let mut rule_id = 1;

    // Add file read rules if enabled
    if agent.enable_file_read {
        // Project directory access
        rules.push(crate::sandbox::profile::SandboxRule {
            id: Some(rule_id),
            profile_id: 0,
            operation_type: "file_read_all".to_string(),
            pattern_type: "subpath".to_string(),
            pattern_value: "{{PROJECT_PATH}}".to_string(),
            enabled: true,
            platform_support: Some(r#"["linux", "macos", "windows"]"#.to_string()),
            created_at: String::new(),
        });
        rule_id += 1;

        // System libraries
        for (path, platforms) in &[
            ("/usr/lib", r#"["linux", "macos"]"#),
            ("/usr/local/lib", r#"["linux", "macos"]"#),
            ("/System/Library", r#"["macos"]"#),
        ] {
            rules.push(crate::sandbox::profile::SandboxRule {
                id: Some(rule_id),
                profile_id: 0,
                operation_type: "file_read_all".to_string(),
                pattern_type: "subpath".to_string(),
                pattern_value: path.to_string(),
                enabled: true,
                platform_support: Some(platforms.to_string()),
                created_at: String::new(),
            });
            rule_id += 1;
        }

        // File metadata access
        rules.push(crate::sandbox::profile::SandboxRule {
            id: Some(rule_id),
            profile_id: 0,
            operation_type: "file_read_metadata".to_string(),
            pattern_type: "subpath".to_string(),
            pattern_value: "/".to_string(),
            enabled: true,
            platform_support: Some(r#"["macos"]"#.to_string()),
            created_at: String::new(),
        });
        rule_id += 1;
    }

    // Add file write rules if enabled
    if agent.enable_file_write {
        rules.push(crate::sandbox::profile::SandboxRule {
            id: Some(rule_id),
            profile_id: 0,
            operation_type: "file_write_all".to_string(),
            pattern_type: "subpath".to_string(),
            pattern_value: "{{PROJECT_PATH}}".to_string(),
            enabled: true,
            platform_support: Some(r#"["linux", "macos", "windows"]"#.to_string()),
            created_at: String::new(),
        });
        rule_id += 1;
    }

    // Add network rules if enabled
    if agent.enable_network {
        rules.push(crate::sandbox::profile::SandboxRule {
            id: Some(rule_id),
            profile_id: 0,
            operation_type: "network_outbound".to_string(),
            pattern_type: "all".to_string(),
            pattern_value: "".to_string(),
            enabled: true,
            platform_support: Some(r#"["linux", "macos"]"#.to_string()),
            created_at: String::new(),
        });
        rule_id += 1;
    }

    // Always add essential system paths (needed for executables to run)
    for (path, platforms) in &[
        ("/usr/bin", r#"["linux", "macos"]"#),
        ("/opt/homebrew/bin", r#"["macos"]"#),
        ("/usr/local/bin", r#"["linux", "macos"]"#),
        ("/bin", r#"["linux", "macos"]"#),
        ("/usr/sbin", r#"["linux", "macos"]"#),
    ] {
        rules.push(crate::sandbox::profile::SandboxRule {
            id: Some(rule_id),
            profile_id: 0,
            operation_type: "file_read_all".to_string(),
            pattern_type: "subpath".to_string(),
            pattern_value: path.to_string(),
            enabled: true,
            platform_support: Some(platforms.to_string()),
            created_at: String::new(),
        });
        rule_id += 1;
    }

    rules
}

/// Format duration in human-readable format
pub fn format_duration(ms: i64) -> String {
    let seconds = ms / 1000;
    let minutes = seconds / 60;
    let hours = minutes / 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes % 60)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds % 60)
    } else {
        format!("{}s", seconds)
    }
}

/// Format cost in USD
pub fn format_cost(cost: f64) -> String {
    if cost < 0.01 {
        format!("${:.4}", cost)
    } else if cost < 1.0 {
        format!("${:.3}", cost)
    } else {
        format!("${:.2}", cost)
    }
}

/// Create a tokio Command with proper environment variables
pub fn create_command_with_env(program: &str) -> tokio::process::Command {
    // Convert std::process::Command to tokio::process::Command
    let _std_cmd = crate::claude_binary::create_command_with_env(program);

    // Create a new tokio Command from the program path
    let mut tokio_cmd = tokio::process::Command::new(program);

    // Copy over all environment variables from the std::process::Command
    // This is a workaround since we can't directly convert between the two types
    for (key, value) in std::env::vars() {
        if key == "PATH"
            || key == "HOME"
            || key == "USER"
            || key == "SHELL"
            || key == "LANG"
            || key == "LC_ALL"
            || key.starts_with("LC_")
            || key == "NODE_PATH"
            || key == "NVM_DIR"
            || key == "NVM_BIN"
            || key == "HOMEBREW_PREFIX"
            || key == "HOMEBREW_CELLAR"
        {
            tokio_cmd.env(&key, &value);
        }
    }

    // Add NVM support if the program is in an NVM directory
    if program.contains("/.nvm/versions/node/") {
        if let Some(node_bin_dir) = std::path::Path::new(program).parent() {
            if let Some(node_bin_str) = node_bin_dir.to_str() {
                tokio_cmd.env("NVM_BIN", node_bin_str);
                
                // Ensure the NVM bin directory is in the PATH
                if let Ok(current_path) = std::env::var("PATH") {
                    let new_path = format!("{}:{}", node_bin_str, current_path);
                    tokio_cmd.env("PATH", new_path);
                }
            }
        }
    }

    // Add Homebrew binary paths if on macOS
    #[cfg(target_os = "macos")]
    {
        if let Ok(current_path) = std::env::var("PATH") {
            let new_path = format!("/opt/homebrew/bin:/usr/local/bin:{}", current_path);
            tokio_cmd.env("PATH", new_path);
        }
    }

    tokio_cmd
}

/// Read JSONL content from a session file
pub async fn read_session_jsonl(session_id: &str, project_path: &str) -> Result<String, String> {
    let claude_dir = dirs::home_dir()
        .ok_or("Failed to get home directory")?
        .join(".claude")
        .join("projects");

    // Encode project path to match Claude Code's directory naming
    let encoded_project = project_path.replace('/', "-");
    let project_dir = claude_dir.join(&encoded_project);
    let session_file = project_dir.join(format!("{}.jsonl", session_id));

    if !session_file.exists() {
        return Err(format!(
            "Session file not found: {}",
            session_file.display()
        ));
    }

    match tokio::fs::read_to_string(&session_file).await {
        Ok(content) => Ok(content),
        Err(e) => Err(format!("Failed to read session file: {}", e)),
    }
}

/// Parse usage limit error from output
pub fn parse_usage_limit_error(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.contains("Usage limit exceeded") || line.contains("rate limit") {
            // Extract the reset time if available
            if let Some(reset_idx) = line.find("resets at") {
                let reset_time = &line[reset_idx + 10..];
                if let Some(end_idx) = reset_time.find('.') {
                    return Some(reset_time[..end_idx].trim().to_string());
                }
            }
            // Return a default time if we can't parse it
            return Some(chrono::Utc::now().to_rfc3339());
        }
    }
    None
}

/// Kill process tree (Unix-specific)
#[cfg(unix)]
pub fn kill_process_tree(pid: u32) -> Result<(), AgentError> {
    use std::process::Command;

    // Use pkill to kill the process group
    let output = Command::new("pkill")
        .args(&["-TERM", "-P", &pid.to_string()])
        .output()
        .map_err(|e| AgentError::Process(format!("Failed to execute pkill: {}", e)))?;

    if !output.status.success() {
        // Try direct kill as fallback
        kill_process(pid)?;
    }

    Ok(())
}

#[cfg(windows)]
pub fn kill_process_tree(pid: u32) -> Result<(), AgentError> {
    // On Windows, taskkill /F /T kills the process tree
    use std::process::Command;

    Command::new("taskkill")
        .args(&["/F", "/T", "/PID", &pid.to_string()])
        .output()
        .map_err(|e| AgentError::Process(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
#[path = "helpers_tests.rs"]
mod tests;