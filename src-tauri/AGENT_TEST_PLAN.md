# Comprehensive Test Plan for Refactored Agents Module

## Executive Summary

This test plan provides comprehensive coverage for the refactored agents module in the Claudia application. The plan is organized by test priority (Critical, High, Medium, Low) and includes specific test cases for each module.

### Coverage Targets
- **Critical Paths**: 100% coverage (types, database operations, process management)
- **High Priority**: >90% coverage (service layer, error handling, command validation)
- **Medium Priority**: >85% coverage (helpers, serialization, event emission)
- **Overall Target**: >85% coverage

## Test Categories by Priority

### Critical Priority Tests
Focus on core functionality that could cause data loss or system failure:
- Type safety and newtype wrapper validation
- Database CRUD operations and transaction handling
- Process lifecycle management (start, stop, cleanup)
- Session isolation and data integrity

### High Priority Tests
Essential business logic and error handling:
- Service layer operations
- Builder pattern validation
- Error conversions and propagation
- Command parameter validation
- Sandbox rule generation

### Medium Priority Tests
Supporting functionality and edge cases:
- Helper function correctness
- Cost calculation accuracy
- Datetime parsing and formatting
- Platform-specific code paths
- Event emission correctness

### Low Priority Tests
Simple utilities and constants:
- Display trait implementations
- Getter methods
- Constant values
- Simple formatting functions

## Module-by-Module Test Requirements

### 1. types.rs - Type Safety and Validation

#### Critical Tests
```rust
#[cfg(test)]
mod newtype_tests {
    use super::*;

    // AgentId validation
    #[test]
    fn test_agent_id_validation() {
        // Valid cases
        assert!(AgentId::new(1).is_ok());
        assert!(AgentId::new(i64::MAX).is_ok());
        
        // Invalid cases
        assert!(AgentId::new(0).is_err());
        assert!(AgentId::new(-1).is_err());
    }

    #[test]
    fn test_agent_id_from_str() {
        assert_eq!(AgentId::from_str("123").unwrap().inner(), 123);
        assert!(AgentId::from_str("0").is_err());
        assert!(AgentId::from_str("abc").is_err());
        assert!(AgentId::from_str("").is_err());
    }

    // RunId validation
    #[test]
    fn test_run_id_validation() {
        assert!(RunId::new(1).is_ok());
        assert!(RunId::new(0).is_err());
        assert!(RunId::new(-1).is_err());
    }

    // SessionId validation
    #[test]
    fn test_session_id_validation() {
        // Valid UUID
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(SessionId::new(valid_uuid.to_string()).is_ok());
        
        // Invalid cases
        assert!(SessionId::new("".to_string()).is_err());
        assert!(SessionId::new("not-a-uuid".to_string()).is_err());
        assert!(SessionId::new("123".to_string()).is_err());
    }

    #[test]
    fn test_session_id_generation() {
        let id1 = SessionId::generate();
        let id2 = SessionId::generate();
        assert_ne!(id1.inner(), id2.inner());
        assert!(SessionId::is_valid_uuid(id1.inner()));
    }
}

#[cfg(test)]
mod builder_tests {
    use super::*;

    #[test]
    fn test_agent_create_builder_success() {
        let agent = AgentCreate::builder()
            .name("Test Agent")
            .icon("🤖")
            .system_prompt("You are a helpful assistant")
            .model(ModelType::Opus4)
            .build()
            .unwrap();
        
        assert_eq!(agent.name, "Test Agent");
        assert_eq!(agent.model, "opus-4");
    }

    #[test]
    fn test_agent_create_builder_missing_required() {
        // Missing name
        let result = AgentCreate::builder()
            .icon("🤖")
            .system_prompt("prompt")
            .build();
        assert!(result.is_err());
        
        // Empty name
        let result = AgentCreate::builder()
            .name("")
            .icon("🤖")
            .system_prompt("prompt")
            .build();
        assert!(result.is_err());
        
        // Whitespace-only name
        let result = AgentCreate::builder()
            .name("   ")
            .icon("🤖")
            .system_prompt("prompt")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_agent_create_validation() {
        let mut agent = AgentCreate {
            name: "Test".to_string(),
            icon: "🤖".to_string(),
            system_prompt: "prompt".to_string(),
            default_task: None,
            model: "invalid-model".to_string(),
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: true,
            enable_network: false,
        };
        
        assert!(agent.validate().is_ok());
        
        agent.name = "   ".to_string();
        assert!(agent.validate().is_err());
    }
}

#[cfg(test)]
mod enum_tests {
    use super::*;

    #[test]
    fn test_run_status_terminal_states() {
        assert!(RunStatus::Completed.is_terminal());
        assert!(RunStatus::Failed.is_terminal());
        assert!(RunStatus::Cancelled.is_terminal());
        assert!(!RunStatus::Running.is_terminal());
        assert!(!RunStatus::Pending.is_terminal());
    }

    #[test]
    fn test_run_status_active_states() {
        assert!(RunStatus::Running.is_active());
        assert!(!RunStatus::Pending.is_active());
        assert!(!RunStatus::Completed.is_active());
    }

    #[test]
    fn test_model_type_parsing() {
        assert_eq!(ModelType::from_str("opus-4"), ModelType::Opus4);
        assert_eq!(ModelType::from_str("OPUS4"), ModelType::Opus4);
        assert_eq!(ModelType::from_str("unknown"), ModelType::Sonnet); // default
    }

    #[test]
    fn test_model_pricing() {
        let (input, output, cache_write, cache_read) = ModelType::Opus4.get_pricing();
        assert_eq!(input, OPUS_4_INPUT_PRICE);
        assert_eq!(output, OPUS_4_OUTPUT_PRICE);
        assert_eq!(cache_write, OPUS_4_CACHE_WRITE_PRICE);
        assert_eq!(cache_read, OPUS_4_CACHE_READ_PRICE);
    }
}
```

