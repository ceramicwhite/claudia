use rusqlite::params;
use tauri::State;

use super::database::AgentDb;
use super::models::{Agent, AgentRun, AgentRunWithMetrics};
use super::utils::get_agent_run_with_metrics;

/// List all agents
#[tauri::command]
pub async fn list_agents(db: State<'_, AgentDb>) -> Result<Vec<Agent>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, name, icon, system_prompt, default_task, model, sandbox_enabled, enable_file_read, enable_file_write, enable_network, created_at, updated_at FROM agents ORDER BY created_at DESC")
        .map_err(|e| e.to_string())?;

    let agents = stmt
        .query_map([], |row| {
            Ok(Agent {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                icon: row.get(2)?,
                system_prompt: row.get(3)?,
                default_task: row.get(4)?,
                model: row
                    .get::<_, String>(5)
                    .unwrap_or_else(|_| "sonnet".to_string()),
                sandbox_enabled: row.get::<_, bool>(6).unwrap_or(true),
                enable_file_read: row.get::<_, bool>(7).unwrap_or(true),
                enable_file_write: row.get::<_, bool>(8).unwrap_or(true),
                enable_network: row.get::<_, bool>(9).unwrap_or(false),
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(agents)
}

/// Create a new agent
#[tauri::command]
pub async fn create_agent(
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
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let model = model.unwrap_or_else(|| "sonnet".to_string());
    let sandbox_enabled = sandbox_enabled.unwrap_or(true);
    let enable_file_read = enable_file_read.unwrap_or(true);
    let enable_file_write = enable_file_write.unwrap_or(true);
    let enable_network = enable_network.unwrap_or(false);

    conn.execute(
        "INSERT INTO agents (name, icon, system_prompt, default_task, model, sandbox_enabled, enable_file_read, enable_file_write, enable_network) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![name, icon, system_prompt, default_task, model, sandbox_enabled, enable_file_read, enable_file_write, enable_network],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    // Fetch the created agent
    let agent = conn
        .query_row(
            "SELECT id, name, icon, system_prompt, default_task, model, sandbox_enabled, enable_file_read, enable_file_write, enable_network, created_at, updated_at FROM agents WHERE id = ?1",
            params![id],
            |row| {
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
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(agent)
}

/// Update an existing agent
#[tauri::command]
pub async fn update_agent(
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
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let model = model.unwrap_or_else(|| "sonnet".to_string());

    // Build dynamic query based on provided parameters
    let mut query =
        "UPDATE agents SET name = ?1, icon = ?2, system_prompt = ?3, default_task = ?4, model = ?5"
            .to_string();
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![
        Box::new(name),
        Box::new(icon),
        Box::new(system_prompt),
        Box::new(default_task),
        Box::new(model),
    ];
    let mut param_count = 5;

    if let Some(se) = sandbox_enabled {
        param_count += 1;
        query.push_str(&format!(", sandbox_enabled = ?{}", param_count));
        params_vec.push(Box::new(se));
    }
    if let Some(efr) = enable_file_read {
        param_count += 1;
        query.push_str(&format!(", enable_file_read = ?{}", param_count));
        params_vec.push(Box::new(efr));
    }
    if let Some(efw) = enable_file_write {
        param_count += 1;
        query.push_str(&format!(", enable_file_write = ?{}", param_count));
        params_vec.push(Box::new(efw));
    }
    if let Some(en) = enable_network {
        param_count += 1;
        query.push_str(&format!(", enable_network = ?{}", param_count));
        params_vec.push(Box::new(en));
    }

    param_count += 1;
    query.push_str(&format!(" WHERE id = ?{}", param_count));
    params_vec.push(Box::new(id));

    conn.execute(
        &query,
        rusqlite::params_from_iter(params_vec.iter().map(|p| p.as_ref())),
    )
    .map_err(|e| e.to_string())?;

    // Fetch the updated agent
    let agent = conn
        .query_row(
            "SELECT id, name, icon, system_prompt, default_task, model, sandbox_enabled, enable_file_read, enable_file_write, enable_network, created_at, updated_at FROM agents WHERE id = ?1",
            params![id],
            |row| {
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
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(agent)
}

/// Delete an agent
#[tauri::command]
pub async fn delete_agent(db: State<'_, AgentDb>, id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM agents WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Get a single agent by ID
#[tauri::command]
pub async fn get_agent(db: State<'_, AgentDb>, id: i64) -> Result<Agent, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let agent = conn
        .query_row(
            "SELECT id, name, icon, system_prompt, default_task, model, sandbox_enabled, enable_file_read, enable_file_write, enable_network, created_at, updated_at FROM agents WHERE id = ?1",
            params![id],
            |row| {
                Ok(Agent {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    icon: row.get(2)?,
                    system_prompt: row.get(3)?,
                    default_task: row.get(4)?,
                    model: row.get::<_, String>(5).unwrap_or_else(|_| "sonnet".to_string()),
                    sandbox_enabled: row.get::<_, bool>(6).unwrap_or(true),
                    enable_file_read: row.get::<_, bool>(7).unwrap_or(true),
                    enable_file_write: row.get::<_, bool>(8).unwrap_or(true),
                    enable_network: row.get::<_, bool>(9).unwrap_or(false),
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(agent)
}

/// List agent runs (optionally filtered by agent_id)
#[tauri::command]
pub async fn list_agent_runs(
    db: State<'_, AgentDb>,
    agent_id: Option<i64>,
) -> Result<Vec<AgentRun>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let query = if agent_id.is_some() {
        "SELECT id, agent_id, agent_name, agent_icon, task, model, project_path, session_id, status, pid, process_started_at, scheduled_start_time, created_at, completed_at, usage_limit_reset_time, auto_resume_enabled, resume_count, parent_run_id 
         FROM agent_runs WHERE agent_id = ?1 ORDER BY created_at DESC"
    } else {
        "SELECT id, agent_id, agent_name, agent_icon, task, model, project_path, session_id, status, pid, process_started_at, scheduled_start_time, created_at, completed_at, usage_limit_reset_time, auto_resume_enabled, resume_count, parent_run_id 
         FROM agent_runs ORDER BY created_at DESC"
    };

    let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;

    let run_mapper = |row: &rusqlite::Row| -> rusqlite::Result<AgentRun> {
        Ok(AgentRun {
            id: Some(row.get(0)?),
            agent_id: row.get(1)?,
            agent_name: row.get(2)?,
            agent_icon: row.get(3)?,
            task: row.get(4)?,
            model: row.get(5)?,
            project_path: row.get(6)?,
            session_id: row.get(7)?,
            status: row
                .get::<_, String>(8)
                .unwrap_or_else(|_| "pending".to_string()),
            pid: row
                .get::<_, Option<i64>>(9)
                .ok()
                .flatten()
                .map(|p| p as u32),
            process_started_at: row.get(10)?,
            scheduled_start_time: row.get(11)?,
            created_at: row.get(12)?,
            completed_at: row.get(13)?,
            usage_limit_reset_time: row.get(14)?,
            auto_resume_enabled: row.get(15)?,
            resume_count: row.get(16)?,
            parent_run_id: row.get(17)?,
        })
    };

    let runs = if let Some(aid) = agent_id {
        stmt.query_map(params![aid], run_mapper)
    } else {
        stmt.query_map(params![], run_mapper)
    }
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(runs)
}

/// Get a single agent run by ID
#[tauri::command]
pub async fn get_agent_run(db: State<'_, AgentDb>, id: i64) -> Result<AgentRun, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let run = conn
        .query_row(
            "SELECT id, agent_id, agent_name, agent_icon, task, model, project_path, session_id, status, pid, process_started_at, scheduled_start_time, created_at, completed_at, usage_limit_reset_time, auto_resume_enabled, resume_count, parent_run_id 
             FROM agent_runs WHERE id = ?1",
            params![id],
            |row| {
                Ok(AgentRun {
                    id: Some(row.get(0)?),
                    agent_id: row.get(1)?,
                    agent_name: row.get(2)?,
                    agent_icon: row.get(3)?,
                    task: row.get(4)?,
                    model: row.get(5)?,
                    project_path: row.get(6)?,
                    session_id: row.get(7)?,
                    status: row
                        .get::<_, String>(8)
                        .unwrap_or_else(|_| "pending".to_string()),
                    pid: row
                        .get::<_, Option<i64>>(9)
                        .ok()
                        .flatten()
                        .map(|p| p as u32),
                    process_started_at: row.get(10)?,
                    scheduled_start_time: row.get(11)?,
                    created_at: row.get(12)?,
                    completed_at: row.get(13)?,
                    usage_limit_reset_time: row.get(14)?,
                    auto_resume_enabled: row.get(15)?,
                    resume_count: row.get(16)?,
                    parent_run_id: row.get(17)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(run)
}

/// List agent runs with metrics
#[tauri::command]
pub async fn list_agent_runs_with_metrics(
    db: State<'_, AgentDb>,
    agent_id: Option<i64>,
) -> Result<Vec<AgentRunWithMetrics>, String> {
    let runs = list_agent_runs(db, agent_id).await?;
    
    let mut runs_with_metrics = Vec::new();
    for run in runs {
        let with_metrics = get_agent_run_with_metrics(run).await;
        runs_with_metrics.push(with_metrics);
    }
    
    Ok(runs_with_metrics)
}