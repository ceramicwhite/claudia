use crate::commands::agents::{
    constants, error::AgentError, pool::SqlitePool, types::*, AgentRunMetrics,
};
use anyhow::Result;
use chrono;
use rusqlite::params;
use serde_json::Value as JsonValue;

/// Repository trait for database operations
pub trait AgentRepository {
    fn find_all_agents(&self) -> Result<Vec<Agent>, AgentError>;
    fn find_agent_by_id(&self, id: i64) -> Result<Agent, AgentError>;
    fn create_agent(&self, agent: NewAgent) -> Result<Agent, AgentError>;
    fn update_agent(&self, id: i64, agent: UpdateAgent) -> Result<Agent, AgentError>;
    fn delete_agent(&self, id: i64) -> Result<(), AgentError>;

    fn find_all_runs(&self, agent_id: Option<i64>) -> Result<Vec<AgentRun>, AgentError>;
    fn find_run_by_id(&self, id: i64) -> Result<AgentRun, AgentError>;
    fn find_run_by_session_id(&self, session_id: &str) -> Result<Option<AgentRun>, AgentError>;
    fn find_running_runs(&self) -> Result<Vec<AgentRun>, AgentError>;
    fn find_scheduled_runs(&self) -> Result<Vec<AgentRun>, AgentError>;

    fn create_run(&self, run: NewAgentRun) -> Result<AgentRun, AgentError>;
    fn update_run_status(
        &self,
        id: i64,
        status: &str,
        pid: Option<u32>,
        started_at: Option<String>,
    ) -> Result<(), AgentError>;
    fn update_run_completion(&self, id: i64, status: &str) -> Result<(), AgentError>;
    fn update_run_usage_limit(
        &self,
        id: i64,
        reset_time: &str,
        auto_resume: bool,
    ) -> Result<(), AgentError>;

    fn store_jsonl_output(
        &self,
        run_id: i64,
        line_number: i64,
        content: &str,
    ) -> Result<(), AgentError>;
    fn get_jsonl_output(&self, run_id: i64) -> Result<String, AgentError>;
    fn get_last_line_number(&self, run_id: i64) -> Result<i64, AgentError>;

    #[allow(dead_code)]
    fn store_sandbox_violation(&self, violation: SandboxViolation) -> Result<(), AgentError>;

    fn get_setting(&self, key: &str) -> Result<Option<String>, AgentError>;
    fn set_setting(&self, key: &str, value: &str) -> Result<(), AgentError>;

    fn calculate_run_metrics(&self, run_id: i64) -> Result<AgentRunMetrics, AgentError>;
}

/// SQL repository implementation
pub struct SqliteAgentRepository {
    pool: SqlitePool,
}

impl SqliteAgentRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    
    // Helper method to get a connection from the pool
    fn get_conn(&self) -> Result<r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>, AgentError> {
        self.pool.get()
            .map_err(|e| AgentError::Other(format!("Failed to get connection from pool: {}", e)))
    }
}

impl AgentRepository for SqliteAgentRepository {
    
