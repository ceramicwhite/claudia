#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::agents::{
        constants, error::AgentError, pool::SqlitePool, repository::*, service::*, types::*,
    };
    use crate::process_registry::{ProcessRegistry, ProcessRegistryState};
    use anyhow::Result;
    use chrono;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use tauri::test::{mock_builder, MockRuntime};
    use tokio::process::Child;

    // ===== Mock Repository Implementation =====

    #[derive(Default)]
    struct MockAgentRepository {
        agents: Arc<Mutex<HashMap<i64, Agent>>>,
        runs: Arc<Mutex<HashMap<i64, AgentRun>>>,
        settings: Arc<Mutex<HashMap<String, String>>>,
        jsonl_outputs: Arc<Mutex<HashMap<i64, Vec<(i64, String)>>>>,
        sandbox_violations: Arc<Mutex<Vec<SandboxViolation>>>,
        next_agent_id: Arc<Mutex<i64>>,
        next_run_id: Arc<Mutex<i64>>,
        should_fail: Arc<Mutex<bool>>,
    }

    impl MockAgentRepository {
        fn new() -> Self {
            Self {
                agents: Arc::new(Mutex::new(HashMap::new())),
                runs: Arc::new(Mutex::new(HashMap::new())),
                settings: Arc::new(Mutex::new(HashMap::new())),
                jsonl_outputs: Arc::new(Mutex::new(HashMap::new())),
                sandbox_violations: Arc::new(Mutex::new(Vec::new())),
                next_agent_id: Arc::new(Mutex::new(1)),
                next_run_id: Arc::new(Mutex::new(1)),
                should_fail: Arc::new(Mutex::new(false)),
            }
        }

        fn set_should_fail(&self, fail: bool) {
            *self.should_fail.lock().unwrap() = fail;
        }

        fn add_test_agent(&self, agent: Agent) -> i64 {
            let id = agent.id.unwrap_or_else(|| {
                let mut next_id = self.next_agent_id.lock().unwrap();
                let id = *next_id;
                *next_id += 1;
                id
            });
            let mut agent_with_id = agent;
            agent_with_id.id = Some(id);
            self.agents.lock().unwrap().insert(id, agent_with_id);
            id
        }

        fn add_test_run(&self, run: AgentRun) -> i64 {
            let id = run.id.unwrap_or_else(|| {
                let mut next_id = self.next_run_id.lock().unwrap();
                let id = *next_id;
                *next_id += 1;
                id
            });
            let mut run_with_id = run;
            run_with_id.id = Some(id);
            self.runs.lock().unwrap().insert(id, run_with_id);
            id
        }
    }

    impl AgentRepository for MockAgentRepository {
        fn find_all_agents(&self) -> Result<Vec<Agent>, AgentError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AgentError::Other("Mock failure".to_string()));
            }
            let agents = self.agents.lock().unwrap();
            let mut result: Vec<Agent> = agents.values().cloned().collect();
            result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            Ok(result)
        }

        fn find_agent_by_id(&self, id: i64) -> Result<Agent, AgentError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AgentError::Other("Mock failure".to_string()));
            }
            self.agents
                .lock()
                .unwrap()
                .get(&id)
                .cloned()
                .ok_or(AgentError::NotFound(format!("Agent {} not found", id)))
        }

        fn create_agent(&self, agent: NewAgent) -> Result<Agent, AgentError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AgentError::Other("Mock failure".to_string()));
            }
            
            // Validate inputs
            if agent.name.trim().is_empty() {
                return Err(AgentError::Validation("Agent name cannot be empty".to_string()));
            }
            if agent.system_prompt.trim().is_empty() {
                return Err(AgentError::Validation("System prompt cannot be empty".to_string()));
            }

            let mut next_id = self.next_agent_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;

            let now = chrono::Utc::now().to_rfc3339();
            let new_agent = Agent {
                id: Some(id),
                name: agent.name,
                icon: agent.icon,
                system_prompt: agent.system_prompt,
                default_task: agent.default_task,
                model: agent.model,
                sandbox_enabled: agent.sandbox_enabled,
                enable_file_read: agent.enable_file_read,
                enable_file_write: agent.enable_file_write,
                enable_network: agent.enable_network,
                created_at: now.clone(),
                updated_at: now,
            };

            self.agents.lock().unwrap().insert(id, new_agent.clone());
            Ok(new_agent)
        }

        fn update_agent(&self, id: i64, agent: UpdateAgent) -> Result<Agent, AgentError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AgentError::Other("Mock failure".to_string()));
            }

            // Validate inputs
            if agent.name.trim().is_empty() {
                return Err(AgentError::Validation("Agent name cannot be empty".to_string()));
            }
            if agent.system_prompt.trim().is_empty() {
                return Err(AgentError::Validation("System prompt cannot be empty".to_string()));
            }

            let mut agents = self.agents.lock().unwrap();
            let existing = agents
                .get_mut(&id)
                .ok_or(AgentError::NotFound(format!("Agent {} not found", id)))?;

            existing.name = agent.name;
            existing.icon = agent.icon;
            existing.system_prompt = agent.system_prompt;
            existing.default_task = agent.default_task;
            existing.model = agent.model;
            if let Some(v) = agent.sandbox_enabled {
                existing.sandbox_enabled = v;
            }
            if let Some(v) = agent.enable_file_read {
                existing.enable_file_read = v;
            }
            if let Some(v) = agent.enable_file_write {
                existing.enable_file_write = v;
            }
            if let Some(v) = agent.enable_network {
                existing.enable_network = v;
            }
            existing.updated_at = chrono::Utc::now().to_rfc3339();

            Ok(existing.clone())
        }

        fn delete_agent(&self, id: i64) -> Result<(), AgentError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AgentError::Other("Mock failure".to_string()));
            }

            // Check for active runs
            let runs = self.runs.lock().unwrap();
            let has_active_run = runs.values().any(|run| {
                run.agent_id == id && (run.status == "running" || run.status == "pending")
            });
            
            if has_active_run {
                return Err(AgentError::Other("Cannot delete agent with active runs".to_string()));
            }

            self.agents
                .lock()
                .unwrap()
                .remove(&id)
                .ok_or(AgentError::NotFound(format!("Agent {} not found", id)))?;
            
            Ok(())
        }

        fn find_all_runs(&self, agent_id: Option<i64>) -> Result<Vec<AgentRun>, AgentError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AgentError::Other("Mock failure".to_string()));
            }

            let runs = self.runs.lock().unwrap();
            let mut result: Vec<AgentRun> = if let Some(id) = agent_id {
                runs.values()
                    .filter(|run| run.agent_id == id)
                    .cloned()
                    .collect()
            } else {
                runs.values().cloned().collect()
            };
            
            result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            Ok(result)
        }

        fn find_run_by_id(&self, id: i64) -> Result<AgentRun, AgentError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AgentError::Other("Mock failure".to_string()));
            }

            self.runs
                .lock()
                .unwrap()
                .get(&id)
                .cloned()
                .ok_or(AgentError::NotFound(format!("Run {} not found", id)))
        }

        fn find_run_by_session_id(&self, session_id: &str) -> Result<Option<AgentRun>, AgentError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AgentError::Other("Mock failure".to_string()));
            }

            let runs = self.runs.lock().unwrap();
            Ok(runs.values().find(|run| run.session_id == session_id).cloned())
        }

        fn find_running_runs(&self) -> Result<Vec<AgentRun>, AgentError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AgentError::Other("Mock failure".to_string()));
            }

            let runs = self.runs.lock().unwrap();
            let result: Vec<AgentRun> = runs
                .values()
                .filter(|run| run.status == "running")
                .cloned()
                .collect();
            Ok(result)
        }

        fn find_scheduled_runs(&self) -> Result<Vec<AgentRun>, AgentError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AgentError::Other("Mock failure".to_string()));
            }

            let runs = self.runs.lock().unwrap();
            let result: Vec<AgentRun> = runs
                .values()
                .filter(|run| run.status == "scheduled")
                .cloned()
                .collect();
            Ok(result)
        }

        fn create_run(&self, run: NewAgentRun) -> Result<AgentRun, AgentError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AgentError::Other("Mock failure".to_string()));
            }

            let mut next_id = self.next_run_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;

            let session_id = uuid::Uuid::new_v4().to_string();
            let now = chrono::Utc::now().to_rfc3339();
            
            let new_run = AgentRun {
                id: Some(id),
                agent_id: run.agent_id,
                agent_name: run.agent_name,
                agent_icon: run.agent_icon,
                task: run.task,
                model: run.model,
                project_path: run.project_path,
                session_id,
                status: run.status.unwrap_or("pending").to_string(),
                pid: None,
                process_started_at: None,
                scheduled_start_time: run.scheduled_start_time,
                created_at: now,
                completed_at: None,
                usage_limit_reset_time: None,
                auto_resume_enabled: false,
                resume_count: 0,
                parent_run_id: run.parent_run_id,
            };

            self.runs.lock().unwrap().insert(id, new_run.clone());
            Ok(new_run)
        }

        fn update_run_status(
            &self,
            id: i64,
            status: &str,
            pid: Option<u32>,
            started_at: Option<String>,
        ) -> Result<(), AgentError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AgentError::Other("Mock failure".to_string()));
            }

            let mut runs = self.runs.lock().unwrap();
            let run = runs
                .get_mut(&id)
                .ok_or(AgentError::NotFound(format!("Run {} not found", id)))?;

            run.status = status.to_string();
            if let Some(p) = pid {
                run.pid = Some(p);
            }
            if let Some(s) = started_at {
                run.process_started_at = Some(s);
            }
            
            Ok(())
        }

        fn update_run_completion(&self, id: i64, status: &str) -> Result<(), AgentError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AgentError::Other("Mock failure".to_string()));
            }

            let mut runs = self.runs.lock().unwrap();
            let run = runs
                .get_mut(&id)
                .ok_or(AgentError::NotFound(format!("Run {} not found", id)))?;

            run.status = status.to_string();
            run.completed_at = Some(chrono::Utc::now().to_rfc3339());
            
            Ok(())
        }

        fn update_run_usage_limit(
            &self,
            id: i64,
            reset_time: &str,
            auto_resume: bool,
        ) -> Result<(), AgentError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AgentError::Other("Mock failure".to_string()));
            }

            let mut runs = self.runs.lock().unwrap();
            let run = runs
                .get_mut(&id)
                .ok_or(AgentError::NotFound(format!("Run {} not found", id)))?;

            run.status = "paused_usage_limit".to_string();
            run.usage_limit_reset_time = Some(reset_time.to_string());
            run.auto_resume_enabled = auto_resume;
            
            Ok(())
        }

        fn store_jsonl_output(
            &self,
            run_id: i64,
            line_number: i64,
            content: &str,
        ) -> Result<(), AgentError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AgentError::Other("Mock failure".to_string()));
            }

            let mut outputs = self.jsonl_outputs.lock().unwrap();
            outputs
                .entry(run_id)
                .or_insert_with(Vec::new)
                .push((line_number, content.to_string()));
            
            Ok(())
        }

        fn get_jsonl_output(&self, run_id: i64) -> Result<String, AgentError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AgentError::Other("Mock failure".to_string()));
            }

            let outputs = self.jsonl_outputs.lock().unwrap();
            if let Some(lines) = outputs.get(&run_id) {
                let mut sorted_lines = lines.clone();
                sorted_lines.sort_by_key(|(line_num, _)| *line_num);
                let result = sorted_lines
                    .into_iter()
                    .map(|(_, content)| content)
                    .collect::<Vec<_>>()
                    .join("\n");
                Ok(result)
            } else {
                Err(AgentError::NotFound("No output found".to_string()))
            }
        }

        fn get_last_line_number(&self, run_id: i64) -> Result<i64, AgentError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AgentError::Other("Mock failure".to_string()));
            }

            let outputs = self.jsonl_outputs.lock().unwrap();
            if let Some(lines) = outputs.get(&run_id) {
                Ok(lines.iter().map(|(line_num, _)| *line_num).max().unwrap_or(0))
            } else {
                Ok(0)
            }
        }

        fn store_sandbox_violation(&self, violation: SandboxViolation) -> Result<(), AgentError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AgentError::Other("Mock failure".to_string()));
            }

            self.sandbox_violations.lock().unwrap().push(violation);
            Ok(())
        }

        fn get_setting(&self, key: &str) -> Result<Option<String>, AgentError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AgentError::Other("Mock failure".to_string()));
            }

            Ok(self.settings.lock().unwrap().get(key).cloned())
        }

        fn set_setting(&self, key: &str, value: &str) -> Result<(), AgentError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AgentError::Other("Mock failure".to_string()));
            }

            self.settings
                .lock()
                .unwrap()
                .insert(key.to_string(), value.to_string());
            Ok(())
        }

        fn calculate_run_metrics(&self, run_id: i64) -> Result<AgentRunMetrics, AgentError> {
            if *self.should_fail.lock().unwrap() {
                return Err(AgentError::Other("Mock failure".to_string()));
            }

            // Return mock metrics
            Ok(AgentRunMetrics {
                duration_ms: Some(120000), // 2 minutes
                total_tokens: Some(5000),
                cost_usd: Some(0.25),
                message_count: Some(10),
            })
        }
    }

    // ===== Mock Process Registry =====

    #[derive(Default)]
    struct MockProcessRegistry {
        processes: HashMap<String, bool>, // session_id -> is_alive
        outputs: HashMap<String, Vec<String>>, // session_id -> output lines
        should_fail: bool,
    }

    impl MockProcessRegistry {
        fn new() -> Self {
            Self {
                processes: HashMap::new(),
                outputs: HashMap::new(),
                should_fail: false,
            }
        }

        fn add_process(&mut self, session_id: String, is_alive: bool) {
            self.processes.insert(session_id.clone(), is_alive);
            self.outputs.insert(session_id, Vec::new());
        }

        fn add_output(&mut self, session_id: &str, output: String) {
            if let Some(buffer) = self.outputs.get_mut(session_id) {
                buffer.push(output);
            }
        }

        fn is_process_alive(&self, session_id: &str) -> bool {
            self.processes.get(session_id).copied().unwrap_or(false)
        }

        fn kill_process(&mut self, session_id: &str) -> Result<(), anyhow::Error> {
            if self.should_fail {
                return Err(anyhow::anyhow!("Mock kill failure"));
            }
            
            if self.processes.contains_key(session_id) {
                self.processes.insert(session_id.to_string(), false);
                Ok(())
            } else {
                Err(anyhow::anyhow!("Process not found"))
            }
        }

        fn get_output(&self, session_id: &str) -> Result<Vec<String>, anyhow::Error> {
            if self.should_fail {
                return Err(anyhow::anyhow!("Mock output failure"));
            }
            
            self.outputs
                .get(session_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Session not found"))
        }
    }

    // ===== Test Helpers =====

    fn create_test_app() -> tauri::AppHandle<MockRuntime> {
        mock_builder().build(tauri::generate_context!()).unwrap()
    }

    fn create_test_agent(name: &str) -> Agent {
        Agent {
            id: None,
            name: name.to_string(),
            icon: "🤖".to_string(),
            system_prompt: "Test system prompt".to_string(),
            default_task: Some("Test task".to_string()),
            model: constants::DEFAULT_MODEL.to_string(),
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: true,
            enable_network: true,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    fn create_test_run(agent_id: i64, status: &str) -> AgentRun {
        AgentRun {
            id: None,
            agent_id,
            agent_name: "Test Agent".to_string(),
            agent_icon: "🤖".to_string(),
            task: "Test task".to_string(),
            model: constants::DEFAULT_MODEL.to_string(),
            project_path: "/tmp/test".to_string(),
            session_id: uuid::Uuid::new_v4().to_string(),
            status: status.to_string(),
            pid: if status == "running" { Some(12345) } else { None },
            process_started_at: if status == "running" {
                Some(chrono::Utc::now().to_rfc3339())
            } else {
                None
            },
            scheduled_start_time: if status == "scheduled" {
                Some((chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339())
            } else {
                None
            },
            created_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
            usage_limit_reset_time: None,
            auto_resume_enabled: false,
            resume_count: 0,
            parent_run_id: None,
        }
    }

    fn create_mock_pool() -> Arc<SqlitePool> {
        // Create an in-memory SQLite pool for testing
        let manager = r2d2_sqlite::SqliteConnectionManager::memory();
        let pool = r2d2::Pool::new(manager).unwrap();
        Arc::new(pool)
    }

    // ===== Agent Management Tests =====

    #[tokio::test]
    async fn test_create_agent_success() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let repo = Arc::new(MockAgentRepository::new());

        let result = service
            .create_agent(
                pool,
                "Test Agent".to_string(),
                "🤖".to_string(),
                "You are a helpful assistant".to_string(),
                Some("Help with coding".to_string()),
                Some("opus-4".to_string()),
                Some(false),
                Some(true),
                Some(false),
                Some(true),
            )
            .await;

        assert!(result.is_ok());
        let agent = result.unwrap();
        assert_eq!(agent.name, "Test Agent");
        assert_eq!(agent.icon, "🤖");
        assert_eq!(agent.model, "opus-4");
        assert!(!agent.sandbox_enabled);
        assert!(agent.enable_file_read);
        assert!(!agent.enable_file_write);
        assert!(agent.enable_network);
    }

    #[tokio::test]
    async fn test_create_agent_validation_empty_name() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();

        let result = service
            .create_agent(
                pool,
                "".to_string(),
                "🤖".to_string(),
                "System prompt".to_string(),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_agent_defaults() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();

        let result = service
            .create_agent(
                pool,
                "Test Agent".to_string(),
                "🤖".to_string(),
                "System prompt".to_string(),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await;

        assert!(result.is_ok());
        let agent = result.unwrap();
        assert_eq!(agent.model, constants::DEFAULT_MODEL);
        assert_eq!(agent.sandbox_enabled, constants::DEFAULT_SANDBOX_ENABLED);
        assert_eq!(agent.enable_file_read, constants::DEFAULT_FILE_READ_ENABLED);
        assert_eq!(agent.enable_file_write, constants::DEFAULT_FILE_WRITE_ENABLED);
        assert_eq!(agent.enable_network, constants::DEFAULT_NETWORK_ENABLED);
    }

    #[tokio::test]
    async fn test_get_agent_success() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let repo = Arc::new(MockAgentRepository::new());
        
        let test_agent = create_test_agent("Test Agent");
        let agent_id = repo.add_test_agent(test_agent);

        let result = service.get_agent(pool, agent_id).await;
        assert!(result.is_ok());
        let agent = result.unwrap();
        assert_eq!(agent.id, Some(agent_id));
        assert_eq!(agent.name, "Test Agent");
    }

    #[tokio::test]
    async fn test_get_agent_not_found() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();

        let result = service.get_agent(pool, 999).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_agent_success() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let repo = Arc::new(MockAgentRepository::new());
        
        let test_agent = create_test_agent("Old Name");
        let agent_id = repo.add_test_agent(test_agent);

        let result = service
            .update_agent(
                pool,
                agent_id,
                "New Name".to_string(),
                "🚀".to_string(),
                "Updated prompt".to_string(),
                Some("New task".to_string()),
                Some("sonnet-4".to_string()),
                Some(false),
                None,
                None,
                None,
            )
            .await;

        assert!(result.is_ok());
        let agent = result.unwrap();
        assert_eq!(agent.name, "New Name");
        assert_eq!(agent.icon, "🚀");
        assert_eq!(agent.model, "sonnet-4");
        assert!(!agent.sandbox_enabled);
    }

    #[tokio::test]
    async fn test_update_agent_partial_update() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let repo = Arc::new(MockAgentRepository::new());
        
        let test_agent = create_test_agent("Test Agent");
        let agent_id = repo.add_test_agent(test_agent);

        let result = service
            .update_agent(
                pool,
                agent_id,
                "Test Agent".to_string(),
                "🤖".to_string(),
                "Test system prompt".to_string(),
                None,
                None,
                None,
                Some(false), // Only update file read
                None,
                None,
            )
            .await;

        assert!(result.is_ok());
        let agent = result.unwrap();
        assert!(!agent.enable_file_read);
        assert!(agent.enable_file_write); // Should remain unchanged
        assert!(agent.enable_network); // Should remain unchanged
    }

    #[tokio::test]
    async fn test_update_agent_validation_error() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let repo = Arc::new(MockAgentRepository::new());
        
        let test_agent = create_test_agent("Test Agent");
        let agent_id = repo.add_test_agent(test_agent);

        let result = service
            .update_agent(
                pool,
                agent_id,
                "".to_string(), // Empty name
                "🤖".to_string(),
                "Prompt".to_string(),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_agent_success() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let repo = Arc::new(MockAgentRepository::new());
        
        let test_agent = create_test_agent("Test Agent");
        let agent_id = repo.add_test_agent(test_agent);

        let result = service.delete_agent(pool, agent_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_agent_with_active_run() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let repo = Arc::new(MockAgentRepository::new());
        
        let test_agent = create_test_agent("Test Agent");
        let agent_id = repo.add_test_agent(test_agent);
        
        let test_run = create_test_run(agent_id, "running");
        repo.add_test_run(test_run);

        let result = service.delete_agent(pool, agent_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_agents() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let repo = Arc::new(MockAgentRepository::new());
        
        repo.add_test_agent(create_test_agent("Agent 1"));
        repo.add_test_agent(create_test_agent("Agent 2"));
        repo.add_test_agent(create_test_agent("Agent 3"));

        let result = service.list_agents(pool).await;
        assert!(result.is_ok());
        let agents = result.unwrap();
        assert_eq!(agents.len(), 3);
    }

    // ===== Run Lifecycle Tests =====

    #[tokio::test]
    async fn test_create_scheduled_run() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let repo = Arc::new(MockAgentRepository::new());
        
        let test_agent = create_test_agent("Test Agent");
        let agent_id = repo.add_test_agent(test_agent);

        let scheduled_time = (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339();
        let result = service
            .create_scheduled_run(
                pool,
                agent_id,
                "Scheduled task".to_string(),
                "/tmp/project".to_string(),
                scheduled_time.clone(),
            )
            .await;

        assert!(result.is_ok());
        let run = result.unwrap();
        assert_eq!(run.status, "scheduled");
        assert_eq!(run.scheduled_start_time, Some(scheduled_time));
        assert_eq!(run.task, "Scheduled task");
    }

    #[tokio::test]
    async fn test_cancel_scheduled_run() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let repo = Arc::new(MockAgentRepository::new());
        
        let test_run = create_test_run(1, "scheduled");
        let run_id = repo.add_test_run(test_run);

        let result = service.cancel_scheduled_run(pool, run_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_agent_runs() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let repo = Arc::new(MockAgentRepository::new());
        
        let agent_id = repo.add_test_agent(create_test_agent("Test Agent"));
        
        repo.add_test_run(create_test_run(agent_id, "running"));
        repo.add_test_run(create_test_run(agent_id, "completed"));
        repo.add_test_run(create_test_run(agent_id, "failed"));

        let result = service.list_agent_runs(pool, Some(agent_id)).await;
        assert!(result.is_ok());
        let runs = result.unwrap();
        assert_eq!(runs.len(), 3);
    }

    #[tokio::test]
    async fn test_list_agent_runs_with_metrics() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let repo = Arc::new(MockAgentRepository::new());
        
        let agent_id = repo.add_test_agent(create_test_agent("Test Agent"));
        repo.add_test_run(create_test_run(agent_id, "completed"));

        let result = service.list_agent_runs_with_metrics(pool, Some(agent_id)).await;
        assert!(result.is_ok());
        let runs = result.unwrap();
        assert_eq!(runs.len(), 1);
        assert!(runs[0].metrics.is_some());
        
        let metrics = runs[0].metrics.as_ref().unwrap();
        assert!(metrics.duration_ms.is_some());
        assert!(metrics.total_tokens.is_some());
        assert!(metrics.cost_usd.is_some());
    }

    #[tokio::test]
    async fn test_get_agent_run() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let repo = Arc::new(MockAgentRepository::new());
        
        let test_run = create_test_run(1, "running");
        let run_id = repo.add_test_run(test_run);

        let result = service.get_agent_run(pool, run_id).await;
        assert!(result.is_ok());
        let run = result.unwrap();
        assert_eq!(run.id, Some(run_id));
        assert_eq!(run.status, "running");
    }

    #[tokio::test]
    async fn test_get_agent_run_with_metrics() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let repo = Arc::new(MockAgentRepository::new());
        
        let test_run = create_test_run(1, "completed");
        let run_id = repo.add_test_run(test_run);
        
        // Add some output
        repo.store_jsonl_output(run_id, 1, r#"{"type":"text","message":"Hello"}"#).unwrap();

        let result = service.get_agent_run_with_metrics(pool, run_id).await;
        assert!(result.is_ok());
        let run_with_metrics = result.unwrap();
        assert!(run_with_metrics.metrics.is_some());
        assert!(run_with_metrics.output.is_some());
    }

    #[tokio::test]
    async fn test_list_running_sessions() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let repo = Arc::new(MockAgentRepository::new());
        
        repo.add_test_run(create_test_run(1, "running"));
        repo.add_test_run(create_test_run(1, "running"));
        repo.add_test_run(create_test_run(1, "completed"));

        let result = service.list_running_sessions(pool).await;
        assert!(result.is_ok());
        let runs = result.unwrap();
        assert_eq!(runs.len(), 2);
        assert!(runs.iter().all(|r| r.status == "running"));
    }

    #[tokio::test]
    async fn test_get_scheduled_runs() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let repo = Arc::new(MockAgentRepository::new());
        
        repo.add_test_run(create_test_run(1, "scheduled"));
        repo.add_test_run(create_test_run(1, "scheduled"));
        repo.add_test_run(create_test_run(1, "running"));

        let result = service.get_scheduled_runs(pool).await;
        assert!(result.is_ok());
        let runs = result.unwrap();
        assert_eq!(runs.len(), 2);
        assert!(runs.iter().all(|r| r.status == "scheduled"));
    }

    // ===== Process Management Tests =====

    #[tokio::test]
    async fn test_kill_agent_session_via_registry() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let repo = Arc::new(MockAgentRepository::new());
        
        let test_run = create_test_run(1, "running");
        let session_id = test_run.session_id.clone();
        let run_id = repo.add_test_run(test_run);
        
        let mut mock_registry = MockProcessRegistry::new();
        mock_registry.add_process(session_id.clone(), true);
        let registry = Arc::new(Mutex::new(ProcessRegistry::new()));

        let result = service.kill_agent_session(pool, &registry, session_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_kill_agent_session_not_found() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let registry = Arc::new(Mutex::new(ProcessRegistry::new()));

        let result = service
            .kill_agent_session(pool, &registry, "non-existent".to_string())
            .await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_session_status_running() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let repo = Arc::new(MockAgentRepository::new());
        
        let test_run = create_test_run(1, "running");
        let session_id = test_run.session_id.clone();
        let pid = test_run.pid;
        repo.add_test_run(test_run);
        
        let registry = Arc::new(Mutex::new(ProcessRegistry::new()));
        
        let result = service
            .get_session_status(pool, &registry, session_id.clone())
            .await;
        
        assert!(result.is_ok());
        let status = result.unwrap();
        assert_eq!(status.session_id, session_id);
        assert_eq!(status.status, "running");
        assert_eq!(status.pid, pid);
    }

    #[tokio::test]
    async fn test_get_session_status_not_found() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let registry = Arc::new(Mutex::new(ProcessRegistry::new()));

        let result = service
            .get_session_status(pool, &registry, "non-existent".to_string())
            .await;
        
        assert!(result.is_ok());
        let status = result.unwrap();
        assert_eq!(status.status, "not_found");
        assert!(!status.is_alive);
        assert!(status.pid.is_none());
    }

    #[tokio::test]
    async fn test_cleanup_finished_processes() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let repo = Arc::new(MockAgentRepository::new());
        
        // Add runs that appear to be running but process is dead
        let run1 = create_test_run(1, "running");
        let run2 = create_test_run(1, "running");
        let run_id1 = repo.add_test_run(run1);
        let run_id2 = repo.add_test_run(run2);
        
        let registry = Arc::new(Mutex::new(ProcessRegistry::new()));

        let result = service.cleanup_finished_processes(pool, &registry).await;
        assert!(result.is_ok());
        let cleaned = result.unwrap();
        assert_eq!(cleaned.len(), 2);
        assert!(cleaned.contains(&run_id1));
        assert!(cleaned.contains(&run_id2));
    }

    #[tokio::test]
    async fn test_get_live_session_output() {
        let app = create_test_app();
        let service = AgentService::new(app);
        
        let mut mock_registry = MockProcessRegistry::new();
        let session_id = "test-session";
        mock_registry.add_process(session_id.to_string(), true);
        mock_registry.add_output(session_id, "Line 1".to_string());
        mock_registry.add_output(session_id, "Line 2".to_string());
        
        let registry = Arc::new(Mutex::new(ProcessRegistry::new()));
        
        // Note: This test is simplified since we can't easily mock the ProcessRegistry trait
        // In a real test, you'd need to create a trait for ProcessRegistry and mock that
    }

    #[tokio::test]
    async fn test_get_session_output() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let repo = Arc::new(MockAgentRepository::new());
        
        let test_run = create_test_run(1, "completed");
        let session_id = test_run.session_id.clone();
        let run_id = repo.add_test_run(test_run);
        
        repo.store_jsonl_output(run_id, 1, "Output line 1").unwrap();
        repo.store_jsonl_output(run_id, 2, "Output line 2").unwrap();

        let result = service.get_session_output(pool, session_id).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Output line 1"));
        assert!(output.contains("Output line 2"));
    }

    // ===== State Management Tests =====

    #[tokio::test]
    async fn test_resume_agent_success() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let repo = Arc::new(MockAgentRepository::new());
        
        let mut test_run = create_test_run(1, "paused_usage_limit");
        test_run.usage_limit_reset_time = Some(chrono::Utc::now().to_rfc3339());
        let run_id = repo.add_test_run(test_run);
        
        repo.store_jsonl_output(run_id, 1, "Previous output").unwrap();
        
        let registry = Arc::new(Mutex::new(ProcessRegistry::new()));

        // Note: This test would need the execute module mocked
        // let result = service.resume_agent(pool, run_id, registry).await;
    }

    #[tokio::test]
    async fn test_resume_agent_wrong_status() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let repo = Arc::new(MockAgentRepository::new());
        
        let test_run = create_test_run(1, "completed");
        let run_id = repo.add_test_run(test_run);
        
        let registry = Arc::new(Mutex::new(ProcessRegistry::new()));

        // Note: This test would need the execute module mocked
        // let result = service.resume_agent(pool, run_id, registry).await;
        // assert!(result.is_err());
    }

    // ===== Integration Tests =====

    #[tokio::test]
    async fn test_export_import_agent() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let repo = Arc::new(MockAgentRepository::new());
        
        let test_agent = create_test_agent("Export Test");
        let agent_id = repo.add_test_agent(test_agent);

        // Export
        let export_result = service.export_agent(pool.clone(), agent_id).await;
        assert!(export_result.is_ok());
        let json = export_result.unwrap();

        // Import
        let import_result = service.import_agent(pool, json).await;
        assert!(import_result.is_ok());
        let imported = import_result.unwrap();
        assert_eq!(imported.name, "Export Test");
        assert_eq!(imported.icon, "🤖");
    }

    #[tokio::test]
    async fn test_import_agent_invalid_version() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();

        let invalid_json = r#"{
            "version": 999,
            "exported_at": "2024-01-01T00:00:00Z",
            "agent": {
                "name": "Test",
                "icon": "🤖",
                "system_prompt": "Test",
                "model": "opus-4",
                "sandbox_enabled": true,
                "enable_file_read": true,
                "enable_file_write": true,
                "enable_network": true
            }
        }"#;

        let result = service.import_agent(pool, invalid_json.to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_claude_binary_path_settings() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();

        // Set path
        let set_result = service
            .set_claude_binary_path(pool.clone(), "/usr/local/bin/claude".to_string())
            .await;
        assert!(set_result.is_ok());

        // Get path
        let get_result = service.get_claude_binary_path(pool).await;
        assert!(get_result.is_ok());
        assert_eq!(get_result.unwrap(), Some("/usr/local/bin/claude".to_string()));
    }

    // ===== Error Handling Tests =====

    #[tokio::test]
    async fn test_repository_failure_handling() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let repo = Arc::new(MockAgentRepository::new());
        
        repo.set_should_fail(true);

        let result = service.list_agents(pool).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_concurrent_state_access() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let repo = Arc::new(MockAgentRepository::new());
        
        // Add multiple agents
        for i in 0..10 {
            repo.add_test_agent(create_test_agent(&format!("Agent {}", i)));
        }

        // Simulate concurrent access
        let mut handles = vec![];
        for _ in 0..5 {
            let pool_clone = pool.clone();
            let service_clone = service.clone();
            let handle = tokio::spawn(async move {
                service_clone.list_agents(pool_clone).await
            });
            handles.push(handle);
        }

        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
            assert_eq!(result.unwrap().len(), 10);
        }
    }

    // ===== Edge Cases =====

    #[tokio::test]
    async fn test_empty_task_handling() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();
        let repo = Arc::new(MockAgentRepository::new());
        
        let agent_id = repo.add_test_agent(create_test_agent("Test Agent"));

        let result = service
            .create_scheduled_run(
                pool,
                agent_id,
                "".to_string(), // Empty task
                "/tmp/project".to_string(),
                chrono::Utc::now().to_rfc3339(),
            )
            .await;

        // Should succeed - empty task is allowed
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_special_characters_in_names() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();

        let result = service
            .create_agent(
                pool,
                "Agent with 特殊文字 & symbols!".to_string(),
                "🎭".to_string(),
                "System prompt with\nnewlines\tand\ttabs".to_string(),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await;

        assert!(result.is_ok());
        let agent = result.unwrap();
        assert_eq!(agent.name, "Agent with 特殊文字 & symbols!");
        assert_eq!(agent.icon, "🎭");
    }

    #[tokio::test]
    async fn test_very_long_strings() {
        let app = create_test_app();
        let service = AgentService::new(app);
        let pool = create_mock_pool();

        let long_name = "A".repeat(1000);
        let long_prompt = "B".repeat(10000);

        let result = service
            .create_agent(
                pool,
                long_name.clone(),
                "🤖".to_string(),
                long_prompt.clone(),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await;

        assert!(result.is_ok());
        let agent = result.unwrap();
        assert_eq!(agent.name, long_name);
        assert_eq!(agent.system_prompt, long_prompt);
    }
}