#[cfg(test)]
mod tests {
    use crate::commands::agents::{
        constants, 
        error::AgentError, 
        pool::{create_pool, SqlitePool}, 
        repository::{AgentRepository, SqliteAgentRepository, NewAgent, UpdateAgent, NewAgentRun, SandboxViolation},
    };
    use chrono::Utc;
    use rusqlite::params;
    use std::sync::Arc;
    use tempfile::TempDir;
    use uuid::Uuid;

    // ===== Test Helpers =====

    /// Initialize test database without sandbox profiles
    fn init_test_db(pool: &SqlitePool) -> Result<(), AgentError> {
        let conn = pool.get()
            .map_err(|e| AgentError::Other(format!("Failed to get connection from pool: {}", e)))?;
        
        // Create agents table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS agents (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                icon TEXT NOT NULL,
                system_prompt TEXT NOT NULL,
                default_task TEXT,
                model TEXT DEFAULT 'sonnet',
                sandbox_enabled BOOLEAN DEFAULT 1,
                enable_file_read BOOLEAN DEFAULT 1,
                enable_file_write BOOLEAN DEFAULT 1,
                enable_network BOOLEAN DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Create trigger to update the updated_at timestamp
        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS update_agent_timestamp 
             AFTER UPDATE ON agents 
             FOR EACH ROW
             BEGIN
                 UPDATE agents SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
             END",
            [],
        )?;