    fn find_all_agents(&self) -> Result<Vec<Agent>, AgentError> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, icon, system_prompt, default_task, model, sandbox_enabled, 
             enable_file_read, enable_file_write, enable_network, created_at, updated_at 
             FROM agents ORDER BY created_at DESC",
        )?;

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
                        .unwrap_or_else(|_| constants::DEFAULT_MODEL.to_string()),
                    sandbox_enabled: row
                        .get::<_, bool>(6)
                        .unwrap_or(constants::DEFAULT_SANDBOX_ENABLED),
                    enable_file_read: row
                        .get::<_, bool>(7)
                        .unwrap_or(constants::DEFAULT_FILE_READ_ENABLED),
                    enable_file_write: row
                        .get::<_, bool>(8)
                        .unwrap_or(constants::DEFAULT_FILE_WRITE_ENABLED),
                    enable_network: row
                        .get::<_, bool>(9)
                        .unwrap_or(constants::DEFAULT_NETWORK_ENABLED),
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(agents)
    }

    fn find_agent_by_id(&self, id: i64) -> Result<Agent, AgentError> {
        let conn = self.get_conn()?;
        conn.query_row(
                "SELECT id, name, icon, system_prompt, default_task, model, sandbox_enabled, 
                 enable_file_read, enable_file_write, enable_network, created_at, updated_at 
                 FROM agents WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Agent {
                        id: Some(row.get(0)?),
                        name: row.get(1)?,
                        icon: row.get(2)?,
                        system_prompt: row.get(3)?,
                        default_task: row.get(4)?,
                        model: row.get::<_, String>(5).unwrap_or_else(|_| {
                            constants::DEFAULT_MODEL.to_string()
                        }),
                        sandbox_enabled: row
                            .get::<_, bool>(6)
                            .unwrap_or(constants::DEFAULT_SANDBOX_ENABLED),
                        enable_file_read: row
                            .get::<_, bool>(7)
                            .unwrap_or(constants::DEFAULT_FILE_READ_ENABLED),
                        enable_file_write: row
                            .get::<_, bool>(8)
                            .unwrap_or(constants::DEFAULT_FILE_WRITE_ENABLED),
                        enable_network: row
                            .get::<_, bool>(9)
                            .unwrap_or(constants::DEFAULT_NETWORK_ENABLED),
                        created_at: row.get(10)?,
                        updated_at: row.get(11)?,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => AgentError::AgentNotFound(id),
                _ => AgentError::Database(e),
            })
    }

    fn create_agent(&self, agent: NewAgent) -> Result<Agent, AgentError> {
        let conn = self.get_conn()?;
        conn.execute(
            "INSERT INTO agents (name, icon, system_prompt, default_task, model, sandbox_enabled, 
             enable_file_read, enable_file_write, enable_network) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                agent.name,
                agent.icon,
                agent.system_prompt,
                agent.default_task,
                agent.model,
                agent.sandbox_enabled,
                agent.enable_file_read,
                agent.enable_file_write,
                agent.enable_network
            ],
        )?;

        let id = conn.last_insert_rowid();
        self.find_agent_by_id(id)
    }

    fn update_agent(&self, id: i64, agent: UpdateAgent) -> Result<Agent, AgentError> {
        let conn = self.get_conn()?;
        // Build dynamic query based on provided parameters
        let mut query =
            "UPDATE agents SET name = ?1, icon = ?2, system_prompt = ?3, default_task = ?4, model = ?5"
                .to_string();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![
            Box::new(agent.name),
            Box::new(agent.icon),
            Box::new(agent.system_prompt),
            Box::new(agent.default_task),
            Box::new(agent.model),
        ];
        let mut param_count = 5;

        if let Some(se) = agent.sandbox_enabled {
            param_count += 1;
            query.push_str(&format!(", sandbox_enabled = ?{}", param_count));
            params_vec.push(Box::new(se));
        }
        if let Some(efr) = agent.enable_file_read {
            param_count += 1;
            query.push_str(&format!(", enable_file_read = ?{}", param_count));
            params_vec.push(Box::new(efr));
        }
        if let Some(efw) = agent.enable_file_write {
            param_count += 1;
            query.push_str(&format!(", enable_file_write = ?{}", param_count));
            params_vec.push(Box::new(efw));
        }
        if let Some(en) = agent.enable_network {
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
        )?;

        self.find_agent_by_id(id)
    }

    fn delete_agent(&self, id: i64) -> Result<(), AgentError> {
        let conn = self.get_conn()?;
        conn.execute("DELETE FROM agents WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn find_all_runs(&self, agent_id: Option<i64>) -> Result<Vec<AgentRun>, AgentError> {
        let conn = self.get_conn()?;
        let query = if agent_id.is_some() {
            "SELECT id, agent_id, agent_name, agent_icon, task, model, project_path, session_id, 
             status, pid, process_started_at, scheduled_start_time, created_at, completed_at, 
             usage_limit_reset_time, auto_resume_enabled, resume_count, parent_run_id 
             FROM agent_runs WHERE agent_id = ?1 ORDER BY created_at DESC"
        } else {
            "SELECT id, agent_id, agent_name, agent_icon, task, model, project_path, session_id, 
             status, pid, process_started_at, scheduled_start_time, created_at, completed_at, 
             usage_limit_reset_time, auto_resume_enabled, resume_count, parent_run_id 
             FROM agent_runs ORDER BY created_at DESC"
        };

        let mut stmt = conn.prepare(query)?;

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
        }?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(runs)
    }

    fn find_run_by_id(&self, id: i64) -> Result<AgentRun, AgentError> {
        let conn = self.get_conn()?;
        conn.query_row(
                "SELECT id, agent_id, agent_name, agent_icon, task, model, project_path, session_id, 
                 status, pid, process_started_at, scheduled_start_time, created_at, completed_at, 
                 usage_limit_reset_time, auto_resume_enabled, resume_count, parent_run_id 
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
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    AgentError::Other(format!("Agent run with id {} not found", id))
                }
                _ => AgentError::Database(e),
            })
    }

    fn find_run_by_session_id(&self, session_id: &str) -> Result<Option<AgentRun>, AgentError> {
        let conn = self.get_conn()?;
        match conn.query_row(
            "SELECT id, agent_id, agent_name, agent_icon, task, model, project_path, session_id, 
             status, pid, process_started_at, scheduled_start_time, created_at, completed_at, 
             usage_limit_reset_time, auto_resume_enabled, resume_count, parent_run_id 
             FROM agent_runs WHERE session_id = ?1",
            params![session_id],
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
        ) {
            Ok(run) => Ok(Some(run)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AgentError::Database(e)),
        }
    }

    fn find_running_runs(&self) -> Result<Vec<AgentRun>, AgentError> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, agent_id, agent_name, agent_icon, task, model, project_path, session_id, 
             status, pid, process_started_at, scheduled_start_time, created_at, completed_at, 
             usage_limit_reset_time, auto_resume_enabled, resume_count, parent_run_id 
             FROM agent_runs WHERE status = 'running' ORDER BY created_at DESC",
        )?;

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
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(runs)
    }

    fn find_scheduled_runs(&self) -> Result<Vec<AgentRun>, AgentError> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, agent_id, agent_name, agent_icon, task, model, project_path, session_id, 
             status, pid, process_started_at, scheduled_start_time, created_at, completed_at, 
             usage_limit_reset_time, auto_resume_enabled, resume_count, parent_run_id 
             FROM agent_runs WHERE status = 'scheduled' ORDER BY scheduled_start_time ASC",
        )?;

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
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(runs)
    }

    fn create_run(&self, run: NewAgentRun) -> Result<AgentRun, AgentError> {
        let conn = self.get_conn()?;
        let session_id = uuid::Uuid::new_v4().to_string();

        conn.execute(
            "INSERT INTO agent_runs (agent_id, agent_name, agent_icon, task, model, project_path, 
             session_id, status, scheduled_start_time, parent_run_id) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                run.agent_id,
                run.agent_name,
                run.agent_icon,
                run.task,
                run.model,
                run.project_path,
                session_id,
                run.status.unwrap_or("pending"),
                run.scheduled_start_time,
                run.parent_run_id
            ],
        )?;

        let id = conn.last_insert_rowid();
        self.find_run_by_id(id)
    }

    fn update_run_status(
        &self,
        id: i64,
        status: &str,
        pid: Option<u32>,
        started_at: Option<String>,
    ) -> Result<(), AgentError> {
        let conn = self.get_conn()?;
        if let (Some(p), Some(started)) = (pid, started_at) {
            conn.execute(
                "UPDATE agent_runs SET status = ?1, pid = ?2, process_started_at = ?3 WHERE id = ?4",
                params![status, p as i64, started, id],
            )?;
        } else {
            conn.execute(
                "UPDATE agent_runs SET status = ?1 WHERE id = ?2",
                params![status, id],
            )?;
        }
        Ok(())
    }

    fn update_run_completion(&self, id: i64, status: &str) -> Result<(), AgentError> {
        let conn = self.get_conn()?;
        let completed_at = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE agent_runs SET status = ?1, completed_at = ?2 WHERE id = ?3",
            params![status, completed_at, id],
        )?;
        Ok(())
    }

    fn update_run_usage_limit(
        &self,
        id: i64,
        reset_time: &str,
        auto_resume: bool,
    ) -> Result<(), AgentError> {
        let conn = self.get_conn()?;
        conn.execute(
            "UPDATE agent_runs SET status = 'paused_usage_limit', usage_limit_reset_time = ?1, 
             auto_resume_enabled = ?2 WHERE id = ?3",
            params![reset_time, auto_resume, id],
        )?;
        Ok(())
    }

    fn store_jsonl_output(
        &self,
        run_id: i64,
        line_number: i64,
        content: &str,
    ) -> Result<(), AgentError> {
        let conn = self.get_conn()?;
        let timestamp = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT OR REPLACE INTO jsonl_outputs (run_id, line_number, timestamp, content) 
             VALUES (?1, ?2, ?3, ?4)",
            params![run_id, line_number, timestamp, content],
        )?;
        Ok(())
    }

    fn get_jsonl_output(&self, run_id: i64) -> Result<String, AgentError> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT content FROM jsonl_outputs WHERE run_id = ?1 ORDER BY line_number ASC",
        )?;

        let lines = stmt
            .query_map(params![run_id], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(lines.join("\n"))
    }

    fn get_last_line_number(&self, run_id: i64) -> Result<i64, AgentError> {
        let conn = self.get_conn()?;
        let line_number: Option<i64> = conn.query_row(
            "SELECT MAX(line_number) FROM jsonl_outputs WHERE run_id = ?1",
            params![run_id],
            |row| row.get(0),
        )?;

        Ok(line_number.unwrap_or(0))
    }

    fn store_sandbox_violation(&self, violation: SandboxViolation) -> Result<(), AgentError> {
        let conn = self.get_conn()?;
        conn.execute(
            "INSERT INTO sandbox_violations (run_id, operation_type, resource, reason) 
             VALUES (?1, ?2, ?3, ?4)",
            params![
                violation.run_id,
                violation.operation_type,
                violation.resource,
                violation.reason
            ],
        )?;
        Ok(())
    }

    fn get_setting(&self, key: &str) -> Result<Option<String>, AgentError> {
        let conn = self.get_conn()?;
        match conn.query_row(
            "SELECT value FROM app_settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        ) {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AgentError::Database(e)),
        }
    }

    fn set_setting(&self, key: &str, value: &str) -> Result<(), AgentError> {
        let conn = self.get_conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    fn calculate_run_metrics(&self, run_id: i64) -> Result<AgentRunMetrics, AgentError> {
        let conn = self.get_conn()?;
        // Get all JSONL content for the run
        let mut stmt = conn.prepare(
            "SELECT content FROM jsonl_outputs WHERE run_id = ?1 ORDER BY line_number ASC",
        )?;

        let lines = stmt
            .query_map(params![run_id], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;

        let mut total_tokens = 0i64;
        let mut cost_usd = 0.0f64;
        let mut message_count = 0i64;
        let mut start_time: Option<chrono::DateTime<chrono::Utc>> = None;
        let mut end_time: Option<chrono::DateTime<chrono::Utc>> = None;

        for line in lines {
            if let Ok(json) = serde_json::from_str::<JsonValue>(&line) {
                // Track timestamps
                if let Some(timestamp_str) = json.get("timestamp").and_then(|t| t.as_str()) {
                    if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(timestamp_str) {
                        let timestamp = timestamp.with_timezone(&chrono::Utc);
                        match start_time {
                            None => start_time = Some(timestamp),
                            Some(existing) if timestamp < existing => start_time = Some(timestamp),
                            _ => {}
                        }
                        match end_time {
                            None => end_time = Some(timestamp),
                            Some(existing) if timestamp > existing => end_time = Some(timestamp),
                            _ => {}
                        }
                    }
                }

                // Count messages
                if json.get("type").and_then(|t| t.as_str()) == Some("text") {
                    message_count += 1;
                }

                // Sum usage stats
                if let Some(usage) = json.get("usage") {
                    if let Some(total) = usage.get("totalTokens").and_then(|t| t.as_i64()) {
                        total_tokens += total;
                    }
                    if let Some(cache_read) = usage.get("cacheReadTokens").and_then(|t| t.as_i64())
                    {
                        // Add cache read tokens to total if not included
                        total_tokens += cache_read;
                    }
                }

                // Sum costs
                if let Some(cost) = json.get("cost").and_then(|c| c.as_f64()) {
                    cost_usd += cost;
                }
            }
        }

        // Calculate duration
        let duration_ms = match (start_time, end_time) {
            (Some(start), Some(end)) => Some((end - start).num_milliseconds()),
            _ => None,
        };

        Ok(AgentRunMetrics {
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
        })
    }
}

/// Create new agent input
#[derive(Debug)]
pub struct NewAgent {
    pub name: String,
    pub icon: String,
    pub system_prompt: String,
    pub default_task: Option<String>,
    pub model: String,
    pub sandbox_enabled: bool,
    pub enable_file_read: bool,
    pub enable_file_write: bool,
    pub enable_network: bool,
}

/// Update agent input
#[derive(Debug)]
pub struct UpdateAgent {
    pub name: String,
    pub icon: String,
    pub system_prompt: String,
    pub default_task: Option<String>,
    pub model: String,
    pub sandbox_enabled: Option<bool>,
    pub enable_file_read: Option<bool>,
    pub enable_file_write: Option<bool>,
    pub enable_network: Option<bool>,
}

/// Create new agent run input
#[derive(Debug)]
pub struct NewAgentRun {
    pub agent_id: i64,
    pub agent_name: String,
    pub agent_icon: String,
    pub task: String,
    pub model: String,
    pub project_path: String,
    pub status: Option<&'static str>,
    pub scheduled_start_time: Option<String>,
    pub parent_run_id: Option<i64>,
}

/// Sandbox violation record
#[derive(Debug)]
pub struct SandboxViolation {
    pub run_id: i64,
    pub operation_type: String,
    pub resource: String,
    pub reason: String,
}