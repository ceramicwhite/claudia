use crate::commands::agents::{AgentRun, AgentDb};
use crate::process::ProcessRegistryState;
use chrono::Utc;
use log::{debug, error, info, warn};
use rusqlite::params;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
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

/// Check for scheduled runs that need to be executed and execute them
async fn check_and_execute_scheduled_agents(app: &AppHandle) -> Result<(), String> {
    // Get current time in UTC
    let now_utc = Utc::now();
    let now_iso = now_utc.to_rfc3339();
    
    debug!("Checking for scheduled runs at {}", now_iso);
    
    // Find scheduled runs that need to be executed - do all DB work in a block
    let runs_to_execute: Vec<AgentRun> = {
        let db = app.state::<AgentDb>();
        let conn = db.0.lock().map_err(|e| e.to_string())?;
        
        let mut stmt = conn.prepare(
            "SELECT id, agent_id, agent_name, agent_icon, task, model, project_path, session_id, 
                    status, pid, process_started_at, scheduled_start_time, created_at, completed_at 
             FROM agent_runs 
             WHERE status = 'scheduled' 
               AND scheduled_start_time IS NOT NULL 
               AND scheduled_start_time <= ?1
             ORDER BY scheduled_start_time ASC"
        ).map_err(|e| e.to_string())?;
        
        let runs = stmt.query_map(params![now_iso], |row| {
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
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
        
        drop(stmt);
        
        // Update status to prevent re-execution
        for run in &runs {
            if let Some(run_id) = run.id {
                conn.execute(
                    "UPDATE agent_runs SET status = 'pending' WHERE id = ?1",
                    params![run_id]
                ).map_err(|e| e.to_string())?;
            }
        }
        
        runs
    }; // DB connection is dropped here
    
    // Now execute the runs without holding the DB connection
    let db = app.state::<AgentDb>();
    let registry = app.state::<ProcessRegistryState>();
    
    for run in runs_to_execute {
        let run_id = run.id.unwrap();
        let agent_id = run.agent_id;
        let agent_name = run.agent_name.clone();
        let task = run.task.clone();
        let model = run.model.clone();
        let project_path = run.project_path.clone();
        
        info!("Executing scheduled run {} for agent '{}' with task: {}", run_id, agent_name, task);
        
        // Execute the agent
        match crate::commands::agents::execute_agent(
            app.clone(),
            agent_id,
            project_path,
            task,
            Some(model),
            db.clone(),
            registry.clone()
        ).await {
            Ok(new_run_id) => {
                info!("Successfully started scheduled agent '{}' with new run ID: {}", agent_name, new_run_id);
                
                // Mark the original scheduled run as completed
                if let Ok(conn) = db.0.lock() {
                    let _ = conn.execute(
                        "UPDATE agent_runs SET status = 'completed', completed_at = CURRENT_TIMESTAMP WHERE id = ?1",
                        params![run_id]
                    );
                }
            }
            Err(e) => {
                error!("Failed to execute scheduled agent '{}': {}", agent_name, e);
                
                // Mark the scheduled run as failed
                if let Ok(conn) = db.0.lock() {
                    let _ = conn.execute(
                        "UPDATE agent_runs SET status = 'failed', completed_at = CURRENT_TIMESTAMP WHERE id = ?1",
                        params![run_id]
                    );
                }
            }
        }
        
        // Add a small delay between agent executions to avoid overwhelming the system
        sleep(std::time::Duration::from_secs(2)).await;
    }
    
    Ok(())
}