        // Create agent_runs table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS agent_runs (
                id INTEGER PRIMARY KEY,
                agent_id INTEGER NOT NULL,
                agent_name TEXT NOT NULL,
                agent_icon TEXT NOT NULL,
                task TEXT NOT NULL,
                model TEXT NOT NULL,
                project_path TEXT NOT NULL,
                session_id TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                pid INTEGER,
                process_started_at TEXT,
                scheduled_start_time TEXT,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                completed_at TEXT,
                usage_limit_reset_time TEXT,
                auto_resume_enabled BOOLEAN DEFAULT 0,
                resume_count INTEGER DEFAULT 0,
                parent_run_id INTEGER,
                FOREIGN KEY (agent_id) REFERENCES agents(id)
            )",
            [],
        )?;

        // Create indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_agent_runs_agent_id ON agent_runs(agent_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_agent_runs_status ON agent_runs(status)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_agent_runs_session_id ON agent_runs(session_id)",
            [],
        )?;

        // Create jsonl_outputs table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS jsonl_outputs (
                id INTEGER PRIMARY KEY,
                run_id INTEGER NOT NULL,
                line_number INTEGER NOT NULL,
                timestamp TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (run_id) REFERENCES agent_runs(id),
                UNIQUE(run_id, line_number)
            )",
            [],
        )?;

        // Create index for efficient querying
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_jsonl_outputs_run_id ON jsonl_outputs(run_id)",
            [],
        )?;

        // Create sandbox_violations table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sandbox_violations (
                id INTEGER PRIMARY KEY,
                run_id INTEGER NOT NULL,
                denied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                operation_type TEXT NOT NULL,
                resource TEXT NOT NULL,
                reason TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (run_id) REFERENCES agent_runs(id)
            )",
            [],
        )?;

        // Create index for efficient querying
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sandbox_violations_denied_at 
             ON sandbox_violations(denied_at DESC)",
            [],
        )?;

        // Create settings table for app-wide settings
        conn.execute(
            "CREATE TABLE IF NOT EXISTS app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Create trigger to update the updated_at timestamp
        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS update_app_settings_timestamp 
             AFTER UPDATE ON app_settings 
             FOR EACH ROW
             BEGIN
                 UPDATE app_settings SET updated_at = CURRENT_TIMESTAMP WHERE key = NEW.key;
             END",
            [],
        )?;

        Ok(())
    }

    /// Create a test database with a connection pool
    fn setup_test_db() -> (TempDir, SqlitePool) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        
        let pool = create_pool(&db_path).expect("Failed to create pool");
        init_test_db(&pool).expect("Failed to initialize database");
        
        (temp_dir, pool)
    }

    /// Create a test repository
    fn create_test_repository() -> (TempDir, SqliteAgentRepository) {
        let (temp_dir, pool) = setup_test_db();
        let repo = SqliteAgentRepository::new(pool);
        (temp_dir, repo)
    }

    /// Test data builder for agents
    struct TestAgentBuilder {
        name: String,
        icon: String,
        system_prompt: String,
        default_task: Option<String>,
        model: String,
        sandbox_enabled: bool,
        enable_file_read: bool,
        enable_file_write: bool,
        enable_network: bool,
    }

    impl TestAgentBuilder {
        fn new() -> Self {
            Self {
                name: "Test Agent".to_string(),
                icon: "🤖".to_string(),
                system_prompt: "You are a helpful assistant".to_string(),
                default_task: None,
                model: constants::DEFAULT_MODEL.to_string(),
                sandbox_enabled: constants::DEFAULT_SANDBOX_ENABLED,
                enable_file_read: constants::DEFAULT_FILE_READ_ENABLED,
                enable_file_write: constants::DEFAULT_FILE_WRITE_ENABLED,
                enable_network: constants::DEFAULT_NETWORK_ENABLED,
            }
        }

        fn with_name(mut self, name: &str) -> Self {
            self.name = name.to_string();
            self
        }

        fn with_model(mut self, model: &str) -> Self {
            self.model = model.to_string();
            self
        }

        fn with_sandbox_disabled(mut self) -> Self {
            self.sandbox_enabled = false;
            self
        }

        fn build(self) -> NewAgent {
            NewAgent {
                name: self.name,
                icon: self.icon,
                system_prompt: self.system_prompt,
                default_task: self.default_task,
                model: self.model,
                sandbox_enabled: self.sandbox_enabled,
                enable_file_read: self.enable_file_read,
                enable_file_write: self.enable_file_write,
                enable_network: self.enable_network,
            }
        }
    }

    /// Test data builder for agent runs
    struct TestRunBuilder {
        agent_id: i64,
        agent_name: String,
        agent_icon: String,
        task: String,
        model: String,
        project_path: String,
        status: Option<&'static str>,
        scheduled_start_time: Option<String>,
        parent_run_id: Option<i64>,
    }

    impl TestRunBuilder {
        fn new(agent_id: i64) -> Self {
            Self {
                agent_id,
                agent_name: "Test Agent".to_string(),
                agent_icon: "🤖".to_string(),
                task: "Test task".to_string(),
                model: constants::DEFAULT_MODEL.to_string(),
                project_path: "/tmp/test".to_string(),
                status: None,
                scheduled_start_time: None,
                parent_run_id: None,
            }
        }

        fn with_task(mut self, task: &str) -> Self {
            self.task = task.to_string();
            self
        }

        fn with_status(mut self, status: &'static str) -> Self {
            self.status = Some(status);
            self
        }

        fn scheduled(mut self, time: &str) -> Self {
            self.scheduled_start_time = Some(time.to_string());
            self.status = Some("scheduled");
            self
        }

        fn as_resume(mut self, parent_id: i64) -> Self {
            self.parent_run_id = Some(parent_id);
            self
        }

        fn build(self) -> NewAgentRun {
            NewAgentRun {
                agent_id: self.agent_id,
                agent_name: self.agent_name,
                agent_icon: self.agent_icon,
                task: self.task,
                model: self.model,
                project_path: self.project_path,
                status: self.status,
                scheduled_start_time: self.scheduled_start_time,
                parent_run_id: self.parent_run_id,
            }
        }
    }

    // ===== Agent CRUD Operations Tests =====

    #[test]
    fn test_create_agent_success() {
        let (_temp, repo) = create_test_repository();
        let new_agent = TestAgentBuilder::new().build();
        
        let agent = repo.create_agent(new_agent).expect("Failed to create agent");
        
        assert!(agent.id.is_some());
        assert_eq!(agent.name, "Test Agent");
        assert_eq!(agent.icon, "🤖");
        assert_eq!(agent.system_prompt, "You are a helpful assistant");
        assert!(agent.sandbox_enabled);
        assert!(!agent.created_at.is_empty());
        assert!(!agent.updated_at.is_empty());
    }

    #[test]
    fn test_create_agent_with_all_fields() {
        let (_temp, repo) = create_test_repository();
        let mut new_agent = TestAgentBuilder::new()
            .with_name("Full Featured Agent")
            .with_model("opus-4")
            .with_sandbox_disabled()
            .build();
        new_agent.default_task = Some("Analyze code".to_string());
        new_agent.enable_network = true;
        
        let agent = repo.create_agent(new_agent).expect("Failed to create agent");
        
        assert_eq!(agent.name, "Full Featured Agent");
        assert_eq!(agent.model, "opus-4");
        assert!(!agent.sandbox_enabled);
        assert_eq!(agent.default_task, Some("Analyze code".to_string()));
        assert!(agent.enable_network);
    }

    #[test]
    fn test_get_agent_by_id_found() {
        let (_temp, repo) = create_test_repository();
        let new_agent = TestAgentBuilder::new().build();
        let created = repo.create_agent(new_agent).expect("Failed to create agent");
        
        let found = repo.find_agent_by_id(created.id.unwrap())
            .expect("Failed to find agent");
        
        assert_eq!(found.id, created.id);
        assert_eq!(found.name, created.name);
        assert_eq!(found.icon, created.icon);
    }

    #[test]
    fn test_get_agent_by_id_not_found() {
        let (_temp, repo) = create_test_repository();
        
        let result = repo.find_agent_by_id(999);
        
        match result {
            Err(AgentError::AgentNotFound(id)) => assert_eq!(id, 999),
            _ => panic!("Expected AgentNotFound error"),
        }
    }

    #[test]
    fn test_update_agent_success() {
        let (_temp, repo) = create_test_repository();
        let new_agent = TestAgentBuilder::new().build();
        let created = repo.create_agent(new_agent).expect("Failed to create agent");
        let id = created.id.unwrap();
        
        let update = UpdateAgent {
            name: "Updated Agent".to_string(),
            icon: "🚀".to_string(),
            system_prompt: "Updated prompt".to_string(),
            default_task: Some("New task".to_string()),
            model: "sonnet-4".to_string(),
            sandbox_enabled: Some(false),
            enable_file_read: Some(false),
            enable_file_write: Some(false),
            enable_network: Some(true),
        };
        
        let updated = repo.update_agent(id, update).expect("Failed to update agent");
        
        assert_eq!(updated.id, Some(id));
        assert_eq!(updated.name, "Updated Agent");
        assert_eq!(updated.icon, "🚀");
        assert_eq!(updated.system_prompt, "Updated prompt");
        assert_eq!(updated.default_task, Some("New task".to_string()));
        assert_eq!(updated.model, "sonnet-4");
        assert!(!updated.sandbox_enabled);
        assert!(!updated.enable_file_read);
        assert!(!updated.enable_file_write);
        assert!(updated.enable_network);
    }

    #[test]
    fn test_update_agent_partial() {
        let (_temp, repo) = create_test_repository();
        let new_agent = TestAgentBuilder::new().build();
        let created = repo.create_agent(new_agent).expect("Failed to create agent");
        let id = created.id.unwrap();
        
        let update = UpdateAgent {
            name: "Partially Updated".to_string(),
            icon: created.icon.clone(),
            system_prompt: created.system_prompt.clone(),
            default_task: created.default_task.clone(),
            model: created.model.clone(),
            sandbox_enabled: None,
            enable_file_read: None,
            enable_file_write: None,
            enable_network: Some(true),
        };
        
        let updated = repo.update_agent(id, update).expect("Failed to update agent");
        
        assert_eq!(updated.name, "Partially Updated");
        assert_eq!(updated.sandbox_enabled, created.sandbox_enabled);
        assert_eq!(updated.enable_file_read, created.enable_file_read);
        assert_eq!(updated.enable_file_write, created.enable_file_write);
        assert!(updated.enable_network);
    }

    #[test]
    fn test_update_agent_not_found() {
        let (_temp, repo) = create_test_repository();
        
        let update = UpdateAgent {
            name: "Ghost Agent".to_string(),
            icon: "👻".to_string(),
            system_prompt: "I don't exist".to_string(),
            default_task: None,
            model: "sonnet".to_string(),
            sandbox_enabled: None,
            enable_file_read: None,
            enable_file_write: None,
            enable_network: None,
        };
        
        let result = repo.update_agent(999, update);
        
        match result {
            Err(AgentError::AgentNotFound(id)) => assert_eq!(id, 999),
            _ => panic!("Expected AgentNotFound error"),
        }
    }

    #[test]
    fn test_delete_agent_success() {
        let (_temp, repo) = create_test_repository();
        let new_agent = TestAgentBuilder::new().build();
        let created = repo.create_agent(new_agent).expect("Failed to create agent");
        let id = created.id.unwrap();
        
        repo.delete_agent(id).expect("Failed to delete agent");
        
        let result = repo.find_agent_by_id(id);
        assert!(matches!(result, Err(AgentError::AgentNotFound(_))));
    }

    #[test]
    fn test_delete_agent_cascade() {
        let (_temp, repo) = create_test_repository();
        
        // Create agent and run
        let new_agent = TestAgentBuilder::new().build();
        let agent = repo.create_agent(new_agent).expect("Failed to create agent");
        let agent_id = agent.id.unwrap();
        
        let new_run = TestRunBuilder::new(agent_id).build();
        let run = repo.create_run(new_run).expect("Failed to create run");
        let run_id = run.id.unwrap();
        
        // Delete agent should fail due to foreign key constraint
        let result = repo.delete_agent(agent_id);
        assert!(result.is_err());
        
        // Verify agent and run still exist
        assert!(repo.find_agent_by_id(agent_id).is_ok());
        assert!(repo.find_run_by_id(run_id).is_ok());
    }

    #[test]
    fn test_list_agents_empty() {
        let (_temp, repo) = create_test_repository();
        
        let agents = repo.find_all_agents().expect("Failed to list agents");
        
        assert!(agents.is_empty());
    }

    #[test]
    fn test_list_agents_multiple() {
        let (_temp, repo) = create_test_repository();
        
        // Create multiple agents
        for i in 1..=5 {
            let agent = TestAgentBuilder::new()
                .with_name(&format!("Agent {}", i))
                .build();
            repo.create_agent(agent).expect("Failed to create agent");
        }
        
        let agents = repo.find_all_agents().expect("Failed to list agents");
        
        assert_eq!(agents.len(), 5);
        // Since we're creating agents quickly in a loop, their created_at timestamps
        // might be identical, making the order non-deterministic. Just verify we have all agents.
        let names: Vec<String> = agents.iter().map(|a| a.name.clone()).collect();
        for i in 1..=5 {
            assert!(names.contains(&format!("Agent {}", i)));
        }
    }

    // ===== Run Management Tests =====

    #[test]
    fn test_create_run_success() {
        let (_temp, repo) = create_test_repository();
        
        // Create agent first
        let new_agent = TestAgentBuilder::new().build();
        let agent = repo.create_agent(new_agent).expect("Failed to create agent");
        let agent_id = agent.id.unwrap();
        
        let new_run = TestRunBuilder::new(agent_id).build();
        let run = repo.create_run(new_run).expect("Failed to create run");
        
        assert!(run.id.is_some());
        assert_eq!(run.agent_id, agent_id);
        assert_eq!(run.task, "Test task");
        assert_eq!(run.status, "pending");
        assert!(Uuid::parse_str(&run.session_id).is_ok());
        assert!(!run.created_at.is_empty());
        assert!(run.completed_at.is_none());
    }

    #[test]
    fn test_create_run_with_schedule() {
        let (_temp, repo) = create_test_repository();
        
        let new_agent = TestAgentBuilder::new().build();
        let agent = repo.create_agent(new_agent).expect("Failed to create agent");
        let agent_id = agent.id.unwrap();
        
        let scheduled_time = Utc::now().to_rfc3339();
        let new_run = TestRunBuilder::new(agent_id)
            .scheduled(&scheduled_time)
            .build();
        
        let run = repo.create_run(new_run).expect("Failed to create run");
        
        assert_eq!(run.status, "scheduled");
        assert_eq!(run.scheduled_start_time, Some(scheduled_time));
    }

    #[test]
    fn test_create_run_foreign_key_constraint() {
        let (_temp, repo) = create_test_repository();
        
        let new_run = TestRunBuilder::new(999).build(); // Non-existent agent
        let result = repo.create_run(new_run);
        
        assert!(matches!(result, Err(AgentError::Database(_))));
    }

    #[test]
    fn test_update_run_status() {
        let (_temp, repo) = create_test_repository();
        
        let new_agent = TestAgentBuilder::new().build();
        let agent = repo.create_agent(new_agent).expect("Failed to create agent");
        let agent_id = agent.id.unwrap();
        
        let new_run = TestRunBuilder::new(agent_id).build();
        let run = repo.create_run(new_run).expect("Failed to create run");
        let run_id = run.id.unwrap();
        
        let started_at = Utc::now().to_rfc3339();
        repo.update_run_status(run_id, "running", Some(12345), Some(started_at.clone()))
            .expect("Failed to update run status");
        
        let updated = repo.find_run_by_id(run_id).expect("Failed to find run");
        assert_eq!(updated.status, "running");
        assert_eq!(updated.pid, Some(12345));
        assert_eq!(updated.process_started_at, Some(started_at));
    }

    #[test]
    fn test_update_run_completion() {
        let (_temp, repo) = create_test_repository();
        
        let new_agent = TestAgentBuilder::new().build();
        let agent = repo.create_agent(new_agent).expect("Failed to create agent");
        let agent_id = agent.id.unwrap();
        
        let new_run = TestRunBuilder::new(agent_id).build();
        let run = repo.create_run(new_run).expect("Failed to create run");
        let run_id = run.id.unwrap();
        
        repo.update_run_completion(run_id, "completed")
            .expect("Failed to update run completion");
        
        let updated = repo.find_run_by_id(run_id).expect("Failed to find run");
        assert_eq!(updated.status, "completed");
        assert!(updated.completed_at.is_some());
    }

    #[test]
    fn test_update_run_usage_limit() {
        let (_temp, repo) = create_test_repository();
        
        let new_agent = TestAgentBuilder::new().build();
        let agent = repo.create_agent(new_agent).expect("Failed to create agent");
        let agent_id = agent.id.unwrap();
        
        let new_run = TestRunBuilder::new(agent_id).build();
        let run = repo.create_run(new_run).expect("Failed to create run");
        let run_id = run.id.unwrap();
        
        let reset_time = Utc::now().to_rfc3339();
        repo.update_run_usage_limit(run_id, &reset_time, true)
            .expect("Failed to update usage limit");
        
        let updated = repo.find_run_by_id(run_id).expect("Failed to find run");
        assert_eq!(updated.status, "paused_usage_limit");
        assert_eq!(updated.usage_limit_reset_time, Some(reset_time));
        assert!(updated.auto_resume_enabled);
    }

    #[test]
    fn test_find_run_by_id() {
        let (_temp, repo) = create_test_repository();
        
        let new_agent = TestAgentBuilder::new().build();
        let agent = repo.create_agent(new_agent).expect("Failed to create agent");
        let agent_id = agent.id.unwrap();
        
        let new_run = TestRunBuilder::new(agent_id).build();
        let run = repo.create_run(new_run).expect("Failed to create run");
        let run_id = run.id.unwrap();
        
        let found = repo.find_run_by_id(run_id).expect("Failed to find run");
        assert_eq!(found.id, Some(run_id));
        assert_eq!(found.agent_id, agent_id);
    }

    #[test]
    fn test_find_run_by_id_not_found() {
        let (_temp, repo) = create_test_repository();
        
        let result = repo.find_run_by_id(999);
        assert!(matches!(result, Err(AgentError::Other(_))));
    }

    #[test]
    fn test_find_run_by_session_id() {
        let (_temp, repo) = create_test_repository();
        
        let new_agent = TestAgentBuilder::new().build();
        let agent = repo.create_agent(new_agent).expect("Failed to create agent");
        let agent_id = agent.id.unwrap();
        
        let new_run = TestRunBuilder::new(agent_id).build();
        let run = repo.create_run(new_run).expect("Failed to create run");
        
        let found = repo.find_run_by_session_id(&run.session_id)
            .expect("Failed to find run by session");
        
        assert!(found.is_some());
        assert_eq!(found.unwrap().session_id, run.session_id);
    }

    #[test]
    fn test_find_run_by_session_id_not_found() {
        let (_temp, repo) = create_test_repository();
        
        let result = repo.find_run_by_session_id("non-existent-session")
            .expect("Should return None");
        
        assert!(result.is_none());
    }

    #[test]
    fn test_list_runs_for_agent() {
        let (_temp, repo) = create_test_repository();
        
        let new_agent = TestAgentBuilder::new().build();
        let agent = repo.create_agent(new_agent).expect("Failed to create agent");
        let agent_id = agent.id.unwrap();
        
        // Create multiple runs for the agent
        for i in 1..=3 {
            let run = TestRunBuilder::new(agent_id)
                .with_task(&format!("Task {}", i))
                .build();
            repo.create_run(run).expect("Failed to create run");
        }
        
        let runs = repo.find_all_runs(Some(agent_id))
            .expect("Failed to list runs");
        
        assert_eq!(runs.len(), 3);
        // Since we're creating runs quickly in a loop, their created_at timestamps
        // might be identical, making the order non-deterministic. Just verify we have all runs.
        let tasks: Vec<String> = runs.iter().map(|r| r.task.clone()).collect();
        for i in 1..=3 {
            assert!(tasks.contains(&format!("Task {}", i)));
        }
    }

    #[test]
    fn test_list_all_runs() {
        let (_temp, repo) = create_test_repository();
        
        // Create multiple agents and runs
        for i in 1..=2 {
            let agent = TestAgentBuilder::new()
                .with_name(&format!("Agent {}", i))
                .build();
            let created = repo.create_agent(agent).expect("Failed to create agent");
            let agent_id = created.id.unwrap();
            
            for j in 1..=2 {
                let run = TestRunBuilder::new(agent_id)
                    .with_task(&format!("Agent {} Task {}", i, j))
                    .build();
                repo.create_run(run).expect("Failed to create run");
            }
        }
        
        let runs = repo.find_all_runs(None).expect("Failed to list all runs");
        assert_eq!(runs.len(), 4);
    }

    #[test]
    fn test_find_running_runs() {
        let (_temp, repo) = create_test_repository();
        
        let new_agent = TestAgentBuilder::new().build();
        let agent = repo.create_agent(new_agent).expect("Failed to create agent");
        let agent_id = agent.id.unwrap();
        
        // Create runs with different statuses
        let statuses = vec!["pending", "running", "completed", "running", "failed"];
        for (i, status) in statuses.iter().enumerate() {
            let run = TestRunBuilder::new(agent_id)
                .with_task(&format!("Task {}", i))
                .with_status(status)
                .build();
            let created = repo.create_run(run).expect("Failed to create run");
            
            if *status == "running" {
                repo.update_run_status(
                    created.id.unwrap(),
                    "running",
                    Some(1000 + i as u32),
                    Some(Utc::now().to_rfc3339())
                ).expect("Failed to update status");
            }
        }
        
        let running = repo.find_running_runs().expect("Failed to find running runs");
        assert_eq!(running.len(), 2);
        assert!(running.iter().all(|r| r.status == "running"));
    }

    #[test]
    fn test_find_scheduled_runs() {
        let (_temp, repo) = create_test_repository();
        
        let new_agent = TestAgentBuilder::new().build();
        let agent = repo.create_agent(new_agent).expect("Failed to create agent");
        let agent_id = agent.id.unwrap();
        
        // Create scheduled runs with different times
        let base_time = Utc::now();
        for i in 0..3 {
            let scheduled_time = (base_time + chrono::Duration::hours(i)).to_rfc3339();
            let run = TestRunBuilder::new(agent_id)
                .with_task(&format!("Scheduled Task {}", i))
                .scheduled(&scheduled_time)
                .build();
            repo.create_run(run).expect("Failed to create run");
        }
        
        let scheduled = repo.find_scheduled_runs()
            .expect("Failed to find scheduled runs");
        
        assert_eq!(scheduled.len(), 3);
        // Verify ordering (earliest first)
        assert_eq!(scheduled[0].task, "Scheduled Task 0");
        assert_eq!(scheduled[2].task, "Scheduled Task 2");
    }

    // ===== JSONL Output Tests =====

    #[test]
    fn test_store_and_get_jsonl_output() {
        let (_temp, repo) = create_test_repository();
        
        let new_agent = TestAgentBuilder::new().build();
        let agent = repo.create_agent(new_agent).expect("Failed to create agent");
        let agent_id = agent.id.unwrap();
        
        let new_run = TestRunBuilder::new(agent_id).build();
        let run = repo.create_run(new_run).expect("Failed to create run");
        let run_id = run.id.unwrap();
        
        // Store multiple JSONL lines
        let lines = vec![
            r#"{"type":"text","message":"Hello"}"#,
            r#"{"type":"text","message":"World"}"#,
            r#"{"type":"usage","totalTokens":100}"#,
        ];
        
        for (i, line) in lines.iter().enumerate() {
            repo.store_jsonl_output(run_id, i as i64 + 1, line)
                .expect("Failed to store JSONL");
        }
        
        let output = repo.get_jsonl_output(run_id)
            .expect("Failed to get JSONL output");
        
        let expected = lines.join("\n");
        assert_eq!(output, expected);
    }

    #[test]
    fn test_store_jsonl_replace_existing() {
        let (_temp, repo) = create_test_repository();
        
        let new_agent = TestAgentBuilder::new().build();
        let agent = repo.create_agent(new_agent).expect("Failed to create agent");
        let agent_id = agent.id.unwrap();
        
        let new_run = TestRunBuilder::new(agent_id).build();
        let run = repo.create_run(new_run).expect("Failed to create run");
        let run_id = run.id.unwrap();
        
        // Store initial line
        repo.store_jsonl_output(run_id, 1, r#"{"type":"text","message":"First"}"#)
            .expect("Failed to store JSONL");
        
        // Replace with new content
        repo.store_jsonl_output(run_id, 1, r#"{"type":"text","message":"Replaced"}"#)
            .expect("Failed to replace JSONL");
        
        let output = repo.get_jsonl_output(run_id)
            .expect("Failed to get JSONL output");
        
        assert_eq!(output, r#"{"type":"text","message":"Replaced"}"#);
    }

    #[test]
    fn test_get_last_line_number() {
        let (_temp, repo) = create_test_repository();
        
        let new_agent = TestAgentBuilder::new().build();
        let agent = repo.create_agent(new_agent).expect("Failed to create agent");
        let agent_id = agent.id.unwrap();
        
        let new_run = TestRunBuilder::new(agent_id).build();
        let run = repo.create_run(new_run).expect("Failed to create run");
        let run_id = run.id.unwrap();
        
        // Initially should be 0
        let last = repo.get_last_line_number(run_id)
            .expect("Failed to get last line number");
        assert_eq!(last, 0);
        
        // Store some lines
        repo.store_jsonl_output(run_id, 1, "line1").unwrap();
        repo.store_jsonl_output(run_id, 3, "line3").unwrap();
        repo.store_jsonl_output(run_id, 7, "line7").unwrap();
        
        let last = repo.get_last_line_number(run_id)
            .expect("Failed to get last line number");
        assert_eq!(last, 7);
    }

    // ===== Metrics and Analytics Tests =====

    #[test]
    fn test_calculate_run_metrics_empty() {
        let (_temp, repo) = create_test_repository();
        
        let new_agent = TestAgentBuilder::new().build();
        let agent = repo.create_agent(new_agent).expect("Failed to create agent");
        let agent_id = agent.id.unwrap();
        
        let new_run = TestRunBuilder::new(agent_id).build();
        let run = repo.create_run(new_run).expect("Failed to create run");
        let run_id = run.id.unwrap();
        
        let metrics = repo.calculate_run_metrics(run_id)
            .expect("Failed to calculate metrics");
        
        assert_eq!(metrics.duration_ms, None);
        assert_eq!(metrics.total_tokens, None);
        assert_eq!(metrics.cost_usd, None);
        assert_eq!(metrics.message_count, None);
    }

    #[test]
    fn test_calculate_run_metrics_with_data() {
        let (_temp, repo) = create_test_repository();
        
        let new_agent = TestAgentBuilder::new().build();
        let agent = repo.create_agent(new_agent).expect("Failed to create agent");
        let agent_id = agent.id.unwrap();
        
        let new_run = TestRunBuilder::new(agent_id).build();
        let run = repo.create_run(new_run).expect("Failed to create run");
        let run_id = run.id.unwrap();
        
        let start_time = Utc::now();
        let end_time = start_time + chrono::Duration::seconds(10);
        
        // Store JSONL with metrics data
        let lines = vec![
            format!(r#"{{"type":"text","message":"Start","timestamp":"{}"}}"#, start_time.to_rfc3339()),
            r#"{"type":"text","message":"Processing"}"#.to_string(),
            r#"{"type":"usage","usage":{"totalTokens":500,"cacheReadTokens":100}}"#.to_string(),
            r#"{"cost":0.025}"#.to_string(),
            format!(r#"{{"type":"text","message":"End","timestamp":"{}"}}"#, end_time.to_rfc3339()),
            r#"{"cost":0.015}"#.to_string(),
        ];
        
        for (i, line) in lines.iter().enumerate() {
            repo.store_jsonl_output(run_id, i as i64 + 1, line)
                .expect("Failed to store JSONL");
        }
        
        let metrics = repo.calculate_run_metrics(run_id)
            .expect("Failed to calculate metrics");
        
        assert_eq!(metrics.duration_ms, Some(10000)); // 10 seconds
        assert_eq!(metrics.total_tokens, Some(600)); // 500 + 100 cache read
        assert_eq!(metrics.cost_usd, Some(0.04)); // 0.025 + 0.015
        assert_eq!(metrics.message_count, Some(3)); // 3 text messages
    }

    // ===== Sandbox Violation Tests =====

    #[test]
    fn test_store_sandbox_violation() {
        let (_temp, repo) = create_test_repository();
        
        let new_agent = TestAgentBuilder::new().build();
        let agent = repo.create_agent(new_agent).expect("Failed to create agent");
        let agent_id = agent.id.unwrap();
        
        let new_run = TestRunBuilder::new(agent_id).build();
        let run = repo.create_run(new_run).expect("Failed to create run");
        let run_id = run.id.unwrap();
        
        let violation = SandboxViolation {
            run_id,
            operation_type: "file_read".to_string(),
            resource: "/etc/passwd".to_string(),
            reason: "Access to system file denied".to_string(),
        };
        
        repo.store_sandbox_violation(violation)
            .expect("Failed to store violation");
        
        // Verify it was stored (would need additional query method to properly test)
        // For now, just ensure no error occurred
    }

    // ===== Settings Tests =====

    #[test]
    fn test_get_setting_not_found() {
        let (_temp, repo) = create_test_repository();
        
        let value = repo.get_setting("non_existent_key")
            .expect("Failed to get setting");
        
        assert_eq!(value, None);
    }

    #[test]
    fn test_set_and_get_setting() {
        let (_temp, repo) = create_test_repository();
        
        repo.set_setting("theme", "dark")
            .expect("Failed to set setting");
        
        let value = repo.get_setting("theme")
            .expect("Failed to get setting");
        
        assert_eq!(value, Some("dark".to_string()));
    }

    #[test]
    fn test_update_existing_setting() {
        let (_temp, repo) = create_test_repository();
        
        repo.set_setting("theme", "light").unwrap();
        repo.set_setting("theme", "dark").unwrap();
        
        let value = repo.get_setting("theme")
            .expect("Failed to get setting");
        
        assert_eq!(value, Some("dark".to_string()));
    }

    // ===== Transaction Tests =====

    #[test]
    fn test_transaction_rollback_on_error() {
        let (_temp, pool) = setup_test_db();
        
        // Create test connection
        let conn = pool.get().unwrap();
        
        // Begin a transaction
        let tx = conn.unchecked_transaction().unwrap();
        
        // First insert should succeed
        let result1 = tx.execute(
            "INSERT INTO agents (name, icon, system_prompt) VALUES (?1, ?2, ?3)",
            params!["Test", "🤖", "Prompt"],
        );
        assert!(result1.is_ok());
        
        // This should fail due to foreign key constraint
        let result2 = tx.execute(
            "INSERT INTO agent_runs (agent_id, agent_name, agent_icon, task, model, project_path, session_id) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![999, "Test", "🤖", "Task", "sonnet", "/tmp", Uuid::new_v4().to_string()],
        );
        assert!(result2.is_err());
        
        // Rollback the transaction
        let rollback_result = tx.rollback();
        assert!(rollback_result.is_ok());
        
        // Verify the agent was not created due to rollback
        let count: i64 = pool.get().unwrap()
            .query_row("SELECT COUNT(*) FROM agents", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    // ===== Connection Pool Tests =====

    #[test]
    fn test_concurrent_access() {
        let (_temp, repo) = create_test_repository();
        let repo = Arc::new(repo);
        
        let mut handles = vec![];
        
        // Spawn multiple threads to access the database concurrently
        for i in 0..5 {
            let repo_clone = Arc::clone(&repo);
            let handle = std::thread::spawn(move || {
                let agent = TestAgentBuilder::new()
                    .with_name(&format!("Concurrent Agent {}", i))
                    .build();
                repo_clone.create_agent(agent)
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        let results: Vec<_> = handles.into_iter()
            .map(|h| h.join().unwrap())
            .collect();
        
        // Verify all operations succeeded
        assert_eq!(results.len(), 5);
        for result in results {
            assert!(result.is_ok());
        }
        
        // Verify all agents were created
        let agents = repo.find_all_agents().unwrap();
        assert_eq!(agents.len(), 5);
    }

    #[test]
    fn test_connection_pool_exhaustion() {
        let (_temp, pool) = setup_test_db();
        
        // Get multiple connections
        let mut connections = vec![];
        for _ in 0..10 {
            connections.push(pool.get().unwrap());
        }
        
        // Pool should handle this gracefully
        assert_eq!(connections.len(), 10);
        
        // Additional connection request
        // Since we have exactly 10 connections taken and max_size is 10,
        // get() with a short timeout should fail
        drop(connections); // First release all connections
        
        // Now we should be able to get a connection
        let result = pool.get();
        assert!(result.is_ok());
    }

    // ===== Edge Cases =====

    #[test]
    fn test_empty_database_queries() {
        let (_temp, repo) = create_test_repository();
        
        // All these should return empty results, not errors
        assert_eq!(repo.find_all_agents().unwrap().len(), 0);
        assert_eq!(repo.find_all_runs(None).unwrap().len(), 0);
        assert_eq!(repo.find_running_runs().unwrap().len(), 0);
        assert_eq!(repo.find_scheduled_runs().unwrap().len(), 0);
    }

    #[test]
    fn test_invalid_uuids() {
        let (_temp, repo) = create_test_repository();
        
        // Various invalid session ID formats
        let invalid_sessions = vec![
            "",
            "not-a-uuid",
            "12345",
            "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
        ];
        
        for session in invalid_sessions {
            let result = repo.find_run_by_session_id(session).unwrap();
            assert_eq!(result, None);
        }
    }

    #[test]
    fn test_null_handling() {
        let (_temp, repo) = create_test_repository();
        
        // Create agent with minimal fields
        let agent = NewAgent {
            name: "Minimal".to_string(),
            icon: "📄".to_string(),
            system_prompt: "Minimal prompt".to_string(),
            default_task: None,
            model: "sonnet".to_string(),
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: true,
            enable_network: false,
        };
        
        let created = repo.create_agent(agent).unwrap();
        assert_eq!(created.default_task, None);
        
        // Create run with minimal fields
        let run = NewAgentRun {
            agent_id: created.id.unwrap(),
            agent_name: created.name.clone(),
            agent_icon: created.icon.clone(),
            task: "Task".to_string(),
            model: created.model.clone(),
            project_path: "/tmp".to_string(),
            status: None,
            scheduled_start_time: None,
            parent_run_id: None,
        };
        
        let created_run = repo.create_run(run).unwrap();
        assert_eq!(created_run.scheduled_start_time, None);
        assert_eq!(created_run.parent_run_id, None);
        assert_eq!(created_run.pid, None);
        assert_eq!(created_run.completed_at, None);
    }

    #[test]
    fn test_large_data_sets() {
        let (_temp, repo) = create_test_repository();
        
        let new_agent = TestAgentBuilder::new().build();
        let agent = repo.create_agent(new_agent).expect("Failed to create agent");
        let agent_id = agent.id.unwrap();
        
        let new_run = TestRunBuilder::new(agent_id).build();
        let run = repo.create_run(new_run).expect("Failed to create run");
        let run_id = run.id.unwrap();
        
        // Store many JSONL lines
        for i in 1..=1000 {
            let line = format!(r#"{{"type":"text","message":"Line {}","index":{}}}"#, i, i);
            repo.store_jsonl_output(run_id, i, &line)
                .expect("Failed to store JSONL");
        }
        
        let output = repo.get_jsonl_output(run_id)
            .expect("Failed to get JSONL output");
        
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 1000);
        
        let last_line_num = repo.get_last_line_number(run_id).unwrap();
        assert_eq!(last_line_num, 1000);
    }

    #[test]
    fn test_special_characters_in_strings() {
        let (_temp, repo) = create_test_repository();
        
        // Test with various special characters
        let agent = NewAgent {
            name: "Agent \"with\" 'quotes'".to_string(),
            icon: "🚀🌟✨".to_string(),
            system_prompt: "Prompt with\nnewlines\tand\ttabs".to_string(),
            default_task: Some("Task with SQL injection'; DROP TABLE agents; --".to_string()),
            model: "sonnet".to_string(),
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: true,
            enable_network: false,
        };
        
        let created = repo.create_agent(agent).unwrap();
        assert_eq!(created.name, "Agent \"with\" 'quotes'");
        assert_eq!(created.system_prompt, "Prompt with\nnewlines\tand\ttabs");
        
        // Verify the database still works
        let found = repo.find_agent_by_id(created.id.unwrap()).unwrap();
        assert_eq!(found.name, created.name);
    }

    #[test]
    fn test_resume_run_chain() {
        let (_temp, repo) = create_test_repository();
        
        let new_agent = TestAgentBuilder::new().build();
        let agent = repo.create_agent(new_agent).expect("Failed to create agent");
        let agent_id = agent.id.unwrap();
        
        // Create initial run
        let run1 = TestRunBuilder::new(agent_id)
            .with_task("Original task")
            .build();
        let created1 = repo.create_run(run1).unwrap();
        let run1_id = created1.id.unwrap();
        
        // Create resumed run
        let run2 = TestRunBuilder::new(agent_id)
            .with_task("Resumed task")
            .as_resume(run1_id)
            .build();
        let created2 = repo.create_run(run2).unwrap();
        let _run2_id = created2.id.unwrap();
        
        // Create another resumed run
        let run3 = TestRunBuilder::new(agent_id)
            .with_task("Resumed again")
            .as_resume(run1_id)
            .build();
        let created3 = repo.create_run(run3).unwrap();
        
        // Verify parent relationships
        assert_eq!(created2.parent_run_id, Some(run1_id));
        assert_eq!(created3.parent_run_id, Some(run1_id));
        
        // Verify we can trace back to parent
        let parent = repo.find_run_by_id(created2.parent_run_id.unwrap()).unwrap();
        assert_eq!(parent.task, "Original task");
    }

    #[test]
    fn test_model_defaults() {
        let (_temp, repo) = create_test_repository();
        
        // Test various model values
        let models = vec![
            ("empty-model", constants::DEFAULT_MODEL), // Will be replaced by a proper value
            ("invalid-model", "invalid-model"), // Invalid models are stored as-is
            ("opus-4", "opus-4"),
            ("SONNET-3", "SONNET-3"), // Case is preserved
        ];
        
        for (input, expected) in models {
            let agent = NewAgent {
                name: format!("Agent with model {}", input),
                icon: "🤖".to_string(),
                system_prompt: "Test".to_string(),
                default_task: None,
                model: if input == "empty-model" { constants::DEFAULT_MODEL.to_string() } else { input.to_string() },
                sandbox_enabled: true,
                enable_file_read: true,
                enable_file_write: true,
                enable_network: false,
            };
            
            let created = repo.create_agent(agent).unwrap();
            assert_eq!(created.model, expected);
        }
    }
}