### 2. error.rs - Error Handling

#### High Priority Tests
```rust
#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn test_error_conversions() {
        // From rusqlite::Error
        let sqlite_err = rusqlite::Error::SqliteSingleThreadedMode;
        let agent_err: AgentError = sqlite_err.into();
        assert!(matches!(agent_err, AgentError::Database(_)));
        
        // From std::io::Error
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let agent_err: AgentError = io_err.into();
        assert!(matches!(agent_err, AgentError::Io(_)));
        
        // From serde_json::Error
        let json_err = serde_json::from_str::<String>("invalid").unwrap_err();
        let agent_err: AgentError = json_err.into();
        assert!(matches!(agent_err, AgentError::Serialization(_)));
    }

    #[test]
    fn test_error_to_string() {
        let err = AgentError::AgentNotFound(123);
        assert_eq!(err.to_string(), "Agent not found: 123");
        
        let err = AgentError::RunNotFound(456);
        assert_eq!(err.to_string(), "Run not found: 456");
        
        let err = AgentError::InvalidStatus("bad_status".to_string());
        assert_eq!(err.to_string(), "Invalid status: bad_status");
    }

    #[test]
    fn test_error_serialization() {
        let err = AgentError::Process("Process failed".to_string());
        let serialized = serde_json::to_string(&err).unwrap();
        assert_eq!(serialized, "\"Process error: Process failed\"");
    }
}
```

### 3. repository.rs - Database Operations

