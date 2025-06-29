use crate::commands::agents::{Agent, AgentDb};
use crate::process::ProcessRegistryState;
use chrono::Utc;
use log::{debug, error, info, warn};
use rusqlite::params;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::sync::Mutex;
use tokio::time::{interval, sleep};

/// Scheduler state for managing scheduled agent executions
pub struct SchedulerState {
    pub is_running: Arc<Mutex<bool>>,
}

impl SchedulerState {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(Mutex::new(false)),
        }
    }
}

/// Start the scheduler to monitor and execute scheduled agents
pub async fn start_scheduler(app: AppHandle) {
    let scheduler_state = app.state::<SchedulerState>();
    let mut is_running = scheduler_state.is_running.lock().await;
    
    if *is_running {
        warn!("Scheduler is already running");
        return;
    }
    
    *is_running = true;
    drop(is_running);
    
    info!("Starting agent scheduler");
    
    // Clone the app handle for the spawned task
    let app_handle = app.clone();
    
    tokio::spawn(async move {
        let mut check_interval = interval(std::time::Duration::from_secs(30)); // Check every 30 seconds
        
        loop {
            check_interval.tick().await;
            
            // Check if scheduler should continue running
            let scheduler_state = app_handle.state::<SchedulerState>();
            let is_running = scheduler_state.is_running.lock().await;
            if !*is_running {
                info!("Scheduler stopped");
                break;
            }
            drop(is_running);
            
            // Check for agents that need to be executed
            if let Err(e) = check_and_execute_scheduled_agents(&app_handle).await {
                error!("Error checking scheduled agents: {}", e);
            }
        }
    });
}

/// Stop the scheduler
pub async fn stop_scheduler(app: &AppHandle) {
    let scheduler_state = app.state::<SchedulerState>();
    let mut is_running = scheduler_state.is_running.lock().await;
    *is_running = false;
    info!("Scheduler stop requested");
}

/// Check for agents that need to be executed and execute them
async fn check_and_execute_scheduled_agents(app: &AppHandle) -> Result<(), String> {
    // Get current time in UTC
    let now_utc = Utc::now();
    let now_iso = now_utc.to_rfc3339();
    
    debug!("Checking for scheduled agents at {}", now_iso);
    
    // Find agents that need to be executed - do all DB work in a block
    let agents_to_execute: Vec<(Agent, String)> = {
        let db = app.state::<AgentDb>();
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        
        let mut stmt = conn.prepare(
            "SELECT id, name, icon, system_prompt, default_task, model, sandbox_enabled, 
                    enable_file_read, enable_file_write, enable_network, scheduled_start_time, 
                    created_at, updated_at 
             FROM agents 
             WHERE scheduled_start_time IS NOT NULL 
               AND scheduled_start_time <= ?1
             ORDER BY scheduled_start_time ASC"
        ).map_err(|e| e.to_string())?;
        
        let agents = stmt.query_map(params![now_iso], |row| {
            Ok(Agent {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                icon: row.get(2)?,
                system_prompt: row.get(3)?,
                default_task: row.get(4)?,
                model: row.get(5)?,
                sandbox_enabled: row.get(6)?,
                enable_file_read: row.get(7)?,
                enable_file_write: row.get(8)?,
                enable_network: row.get(9)?,
                scheduled_start_time: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
        
        drop(stmt);
        
        // Process agents and collect those that should be executed
        let mut agents_with_paths = Vec::new();
        
        for agent in agents {
            let agent_id = agent.id.unwrap();
            let agent_name = agent.name.clone();
            
            info!("Found scheduled agent '{}' (ID: {}) ready to execute", agent_name, agent_id);
            
            // Check if this agent is already running
            let running_runs = conn.query_row(
                "SELECT COUNT(*) FROM agent_runs 
                 WHERE agent_id = ?1 AND status = 'running'",
                params![agent_id],
                |row| row.get::<_, i64>(0)
            ).map_err(|e| e.to_string())?;
            
            if running_runs > 0 {
                warn!("Agent '{}' is already running, skipping scheduled execution", agent_name);
                continue;
            }
            
            // Clear the scheduled_start_time so it doesn't run again
            conn.execute(
                "UPDATE agents SET scheduled_start_time = NULL WHERE id = ?1",
                params![agent_id]
            ).map_err(|e| e.to_string())?;
            
            // Get the project path from the most recent run of this agent, or use home directory
            let project_path = conn.query_row(
                "SELECT project_path FROM agent_runs 
                 WHERE agent_id = ?1 
                 ORDER BY created_at DESC 
                 LIMIT 1",
                params![agent_id],
                |row| row.get::<_, String>(0)
            ).unwrap_or_else(|_| {
                dirs::home_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "/".to_string())
            });
            
            agents_with_paths.push((agent, project_path));
        }
        
        agents_with_paths
    }; // DB connection is dropped here
    
    // Now execute the agents without holding the DB connection
    let db = app.state::<AgentDb>();
    let registry = app.state::<ProcessRegistryState>();
    
    for (agent, project_path) in agents_to_execute {
        let agent_id = agent.id.unwrap();
        let agent_name = agent.name.clone();
        let task = agent.default_task.clone()
            .unwrap_or_else(|| "Scheduled execution".to_string());
        
        info!("Executing scheduled agent '{}' with task: {}", agent_name, task);
        
        // Execute the agent
        match crate::commands::agents::execute_agent(
            app.clone(),
            agent_id,
            project_path,
            task,
            Some(agent.model.clone()),
            db.clone(),
            registry.clone()
        ).await {
            Ok(run_id) => {
                info!("Successfully started scheduled agent '{}' with run ID: {}", agent_name, run_id);
            }
            Err(e) => {
                error!("Failed to execute scheduled agent '{}': {}", agent_name, e);
            }
        }
        
        // Add a small delay between agent executions to avoid overwhelming the system
        sleep(std::time::Duration::from_secs(2)).await;
    }
    
    Ok(())
}

/// Get a list of upcoming scheduled agents
#[tauri::command]
pub async fn get_scheduled_agents(db: State<'_, AgentDb>) -> Result<Vec<Agent>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare(
        "SELECT id, name, icon, system_prompt, default_task, model, sandbox_enabled, 
                enable_file_read, enable_file_write, enable_network, scheduled_start_time, 
                created_at, updated_at 
         FROM agents 
         WHERE scheduled_start_time IS NOT NULL 
         ORDER BY scheduled_start_time ASC"
    ).map_err(|e| e.to_string())?;
    
    let agents = stmt.query_map([], |row| {
        Ok(Agent {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            icon: row.get(2)?,
            system_prompt: row.get(3)?,
            default_task: row.get(4)?,
            model: row.get(5)?,
            sandbox_enabled: row.get(6)?,
            enable_file_read: row.get(7)?,
            enable_file_write: row.get(8)?,
            enable_network: row.get(9)?,
            scheduled_start_time: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;
    
    Ok(agents)
}

/// Clear the scheduled start time for an agent
#[tauri::command]
pub async fn clear_agent_schedule(db: State<'_, AgentDb>, agent_id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    
    conn.execute(
        "UPDATE agents SET scheduled_start_time = NULL WHERE id = ?1",
        params![agent_id]
    ).map_err(|e| e.to_string())?;
    
    info!("Cleared schedule for agent ID: {}", agent_id);
    Ok(())
}