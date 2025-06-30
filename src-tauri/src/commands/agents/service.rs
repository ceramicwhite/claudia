use crate::commands::agents::{
    constants, error::AgentError, helpers, pool::SqlitePool, repository::*, types::*, AgentRunMetrics,
    AgentRunWithMetrics,
};
use crate::process_registry::ProcessRegistry;
use anyhow::Result;
use chrono;
use log::{debug, error, info, warn};
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};

/// Agent service for business logic
pub struct AgentService {
    app_handle: AppHandle,
    app_data_dir: std::path::PathBuf,
}

impl AgentService {
    pub fn new(app_handle: AppHandle) -> Self {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .expect("Failed to get app data dir");

        Self {
            app_handle,
            app_data_dir,
        }
    }

    /// List all agents
    pub async fn list_agents(&self, pool: Arc<SqlitePool>) -> Result<Vec<Agent>, AgentError> {
        let repo = SqliteAgentRepository::new(pool.as_ref().clone());
        repo.find_all_agents()
    }

    /// Get a single agent
    pub async fn get_agent(&self, pool: Arc<SqlitePool>, id: i64) -> Result<Agent, AgentError> {
        let repo = SqliteAgentRepository::new(pool.as_ref().clone());
        repo.find_agent_by_id(id)
    }

    /// Create a new agent
    pub async fn create_agent(
        &self,
        pool: Arc<SqlitePool>,
        name: String,
        icon: String,
        system_prompt: String,
        default_task: Option<String>,
        model: Option<String>,
        sandbox_enabled: Option<bool>,
        enable_file_read: Option<bool>,
        enable_file_write: Option<bool>,
        enable_network: Option<bool>,
    ) -> Result<Agent, AgentError> {
        let repo = SqliteAgentRepository::new(pool.as_ref().clone());

        let new_agent = NewAgent {
            name,
            icon,
            system_prompt,
            default_task,
            model: model.unwrap_or_else(|| constants::DEFAULT_MODEL.to_string()),
            sandbox_enabled: sandbox_enabled.unwrap_or(constants::DEFAULT_SANDBOX_ENABLED),
            enable_file_read: enable_file_read.unwrap_or(constants::DEFAULT_FILE_READ_ENABLED),
            enable_file_write: enable_file_write.unwrap_or(constants::DEFAULT_FILE_WRITE_ENABLED),
            enable_network: enable_network.unwrap_or(constants::DEFAULT_NETWORK_ENABLED),
        };

        repo.create_agent(new_agent)
    }

    /// Update an existing agent
    pub async fn update_agent(
        &self,
        pool: Arc<SqlitePool>,
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
    ) -> Result<Agent, AgentError> {
        let repo = SqliteAgentRepository::new(pool.as_ref().clone());

        let update_agent = UpdateAgent {
            name,
            icon,
            system_prompt,
            default_task,
            model: model.unwrap_or_else(|| constants::DEFAULT_MODEL.to_string()),
            sandbox_enabled,
            enable_file_read,
            enable_file_write,
            enable_network,
        };

        repo.update_agent(id, update_agent)
    }

    /// Delete an agent
    pub async fn delete_agent(&self, pool: Arc<SqlitePool>, id: i64) -> Result<(), AgentError> {
        let repo = SqliteAgentRepository::new(pool.as_ref().clone());
        repo.delete_agent(id)
    }

    /// List agent runs
    pub async fn list_agent_runs(
        &self,
        pool: Arc<SqlitePool>,
        agent_id: Option<i64>,
    ) -> Result<Vec<AgentRun>, AgentError> {
        let repo = SqliteAgentRepository::new(pool.as_ref().clone());
        repo.find_all_runs(agent_id)
    }

    /// List agent runs with calculated metrics
    pub async fn list_agent_runs_with_metrics(
        &self,
        pool: Arc<SqlitePool>,
        agent_id: Option<i64>,
    ) -> Result<Vec<AgentRunWithMetrics>, AgentError> {
        let repo = SqliteAgentRepository::new(pool.as_ref().clone());
        let runs = repo.find_all_runs(agent_id)?;

        let mut runs_with_metrics = Vec::new();
        for run in runs {
            let metrics = self.get_run_metrics(pool.clone(), &run).await;
            runs_with_metrics.push(AgentRunWithMetrics {
                run,
                metrics: Some(metrics),
                output: None,
            });
        }

        Ok(runs_with_metrics)
    }