#### Critical Tests
```rust
#[cfg(test)]
mod repository_tests {
    use super::*;
    use tempfile::tempdir;

    fn setup_test_db() -> SqlitePool {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let pool = create_pool(db_path).unwrap();
        init_pool_db(&pool).unwrap();
        pool
    }

    #[test]
    fn test_agent_crud_operations() {
        let pool = setup_test_db();
        let repo = SqliteAgentRepository::new(pool);
        
        // Create
        let new_agent = NewAgent {
            name: "Test Agent".to_string(),
            icon: "🤖".to_string(),
            system_prompt: "Test prompt".to_string(),
            default_task: Some("Default task".to_string()),
            model: "opus-4".to_string(),
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: false,
            enable_network: false,
        };
        
        let created = repo.create_agent(new_agent).unwrap();
        assert!(created.id.is_some());
        assert_eq!(created.name, "Test Agent");
        
        // Read
        let agent_id = created.id.unwrap();
        let fetched = repo.find_agent_by_id(agent_id).unwrap();
        assert_eq!(fetched.name, created.name);
        
        // Update
        let update = UpdateAgent {
            name: "Updated Agent".to_string(),
            icon: "🤖".to_string(),
            system_prompt: "Updated prompt".to_string(),
            default_task: None,
            model: "sonnet-4".to_string(),
            sandbox_enabled: Some(false),
            enable_file_read: Some(false),
            enable_file_write: Some(true),
            enable_network: Some(true),
        };
        
        let updated = repo.update_agent(agent_id, update).unwrap();
        assert_eq!(updated.name, "Updated Agent");
        assert_eq!(updated.model, "sonnet-4");
        assert!(!updated.sandbox_enabled);
        
        // List
        let all_agents = repo.find_all_agents().unwrap();
        assert_eq!(all_agents.len(), 1);
        
        // Delete
        repo.delete_agent(agent_id).unwrap();
        let result = repo.find_agent_by_id(agent_id);
        assert!(matches!(result, Err(AgentError::AgentNotFound(_))));
    }

    #[test]
    fn test_run_lifecycle() {
        let pool = setup_test_db();
        let repo = SqliteAgentRepository::new(pool);
        
        // Create agent first
        let agent = repo.create_agent(NewAgent {
            name: "Test".to_string(),
            icon: "🤖".to_string(),
            system_prompt: "Test".to_string(),
            default_task: None,
            model: "sonnet".to_string(),
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: true,
            enable_network: false,
        }).unwrap();
        
        let agent_id = agent.id.unwrap();
        
        // Create run
        let new_run = NewAgentRun {
            agent_id,
            agent_name: "Test".to_string(),
            agent_icon: "🤖".to_string(),
            task: "Test task".to_string(),
            model: "sonnet".to_string(),
            project_path: "/test/path".to_string(),
            status: Some("pending"),
            scheduled_start_time: None,
            parent_run_id: None,
        };
        
        let run = repo.create_run(new_run).unwrap();
        let run_id = run.id.unwrap();
        assert_eq!(run.status, "pending");
        
        // Update status to running
        let started_at = chrono::Utc::now().to_rfc3339();
        repo.update_run_status(run_id, "running", Some(12345), Some(started_at)).unwrap();
        
        let updated = repo.find_run_by_id(run_id).unwrap();
        assert_eq!(updated.status, "running");
        assert_eq!(updated.pid, Some(12345));
        
        // Complete run
        repo.update_run_completion(run_id, "completed").unwrap();
        
        let completed = repo.find_run_by_id(run_id).unwrap();
        assert_eq!(completed.status, "completed");
        assert!(completed.completed_at.is_some());
    }

    #[test]
    fn test_jsonl_storage() {
        let pool = setup_test_db();
        let repo = SqliteAgentRepository::new(pool);
        
        // Store lines
        repo.store_jsonl_output(1, 1, r#"{"type": "text", "message": "Line 1"}"#).unwrap();
        repo.store_jsonl_output(1, 2, r#"{"type": "text", "message": "Line 2"}"#).unwrap();
        repo.store_jsonl_output(1, 3, r#"{"type": "text", "message": "Line 3"}"#).unwrap();
        
        // Get last line number
        let last_line = repo.get_last_line_number(1).unwrap();
        assert_eq!(last_line, 3);
        
        // Get all output
        let output = repo.get_jsonl_output(1).unwrap();
        assert!(output.contains("Line 1"));
        assert!(output.contains("Line 2"));
        assert!(output.contains("Line 3"));
        
        // Test overwrite
        repo.store_jsonl_output(1, 2, r#"{"type": "text", "message": "Updated Line 2"}"#).unwrap();
        let output = repo.get_jsonl_output(1).unwrap();
        assert!(output.contains("Updated Line 2"));
        assert!(!output.contains("\"Line 2\""));
    }

    #[test]
    fn test_run_queries() {
        let pool = setup_test_db();
        let repo = SqliteAgentRepository::new(pool);
        
        // Create test data
        let agent = repo.create_agent(NewAgent {
            name: "Test".to_string(),
            icon: "🤖".to_string(),
            system_prompt: "Test".to_string(),
            default_task: None,
            model: "sonnet".to_string(),
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: true,
            enable_network: false,
        }).unwrap();
        
        let agent_id = agent.id.unwrap();
        
        // Create multiple runs with different statuses
        for (i, status) in ["pending", "running", "completed", "scheduled"].iter().enumerate() {
            let mut new_run = NewAgentRun {
                agent_id,
                agent_name: "Test".to_string(),
                agent_icon: "🤖".to_string(),
                task: format!("Task {}", i),
                model: "sonnet".to_string(),
                project_path: "/test".to_string(),
                status: Some(status),
                scheduled_start_time: if *status == "scheduled" {
                    Some(chrono::Utc::now().to_rfc3339())
                } else {
                    None
                },
                parent_run_id: None,
            };
            
            repo.create_run(new_run).unwrap();
        }
        
        // Test queries
        let all_runs = repo.find_all_runs(None).unwrap();
        assert_eq!(all_runs.len(), 4);
        
        let agent_runs = repo.find_all_runs(Some(agent_id)).unwrap();
        assert_eq!(agent_runs.len(), 4);
        
        let running_runs = repo.find_running_runs().unwrap();
        assert_eq!(running_runs.len(), 1);
        
        let scheduled_runs = repo.find_scheduled_runs().unwrap();
        assert_eq!(scheduled_runs.len(), 1);
    }

    #[test]
    fn test_sandbox_violations() {
        let pool = setup_test_db();
        let repo = SqliteAgentRepository::new(pool);
        
        let violation = SandboxViolation {
            run_id: 1,
            operation_type: "file_write".to_string(),
            resource: "/etc/passwd".to_string(),
            reason: "Access denied".to_string(),
        };
        
        repo.store_sandbox_violation(violation).unwrap();
        
        // Verify stored (would need a query method in real implementation)
        let conn = pool.get().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sandbox_violations WHERE run_id = 1",
            [],
            |row| row.get(0)
        ).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_settings() {
        let pool = setup_test_db();
        let repo = SqliteAgentRepository::new(pool);
        
        // Get non-existent setting
        let value = repo.get_setting("test_key").unwrap();
        assert!(value.is_none());
        
        // Set setting
        repo.set_setting("test_key", "test_value").unwrap();
        
        // Get existing setting
        let value = repo.get_setting("test_key").unwrap();
        assert_eq!(value, Some("test_value".to_string()));
        
        // Update setting
        repo.set_setting("test_key", "updated_value").unwrap();
        let value = repo.get_setting("test_key").unwrap();
        assert_eq!(value, Some("updated_value".to_string()));
    }

    #[test]
    fn test_calculate_run_metrics() {
        let pool = setup_test_db();
        let repo = SqliteAgentRepository::new(pool);
        
        // Store JSONL with metrics
        let run_id = 1;
        repo.store_jsonl_output(run_id, 1, r#"{
            "type": "text",
            "timestamp": "2024-01-01T00:00:00Z",
            "message": "Starting"
        }"#).unwrap();
        
        repo.store_jsonl_output(run_id, 2, r#"{
            "type": "text",
            "timestamp": "2024-01-01T00:00:30Z",
            "message": "Processing",
            "usage": {
                "totalTokens": 1000,
                "cacheReadTokens": 100
            },
            "cost": 0.05
        }"#).unwrap();
        
        repo.store_jsonl_output(run_id, 3, r#"{
            "type": "text",
            "timestamp": "2024-01-01T00:01:00Z",
            "message": "Complete",
            "usage": {
                "totalTokens": 500
            },
            "cost": 0.025
        }"#).unwrap();
        
        let metrics = repo.calculate_run_metrics(run_id).unwrap();
        assert_eq!(metrics.duration_ms, Some(60000)); // 1 minute
        assert_eq!(metrics.total_tokens, Some(1600)); // 1000 + 500 + 100
        assert_eq!(metrics.cost_usd, Some(0.075)); // 0.05 + 0.025
        assert_eq!(metrics.message_count, Some(3));
    }

    #[test]
    fn test_transaction_rollback() {
        let pool = setup_test_db();
        let conn = pool.get().unwrap();
        
        // Start transaction
        let tx = conn.unchecked_transaction().unwrap();
        
        // Insert agent
        tx.execute(
            "INSERT INTO agents (name, icon, system_prompt) VALUES (?, ?, ?)",
            params!["Test", "🤖", "Test"]
        ).unwrap();
        
        // Rollback
        tx.rollback().unwrap();
        
        // Verify not inserted
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM agents",
            [],
            |row| row.get(0)
        ).unwrap();
        assert_eq!(count, 0);
    }
}
```

