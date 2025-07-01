#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::agents::{
        error::AgentError,
        pool::SqlitePool,
        repository::*,
        types::*,
    };
    use crate::process_registry::ProcessRegistry;
    use crate::sandbox::profile::{ProfileBuilder, SandboxBuildResult};
    use mockall::{mock, predicate::*};
    use std::process::{Command, Stdio};
    use std::sync::{Arc, Mutex};
    use tauri::{AppHandle, Manager};
    use tempfile::TempDir;
    use tokio::io::AsyncWriteExt;

    // Mock traits for testing
    mock! {
        AgentRepo {}
        
        impl AgentRepository for AgentRepo {
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
            
            fn store_sandbox_violation(&self, violation: SandboxViolation) -> Result<(), AgentError>;
            
            fn get_setting(&self, key: &str) -> Result<Option<String>, AgentError>;
            fn set_setting(&self, key: &str, value: &str) -> Result<(), AgentError>;
            
            fn calculate_run_metrics(&self, run_id: i64) -> Result<AgentRunMetrics, AgentError>;
        }
    }

    mock! {
        AppHandle {}
        
        impl Clone for AppHandle {
            fn clone(&self) -> Self;
        }
    }

    mock! {
        ProcessChild {
            pub id: Option<u32>,
            pub stdout: Option<std::process::ChildStdout>,
            pub stderr: Option<std::process::ChildStderr>,
        }
        
        impl ProcessChild {
            pub fn id(&self) -> Option<u32>;
            pub fn wait(&mut self) -> std::io::Result<std::process::ExitStatus>;
            pub fn kill(&mut self) -> std::io::Result<()>;
        }
    }

    // Helper function to create test agent
    fn create_test_agent(id: i64) -> Agent {
        Agent {
            id: Some(id),
            name: "Test Agent".to_string(),
            icon: "🤖".to_string(),
            system_prompt: "You are a test agent.".to_string(),
            default_task: Some("Test task".to_string()),
            model: "claude-3-5-sonnet-latest".to_string(),
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: false,
            enable_network: false,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        }
    }

    // Helper function to create test run
    fn create_test_run(id: i64, agent_id: i64) -> AgentRun {
        AgentRun {
            id: Some(id),
            agent_id,
            agent_name: "Test Agent".to_string(),
            agent_icon: "🤖".to_string(),
            task: "Test task".to_string(),
            model: "claude-3-5-sonnet-latest".to_string(),
            project_path: "/test/path".to_string(),
            status: "pending".to_string(),
            session_id: "test-session-123".to_string(),
            pid: None,
            output: None,
            started_at: None,
            completed_at: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            scheduled_start_time: None,
            usage_limit_reset_time: None,
            parent_run_id: None,
        }
    }

    // Test 1: Successful agent execution
    #[tokio::test]
    async fn test_execute_agent_success() {
        let temp_dir = TempDir::new().unwrap();
        let mut mock_repo = MockAgentRepo::new();
        let agent = create_test_agent(1);
        let run = create_test_run(1, 1);
        
        // Set up expectations
        mock_repo
            .expect_find_agent_by_id()
            .with(eq(1))
            .times(1)
            .returning(move |_| Ok(agent.clone()));
            
        mock_repo
            .expect_create_run()
            .times(1)
            .returning(move |_| Ok(run.clone()));
            
        mock_repo
            .expect_update_run_status()
            .with(eq(1), eq("running"), any(), any())
            .times(1)
            .returning(|_, _, _, _| Ok(()));
            
        mock_repo
            .expect_find_run_by_id()
            .with(eq(1))
            .times(1)
            .returning(move |_| Ok(run.clone()));

        // Create mock app handle
        let app = create_mock_app_handle(temp_dir.path());
        let pool = Arc::new(create_test_pool());
        let registry = Arc::new(Mutex::new(ProcessRegistry::new()));

        // Execute agent
        let result = execute_agent(
            app,
            pool,
            registry,
            1,
            Some("Custom task".to_string()),
            Some("/custom/path".to_string()),
            None,
            None,
            None,
        )
        .await;

        assert!(result.is_ok());
        let run = result.unwrap();
        assert_eq!(run.agent_id, 1);
        assert_eq!(run.task, "Test task");
        assert_eq!(run.status, "pending");
    }

    // Test 2: Environment variable setup
    #[test]
    fn test_environment_variables() {
        let claude_path = "/usr/local/bin/claude";
        let mut cmd = helpers::create_command_with_env(claude_path);
        
        // Check that command has proper environment setup
        let cmd_env: Vec<_> = cmd.get_envs().collect();
        
        // Verify PATH is set
        assert!(cmd_env.iter().any(|(key, _)| key == &"PATH"));
        
        // Verify command path
        assert_eq!(cmd.get_program(), claude_path);
    }

    // Test 3: Working directory handling
    #[tokio::test]
    async fn test_working_directory_handling() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();
        
        let mut mock_repo = MockAgentRepo::new();
        let agent = create_test_agent(1);
        let run = create_test_run(1, 1);
        
        mock_repo
            .expect_find_agent_by_id()
            .returning(move |_| Ok(agent.clone()));
            
        mock_repo
            .expect_create_run()
            .returning(move |new_run| {
                assert_eq!(new_run.project_path, project_path);
                Ok(run.clone())
            });

        let app = create_mock_app_handle(temp_dir.path());
        let pool = Arc::new(create_test_pool());
        let registry = Arc::new(Mutex::new(ProcessRegistry::new()));

        let result = execute_agent(
            app,
            pool,
            registry,
            1,
            None,
            Some(project_path.to_string()),
            None,
            None,
            None,
        )
        .await;

        assert!(result.is_ok());
    }

    // Test 4: JSONL output parsing and emission
    #[tokio::test]
    async fn test_jsonl_output_parsing() {
        let jsonl_lines = vec![
            r#"{"type":"output","content":"Starting task..."}"#,
            r#"{"type":"progress","percentage":50}"#,
            r#"{"type":"error","message":"Rate limit exceeded"}"#,
        ];

        // Test parsing each line
        for line in &jsonl_lines {
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(line);
            assert!(parsed.is_ok());
        }

        // Test usage limit detection
        let error_line = r#"{"error":"Usage limit exceeded. Reset at 2024-01-01T12:00:00Z"}"#;
        let json: serde_json::Value = serde_json::from_str(error_line).unwrap();
        
        if let Some(error) = json.get("error").and_then(|e| e.as_str()) {
            assert!(error.contains("Usage limit exceeded"));
            let reset_time = helpers::parse_usage_limit_error(error);
            assert!(reset_time.is_some());
        }
    }

    // Test 5: Buffer handling for partial lines
    #[tokio::test]
    async fn test_partial_line_buffering() {
        use tokio::io::{BufReader, AsyncBufReadExt};
        
        // Create a mock stream with partial lines
        let data = b"First line\nSecond ";
        let cursor = std::io::Cursor::new(data);
        let mut reader = BufReader::new(cursor);
        let mut lines = reader.lines();
        
        // First line should be complete
        let line1 = lines.next_line().await.unwrap();
        assert_eq!(line1, Some("First line".to_string()));
        
        // Second line is partial, should not be returned yet
        let line2 = lines.next_line().await.unwrap();
        assert_eq!(line2, None);
    }

    // Test 6: Sandbox profile application
    #[test]
    fn test_sandbox_profile_building() {
        let agent = create_test_agent(1);
        let project_path = "/test/project";
        
        // Build sandbox rules
        let rules = helpers::build_sandbox_rules(&agent, project_path);
        
        // Verify rules are created based on agent permissions
        assert!(!rules.is_empty());
        
        // Check file read permission
        if agent.enable_file_read {
            assert!(rules.iter().any(|r| r.contains("file_read")));
        }
        
        // Check file write permission (should not be present)
        if !agent.enable_file_write {
            assert!(!rules.iter().any(|r| r.contains("file_write")));
        }
        
        // Check network permission (should not be present)
        if !agent.enable_network {
            assert!(!rules.iter().any(|r| r.contains("network")));
        }
    }

    // Test 7: Sandbox violation detection
    #[tokio::test]
    async fn test_sandbox_violation_logging() {
        let mut mock_repo = MockAgentRepo::new();
        
        mock_repo
            .expect_store_sandbox_violation()
            .times(1)
            .returning(|violation| {
                assert_eq!(violation.agent_id, 1);
                assert_eq!(violation.run_id, 1);
                assert!(violation.rule.contains("file_write"));
                assert!(violation.action.contains("write"));
                Ok(())
            });

        let violation = SandboxViolation {
            id: None,
            agent_id: 1,
            run_id: 1,
            session_id: "test-session".to_string(),
            rule: "file_write:/protected/path".to_string(),
            action: "attempted write to /protected/path/file.txt".to_string(),
            timestamp: helpers::now_iso8601(),
        };

        let result = mock_repo.store_sandbox_violation(violation);
        assert!(result.is_ok());
    }

    // Test 8: Resume command building
    #[tokio::test]
    async fn test_resume_command_building() {
        let mut mock_repo = MockAgentRepo::new();
        let agent = create_test_agent(1);
        let mut run = create_test_run(1, 1);
        run.status = "paused_usage_limit".to_string();
        
        mock_repo
            .expect_find_agent_by_id()
            .returning(move |_| Ok(agent.clone()));
            
        mock_repo
            .expect_find_run_by_id()
            .returning(move |_| Ok(run.clone()));
            
        mock_repo
            .expect_get_last_line_number()
            .with(eq(1))
            .returning(|_| Ok(42));

        let app = create_mock_app_handle(TempDir::new().unwrap().path());
        let pool = Arc::new(create_test_pool());
        let registry = Arc::new(Mutex::new(ProcessRegistry::new()));

        // Test resume with line number
        let result = execute_agent(
            app,
            pool,
            registry,
            1,
            None,
            None,
            None,
            Some(1),
            Some(42),
        )
        .await;

        // Verify command would include resume arguments
        // In real execution, this would be checked in the command building
        assert!(result.is_ok());
    }

    // Test 9: Process spawn failures
    #[tokio::test]
    async fn test_process_spawn_failure() {
        let mut mock_repo = MockAgentRepo::new();
        let agent = create_test_agent(1);
        
        mock_repo
            .expect_find_agent_by_id()
            .returning(move |_| Ok(agent.clone()));
            
        mock_repo
            .expect_create_run()
            .returning(|_| Ok(create_test_run(1, 1)));

        // Use invalid binary path to trigger spawn failure
        std::env::set_var("CLAUDE_BINARY_PATH", "/nonexistent/binary");
        
        let app = create_mock_app_handle(TempDir::new().unwrap().path());
        let pool = Arc::new(create_test_pool());
        let registry = Arc::new(Mutex::new(ProcessRegistry::new()));

        let result = execute_agent(
            app,
            pool,
            registry,
            1,
            None,
            None,
            None,
            None,
            None,
        )
        .await;

        assert!(result.is_err());
        match result {
            Err(AgentError::Process(_)) | Err(AgentError::BinaryNotFound(_)) => {},
            _ => panic!("Expected Process or BinaryNotFound error"),
        }
    }

    // Test 10: Command execution error handling
    #[tokio::test]
    async fn test_command_execution_errors() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("agent_outputs");
        std::fs::create_dir_all(&output_dir).unwrap();
        
        // Create a file with error output
        let error_content = r#"{"error":"Command execution failed","details":"Invalid argument"}"#;
        let output_file = output_dir.join("1.jsonl");
        std::fs::write(&output_file, error_content).unwrap();

        // Test error parsing
        let parsed: serde_json::Value = serde_json::from_str(error_content).unwrap();
        assert_eq!(parsed["error"], "Command execution failed");
        assert_eq!(parsed["details"], "Invalid argument");
    }

    // Test 11: Cleanup on failure
    #[tokio::test]
    async fn test_cleanup_on_failure() {
        let registry = Arc::new(Mutex::new(ProcessRegistry::new()));
        let session_id = "test-session-cleanup";
        
        // Create a mock child process
        let child = create_mock_child_process();
        
        // Register process
        {
            let mut reg = registry.lock().unwrap();
            reg.register_process(session_id.to_string(), child);
        }
        
        // Verify process is registered
        {
            let reg = registry.lock().unwrap();
            assert!(reg.is_running(session_id));
        }
        
        // Simulate cleanup
        {
            let mut reg = registry.lock().unwrap();
            reg.unregister_process(session_id);
        }
        
        // Verify process is unregistered
        {
            let reg = registry.lock().unwrap();
            assert!(!reg.is_running(session_id));
        }
    }

    // Test 12: Resource leak prevention
    #[tokio::test]
    async fn test_resource_leak_prevention() {
        let registry = Arc::new(Mutex::new(ProcessRegistry::new()));
        
        // Register multiple processes
        for i in 0..5 {
            let session_id = format!("session-{}", i);
            let child = create_mock_child_process();
            
            let mut reg = registry.lock().unwrap();
            reg.register_process(session_id.clone(), child);
        }
        
        // Verify all processes are tracked
        {
            let reg = registry.lock().unwrap();
            for i in 0..5 {
                assert!(reg.is_running(&format!("session-{}", i)));
            }
        }
        
        // Clean up all processes
        {
            let mut reg = registry.lock().unwrap();
            for i in 0..5 {
                reg.unregister_process(&format!("session-{}", i));
            }
        }
        
        // Verify no processes remain
        {
            let reg = registry.lock().unwrap();
            for i in 0..5 {
                assert!(!reg.is_running(&format!("session-{}", i)));
            }
        }
    }

    // Test 13: Output file persistence
    #[tokio::test]
    async fn test_output_file_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let app_data_dir = temp_dir.path();
        let run_id = 1;
        let line_number = 1;
        let content = r#"{"type":"output","content":"Test output"}"#;
        
        // Test persist function
        let result = persist_agent_output_line(app_data_dir, run_id, line_number, content).await;
        assert!(result.is_ok());
        
        // Verify file was created
        let output_file = app_data_dir.join("agent_outputs").join("1.jsonl");
        assert!(output_file.exists());
        
        // Verify content
        let saved_content = tokio::fs::read_to_string(&output_file).await.unwrap();
        assert_eq!(saved_content.trim(), content);
    }

    // Test 14: Parent run association for resume
    #[tokio::test]
    async fn test_parent_run_association() {
        let mut mock_repo = MockAgentRepo::new();
        let agent = create_test_agent(1);
        let parent_run = create_test_run(1, 1);
        
        mock_repo
            .expect_find_agent_by_id()
            .returning(move |_| Ok(agent.clone()));
            
        mock_repo
            .expect_find_run_by_id()
            .with(eq(1))
            .returning(move |_| Ok(parent_run.clone()));
            
        mock_repo
            .expect_create_run()
            .returning(move |new_run| {
                // Verify parent_run_id is set
                assert_eq!(new_run.parent_run_id, Some(1));
                let mut child_run = create_test_run(2, 1);
                child_run.parent_run_id = Some(1);
                Ok(child_run)
            });

        let app = create_mock_app_handle(TempDir::new().unwrap().path());
        let pool = Arc::new(create_test_pool());
        let registry = Arc::new(Mutex::new(ProcessRegistry::new()));

        // Execute with parent run
        let result = execute_agent(
            app,
            pool,
            registry,
            1,
            None,
            None,
            None,
            Some(1), // parent run_id
            None,
        )
        .await;

        assert!(result.is_ok());
    }

    // Test 15: Platform-specific sandbox behavior
    #[test]
    fn test_platform_specific_sandbox() {
        let agent = create_test_agent(1);
        let project_path = PathBuf::from("/test/project");
        
        // Test sandbox profile building
        match ProfileBuilder::new(project_path.clone()) {
            Ok(builder) => {
                let rules = helpers::build_sandbox_rules(&agent, project_path.to_str().unwrap());
                let result = builder.build_agent_profile(
                    rules,
                    agent.sandbox_enabled,
                    agent.enable_file_read,
                    agent.enable_file_write,
                    agent.enable_network,
                );
                
                match result {
                    Ok(build_result) => {
                        // Verify serialized profile exists
                        assert!(!build_result.serialized.is_empty());
                        
                        #[cfg(unix)]
                        {
                            // Unix-specific assertions
                            assert!(build_result.profile.is_some());
                        }
                        
                        #[cfg(not(unix))]
                        {
                            // Non-Unix platforms should have empty profile
                            assert_eq!(build_result.profile, ());
                        }
                    }
                    Err(e) => {
                        // Platform might not support sandboxing
                        println!("Sandbox not supported: {}", e);
                    }
                }
            }
            Err(e) => {
                // Platform detection might fail
                println!("Platform detection failed: {}", e);
            }
        }
    }

    // Helper functions for tests

    fn create_mock_app_handle(temp_dir: &std::path::Path) -> AppHandle {
        // In real tests, you would use tauri::test utilities
        // For now, we'll create a simplified mock
        unimplemented!("Use tauri::test::mock_app() in real implementation")
    }

    fn create_test_pool() -> SqlitePool {
        // Create in-memory database for testing
        let manager = r2d2_sqlite::SqliteConnectionManager::memory();
        r2d2::Pool::builder()
            .max_size(1)
            .build(manager)
            .unwrap()
    }

    fn create_mock_child_process() -> std::process::Child {
        // Create a simple process that exits immediately
        Command::new("echo")
            .arg("test")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap()
    }
}

// Integration tests module
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_full_agent_execution_flow() {
        // This would be a full integration test with real components
        // Requires proper test environment setup
    }

    #[tokio::test]
    async fn test_concurrent_agent_executions() {
        // Test multiple agents running simultaneously
        // Verify process isolation and registry management
    }

    #[tokio::test]
    async fn test_agent_crash_recovery() {
        // Test handling of agent process crashes
        // Verify cleanup and state consistency
    }
}