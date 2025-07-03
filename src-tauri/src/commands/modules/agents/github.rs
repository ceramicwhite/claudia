use log::info;
use tauri::State;

use super::database::AgentDb;
use super::models::{Agent, AgentExport, GitHubAgentFile, GitHubApiResponse};
use super::import_export::import_agent;

/// Fetch list of agents from GitHub repository
#[tauri::command]
pub async fn fetch_github_agents() -> Result<Vec<GitHubAgentFile>, String> {
    info!("Fetching agents from GitHub repository...");

    let client = reqwest::Client::new();
    let url = "https://api.github.com/repos/getAsterisk/claudia/contents/cc_agents";

    let response = client
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "Claudia-App")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch from GitHub: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("GitHub API error ({}): {}", status, error_text));
    }

    let api_files: Vec<GitHubApiResponse> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse GitHub response: {}", e))?;

    // Filter only .claudia.json files
    let agent_files: Vec<GitHubAgentFile> = api_files
        .into_iter()
        .filter(|f| f.name.ends_with(".claudia.json") && f.file_type == "file")
        .filter_map(|f| {
            f.download_url.map(|download_url| GitHubAgentFile {
                name: f.name,
                path: f.path,
                download_url,
                size: f.size,
                sha: f.sha,
            })
        })
        .collect();

    info!("Found {} agents on GitHub", agent_files.len());
    Ok(agent_files)
}

/// Fetch and preview a specific agent from GitHub
#[tauri::command]
pub async fn fetch_github_agent_content(download_url: String) -> Result<AgentExport, String> {
    info!("Fetching agent content from: {}", download_url);

    let client = reqwest::Client::new();
    let response = client
        .get(&download_url)
        .header("Accept", "application/json")
        .header("User-Agent", "Claudia-App")
        .send()
        .await
        .map_err(|e| format!("Failed to download agent: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to download agent: HTTP {}",
            response.status()
        ));
    }

    let json_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // Parse and validate the agent data
    let export_data: AgentExport = serde_json::from_str(&json_text)
        .map_err(|e| format!("Invalid agent JSON format: {}", e))?;

    // Validate version
    if export_data.version != 1 {
        return Err(format!(
            "Unsupported agent version: {}",
            export_data.version
        ));
    }

    Ok(export_data)
}

/// Import an agent directly from GitHub
#[tauri::command]
pub async fn import_agent_from_github(
    db: State<'_, AgentDb>,
    download_url: String,
) -> Result<Agent, String> {
    // Fetch the agent content
    let export_data = fetch_github_agent_content(download_url).await?;
    
    // Convert to JSON string for import_agent
    let json_data = serde_json::to_string(&export_data)
        .map_err(|e| format!("Failed to serialize agent data: {}", e))?;
    
    // Import using the existing import function
    import_agent(db, json_data).await
}