### 4. service.rs - Business Logic

#### High Priority Tests
```rust
#[cfg(test)]
mod service_tests {
    use super::*;
    use tauri::test::{mock_builder, MockRuntime};

    async fn setup_test_service() -> (AgentService, Arc<SqlitePool>) {
        let app = mock_builder().build(tauri::generate_context!()).unwrap();
        let app_handle = app.handle().clone();
        
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let pool = Arc::new(create_pool(db_path).unwrap());
        init_pool_db(&pool).unwrap();
        
        let service = AgentService::new(app_handle);
        (service, pool)
    }

    #[tokio::test]
    async fn test_create_agent_with_defaults() {
        let (service, pool) = setup_test_service().await;
        
        let agent = service.create_agent(
            pool,
            "Test Agent".to_string(),
            "🤖".to_string(),
            "Test prompt".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
        ).await.unwrap();
        
        assert_eq!(agent.name, "Test Agent");
        assert_eq!(agent.model, DEFAULT_MODEL);
        assert_eq!(agent.sandbox_enabled, DEFAULT_SANDBOX_ENABLED);
        assert_eq!(agent.enable_file_read, DEFAULT_FILE_READ_ENABLED);
        assert_eq!(agent.enable_file_write, DEFAULT_FILE_WRITE_ENABLED);
        assert_eq!(agent.enable_network, DEFAULT_NETWORK_ENABLED);
    }

    #[tokio::test]
    async fn test_execute_agent_run() {
        let (service, pool) = setup_test_service().await;
        let registry = Arc::new(Mutex::new(ProcessRegistry::new()));
        
        // Create agent
        let agent = service.create_agent(
            pool.clone(),
            "Test".to_string(),
            "🤖".to_string(),
            "Test".to_string(),
            Some("echo test".to_string()),
            None,
            None,
            None,
            None,
            None,
        ).await.unwrap();
        
        // Execute (would need mock binary in real test)
        let result = service.execute_agent(
            pool,
            registry,
            agent.id.unwrap(),
            None,
            None,
            None,
            None,
        ).await;
        
        // Would fail without proper mock setup
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_runs_with_metrics() {
        let (service, pool) = setup_test_service().await;
        
        // Create test data
        let repo = SqliteAgentRepository::new(pool.as_ref().clone());
        
        // Create agent
        let agent = repo.create_agent(NewAgent {
            name: "Test".to_string(),
            icon: "🤖".to_string(),
            system_prompt: "Test".to_string(),
            default_task: None,
            model: "sonnet".to_string(),
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: true,
            enable_network: false,
        }).unwrap();
        
        // Create run
        let run = repo.create_run(NewAgentRun {
            agent_id: agent.id.unwrap(),
            agent_name: "Test".to_string(),
            agent_icon: "🤖".to_string(),
            task: "Test".to_string(),
            model: "sonnet".to_string(),
            project_path: "/test".to_string(),
            status: Some("completed"),
            scheduled_start_time: None,
            parent_run_id: None,
        }).unwrap();
        
        // Add JSONL output
        repo.store_jsonl_output(
            run.id.unwrap(),
            1,
            r#"{"type": "text", "timestamp": "2024-01-01T00:00:00Z", "cost": 0.01}"#
        ).unwrap();
        
        // List with metrics
        let runs = service.list_runs_with_metrics(pool, None).await.unwrap();
        assert_eq!(runs.len(), 1);
        assert!(runs[0].metrics.is_some());
        assert_eq!(runs[0].metrics.as_ref().unwrap().cost_usd, Some(0.01));
    }

    #[tokio::test]
    async fn test_cancel_run() {
        let (service, pool) = setup_test_service().await;
        let registry = Arc::new(Mutex::new(ProcessRegistry::new()));
        
        // Would need proper setup with running process
        let result = service.cancel_run(pool, registry, 999).await;
        assert!(result.is_err()); // Run not found
    }

    #[tokio::test]
    async fn test_delete_agent_with_runs() {
        let (service, pool) = setup_test_service().await;
        
        // Create agent
        let agent = service.create_agent(
            pool.clone(),
            "Test".to_string(),
            "🤖".to_string(),
            "Test".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
        ).await.unwrap();
        
        let agent_id = agent.id.unwrap();
        
        // Create run
        let repo = SqliteAgentRepository::new(pool.as_ref().clone());
        repo.create_run(NewAgentRun {
            agent_id,
            agent_name: "Test".to_string(),
            agent_icon: "🤖".to_string(),
            task: "Test".to_string(),
            model: "sonnet".to_string(),
            project_path: "/test".to_string(),
            status: Some("completed"),
            scheduled_start_time: None,
            parent_run_id: None,
        }).unwrap();
        
        // Delete should succeed (cascading delete)
        service.delete_agent(pool, agent_id).await.unwrap();
    }
}
```

