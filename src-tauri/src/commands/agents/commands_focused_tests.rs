#[cfg(test)]
mod tests {
    use super::super::error::AgentError;
    use super::super::types::*;
    use chrono::Utc;
    use serde_json;

    // ===== Type Validation Tests =====

    #[test]
    fn test_agent_id_validation() {
        // Valid IDs
        assert!(AgentId::new(1).is_ok());
        assert!(AgentId::new(100).is_ok());
        assert!(AgentId::new(i64::MAX).is_ok());

        // Invalid IDs
        assert!(AgentId::new(0).is_err());
        assert!(AgentId::new(-1).is_err());
        assert!(AgentId::new(i64::MIN).is_err());
    }

    #[test]
    fn test_agent_id_from_str() {
        assert_eq!(AgentId::from_str("1").unwrap().inner(), 1);
        assert_eq!(AgentId::from_str("999").unwrap().inner(), 999);
        
        assert_eq!(
            AgentId::from_str("0").unwrap_err(),
            "Agent ID must be positive"
        );
        
        assert_eq!(
            AgentId::from_str("-1").unwrap_err(),
            "Agent ID must be positive"
        );
        
        assert_eq!(
            AgentId::from_str("abc").unwrap_err(),
            "Invalid agent ID format"
        );
    }

    #[test]
    fn test_run_id_validation() {
        // Valid IDs
        assert!(RunId::new(1).is_ok());
        assert!(RunId::new(999999).is_ok());

        // Invalid IDs
        assert!(RunId::new(0).is_err());
        assert!(RunId::new(-100).is_err());
    }

    #[test]
    fn test_session_id_validation() {
        // Valid UUIDs
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(SessionId::new(valid_uuid.to_string()).is_ok());

        // Generated UUID should be valid
        let generated = SessionId::generate();
        assert!(!generated.inner().is_empty());

        // Invalid formats
        assert!(SessionId::new("".to_string()).is_err());
        assert!(SessionId::new("not-a-uuid".to_string()).is_err());
        assert!(SessionId::new("12345".to_string()).is_err());
    }

    #[test]
    fn test_run_status_parsing() {
        assert_eq!(RunStatus::from_str("running"), RunStatus::Running);
        assert_eq!(RunStatus::from_str("completed"), RunStatus::Completed);
        assert_eq!(RunStatus::from_str("failed"), RunStatus::Failed);
        assert_eq!(RunStatus::from_str("cancelled"), RunStatus::Cancelled);
        assert_eq!(RunStatus::from_str("scheduled"), RunStatus::Scheduled);
        assert_eq!(RunStatus::from_str("paused_usage_limit"), RunStatus::PausedUsageLimit);
        assert_eq!(RunStatus::from_str("unknown"), RunStatus::Pending); // Default
    }

    #[test]
    fn test_run_status_terminal_states() {
        assert!(!RunStatus::Pending.is_terminal());
        assert!(!RunStatus::Running.is_terminal());
        assert!(RunStatus::Completed.is_terminal());
        assert!(RunStatus::Failed.is_terminal());
        assert!(RunStatus::Cancelled.is_terminal());
        assert!(!RunStatus::Scheduled.is_terminal());
        assert!(!RunStatus::PausedUsageLimit.is_terminal());
    }

    #[test]
    fn test_run_status_active_states() {
        assert!(!RunStatus::Pending.is_active());
        assert!(RunStatus::Running.is_active());
        assert!(!RunStatus::Completed.is_active());
        assert!(!RunStatus::Failed.is_active());
        assert!(!RunStatus::Cancelled.is_active());
        assert!(!RunStatus::Scheduled.is_active());
        assert!(!RunStatus::PausedUsageLimit.is_active());
    }

    #[test]
    fn test_model_type_parsing() {
        assert_eq!(ModelType::from_str("opus-3"), ModelType::Opus3);
        assert_eq!(ModelType::from_str("OPUS-3"), ModelType::Opus3);
        assert_eq!(ModelType::from_str("opus3"), ModelType::Opus3);
        assert_eq!(ModelType::from_str("sonnet-4"), ModelType::Sonnet4);
        assert_eq!(ModelType::from_str("opus"), ModelType::Opus);
        assert_eq!(ModelType::from_str("invalid"), ModelType::Sonnet); // Default
    }