    /// Get a single agent run
    pub async fn get_agent_run(&self, pool: Arc<SqlitePool>, id: i64) -> Result<AgentRun, AgentError> {
        let repo = SqliteAgentRepository::new(pool.as_ref().clone());
        repo.find_run_by_id(id)
    }

    /// Get agent run with real-time metrics
    pub async fn get_agent_run_with_metrics(
        &self,
        pool: Arc<SqlitePool>,
        id: i64,
    ) -> Result<AgentRunWithMetrics, AgentError> {
        let repo = SqliteAgentRepository::new(pool.as_ref().clone());
        let run = repo.find_run_by_id(id)?;
        let metrics = self.get_run_metrics(pool.clone(), &run).await;
        let output = repo.get_jsonl_output(id).ok();

        Ok(AgentRunWithMetrics {
            run,
            metrics: Some(metrics),
            output,
        })
    }

    /// List running sessions
    pub async fn list_running_sessions(
        &self,
        pool: Arc<SqlitePool>,
    ) -> Result<Vec<AgentRun>, AgentError> {
        let repo = SqliteAgentRepository::new(pool.as_ref().clone());
        repo.find_running_runs()
    }

    /// List running sessions with metrics
    pub async fn list_running_sessions_with_metrics(
        &self,
        pool: Arc<SqlitePool>,
    ) -> Result<Vec<AgentRunWithMetrics>, AgentError> {
        let repo = SqliteAgentRepository::new(pool.as_ref().clone());
        let runs = repo.find_running_runs()?;

        let mut runs_with_metrics = Vec::new();
        for run in runs {
            let metrics = self.get_run_metrics(pool.clone(), &run).await;
            runs_with_metrics.push(AgentRunWithMetrics {
                run,
                metrics: Some(metrics),
                output: None,
            });
        }

        Ok(runs_with_metrics)
    }

    /// Get scheduled agent runs
    pub async fn get_scheduled_runs(
        &self,
        pool: Arc<SqlitePool>,
    ) -> Result<Vec<AgentRun>, AgentError> {
        let repo = SqliteAgentRepository::new(pool.as_ref().clone());
        repo.find_scheduled_runs()
    }

    /// Create a scheduled agent run
    pub async fn create_scheduled_run(
        &self,
        pool: Arc<SqlitePool>,
        agent_id: i64,
        task: String,
        project_path: String,
        scheduled_start_time: String,
    ) -> Result<AgentRun, AgentError> {
        let repo = SqliteAgentRepository::new(pool.as_ref().clone());

        // Get agent details
        let agent = repo.find_agent_by_id(agent_id)?;

        let new_run = NewAgentRun {
            agent_id,
            agent_name: agent.name,
            agent_icon: agent.icon,
            task,
            model: agent.model,
            project_path,
            status: Some("scheduled"),
            scheduled_start_time: Some(scheduled_start_time),
            parent_run_id: None,
        };

        repo.create_run(new_run)
    }

    /// Cancel a scheduled agent run
    pub async fn cancel_scheduled_run(&self, pool: Arc<SqlitePool>, run_id: i64) -> Result<(), AgentError> {
        let repo = SqliteAgentRepository::new(pool.as_ref().clone());
        repo.update_run_status(run_id, "cancelled", None, None)
    }

    /// Kill an agent session
    pub async fn kill_agent_session(
        &self,
        pool: Arc<SqlitePool>,
        registry: &Arc<Mutex<ProcessRegistry>>,
        session_id: String,
    ) -> Result<(), AgentError> {
        let repo = SqliteAgentRepository::new(pool.as_ref().clone());

        // Find the run by session ID
        if let Some(run) = repo.find_run_by_session_id(&session_id)? {
            if let Some(run_id) = run.id {
                // Try to kill through registry first
                let mut reg = registry
                    .lock()
                    .map_err(|e| AgentError::Lock(e.to_string()))?;

                if reg.kill_process(&session_id).is_ok() {
                    info!("Killed agent session {} through registry", session_id);
                    repo.update_run_completion(run_id, "killed")?;
                    return Ok(());
                }

                // Fallback to PID-based kill
                if let Some(pid) = run.pid {
                    if let Err(e) = helpers::kill_process_tree(pid) {
                        warn!("Failed to kill process tree: {}", e);
                        // Try simple kill as last resort
                        helpers::kill_process(pid)?;
                    }
                    repo.update_run_completion(run_id, "killed")?;
                    info!("Killed agent session {} (PID: {})", session_id, pid);
                } else {
                    return Err(AgentError::Other("Process has no PID".to_string()));
                }
            }
        } else {
            return Err(AgentError::Other(format!(
                "No running session found with ID: {}",
                session_id
            )));
        }

        Ok(())
    }