### 5. execute.rs - Agent Execution

#### Critical Tests
```rust
#[cfg(test)]
mod execute_tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_execute_agent_with_sandbox() {
        // Would require extensive mocking of:
        // - Claude binary
        // - Process spawning
        // - Sandbox profile creation
        // - Stream reading
        
        // Example structure:
        let app = mock_builder().build(tauri::generate_context!()).unwrap();
        let pool = Arc::new(setup_test_db());
        let registry = Arc::new(Mutex::new(ProcessRegistry::new()));
        
        // Mock binary path
        std::env::set_var("CLAUDE_BINARY_PATH", "/mock/claude");
        
        // Test would verify:
        // - Correct command construction
        // - Sandbox rules applied
        // - Process registration
        // - Event emission
        // - Output streaming
    }

    #[tokio::test]
    async fn test_resume_agent_execution() {
        // Test resume functionality
        // - Verify --resume-from argument added
        // - Verify line number tracking
        // - Verify parent_run_id set
    }

    #[tokio::test]
    async fn test_process_output_parsing() {
        // Test JSONL parsing from process output
        // - Valid JSON lines
        // - Invalid JSON handling
        // - Usage limit detection
        // - Session ID extraction
        // - PID extraction
    }

    #[tokio::test]
    async fn test_sandbox_violation_handling() {
        // Test sandbox violation detection and storage
        // - Parse violation from output
        // - Store in database
        // - Continue execution
    }
}
```

