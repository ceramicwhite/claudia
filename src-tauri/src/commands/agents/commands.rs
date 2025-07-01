use crate::commands::agents::{error::AgentError, service::*, types::*, AgentDb, AgentRunWithMetrics};
use crate::process_registry::ProcessRegistry;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};

/// Tauri command handlers - thin wrappers around service methods

#[tauri::command]
pub async fn list_agents(
    app: AppHandle,
    db: State<'_, AgentDb>,
) -> Result<Vec<Agent>, String> {
    let service = AgentService::new(app);
    let pool = db.0.clone();
    
    service.list_agents(pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_agent(
    app: AppHandle,
    db: State<'_, AgentDb>,
    id: i64,
) -> Result<Agent, String> {
    let service = AgentService::new(app);
    let pool = db.0.clone();
    
    service.get_agent(pool, id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_agent(
    app: AppHandle,
    db: State<'_, AgentDb>,
    name: String,
    icon: String,
    system_prompt: String,
    default_task: Option<String>,
    model: Option<String>,
    sandbox_enabled: Option<bool>,
    enable_file_read: Option<bool>,
    enable_file_write: Option<bool>,
    enable_network: Option<bool>,
) -> Result<Agent, String> {
    let service = AgentService::new(app);
    let pool = db.0.clone();
    
    service.create_agent(
        pool,
        name,
        icon,
        system_prompt,
        default_task,
        model,
        sandbox_enabled,
        enable_file_read,
        enable_file_write,
        enable_network,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_agent(
    app: AppHandle,
    db: State<'_, AgentDb>,
    id: i64,
    name: String,
    icon: String,
    system_prompt: String,
    default_task: Option<String>,
    model: Option<String>,
    sandbox_enabled: Option<bool>,
    enable_file_read: Option<bool>,
    enable_file_write: Option<bool>,
    enable_network: Option<bool>,
) -> Result<Agent, String> {
    let service = AgentService::new(app);
    let pool = db.0.clone();
    
    service.update_agent(
        pool,
        id,
        name,
        icon,
        system_prompt,
        default_task,
        model,
        sandbox_enabled,
        enable_file_read,
        enable_file_write,
        enable_network,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_agent(
    app: AppHandle,
    db: State<'_, AgentDb>,
    id: i64,
) -> Result<(), String> {
    let service = AgentService::new(app);
    let pool = db.0.clone();
    
    service.delete_agent(pool, id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_agent_runs(
    app: AppHandle,
    db: State<'_, AgentDb>,
    agent_id: Option<i64>,
) -> Result<Vec<AgentRun>, String> {
    let service = AgentService::new(app);
    let pool = db.0.clone();
    
    service.list_agent_runs(pool, agent_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_agent_runs_with_metrics(
    app: AppHandle,
    db: State<'_, AgentDb>,
    agent_id: Option<i64>,
) -> Result<Vec<AgentRunWithMetrics>, String> {
    let service = AgentService::new(app);
    let pool = db.0.clone();
    
    service.list_agent_runs_with_metrics(pool, agent_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_agent_run(
    app: AppHandle,
    db: State<'_, AgentDb>,
    id: i64,
) -> Result<AgentRun, String> {
    let service = AgentService::new(app);
    let pool = db.0.clone();
    
    service.get_agent_run(pool, id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_agent_run_with_real_time_metrics(
    app: AppHandle,
    db: State<'_, AgentDb>,
    id: i64,
) -> Result<AgentRunWithMetrics, String> {
    let service = AgentService::new(app);
    let pool = db.0.clone();
    
    service.get_agent_run_with_metrics(pool, id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_running_sessions(
    app: AppHandle,
    db: State<'_, AgentDb>,
) -> Result<Vec<AgentRun>, String> {
    let service = AgentService::new(app);
    let pool = db.0.clone();
    
    service.list_running_sessions(pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_running_sessions_with_metrics(
    app: AppHandle,
    db: State<'_, AgentDb>,
) -> Result<Vec<AgentRunWithMetrics>, String> {
    let service = AgentService::new(app);
    let pool = db.0.clone();
    
    service.list_running_sessions_with_metrics(pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn kill_agent_session(
    app: AppHandle,
    db: State<'_, AgentDb>,
    registry: State<'_, Arc<Mutex<ProcessRegistry>>>,
    session_id: String,
) -> Result<(), String> {
    let service = AgentService::new(app);
    let pool = db.0.clone();
    
    service.kill_agent_session(pool, &registry, session_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_session_status(
    app: AppHandle,
    db: State<'_, AgentDb>,
    registry: State<'_, Arc<Mutex<ProcessRegistry>>>,
    session_id: String,
) -> Result<SessionStatus, String> {
    let service = AgentService::new(app);
    let pool = db.0.clone();
    
    service.get_session_status(pool, &registry, session_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cleanup_finished_processes(
    app: AppHandle,
    db: State<'_, AgentDb>,
    registry: State<'_, Arc<Mutex<ProcessRegistry>>>,
) -> Result<Vec<i64>, String> {
    let service = AgentService::new(app);
    let pool = db.0.clone();
    
    service.cleanup_finished_processes(pool, &registry)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_live_session_output(
    app: AppHandle,
    registry: State<'_, Arc<Mutex<ProcessRegistry>>>,
    session_id: String,
) -> Result<Vec<String>, String> {
    let service = AgentService::new(app);
    
    service.get_live_session_output(&registry, session_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_session_output(
    app: AppHandle,
    db: State<'_, AgentDb>,
    session_id: String,
) -> Result<String, String> {
    let service = AgentService::new(app);
    let pool = db.0.clone();
    
    service.get_session_output(pool, session_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_agent(
    app: AppHandle,
    db: State<'_, AgentDb>,
    id: i64,
) -> Result<String, String> {
    let service = AgentService::new(app);
    let pool = db.0.clone();
    
    service.export_agent(pool, id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_agent_to_file(
    app: AppHandle,
    db: State<'_, AgentDb>,
    id: i64,
    file_path: String,
) -> Result<(), String> {
    let service = AgentService::new(app);
    let pool = db.0.clone();
    
    let json_data = service.export_agent(pool, id).await?;
    
    // Write to file
    tokio::fs::write(&file_path, json_data)
        .await
        .map_err(|e| format!("Failed to write file: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn import_agent(
    app: AppHandle,
    db: State<'_, AgentDb>,
    json_data: String,
) -> Result<Agent, String> {
    let service = AgentService::new(app);
    let pool = db.0.clone();
    
    service.import_agent(pool, json_data)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_agent_from_file(
    app: AppHandle,
    db: State<'_, AgentDb>,
    file_path: String,
) -> Result<Agent, String> {
    let service = AgentService::new(app);
    let pool = db.0.clone();
    
    // Read file
    let json_data = tokio::fs::read_to_string(&file_path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    service.import_agent(pool, json_data)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_scheduled_agent_runs(
    app: AppHandle,
    db: State<'_, AgentDb>,
) -> Result<Vec<AgentRun>, String> {
    let service = AgentService::new(app);
    let pool = db.0.clone();
    
    service.get_scheduled_runs(pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_scheduled_agent_run(
    app: AppHandle,
    db: State<'_, AgentDb>,
    agent_id: i64,
    task: String,
    project_path: String,
    scheduled_start_time: String,
) -> Result<AgentRun, String> {
    let service = AgentService::new(app);
    let pool = db.0.clone();
    
    service.create_scheduled_run(pool, agent_id, task, project_path, scheduled_start_time)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cancel_scheduled_agent_run(
    app: AppHandle,
    db: State<'_, AgentDb>,
    run_id: i64,
) -> Result<(), String> {
    let service = AgentService::new(app);
    let pool = db.0.clone();
    
    service.cancel_scheduled_run(pool, run_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_claude_binary_path(
    app: AppHandle,
    db: State<'_, AgentDb>,
) -> Result<Option<String>, String> {
    let service = AgentService::new(app);
    let pool = db.0.clone();
    
    service.get_claude_binary_path(pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_claude_binary_path(
    app: AppHandle,
    db: State<'_, AgentDb>,
    path: String,
) -> Result<(), String> {
    let service = AgentService::new(app);
    let pool = db.0.clone();
    
    service.set_claude_binary_path(pool, path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resume_agent(
    app: AppHandle,
    db: State<'_, AgentDb>,
    registry: State<'_, Arc<Mutex<ProcessRegistry>>>,
    run_id: i64,
) -> Result<AgentRun, String> {
    let service = AgentService::new(app);
    let pool = db.0.clone();
    
    service.resume_agent(pool, run_id, Arc::clone(&registry))
        .await
        .map_err(|e| e.to_string())
}

// GitHub-related commands

#[tauri::command]
pub async fn fetch_github_agents() -> Result<Vec<GitHubAgentFile>, String> {
    fetch_github_agents_internal()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fetch_github_agent_content(download_url: String) -> Result<AgentExport, String> {
    fetch_github_agent_content_internal(download_url)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_agent_from_github(
    app: AppHandle,
    db: State<'_, AgentDb>,
    download_url: String,
) -> Result<Agent, String> {
    let export = fetch_github_agent_content_internal(download_url).await?;
    let json_data = serde_json::to_string(&export)
        .map_err(|e| format!("Failed to serialize agent data: {}", e))?;
    
    let service = AgentService::new(app);
    let pool = db.0.clone();
    
    service.import_agent(pool, json_data)
        .await
        .map_err(|e| e.to_string())
}

// Helper functions for GitHub operations

async fn fetch_github_agents_internal() -> Result<Vec<GitHubAgentFile>, AgentError> {
    use reqwest;
    
    let client = reqwest::Client::builder()
        .user_agent("Claudia-App")
        .build()
        .map_err(|e| AgentError::Network(e.to_string()))?;

    let url = "https://api.github.com/repos/cktang88/claudia-prompts/contents/agents";
    
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| AgentError::Network(e.to_string()))?;

    if !response.status().is_success() {
        return Err(AgentError::Network(format!(
            "GitHub API returned status: {}",
            response.status()
        )));
    }

    let files: Vec<GitHubAgentFile> = response
        .json()
        .await
        .map_err(|e| AgentError::Network(format!("Failed to parse GitHub response: {}", e)))?;

    // Filter only .json files
    let json_files: Vec<GitHubAgentFile> = files
        .into_iter()
        .filter(|f| f.name.ends_with(".json"))
        .collect();

    Ok(json_files)
}

async fn fetch_github_agent_content_internal(download_url: String) -> Result<AgentExport, AgentError> {
    use reqwest;
    
    let client = reqwest::Client::builder()
        .user_agent("Claudia-App")
        .build()
        .map_err(|e| AgentError::Network(e.to_string()))?;

    let response = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| AgentError::Network(e.to_string()))?;

    if !response.status().is_success() {
        return Err(AgentError::Network(format!(
            "Failed to download agent file: {}",
            response.status()
        )));
    }

    let content = response
        .text()
        .await
        .map_err(|e| AgentError::Network(format!("Failed to read response: {}", e)))?;

    let export: AgentExport = serde_json::from_str(&content)
        .map_err(|e| AgentError::Serialization(e))?;

    Ok(export)
}

// Execute agent command (delegates to execute module)
#[tauri::command]
pub async fn execute_agent(
    app: AppHandle,
    db: State<'_, AgentDb>,
    registry: State<'_, Arc<Mutex<ProcessRegistry>>>,
    agent_id: i64,
    task: Option<String>,
    project_path: Option<String>,
) -> Result<AgentRun, String> {
    let pool = db.0.clone();
    
    super::execute::execute_agent(
        app,
        pool,
        Arc::clone(&registry),
        agent_id,
        task,
        project_path,
        None,
        None,
        None,
    )
    .await
    .map_err(|e| e.to_string())
}

// Stream session output command
#[tauri::command]
pub async fn stream_session_output(
    app: AppHandle,
    db: State<'_, AgentDb>,
    session_id: String,
) -> Result<(), String> {
    let pool = db.0.clone();
    
    super::execute::stream_session_output(app, pool, session_id)
        .await
        .map_err(|e| e.to_string())
}

// List available Claude installations
#[tauri::command]
pub async fn list_claude_installations(
    app: AppHandle,
) -> Result<Vec<ClaudeInstallation>, String> {
    // For now, return a simple implementation
    // This would typically scan common installation locations
    let installations = vec![];
    Ok(installations)
}

// Read agent output file
pub fn read_agent_output_file(app_data_dir: &std::path::Path, run_id: i64) -> Result<String, String> {
    use std::fs;
    
    let output_dir = app_data_dir.join("agent_outputs");
    let output_file = output_dir.join(format!("{}.jsonl", run_id));
    
    if !output_file.exists() {
        return Ok(String::new());
    }
    
    fs::read_to_string(output_file)
        .map_err(|e| format!("Failed to read output file: {}", e))
}

#[cfg(test)]
mod commands_focused_tests;