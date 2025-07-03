use crate::claude_binary;
use anyhow::Result;
use chrono::{self, Datelike};
use log::info;
use serde_json::Value as JsonValue;
use tauri::AppHandle;
use tokio::process::Command;

use super::models::{AgentRun, AgentRunMetrics, AgentRunWithMetrics};

/// Finds the full path to the claude binary
/// This is necessary because macOS apps have a limited PATH environment
pub fn find_claude_binary(app_handle: &AppHandle) -> Result<String, String> {
    claude_binary::find_claude_binary(app_handle)
}

impl AgentRunMetrics {
    /// Calculate metrics from JSONL content
    pub fn from_jsonl(jsonl_content: &str, model: &str) -> Self {
        let mut total_tokens = 0i64;
        let mut total_input_tokens = 0i64;
        let mut total_output_tokens = 0i64;
        let mut total_cache_creation_tokens = 0i64;
        let mut total_cache_read_tokens = 0i64;
        let mut cost_usd = 0.0f64;
        let mut has_cost_field = false;
        let mut message_count = 0i64;
        let mut start_time: Option<chrono::DateTime<chrono::Utc>> = None;
        let mut end_time: Option<chrono::DateTime<chrono::Utc>> = None;

        for line in jsonl_content.lines() {
            if let Ok(json) = serde_json::from_str::<JsonValue>(line) {
                message_count += 1;

                // Track timestamps
                if let Some(timestamp_str) = json.get("timestamp").and_then(|t| t.as_str()) {
                    if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(timestamp_str) {
                        let utc_time = timestamp.with_timezone(&chrono::Utc);
                        if start_time.is_none() || utc_time < start_time.unwrap() {
                            start_time = Some(utc_time);
                        }
                        if end_time.is_none() || utc_time > end_time.unwrap() {
                            end_time = Some(utc_time);
                        }
                    }
                }

                // Extract token usage - check both top-level and nested message.usage
                let usage = json
                    .get("usage")
                    .or_else(|| json.get("message").and_then(|m| m.get("usage")));

                if let Some(usage) = usage {
                    if let Some(input_tokens) = usage.get("input_tokens").and_then(|t| t.as_i64()) {
                        total_tokens += input_tokens;
                        total_input_tokens += input_tokens;
                    }
                    if let Some(output_tokens) = usage.get("output_tokens").and_then(|t| t.as_i64())
                    {
                        total_tokens += output_tokens;
                        total_output_tokens += output_tokens;
                    }
                    if let Some(cache_creation) = usage.get("cache_creation_input_tokens").and_then(|t| t.as_i64()) {
                        total_cache_creation_tokens += cache_creation;
                    }
                    if let Some(cache_read) = usage.get("cache_read_input_tokens").and_then(|t| t.as_i64()) {
                        total_cache_read_tokens += cache_read;
                    }
                }

                // Extract cost information
                if let Some(cost) = json.get("cost").and_then(|c| c.as_f64()) {
                    cost_usd += cost;
                    has_cost_field = true;
                }
            }
        }

        // If no cost field was found but we have tokens, calculate the cost
        if !has_cost_field && total_tokens > 0 {
            // Claude 4 pricing constants (per million tokens)
            const OPUS_4_INPUT_PRICE: f64 = 15.0;
            const OPUS_4_OUTPUT_PRICE: f64 = 75.0;
            const OPUS_4_CACHE_WRITE_PRICE: f64 = 18.75;
            const OPUS_4_CACHE_READ_PRICE: f64 = 1.50;

            const SONNET_4_INPUT_PRICE: f64 = 3.0;
            const SONNET_4_OUTPUT_PRICE: f64 = 15.0;
            const SONNET_4_CACHE_WRITE_PRICE: f64 = 3.75;
            const SONNET_4_CACHE_READ_PRICE: f64 = 0.30;

            // Calculate cost based on model
            let (input_price, output_price, cache_write_price, cache_read_price) =
                if model.contains("opus-4") || model.contains("claude-opus-4") {
                    (
                        OPUS_4_INPUT_PRICE,
                        OPUS_4_OUTPUT_PRICE,
                        OPUS_4_CACHE_WRITE_PRICE,
                        OPUS_4_CACHE_READ_PRICE,
                    )
                } else if model.contains("sonnet-4") || model.contains("claude-sonnet-4") {
                    (
                        SONNET_4_INPUT_PRICE,
                        SONNET_4_OUTPUT_PRICE,
                        SONNET_4_CACHE_WRITE_PRICE,
                        SONNET_4_CACHE_READ_PRICE,
                    )
                } else {
                    // Default to sonnet pricing for unknown models
                    (
                        SONNET_4_INPUT_PRICE,
                        SONNET_4_OUTPUT_PRICE,
                        SONNET_4_CACHE_WRITE_PRICE,
                        SONNET_4_CACHE_READ_PRICE,
                    )
                };

            // Calculate cost (prices are per million tokens)
            let input_cost = (total_input_tokens as f64 / 1_000_000.0) * input_price;
            let output_cost = (total_output_tokens as f64 / 1_000_000.0) * output_price;
            let cache_write_cost = (total_cache_creation_tokens as f64 / 1_000_000.0) * cache_write_price;
            let cache_read_cost = (total_cache_read_tokens as f64 / 1_000_000.0) * cache_read_price;

            cost_usd = input_cost + output_cost + cache_write_cost + cache_read_cost;
        }

        let duration_ms = match (start_time, end_time) {
            (Some(start), Some(end)) => Some((end - start).num_milliseconds()),
            _ => None,
        };

        Self {
            duration_ms,
            total_tokens: if total_tokens > 0 {
                Some(total_tokens)
            } else {
                None
            },
            cost_usd: if cost_usd > 0.0 { Some(cost_usd) } else { None },
            message_count: if message_count > 0 {
                Some(message_count)
            } else {
                None
            },
        }
    }
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