### 6. helpers.rs - Utility Functions

#### Medium Priority Tests
```rust
#[cfg(test)]
mod helpers_tests {
    use super::*;

    #[test]
    fn test_calculate_cost() {
        // Test Opus-4 pricing
        let cost = calculate_cost("opus-4", 1_000_000, 500_000, 100_000, 50_000);
        let expected = 15.0 + 37.5 + 1.875 + 0.075;
        assert!((cost - expected).abs() < 0.001);
        
        // Test Sonnet-4 pricing
        let cost = calculate_cost("sonnet-4", 1_000_000, 500_000, 100_000, 50_000);
        let expected = 3.0 + 7.5 + 0.375 + 0.015;
        assert!((cost - expected).abs() < 0.001);
        
        // Test zero tokens
        let cost = calculate_cost("opus-4", 0, 0, 0, 0);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_parse_datetime() {
        // Valid RFC3339
        let dt = parse_datetime("2024-01-01T12:00:00Z").unwrap();
        assert_eq!(dt.year(), 2024);
        
        // With timezone
        let dt = parse_datetime("2024-01-01T12:00:00+05:00").unwrap();
        assert_eq!(dt.hour(), 7); // UTC hour
        
        // Invalid format
        assert!(parse_datetime("2024-01-01").is_err());
        assert!(parse_datetime("not a date").is_err());
    }

    #[test]
    fn test_extract_json_fields() {
        let json = serde_json::json!({
            "session_id": "test-session",
            "pid": 12345
        });
        
        assert_eq!(extract_session_id(&json), Some("test-session".to_string()));
        assert_eq!(extract_pid(&json), Some(12345));
        
        let empty_json = serde_json::json!({});
        assert!(extract_session_id(&empty_json).is_none());
        assert!(extract_pid(&empty_json).is_none());
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(500), "0s");
        assert_eq!(format_duration(45_000), "45s");
        assert_eq!(format_duration(65_000), "1m 5s");
        assert_eq!(format_duration(3_665_000), "1h 1m");
    }

    #[test]
    fn test_format_cost() {
        assert_eq!(format_cost(0.0001), "$0.0001");
        assert_eq!(format_cost(0.005), "$0.0050");
        assert_eq!(format_cost(0.123), "$0.123");
        assert_eq!(format_cost(1.234), "$1.23");
        assert_eq!(format_cost(12.345), "$12.35");
    }

    #[test]
    fn test_build_sandbox_rules() {
        let agent = Agent {
            id: Some(1),
            name: "Test".to_string(),
            icon: "🤖".to_string(),
            system_prompt: "Test".to_string(),
            default_task: None,
            model: "sonnet".to_string(),
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: false,
            enable_network: true,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };
        
        let rules = build_sandbox_rules(&agent, "/test/project");
        
        // Verify file read rules exist
        assert!(rules.iter().any(|r| 
            r.operation_type == "file_read_all" && 
            r.pattern_value == "{{PROJECT_PATH}}"
        ));
        
        // Verify no file write rules
        assert!(!rules.iter().any(|r| r.operation_type == "file_write_all"));
        
        // Verify network rules exist
        assert!(rules.iter().any(|r| r.operation_type == "network_outbound"));
        
        // Verify essential system paths always included
        assert!(rules.iter().any(|r| 
            r.pattern_value == "/usr/bin" && 
            r.operation_type == "file_read_all"
        ));
    }

    #[test]
    fn test_parse_usage_limit_error() {
        let output = "Error: Usage limit exceeded. Your limit resets at 2024-01-01T12:00:00Z.";
        let reset_time = parse_usage_limit_error(output);
        assert_eq!(reset_time, Some("2024-01-01T12:00:00Z".to_string()));
        
        let output = "Some other error";
        assert!(parse_usage_limit_error(output).is_none());
    }

    #[cfg(unix)]
    #[test]
    fn test_is_process_running() {
        // Test with current process (should be running)
        let current_pid = std::process::id();
        assert!(is_process_running(current_pid));
        
        // Test with likely non-existent PID
        assert!(!is_process_running(999999));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_command_env_setup() {
        let cmd = create_command_with_env("/usr/bin/test");
        
        // Would need to inspect command environment
        // Verify PATH includes Homebrew paths
        // Verify essential env vars preserved
    }
}
```

### 7. pool.rs - Connection Pool