    /// Get session status
    pub async fn get_session_status(
        &self,
        pool: Arc<SqlitePool>,
        registry: &Arc<Mutex<ProcessRegistry>>,
        session_id: String,
    ) -> Result<SessionStatus, AgentError> {
        let repo = SqliteAgentRepository::new(pool.as_ref().clone());

        if let Some(run) = repo.find_run_by_session_id(&session_id)? {
            // Check if process is still alive through registry
            let reg = registry
                .lock()
                .map_err(|e| AgentError::Lock(e.to_string()))?;

            let is_alive = reg.is_process_alive(&session_id);

            Ok(SessionStatus {
                session_id,
                status: run.status,
                is_alive,
                pid: run.pid,
            })
        } else {
            Ok(SessionStatus {
                session_id: session_id.clone(),
                status: "not_found".to_string(),
                is_alive: false,
                pid: None,
            })
        }
    }

    /// Cleanup finished processes
    pub async fn cleanup_finished_processes(
        &self,
        pool: Arc<SqlitePool>,
        registry: &Arc<Mutex<ProcessRegistry>>,
    ) -> Result<Vec<i64>, AgentError> {
        let repo = SqliteAgentRepository::new(pool.as_ref().clone());
        let running_runs = repo.find_running_runs()?;

        let mut cleaned_run_ids = Vec::new();
        let reg = registry
            .lock()
            .map_err(|e| AgentError::Lock(e.to_string()))?;

        for run in running_runs {
            if let Some(run_id) = run.id {
                // Check if process is still alive through registry
                if !reg.is_process_alive(&run.session_id) {
                    info!(
                        "Cleaning up finished process for run {} (session: {})",
                        run_id, run.session_id
                    );

                    // Update status to completed
                    repo.update_run_completion(run_id, "completed")?;
                    cleaned_run_ids.push(run_id);
                }
            }
        }

        if !cleaned_run_ids.is_empty() {
            info!("Cleaned up {} finished processes", cleaned_run_ids.len());
        }

        Ok(cleaned_run_ids)
    }

    /// Get live session output (from registry)
    pub async fn get_live_session_output(
        &self,
        registry: &Arc<Mutex<ProcessRegistry>>,
        session_id: String,
    ) -> Result<Vec<String>, AgentError> {
        let reg = registry
            .lock()
            .map_err(|e| AgentError::Lock(e.to_string()))?;

        reg.get_output(&session_id)
            .map_err(|e| AgentError::Other(e.to_string()))
    }

    /// Get session output (from database)
    pub async fn get_session_output(
        &self,
        pool: Arc<SqlitePool>,
        session_id: String,
    ) -> Result<String, AgentError> {
        let repo = SqliteAgentRepository::new(pool.as_ref().clone());

        if let Some(run) = repo.find_run_by_session_id(&session_id)? {
            if let Some(run_id) = run.id {
                return repo.get_jsonl_output(run_id);
            }
        }

        Err(AgentError::Other(format!(
            "No session found with ID: {}",
            session_id
        )))
    }

    /// Export agent to JSON
    pub async fn export_agent(&self, pool: Arc<SqlitePool>, id: i64) -> Result<String, AgentError> {
        let repo = SqliteAgentRepository::new(pool.as_ref().clone());
        let agent = repo.find_agent_by_id(id)?;

        let export = AgentExport {
            version: 1,
            exported_at: chrono::Utc::now().to_rfc3339(),
            agent: AgentData {
                name: agent.name,
                icon: agent.icon,
                system_prompt: agent.system_prompt,
                default_task: agent.default_task,
                model: agent.model,
                sandbox_enabled: agent.sandbox_enabled,
                enable_file_read: agent.enable_file_read,
                enable_file_write: agent.enable_file_write,
                enable_network: agent.enable_network,
            },
        };

        serde_json::to_string_pretty(&export).map_err(|e| AgentError::Serialization(e))
    }