    info!("Looking for JSONL file at: {}", session_file.display());

    if !session_file.exists() {
        // Check if there might be a resumed session file pattern
        info!("Primary JSONL file not found, checking for alternatives...");
        
        // List all files in the directory
        if let Ok(entries) = std::fs::read_dir(&project_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                        info!("Found JSONL file: {}", path.display());
                    }
                }
            }
        }
        
        return Err(format!(
            "Session file not found: {}",
            session_file.display()
        ));
    }

    match tokio::fs::read_to_string(&session_file).await {
        Ok(content) => {
            info!("Read JSONL file successfully, size: {} bytes", content.len());
            Ok(content)
        },
        Err(e) => Err(format!("Failed to read session file: {}", e)),
    }
}

/// Get agent run with real-time metrics (without full output for performance)
pub async fn get_agent_run_with_metrics(run: AgentRun) -> AgentRunWithMetrics {
    match read_session_jsonl(&run.session_id, &run.project_path).await {
        Ok(jsonl_content) => {
            let metrics = AgentRunMetrics::from_jsonl(&jsonl_content, &run.model);
            AgentRunWithMetrics {
                run,
                metrics: Some(metrics),
                output: None, // Don't include full output for performance
            }
        }
        Err(e) => {
            log::warn!("Failed to read JSONL for session {}: {}", run.session_id, e);
            AgentRunWithMetrics {
                run,
                metrics: None,
                output: None,
            }
        }
    }
}