#### Medium Priority Tests
```rust
#[cfg(test)]
mod pool_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_pool_creation() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        
        let pool = create_pool(&db_path).unwrap();
        
        // Test pool properties
        assert_eq!(pool.max_size(), 10);
        assert_eq!(pool.min_idle(), Some(1));
        
        // Test getting connections
        let conn1 = pool.get().unwrap();
        let conn2 = pool.get().unwrap();
        drop(conn1);
        drop(conn2);
        
        // Test foreign keys enabled
        let conn = pool.get().unwrap();
        let fk_enabled: i32 = conn.query_row(
            "PRAGMA foreign_keys",
            [],
            |row| row.get(0)
        ).unwrap();
        assert_eq!(fk_enabled, 1);
    }

    #[test]
    fn test_init_pool_db_schema() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let pool = create_pool(db_path).unwrap();
        
        init_pool_db(&pool).unwrap();
        
        let conn = pool.get().unwrap();
        
        // Verify tables exist
        let tables = ["agents", "agent_runs", "jsonl_outputs", "sandbox_violations", "app_settings"];
        for table in &tables {
            let count: i32 = conn.query_row(
                &format!("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='{}'", table),
                [],
                |row| row.get(0)
            ).unwrap();
            assert_eq!(count, 1, "Table {} should exist", table);
        }
        
        // Verify indexes exist
        let indexes = ["idx_agent_runs_agent_id", "idx_agent_runs_status", "idx_agent_runs_session_id"];
        for index in &indexes {
            let count: i32 = conn.query_row(
                &format!("SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name='{}'", index),
                [],
                |row| row.get(0)
            ).unwrap();
            assert_eq!(count, 1, "Index {} should exist", index);
        }
        
        // Verify triggers exist
        let triggers = ["update_agent_timestamp", "update_app_settings_timestamp"];
        for trigger in &triggers {
            let count: i32 = conn.query_row(
                &format!("SELECT COUNT(*) FROM sqlite_master WHERE type='trigger' AND name='{}'", trigger),
                [],
                |row| row.get(0)
            ).unwrap();
            assert_eq!(count, 1, "Trigger {} should exist", trigger);
        }
    }

    #[test]
    fn test_migration_handling() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let pool = create_pool(db_path).unwrap();
        
        // Create initial schema without new columns
        let conn = pool.get().unwrap();
        conn.execute(
            "CREATE TABLE agent_runs (
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
                completed_at TEXT
            )",
            [],
        ).unwrap();
        
        // Run init which should add missing columns
        init_pool_db(&pool).unwrap();
        
        // Verify new columns exist
        let columns = ["usage_limit_reset_time", "auto_resume_enabled", "resume_count", "parent_run_id"];
        for column in &columns {
            let count: i32 = conn.query_row(
                &format!("SELECT COUNT(*) FROM pragma_table_info('agent_runs') WHERE name='{}'", column),
                [],
                |row| row.get(0)
            ).unwrap();
            assert_eq!(count, 1, "Column {} should exist", column);
        }
    }
}
```

### 8. commands.rs - Tauri Command Interface

#### High Priority Tests
```rust
#[cfg(test)]
mod command_tests {
    use super::*;
    use tauri::test::{mock_builder, MockRuntime};

    #[tokio::test]
    async fn test_command_error_handling() {
        // Test that service errors are properly converted to strings
        let app = mock_builder().build(tauri::generate_context!()).unwrap();
        let db = State(AgentDb(Arc::new(setup_test_db())));
        
        // Test with non-existent agent
        let result = get_agent(app.handle().clone(), db, 999).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Agent not found"));
    }

    #[tokio::test]
    async fn test_command_parameter_validation() {
        // Test that invalid parameters are caught
        let app = mock_builder().build(tauri::generate_context!()).unwrap();
        let db = State(AgentDb(Arc::new(setup_test_db())));
        
        // Empty name should fail
        let result = create_agent(
            app.handle().clone(),
            db,
            "".to_string(),
            "🤖".to_string(),
            "prompt".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
        ).await;
        
        // Service should validate and return error
        assert!(result.is_err());
    }
}
```

## Integration Test Scenarios

### Scenario 1: Complete Agent Lifecycle
```rust
#[tokio::test]
async fn test_complete_agent_lifecycle() {
    // 1. Create agent
    // 2. Execute task
    // 3. Monitor output
    // 4. Handle completion
    // 5. View metrics
    // 6. Delete agent
}
```

### Scenario 2: Resume After Usage Limit
```rust
#[tokio::test]
async fn test_usage_limit_and_resume() {
    // 1. Start execution
    // 2. Simulate usage limit
    // 3. Pause execution
    // 4. Wait for reset
    // 5. Auto-resume
    // 6. Complete task
}
```