    /// Import agent from JSON
    pub async fn import_agent(
        &self,
        pool: Arc<SqlitePool>,
        json_data: String,
    ) -> Result<Agent, AgentError> {
        let export: AgentExport =
            serde_json::from_str(&json_data).map_err(|e| AgentError::Serialization(e))?;

        if export.version != 1 {
            return Err(AgentError::Other(format!(
                "Unsupported export version: {}",
                export.version
            )));
        }

        let repo = SqliteAgentRepository::new(pool.as_ref().clone());
        let new_agent = NewAgent {
            name: export.agent.name,
            icon: export.agent.icon,
            system_prompt: export.agent.system_prompt,
            default_task: export.agent.default_task,
            model: export.agent.model,
            sandbox_enabled: export.agent.sandbox_enabled,
            enable_file_read: export.agent.enable_file_read,
            enable_file_write: export.agent.enable_file_write,
            enable_network: export.agent.enable_network,
        };

        repo.create_agent(new_agent)
    }

    /// Get Claude binary path
    pub async fn get_claude_binary_path(&self, pool: Arc<SqlitePool>) -> Result<Option<String>, AgentError> {
        let repo = SqliteAgentRepository::new(pool.as_ref().clone());
        repo.get_setting("claude_binary_path")
    }

    /// Set Claude binary path
    pub async fn set_claude_binary_path(
        &self,
        pool: Arc<SqlitePool>,
        path: String,
    ) -> Result<(), AgentError> {
        let repo = SqliteAgentRepository::new(pool.as_ref().clone());
        repo.set_setting("claude_binary_path", &path)
    }

    /// Resume a paused agent run
    pub async fn resume_agent(
        &self,
        pool: Arc<SqlitePool>,
        run_id: i64,
        registry: Arc<Mutex<ProcessRegistry>>,
    ) -> Result<AgentRun, AgentError> {
        let repo = SqliteAgentRepository::new(pool.as_ref().clone());
        let original_run = repo.find_run_by_id(run_id)?;

        if original_run.status != "paused_usage_limit" {
            return Err(AgentError::Other(
                "Can only resume runs that are paused due to usage limit".to_string(),
            ));
        }

        // Get the last line number from the original run
        let last_line_number = repo.get_last_line_number(run_id)?;

        // Create a new run as a continuation
        let new_run = NewAgentRun {
            agent_id: original_run.agent_id,
            agent_name: original_run.agent_name.clone(),
            agent_icon: original_run.agent_icon.clone(),
            task: format!("[Resumed] {}", original_run.task),
            model: original_run.model.clone(),
            project_path: original_run.project_path.clone(),
            status: Some("pending"),
            scheduled_start_time: None,
            parent_run_id: Some(run_id),
        };

        let resumed_run = repo.create_run(new_run)?;

        // Execute the resumed run
        if let Some(resumed_run_id) = resumed_run.id {
            // Use the execute module to run the agent
            let execute_result = super::execute::execute_agent(
                self.app_handle.clone(),
                pool.clone(),
                registry,
                original_run.agent_id,
                Some(original_run.task.clone()),
                Some(original_run.project_path.clone()),
                Some(resumed_run.session_id.clone()),
                Some(resumed_run_id),
                Some(last_line_number),
            )
            .await?;

            // Return the updated run
            repo.find_run_by_id(resumed_run_id)
        } else {
            Err(AgentError::Other("Failed to create resumed run".to_string()))
        }
    }

    // Helper methods

    /// Get runtime metrics for a run
    async fn get_run_metrics(&self, pool: Arc<SqlitePool>, run: &AgentRun) -> AgentRunMetrics {
        if let Some(run_id) = run.id {
            let repo = SqliteAgentRepository::new(pool.as_ref().clone());
            if let Ok(metrics) = repo.calculate_run_metrics(run_id) {
                return metrics;
            }
        }

        // Return empty metrics if calculation fails
        AgentRunMetrics {
            duration_ms: None,
            total_tokens: None,
            cost_usd: None,
            message_count: None,
        }
    }
}

/// Session status response
#[derive(Debug, serde::Serialize)]
pub struct SessionStatus {
    pub session_id: String,
    pub status: String,
    pub is_alive: bool,
    pub pid: Option<u32>,
}