/// Parse usage limit error from Claude output
pub fn parse_usage_limit_error(output: &str) -> Option<String> {
    // Try to parse as JSONL first
    for line in output.lines() {
        if let Ok(json) = serde_json::from_str::<JsonValue>(line) {
            if let Some(msg) = json.get("messageText").and_then(|m| m.as_str()) {
                if msg.contains("I've reached my usage limit") {
                    // Extract the reset time from the message
                    if let Some(reset_time_start) = msg.find("again at") {
                        let reset_time_str = &msg[reset_time_start + 8..];
                        if let Some(period_pos) = reset_time_str.find('.') {
                            let reset_time = &reset_time_str[..period_pos].trim();
                            
                            // Parse the time format: "6:00 PM PST on November 28"
                            // We'll convert this to an ISO 8601 datetime
                            if let Some(parsed_time) = parse_usage_limit_time(reset_time) {
                                return Some(parsed_time);
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Fallback: check raw output
    if output.contains("I've reached my usage limit") {
        // Look for the reset time pattern
        if let Some(reset_time_start) = output.find("again at") {
            let reset_time_str = &output[reset_time_start + 8..];
            if let Some(period_pos) = reset_time_str.find('.') {
                let reset_time = &reset_time_str[..period_pos].trim();
                if let Some(parsed_time) = parse_usage_limit_time(reset_time) {
                    return Some(parsed_time);
                }
            }
        }
    }
    
    None
}

/// Parse usage limit time format like "6:00 PM PST on November 28"
fn parse_usage_limit_time(time_str: &str) -> Option<String> {
    // This is a simplified parser - in production you'd want more robust parsing
    let parts: Vec<&str> = time_str.split(" on ").collect();
    if parts.len() != 2 {
        return None;
    }
    
    let time_part = parts[0];
    let date_part = parts[1];
    
    // Get current year
    let now = chrono::Utc::now();
    let current_year = now.year();
    
    // Parse month and day
    let date_parts: Vec<&str> = date_part.split_whitespace().collect();
    if date_parts.len() != 2 {
        return None;
    }
    
    let month_str = date_parts[0];
    let day_str = date_parts[1];
    
    let month = match month_str.to_lowercase().as_str() {
        "january" => 1,
        "february" => 2,
        "march" => 3,
        "april" => 4,
        "may" => 5,
        "june" => 6,
        "july" => 7,
        "august" => 8,
        "september" => 9,
        "october" => 10,
        "november" => 11,
        "december" => 12,
        _ => return None,
    };
    
    let day: u32 = day_str.parse().ok()?;
    
    // Parse time (e.g., "6:00 PM PST")
    let time_parts: Vec<&str> = time_part.split_whitespace().collect();
    if time_parts.len() < 2 {
        return None;
    }
    
    let time_str = time_parts[0];
    let am_pm = time_parts[1];
    
    let time_components: Vec<&str> = time_str.split(':').collect();
    if time_components.len() != 2 {
        return None;
    }
    
    let mut hour: u32 = time_components[0].parse().ok()?;
    let minute: u32 = time_components[1].parse().ok()?;
    
    // Convert to 24-hour format
    if am_pm.to_uppercase() == "PM" && hour != 12 {
        hour += 12;
    } else if am_pm.to_uppercase() == "AM" && hour == 12 {
        hour = 0;
    }
    
    // Assume PST offset (-8 hours from UTC)
    // In production, you'd parse the timezone properly
    let pst_offset_hours = 8;
    
    // Create the datetime
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    let naive_date = NaiveDate::from_ymd_opt(current_year, month, day)?;
    let naive_time = NaiveTime::from_hms_opt(hour, minute, 0)?;
    let naive_datetime = NaiveDateTime::new(naive_date, naive_time);
    
    // Convert to UTC by adding PST offset
    let utc_datetime = naive_datetime + chrono::Duration::hours(pst_offset_hours);
    
    // Format as ISO 8601
    Some(format!("{}", utc_datetime.format("%Y-%m-%dT%H:%M:%S.000Z")))
}

/// Create a command with proper environment variables
pub fn create_command_with_env(program: &str) -> Command {
    // Convert std::process::Command to tokio::process::Command
    let _std_cmd = crate::claude_binary::create_command_with_env(program);

    // Create a new tokio Command from the program path
    let mut tokio_cmd = Command::new(program);

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
            let current_path = std::env::var("PATH").unwrap_or_default();
            let node_bin_str = node_bin_dir.to_string_lossy();
            if !current_path.contains(&node_bin_str.as_ref()) {
                let new_path = format!("{}:{}", node_bin_str, current_path);
                tokio_cmd.env("PATH", new_path);
            }
        }
    }

    // Ensure PATH contains common Homebrew locations
    if let Ok(existing_path) = std::env::var("PATH") {
        let mut paths: Vec<&str> = existing_path.split(':').collect();
        for p in ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin", "/bin"].iter() {
            if !paths.contains(p) {
                paths.push(p);
            }
        }
        let joined = paths.join(":");
        tokio_cmd.env("PATH", joined);
    } else {
        tokio_cmd.env("PATH", "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin");
    }

    tokio_cmd
}