### Scenario 3: Concurrent Executions
```rust
#[tokio::test]
async fn test_concurrent_agent_executions() {
    // 1. Create multiple agents
    // 2. Execute simultaneously
    // 3. Verify session isolation
    // 4. Verify resource usage
    // 5. Handle cancellations
}
```

### Scenario 4: Sandbox Violations
```rust
#[tokio::test]
async fn test_sandbox_violation_handling() {
    // 1. Create restricted agent
    // 2. Attempt forbidden operations
    // 3. Verify violations logged
    // 4. Verify execution continues
    // 5. Review violation report
}
```

## Performance and Load Tests

### Database Performance
```rust
#[test]
fn test_bulk_jsonl_insertion() {
    // Insert 10,000 JSONL lines
    // Measure insertion time
    // Verify retrieval performance
}

#[test]
fn test_concurrent_database_access() {
    // Spawn 10 threads
    // Perform simultaneous operations
    // Verify no deadlocks
}
```

### Memory Usage
```rust
#[test]
fn test_large_output_handling() {
    // Generate 100MB of JSONL output
    // Verify memory usage stays reasonable
    // Test streaming efficiency
}
```

## Error Recovery Tests

### Process Crash Recovery
```rust
#[test]
fn test_process_crash_recovery() {
    // Start execution
    // Kill process ungracefully
    // Verify status updated
    // Verify can resume
}
```

### Database Recovery
```rust
#[test]
fn test_database_corruption_handling() {
    // Corrupt database file
    // Attempt operations
    // Verify graceful error handling
}
```

## Test Execution Strategy

### Phase 1: Unit Tests
- Run all unit tests for individual modules
- Focus on critical path coverage
- Target: 100% coverage for types, repository

### Phase 2: Integration Tests
- Test module interactions
- Database integration
- Process management
- Target: >90% coverage

### Phase 3: System Tests
- End-to-end scenarios
- Performance testing
- Error recovery
- Target: Key user journeys

### Phase 4: Platform-Specific Tests
- macOS sandbox testing
- Linux namespace testing
- Windows process management
- Cross-platform compatibility

## Continuous Integration Setup

```yaml
# .github/workflows/test-agents.yml
name: Agent Module Tests

on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Run tests
        run: |
          cd src-tauri
          cargo test --package claudia --lib commands::agents -- --nocapture
          
      - name: Generate coverage
        if: matrix.os == 'ubuntu-latest'
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml --workspace
          
      - name: Upload coverage
        if: matrix.os == 'ubuntu-latest'
        uses: codecov/codecov-action@v3
```

## Test Data Management

### Fixtures
```rust
// tests/fixtures/agents.rs
pub fn create_test_agent() -> NewAgent {
    NewAgent {
        name: "Test Agent".to_string(),
        icon: "🤖".to_string(),
        system_prompt: "You are a test assistant".to_string(),
        default_task: Some("echo test".to_string()),
        model: "sonnet".to_string(),
        sandbox_enabled: true,
        enable_file_read: true,
        enable_file_write: false,
        enable_network: false,
    }
}

pub fn create_test_run(agent_id: i64) -> NewAgentRun {
    NewAgentRun {
        agent_id,
        agent_name: "Test Agent".to_string(),
        agent_icon: "🤖".to_string(),
        task: "Test task".to_string(),
        model: "sonnet".to_string(),
        project_path: "/test/path".to_string(),
        status: Some("pending"),
        scheduled_start_time: None,
        parent_run_id: None,
    }
}
```

### Mock Builders
```rust
// tests/mocks/claude_binary.rs
pub struct MockClaudeBinary {
    output: Vec<String>,
    exit_code: i32,
    delay_ms: u64,
}

impl MockClaudeBinary {
    pub fn new() -> Self {
        Self {
            output: vec![],
            exit_code: 0,
            delay_ms: 0,
        }
    }
    
    pub fn with_output(mut self, line: &str) -> Self {
        self.output.push(line.to_string());
        self
    }
    
    pub fn with_usage_limit_error(mut self) -> Self {
        self.output.push(r#"{"type": "error", "message": "Usage limit exceeded. Your limit resets at 2024-01-01T12:00:00Z."}"#.to_string());
        self.exit_code = 1;
        self
    }
}
```

## Summary

This comprehensive test plan covers:
- All modules with specific test cases
- Critical paths with 100% coverage target
- Integration and system test scenarios
- Performance and error recovery testing
- Platform-specific considerations
- CI/CD integration
- Test data management strategies

The plan prioritizes type safety, database integrity, and process management as critical areas while ensuring comprehensive coverage of all functionality. Implementation should follow the phased approach, starting with unit tests for critical modules and expanding to integration and system tests.