    #[test]
    fn test_model_type_display() {
        assert_eq!(format!("{}", ModelType::Opus3), "opus-3");
        assert_eq!(format!("{}", ModelType::Sonnet3), "sonnet-3");
        assert_eq!(format!("{}", ModelType::Opus4), "opus-4");
        assert_eq!(format!("{}", ModelType::Sonnet4), "sonnet-4");
        assert_eq!(format!("{}", ModelType::Sonnet), "sonnet");
        assert_eq!(format!("{}", ModelType::Opus), "opus");
    }

    // ===== Error Handling Tests =====

    #[test]
    fn test_agent_error_conversions() {
        // Test From<AgentError> for String
        let not_found = AgentError::AgentNotFound(42);
        let error_string: String = not_found.into();
        assert!(error_string.contains("Agent not found: 42"));

        let lock_error = AgentError::Lock("Failed to acquire lock".to_string());
        let error_string: String = lock_error.into();
        assert!(error_string.contains("Lock error"));

        let invalid_model = AgentError::InvalidModel("bad-model".to_string());
        let error_string: String = invalid_model.into();
        assert!(error_string.contains("Invalid model: bad-model"));
    }

    #[test]
    fn test_error_serialization() {
        let error = AgentError::InvalidModel("bad-model".to_string());
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("Invalid model: bad-model"));
    }

    // ===== Builder Pattern Tests =====

    #[test]
    fn test_agent_create_builder_success() {
        let agent = AgentCreateBuilder::new()
            .name("Test Agent")
            .icon("🤖")
            .system_prompt("You are a helpful assistant")
            .build()
            .unwrap();

        assert_eq!(agent.name, "Test Agent");
        assert_eq!(agent.icon, "🤖");
        assert_eq!(agent.system_prompt, "You are a helpful assistant");
        assert_eq!(agent.model, "sonnet"); // Default
    }

    #[test]
    fn test_agent_create_builder_with_all_fields() {
        let agent = AgentCreateBuilder::new()
            .name("Full Agent")
            .icon("🎯")
            .system_prompt("Advanced assistant")
            .default_task("Default task")
            .model(ModelType::Opus4)
            .sandbox_enabled(false)
            .enable_file_read(false)
            .enable_file_write(true)
            .enable_network(true)
            .build()
            .unwrap();

        assert_eq!(agent.name, "Full Agent");
        assert_eq!(agent.default_task, Some("Default task".to_string()));
        assert_eq!(agent.model, "opus-4");
        assert!(!agent.sandbox_enabled);
        assert!(!agent.enable_file_read);
        assert!(agent.enable_file_write);
        assert!(agent.enable_network);
    }

    #[test]
    fn test_agent_create_builder_missing_fields() {
        // Missing name
        let result = AgentCreateBuilder::new()
            .icon("🤖")
            .system_prompt("Prompt")
            .build();
        assert_eq!(result.unwrap_err(), "Agent name is required");

        // Missing icon
        let result = AgentCreateBuilder::new()
            .name("Agent")
            .system_prompt("Prompt")
            .build();
        assert_eq!(result.unwrap_err(), "Agent icon is required");

        // Missing system prompt
        let result = AgentCreateBuilder::new()
            .name("Agent")
            .icon("🤖")
            .build();
        assert_eq!(result.unwrap_err(), "System prompt is required");
    }

    #[test]
    fn test_agent_create_builder_validation() {
        // Empty name
        let result = AgentCreateBuilder::new()
            .name("")
            .icon("🤖")
            .system_prompt("Prompt")
            .build();
        assert_eq!(result.unwrap_err(), "Agent name is required");

        // Whitespace-only name
        let result = AgentCreateBuilder::new()
            .name("   ")
            .icon("🤖")
            .system_prompt("Prompt")
            .build();
        assert_eq!(result.unwrap_err(), "Agent name cannot be empty or whitespace");

        // Whitespace-only system prompt
        let result = AgentCreateBuilder::new()
            .name("Agent")
            .icon("🤖")
            .system_prompt("   ")
            .build();
        assert_eq!(result.unwrap_err(), "System prompt cannot be empty or whitespace");
    }

    #[test]
    fn test_agent_create_validation() {
        let valid_agent = AgentCreate {
            name: "Test Agent".to_string(),
            icon: "🤖".to_string(),
            system_prompt: "You are helpful".to_string(),
            default_task: None,
            model: "opus-4".to_string(),
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: true,
            enable_network: false,
        };
        assert!(valid_agent.validate().is_ok());

        let invalid_agent = AgentCreate {
            name: "".to_string(),
            icon: "🤖".to_string(),
            system_prompt: "Prompt".to_string(),
            default_task: None,
            model: "sonnet".to_string(),
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: true,
            enable_network: false,
        };
        assert!(invalid_agent.validate().is_err());
    }

    // ===== Data Structure Tests =====

    #[test]
    fn test_agent_struct_serialization() {
        let agent = Agent {
            id: Some(1),
            name: "Test Agent".to_string(),
            icon: "🤖".to_string(),
            system_prompt: "Test prompt".to_string(),
            default_task: Some("Default task".to_string()),
            model: "opus-4".to_string(),
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: false,
            enable_network: true,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&agent).unwrap();
        let deserialized: Agent = serde_json::from_str(&json).unwrap();
        
        assert_eq!(agent.name, deserialized.name);
        assert_eq!(agent.model, deserialized.model);
        assert_eq!(agent.sandbox_enabled, deserialized.sandbox_enabled);
    }

    #[test]
    fn test_agent_run_struct() {
        let run = AgentRun {
            id: Some(1),
            agent_id: 1,
            agent_name: "Test Agent".to_string(),
            agent_icon: "🤖".to_string(),
            task: "Test task".to_string(),
            model: "opus-4".to_string(),
            project_path: "/tmp/test".to_string(),
            session_id: SessionId::generate().inner().to_string(),
            status: "running".to_string(),
            pid: Some(12345),
            process_started_at: Some(Utc::now().to_rfc3339()),
            scheduled_start_time: None,
            created_at: Utc::now().to_rfc3339(),
            completed_at: None,
            usage_limit_reset_time: None,
            auto_resume_enabled: false,
            resume_count: 0,
            parent_run_id: None,
        };

        let json = serde_json::to_string(&run).unwrap();
        let deserialized: AgentRun = serde_json::from_str(&json).unwrap();
        
        assert_eq!(run.agent_id, deserialized.agent_id);
        assert_eq!(run.status, deserialized.status);
        assert_eq!(run.pid, deserialized.pid);
    }

    #[test]
    fn test_agent_export_format() {
        let export = AgentExport {
            version: 1,
            exported_at: Utc::now().to_rfc3339(),
            agent: AgentData {
                name: "Export Test".to_string(),
                icon: "📤".to_string(),
                system_prompt: "Export prompt".to_string(),
                default_task: Some("Default".to_string()),
                model: "sonnet-4".to_string(),
                sandbox_enabled: true,
                enable_file_read: true,
                enable_file_write: true,
                enable_network: false,
            },
        };

        let json = serde_json::to_string(&export).unwrap();
        let deserialized: AgentExport = serde_json::from_str(&json).unwrap();
        
        assert_eq!(export.version, deserialized.version);
        assert_eq!(export.agent.name, deserialized.agent.name);
    }

    #[test]
    fn test_jsonl_message_parsing() {
        let message = JsonlMessage {
            r#type: "response".to_string(),
            timestamp: Some(Utc::now().to_rfc3339()),
            message: Some("Hello, world!".to_string()),
            model: Some("opus-4".to_string()),
            input_tokens: Some(100),
            output_tokens: Some(50),
            cache_creation_input_tokens: Some(10),
            cache_read_input_tokens: Some(5),
            cost: Some(0.01),
            total_cost: Some(0.05),
        };

        let json = serde_json::to_string(&message).unwrap();
        let deserialized: JsonlMessage = serde_json::from_str(&json).unwrap();
        
        assert_eq!(message.r#type, deserialized.r#type);
        assert_eq!(message.input_tokens, deserialized.input_tokens);
    }

    #[test]
    fn test_agent_run_metrics_structure() {
        let metrics = AgentRunMetrics {
            duration_ms: Some(3600000), // 1 hour
            total_tokens: Some(1000),
            cost_usd: Some(0.05),
            message_count: Some(10),
        };

        assert_eq!(metrics.duration_ms, Some(3600000));
        assert_eq!(metrics.total_tokens, Some(1000));
        assert_eq!(metrics.cost_usd, Some(0.05));
        assert_eq!(metrics.message_count, Some(10));
    }

    #[test]
    fn test_agent_run_with_metrics() {
        let run = AgentRun {
            id: Some(1),
            agent_id: 1,
            agent_name: "Test".to_string(),
            agent_icon: "🤖".to_string(),
            task: "Task".to_string(),
            model: "sonnet".to_string(),
            project_path: "/tmp".to_string(),
            session_id: SessionId::generate().inner().to_string(),
            status: "completed".to_string(),
            pid: None,
            process_started_at: None,
            scheduled_start_time: None,
            created_at: Utc::now().to_rfc3339(),
            completed_at: Some(Utc::now().to_rfc3339()),
            usage_limit_reset_time: None,
            auto_resume_enabled: false,
            resume_count: 0,
            parent_run_id: None,
        };

        let metrics = AgentRunMetrics {
            duration_ms: Some(60000),
            total_tokens: Some(500),
            cost_usd: Some(0.025),
            message_count: Some(5),
        };

        let with_metrics = AgentRunWithMetrics {
            run: run.clone(),
            metrics: Some(metrics),
            output: Some("Output content".to_string()),
        };

        assert_eq!(with_metrics.run.id, Some(1));
        assert_eq!(with_metrics.metrics.as_ref().unwrap().total_tokens, Some(500));
        assert_eq!(with_metrics.output, Some("Output content".to_string()));
    }

    #[test]
    fn test_github_agent_file_structure() {
        let file = GitHubAgentFile {
            name: "test-agent.json".to_string(),
            path: "agents/test-agent.json".to_string(),
            sha: "abc123def456".to_string(),
            size: 2048,
            url: "https://api.github.com/repos/user/repo/contents/agents/test-agent.json".to_string(),
            html_url: "https://github.com/user/repo/blob/main/agents/test-agent.json".to_string(),
            git_url: "https://api.github.com/repos/user/repo/git/blobs/abc123def456".to_string(),
            download_url: "https://raw.githubusercontent.com/user/repo/main/agents/test-agent.json".to_string(),
            file_type: "file".to_string(),
        };

        assert_eq!(file.name, "test-agent.json");
        assert_eq!(file.size, 2048);
        assert!(file.download_url.contains("raw.githubusercontent.com"));
    }

    #[test]
    fn test_sandbox_violation_structure() {
        let violation = SandboxViolation {
            id: Some(1),
            run_id: 1,
            denied_at: Utc::now().to_rfc3339(),
            operation_type: "file_read".to_string(),
            resource: "/etc/passwd".to_string(),
            reason: "Access denied to sensitive file".to_string(),
            created_at: Utc::now().to_rfc3339(),
        };

        assert_eq!(violation.operation_type, "file_read");
        assert_eq!(violation.resource, "/etc/passwd");
        assert_eq!(violation.reason, "Access denied to sensitive file");
    }

    #[test]
    fn test_app_setting_structure() {
        let setting = AppSetting {
            key: "claude_binary_path".to_string(),
            value: "/usr/local/bin/claude".to_string(),
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };

        assert_eq!(setting.key, "claude_binary_path");
        assert_eq!(setting.value, "/usr/local/bin/claude");
    }

    #[test]
    fn test_claude_installation_structure() {
        let installation = ClaudeInstallation {
            path: "/usr/local/bin/claude".to_string(),
            version: Some("1.0.0".to_string()),
            source: "homebrew".to_string(),
        };

        assert_eq!(installation.path, "/usr/local/bin/claude");
        assert_eq!(installation.version, Some("1.0.0".to_string()));
        assert_eq!(installation.source, "homebrew");
    }

    // ===== Edge Case Tests =====

    #[test]
    fn test_unicode_and_special_characters() {
        let agent = Agent {
            id: Some(1),
            name: "Unicode 你好 🌍 Agent".to_string(),
            icon: "🎌".to_string(),
            system_prompt: "Prompt with <>&\"' characters\nand\nnewlines".to_string(),
            default_task: None,
            model: "sonnet".to_string(),
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: true,
            enable_network: false,
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };

        // Test serialization with unicode
        let json = serde_json::to_string(&agent).unwrap();
        let deserialized: Agent = serde_json::from_str(&json).unwrap();
        
        assert_eq!(agent.name, deserialized.name);
        assert_eq!(agent.icon, deserialized.icon);
        assert_eq!(agent.system_prompt, deserialized.system_prompt);
    }

    #[test]
    fn test_large_data_handling() {
        let large_prompt = "A".repeat(10_000); // 10KB of text
        
        let agent = AgentCreate {
            name: "Large Agent".to_string(),
            icon: "📚".to_string(),
            system_prompt: large_prompt.clone(),
            default_task: None,
            model: "sonnet".to_string(),
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: true,
            enable_network: false,
        };

        // Test validation works with large data
        assert!(agent.validate().is_ok());
        assert_eq!(agent.system_prompt.len(), 10_000);
    }

    #[test]
    fn test_session_id_edge_cases() {
        // Test uppercase UUID
        let uppercase = "550E8400-E29B-41D4-A716-446655440000";
        assert!(SessionId::new(uppercase.to_string()).is_ok());
        
        // Test lowercase UUID
        let lowercase = "550e8400-e29b-41d4-a716-446655440000";
        assert!(SessionId::new(lowercase.to_string()).is_ok());
        
        // Test UUID without hyphens (uuid crate accepts this format)
        let no_hyphens = "550e8400e29b41d4a716446655440000";
        assert!(SessionId::new(no_hyphens.to_string()).is_ok());
        
        // Test invalid UUID formats
        let too_short = "550e8400-e29b-41d4";
        assert!(SessionId::new(too_short.to_string()).is_err());
        
        let invalid_chars = "550e8400-e29b-41d4-a716-44665544000g";
        assert!(SessionId::new(invalid_chars.to_string()).is_err());
    }

    #[test]
    fn test_model_type_case_insensitive() {
        assert_eq!(ModelType::from_str("OPUS-3"), ModelType::Opus3);
        assert_eq!(ModelType::from_str("SoNnEt-4"), ModelType::Sonnet4);
        assert_eq!(ModelType::from_str("OpUs"), ModelType::Opus);
    }

    #[test]
    fn test_agent_create_builder_chaining() {
        let agent = AgentCreate::builder()
            .name("Test")
            .name("Test Agent") // Should override
            .icon("👍")
            .icon("🤖") // Should override
            .system_prompt("First")
            .system_prompt("You are a helpful assistant") // Should override
            .build()
            .unwrap();
        
        assert_eq!(agent.name, "Test Agent");
        assert_eq!(agent.icon, "🤖");
        assert_eq!(agent.system_prompt, "You are a helpful assistant");
    }

    // ===== Constants Tests =====

    #[test]
    fn test_default_constants() {
        use crate::commands::agents::constants::*;
        
        assert_eq!(DEFAULT_MODEL, "sonnet");
        assert!(DEFAULT_SANDBOX_ENABLED);
        assert!(DEFAULT_FILE_READ_ENABLED);
        assert!(DEFAULT_FILE_WRITE_ENABLED);
        assert!(!DEFAULT_NETWORK_ENABLED);
        assert_eq!(AGENT_EXPORT_VERSION, 1);
    }

    // ===== File Path Tests =====

    #[test]
    fn test_output_file_path_construction() {
        use std::path::Path;
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().unwrap();
        let app_data_dir = temp_dir.path();
        let run_id = 42;
        
        let output_dir = app_data_dir.join("agent_outputs");
        let output_file = output_dir.join(format!("{}.jsonl", run_id));
        
        assert_eq!(output_file.file_name().unwrap(), "42.jsonl");
        assert!(output_file.parent().unwrap().ends_with("agent_outputs"));
    }

    #[test]
    fn test_read_agent_output_file_missing() {
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().unwrap();
        let result = super::super::commands::read_agent_output_file(temp_dir.path(), 123);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_read_agent_output_file_exists() {
        use std::fs;
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("agent_outputs");
        fs::create_dir_all(&output_dir).unwrap();
        
        let output_file = output_dir.join("123.jsonl");
        let content = r#"{"type":"response","message":"Test output"}"#;
        fs::write(&output_file, content).unwrap();
        
        let result = super::super::commands::read_agent_output_file(temp_dir.path(), 123);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), content);
    }

    // ===== Command Response Format Tests =====

    #[test]
    fn test_error_to_string_conversion() {
        // Test that errors convert to strings properly for Tauri
        let errors = vec![
            AgentError::AgentNotFound(1),
            AgentError::RunNotFound(2),
            AgentError::Lock("test lock".to_string()),
            AgentError::Process("process error".to_string()),
            AgentError::BinaryNotFound("binary".to_string()),
            AgentError::InvalidStatus("bad status".to_string()),
            AgentError::InvalidModel("bad model".to_string()),
            AgentError::Sandbox("sandbox error".to_string()),
            AgentError::Schedule("schedule error".to_string()),
            AgentError::Parse("parse error".to_string()),
            AgentError::Network("network error".to_string()),
            AgentError::Other("other error".to_string()),
        ];

        for error in errors {
            let error_string: String = error.to_string();
            assert!(!error_string.is_empty());
            
            // Verify error can be serialized
            let serialized = serde_json::to_string(&error_string).unwrap();
            assert!(serialized.contains(&error_string));
        }
    }

    #[test]
    fn test_result_serialization_for_tauri() {
        // Test successful result
        let agent = Agent {
            id: Some(1),
            name: "Test".to_string(),
            icon: "🤖".to_string(),
            system_prompt: "Test".to_string(),
            default_task: None,
            model: "sonnet".to_string(),
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: true,
            enable_network: false,
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };

        let result: Result<Agent, String> = Ok(agent);
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"Ok\""));
        assert!(json.contains("\"name\":\"Test\""));

        // Test error result
        let error_result: Result<Agent, String> = Err("Database error".to_string());
        let error_json = serde_json::to_string(&error_result).unwrap();
        assert!(error_json.contains("\"Err\""));
        assert!(error_json.contains("Database error"));
    }

    #[test]
    fn test_agent_run_status_values() {
        // Verify all expected status strings work
        let statuses = vec![
            "pending",
            "running", 
            "completed",
            "failed",
            "cancelled",
            "scheduled",
            "paused_usage_limit",
        ];

        for status in statuses {
            let run_status = RunStatus::from_str(status);
            assert_eq!(run_status.to_string(), status);
        }
    }

    #[test]
    fn test_scheduled_run_time_format() {
        let future_time = (Utc::now() + chrono::Duration::hours(1)).to_rfc3339();
        
        // Verify RFC3339 format
        assert!(future_time.contains("T"));
        assert!(future_time.contains(":"));
        assert!(future_time.len() > 19); // Basic datetime is at least 19 chars
        
        // Verify it can be parsed back
        let parsed = chrono::DateTime::parse_from_rfc3339(&future_time);
        assert!(parsed.is_ok());
    }
}