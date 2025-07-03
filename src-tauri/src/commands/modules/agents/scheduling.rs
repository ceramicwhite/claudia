use rusqlite::params;
use tauri::State;

use super::database::AgentDb;
use super::models::AgentRun;

/// Create a scheduled agent run
#[tauri::command]
pub async fn create_scheduled_agent_run(
    db: State<'_, AgentDb>,
    agent_id: i64,
    project_path: String,
    task: String,
    model: String,
    scheduled_start_time: String,
) -> Result<i64, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    
    // Get the agent details
    let (agent_name, agent_icon) = conn
        .query_row(
            "SELECT name, icon FROM agents WHERE id = ?1",
            params![agent_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .map_err(|e| format!("Failed to find agent: {}", e))?;
    
    // Create a run record with 'scheduled' status
    conn.execute(
        "INSERT INTO agent_runs (agent_id, agent_name, agent_icon, task, model, project_path, session_id, status, scheduled_start_time) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![agent_id, agent_name, agent_icon, task, model, project_path, "", "scheduled", scheduled_start_time],
    )
    .map_err(|e| format!("Failed to create scheduled run: {}", e))?;
    
    let run_id = conn.last_insert_rowid();
    log::info!("Created scheduled agent run {} for agent {} at {}", run_id, agent_id, scheduled_start_time);
    
    Ok(run_id)
}

/// Get a list of all scheduled agent runs
#[tauri::command]
pub async fn get_scheduled_agent_runs(db: State<'_, AgentDb>) -> Result<Vec<AgentRun>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = conn
        .prepare(
            "SELECT id, agent_id, agent_name, agent_icon, task, model, project_path, session_id, status, pid, process_started_at, scheduled_start_time, created_at, completed_at, usage_limit_reset_time, auto_resume_enabled, resume_count, parent_run_id 
             FROM agent_runs 
             WHERE status = 'scheduled' AND scheduled_start_time IS NOT NULL 
             ORDER BY scheduled_start_time ASC"
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
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    
    Ok(runs)
}

/// Cancel a scheduled agent run
#[tauri::command]
pub async fn cancel_scheduled_agent_run(db: State<'_, AgentDb>, run_id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    
    conn.execute(
        "UPDATE agent_runs SET status = 'cancelled', completed_at = CURRENT_TIMESTAMP WHERE id = ?1 AND status = 'scheduled'",
        params![run_id],
    )
    .map_err(|e| format!("Failed to cancel scheduled run: {}", e))?;
    
    log::info!("Cancelled scheduled agent run {}", run_id);
    Ok(())
}