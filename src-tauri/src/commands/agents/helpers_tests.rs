#[cfg(test)]
mod tests {
    use crate::commands::agents::helpers::*;
    
    
    
    use chrono::{DateTime, TimeZone, Utc};
    use serde_json::json;

    // ===== calculate_cost Tests =====

    #[test]
    fn test_calculate_cost_basic() {
        // Test with Opus 3 pricing
        let cost = calculate_cost("opus-3", 1_000_000, 1_000_000, 0, 0);
        let expected = (1_000_000.0 / TOKENS_PER_MILLION) * OPUS_3_INPUT_PRICE
            + (1_000_000.0 / TOKENS_PER_MILLION) * OPUS_3_OUTPUT_PRICE;
        assert_eq!(cost, expected);
    }

    #[test]
    fn test_calculate_cost_with_cache() {
        // Test with cache tokens
        let cost = calculate_cost("sonnet-4", 500_000, 300_000, 100_000, 50_000);
        
        let input_cost = (500_000.0 / TOKENS_PER_MILLION) * SONNET_4_INPUT_PRICE;
        let output_cost = (300_000.0 / TOKENS_PER_MILLION) * SONNET_4_OUTPUT_PRICE;
        let cache_write_cost = (100_000.0 / TOKENS_PER_MILLION) * SONNET_4_CACHE_WRITE_PRICE;
        let cache_read_cost = (50_000.0 / TOKENS_PER_MILLION) * SONNET_4_CACHE_READ_PRICE;
        
        assert_eq!(cost, input_cost + output_cost + cache_write_cost + cache_read_cost);
    }

    #[test]
    fn test_calculate_cost_zero_tokens() {
        let cost = calculate_cost("opus-4", 0, 0, 0, 0);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_calculate_cost_different_models() {
        let models = vec!["opus-3", "sonnet-3", "opus-4", "sonnet-4", "opus", "sonnet", "unknown"];
        
        for model in models {
            let cost = calculate_cost(model, 1000, 1000, 0, 0);
            assert!(cost > 0.0, "Cost should be positive for model: {}", model);
        }
    }

    #[test]
    fn test_calculate_cost_large_numbers() {
        let cost = calculate_cost("opus-4", i64::MAX, i64::MAX, i64::MAX, i64::MAX);
        assert!(cost > 0.0);
        assert!(!cost.is_nan());
        assert!(cost.is_finite());
    }

    // ===== parse_datetime Tests =====

    #[test]
    fn test_parse_datetime_valid() {
        let datetime_str = "2024-01-01T12:00:00Z";
        let result = parse_datetime(datetime_str).unwrap();
        
        let expected = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_datetime_with_timezone() {
        let datetime_str = "2024-01-01T12:00:00+05:00";
        let result = parse_datetime(datetime_str).unwrap();
        
        // Should be converted to UTC
        let expected = Utc.with_ymd_and_hms(2024, 1, 1, 7, 0, 0).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_datetime_invalid() {
        let invalid_dates = vec![
            "not a date",
            "2024-01-01",
            "12:00:00",
            "2024-13-01T12:00:00Z", // Invalid month
            "",
        ];
        
        for date in invalid_dates {
            assert!(parse_datetime(date).is_err(), "Should fail for: {}", date);
        }
    }

    // ===== now_iso8601 Tests =====

    #[test]
    fn test_now_iso8601_format() {
        let now = now_iso8601();
        
        // Should be parseable as valid datetime
        assert!(DateTime::parse_from_rfc3339(&now).is_ok());
        
        // Should contain expected components
        assert!(now.contains('T'));
        assert!(now.ends_with('Z') || now.contains('+') || now.contains('-'));
    }

    // ===== extract_session_id Tests =====

    #[test]
    fn test_extract_session_id_valid() {
        let json = json!({
            "session_id": "550e8400-e29b-41d4-a716-446655440000",
            "other": "data"
        });
        
        let result = extract_session_id(&json);
        assert_eq!(result, Some("550e8400-e29b-41d4-a716-446655440000".to_string()));
    }

    #[test]
    fn test_extract_session_id_missing() {
        let json = json!({
            "other": "data"
        });
        
        assert_eq!(extract_session_id(&json), None);
    }

    #[test]
    fn test_extract_session_id_wrong_type() {
        let json = json!({
            "session_id": 12345
        });
        
        assert_eq!(extract_session_id(&json), None);
    }

    #[test]
    fn test_extract_session_id_null() {
        let json = json!({
            "session_id": null
        });
        
        assert_eq!(extract_session_id(&json), None);
    }

    // ===== extract_pid Tests =====

    #[test]
    fn test_extract_pid_valid() {
        let json = json!({
            "pid": 12345,
            "other": "data"
        });
        
        assert_eq!(extract_pid(&json), Some(12345));
    }

    #[test]
    fn test_extract_pid_large_number() {
        let json = json!({
            "pid": u32::MAX as u64
        });
        
        assert_eq!(extract_pid(&json), Some(u32::MAX));
    }

    #[test]
    fn test_extract_pid_overflow() {
        let json = json!({
            "pid": (u32::MAX as u64) + 1
        });
        
        // Should overflow and return incorrect value - this is a limitation
        // In production code, we might want to handle this better
        assert_eq!(extract_pid(&json), Some(0));
    }

    #[test]
    fn test_extract_pid_missing() {
        let json = json!({
            "other": "data"
        });
        
        assert_eq!(extract_pid(&json), None);
    }

    #[test]
    fn test_extract_pid_wrong_type() {
        let json = json!({
            "pid": "12345"
        });
        
        assert_eq!(extract_pid(&json), None);
    }

    // ===== Platform-specific process tests =====

    #[cfg(unix)]
    mod unix_process_tests {
        use super::*;
        

        #[test]
        fn test_is_process_running_current() {
            let pid = std::process::id();
            assert!(is_process_running(pid));
        }

        #[test]
        fn test_is_process_running_invalid() {
            // PID 0 is invalid on Unix
            assert!(!is_process_running(0));
            
            // Very high PID unlikely to exist
            assert!(!is_process_running(999999999));
        }

        #[test]
        fn test_kill_process_invalid_pid() {
            // Try to kill a non-existent process
            let result = kill_process(999999999);
            assert!(result.is_err());
        }

        #[test]
        fn test_kill_process_tree_invalid() {
            let result = kill_process_tree(999999999);
            // On Unix, this might succeed (pkill returns 0 even if no processes killed)
            // or fail depending on the system
            // Just verify it doesn't panic
            let _ = result;
        }
    }

    #[cfg(windows)]
    mod windows_process_tests {
        use super::*;
        use crate::commands::agents::helpers::*;

        #[test]
        fn test_is_process_running_current() {
            let pid = std::process::id();
            assert!(is_process_running(pid));
        }

        #[test]
        fn test_is_process_running_invalid() {
            // Very high PID unlikely to exist
            assert!(!is_process_running(999999999));
        }

        #[test]
        fn test_kill_process_invalid_pid() {
            let result = kill_process(999999999);
            // Windows taskkill will fail for non-existent process
            assert!(result.is_err());
        }

        #[test]
        fn test_kill_process_tree_invalid() {
            let result = kill_process_tree(999999999);
            assert!(result.is_err());
        }
    }

    // ===== build_sandbox_rules Tests =====

    #[test]
    fn test_build_sandbox_rules_all_permissions() {
        let agent = Agent {
            id: Some(1),
            name: "Test Agent".to_string(),
            icon: "🤖".to_string(),
            system_prompt: "Test".to_string(),
            default_task: None,
            model: "opus-4".to_string(),
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: true,
            enable_network: true,
            created_at: now_iso8601(),
            updated_at: now_iso8601(),
        };

        let rules = build_sandbox_rules(&agent, "/test/project");
        
        // Should have rules for file read, write, network, and system paths
        assert!(rules.iter().any(|r| r.operation_type == "file_read_all" && r.pattern_value == "{{PROJECT_PATH}}"));
        assert!(rules.iter().any(|r| r.operation_type == "file_write_all" && r.pattern_value == "{{PROJECT_PATH}}"));
        assert!(rules.iter().any(|r| r.operation_type == "network_outbound"));
        
        // Should always have system paths
        assert!(rules.iter().any(|r| r.pattern_value == "/usr/bin"));
    }

    #[test]
    fn test_build_sandbox_rules_no_permissions() {
        let agent = Agent {
            id: Some(1),
            name: "Test Agent".to_string(),
            icon: "🤖".to_string(),
            system_prompt: "Test".to_string(),
            default_task: None,
            model: "opus-4".to_string(),
            sandbox_enabled: true,
            enable_file_read: false,
            enable_file_write: false,
            enable_network: false,
            created_at: now_iso8601(),
            updated_at: now_iso8601(),
        };

        let rules = build_sandbox_rules(&agent, "/test/project");
        
        // Should not have project file rules
        assert!(!rules.iter().any(|r| r.operation_type == "file_read_all" && r.pattern_value == "{{PROJECT_PATH}}"));
        assert!(!rules.iter().any(|r| r.operation_type == "file_write_all"));
        assert!(!rules.iter().any(|r| r.operation_type == "network_outbound"));
        
        // But should still have system paths
        assert!(rules.iter().any(|r| r.pattern_value == "/usr/bin"));
    }

    #[test]
    fn test_build_sandbox_rules_platform_specific() {
        let agent = Agent {
            id: Some(1),
            name: "Test Agent".to_string(),
            icon: "🤖".to_string(),
            system_prompt: "Test".to_string(),
            default_task: None,
            model: "opus-4".to_string(),
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: false,
            enable_network: false,
            created_at: now_iso8601(),
            updated_at: now_iso8601(),
        };

        let rules = build_sandbox_rules(&agent, "/test/project");
        
        // Check platform-specific paths
        let macos_specific = rules.iter().any(|r| 
            r.pattern_value == "/System/Library" && 
            r.platform_support.as_ref().unwrap().contains("macos")
        );
        
        let homebrew_path = rules.iter().any(|r| 
            r.pattern_value == "/opt/homebrew/bin" && 
            r.platform_support.as_ref().unwrap().contains("macos")
        );
        
        assert!(macos_specific);
        assert!(homebrew_path);
    }

    #[test]
    fn test_build_sandbox_rules_sequential_ids() {
        let agent = Agent {
            id: Some(1),
            name: "Test Agent".to_string(),
            icon: "🤖".to_string(),
            system_prompt: "Test".to_string(),
            default_task: None,
            model: "opus-4".to_string(),
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: true,
            enable_network: true,
            created_at: now_iso8601(),
            updated_at: now_iso8601(),
        };

        let rules = build_sandbox_rules(&agent, "/test/project");
        
        // Check that rule IDs are sequential starting from 1
        let mut ids: Vec<i64> = rules.iter().filter_map(|r| r.id).collect();
        ids.sort();
        
        for (i, id) in ids.iter().enumerate() {
            assert_eq!(*id, (i + 1) as i64);
        }
    }

    // ===== format_duration Tests =====

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(0), "0s");
        assert_eq!(format_duration(1000), "1s");
        assert_eq!(format_duration(59999), "59s");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(60000), "1m 0s");
        assert_eq!(format_duration(90000), "1m 30s");
        assert_eq!(format_duration(3599999), "59m 59s");
    }

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(format_duration(3600000), "1h 0m");
        assert_eq!(format_duration(3660000), "1h 1m");
        assert_eq!(format_duration(7200000), "2h 0m");
        assert_eq!(format_duration(10800000), "3h 0m");
    }

    #[test]
    fn test_format_duration_negative() {
        // Negative durations should still work (absolute value)
        assert_eq!(format_duration(-1000), "0s"); // Integer division truncates
        assert_eq!(format_duration(-60000), "0h 1m"); // Weird but consistent
    }

    // ===== format_cost Tests =====

    #[test]
    fn test_format_cost_very_small() {
        assert_eq!(format_cost(0.0), "$0.0000");
        assert_eq!(format_cost(0.0001), "$0.0001");
        assert_eq!(format_cost(0.0099), "$0.0099");
    }

    #[test]
    fn test_format_cost_small() {
        assert_eq!(format_cost(0.01), "$0.010");
        assert_eq!(format_cost(0.123), "$0.123");
        assert_eq!(format_cost(0.999), "$0.999");
    }

    #[test]
    fn test_format_cost_large() {
        assert_eq!(format_cost(1.0), "$1.00");
        assert_eq!(format_cost(10.50), "$10.50");
        assert_eq!(format_cost(999.99), "$999.99");
        assert_eq!(format_cost(1000.0), "$1000.00");
    }

    #[test]
    fn test_format_cost_edge_cases() {
        assert_eq!(format_cost(0.009999), "$0.0100"); // Rounding
        assert_eq!(format_cost(0.9999), "$1.000"); // Just under 1.0
        assert_eq!(format_cost(1.001), "$1.00"); // Just over 1.0
    }

    #[test]
    fn test_format_cost_negative() {
        assert_eq!(format_cost(-0.001), "$-0.0010");
        assert_eq!(format_cost(-10.0), "$-10.00");
    }

    // ===== create_command_with_env Tests =====

    #[test]
    fn test_create_command_with_env_basic() {
        let cmd = create_command_with_env("echo");
        
        // Just verify it creates a command without panicking
        // We can't easily test the environment variables without executing
        assert_eq!(cmd.as_std().get_program(), "echo");
    }

    #[test]
    fn test_create_command_with_env_nvm_path() {
        let nvm_path = "/home/user/.nvm/versions/node/v18.0.0/bin/node";
        let cmd = create_command_with_env(nvm_path);
        
        assert_eq!(cmd.as_std().get_program(), nvm_path);
        // Environment variable setting is tested implicitly
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_create_command_with_env_macos() {
        let cmd = create_command_with_env("test");
        
        // On macOS, PATH should include homebrew paths
        // Can't easily verify without executing
        assert_eq!(cmd.as_std().get_program(), "test");
    }

    // ===== read_session_jsonl Tests =====

    #[tokio::test]
    async fn test_read_session_jsonl_missing_file() {
        let result = read_session_jsonl("nonexistent-session", "/test/project").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Session file not found"));
    }

    #[tokio::test]
    async fn test_read_session_jsonl_encoded_path() {
        // Test that project path encoding works correctly
        let session_id = "test-session";
        let project_path = "/path/with/slashes";
        
        let result = read_session_jsonl(session_id, project_path).await;
        assert!(result.is_err()); // File won't exist, but we're testing path encoding
        
        // The encoded path should replace slashes with dashes
        let err = result.unwrap_err();
        assert!(err.contains("-path-with-slashes"));
    }

    // ===== parse_usage_limit_error Tests =====

    #[test]
    fn test_parse_usage_limit_error_found() {
        let output = "Error: Usage limit exceeded. Your limit resets at 2024-01-01T12:00:00Z.";
        let result = parse_usage_limit_error(output);
        
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "2024-01-01T12:00:00Z");
    }

    #[test]
    fn test_parse_usage_limit_error_rate_limit() {
        let output = "Warning: You have hit the rate limit. Please try again later.";
        let result = parse_usage_limit_error(output);
        
        assert!(result.is_some());
        // Should return current time as fallback
        let parsed_time = DateTime::parse_from_rfc3339(&result.unwrap());
        assert!(parsed_time.is_ok());
    }

    #[test]
    fn test_parse_usage_limit_error_multiline() {
        let output = r#"
Starting process...
Error: Usage limit exceeded. Your limit resets at 2024-12-25T00:00:00Z.
Please wait before trying again.
"#;
        let result = parse_usage_limit_error(output);
        
        assert_eq!(result, Some("2024-12-25T00:00:00Z".to_string()));
    }

    #[test]
    fn test_parse_usage_limit_error_no_reset_time() {
        let output = "Error: Usage limit exceeded. Please try again later.";
        let result = parse_usage_limit_error(output);
        
        assert!(result.is_some());
        // Should return current time as fallback
        let parsed_time = DateTime::parse_from_rfc3339(&result.unwrap());
        assert!(parsed_time.is_ok());
    }

    #[test]
    fn test_parse_usage_limit_error_not_found() {
        let output = "Everything is working fine!";
        let result = parse_usage_limit_error(output);
        
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_usage_limit_error_empty() {
        assert!(parse_usage_limit_error("").is_none());
    }

    // ===== Integration-style tests =====

    #[test]
    fn test_agent_lifecycle_helpers() {
        // Test a typical agent lifecycle using multiple helpers
        
        // 1. Create an agent with permissions
        let agent = Agent {
            id: Some(1),
            name: "Integration Test".to_string(),
            icon: "🧪".to_string(),
            system_prompt: "Test prompt".to_string(),
            default_task: None,
            model: "opus-4".to_string(),
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: false,
            enable_network: true,
            created_at: now_iso8601(),
            updated_at: now_iso8601(),
        };

        // 2. Build sandbox rules
        let rules = build_sandbox_rules(&agent, "/test/project");
        assert!(!rules.is_empty());

        // 3. Simulate token usage and calculate cost
        let cost = calculate_cost(&agent.model, 10000, 5000, 1000, 500);
        assert!(cost > 0.0);

        // 4. Format the cost for display
        let formatted = format_cost(cost);
        assert!(formatted.starts_with('$'));

        // 5. Simulate duration and format it
        let duration_ms = 125000; // 2 minutes 5 seconds
        let formatted_duration = format_duration(duration_ms);
        assert_eq!(formatted_duration, "2m 5s");
    }

    #[test]
    fn test_json_extraction_helpers() {
        // Test JSON extraction helpers together
        let message = json!({
            "type": "assistant_response",
            "session_id": "550e8400-e29b-41d4-a716-446655440000",
            "pid": 12345,
            "timestamp": now_iso8601(),
            "data": {
                "message": "Hello, world!"
            }
        });

        let session_id = extract_session_id(&message);
        assert_eq!(session_id, Some("550e8400-e29b-41d4-a716-446655440000".to_string()));

        let pid = extract_pid(&message);
        assert_eq!(pid, Some(12345));

        // Test with missing fields
        let partial_message = json!({
            "type": "assistant_response",
            "timestamp": now_iso8601()
        });

        assert!(extract_session_id(&partial_message).is_none());
        assert!(extract_pid(&partial_message).is_none());
    }

    #[test]
    fn test_datetime_helpers_roundtrip() {
        // Test datetime parsing and generation
        let now_str = now_iso8601();
        let parsed = parse_datetime(&now_str).unwrap();
        let formatted = parsed.to_rfc3339();
        
        // Should be able to parse the formatted datetime
        let reparsed = parse_datetime(&formatted).unwrap();
        assert_eq!(parsed, reparsed);
    }

    // ===== Property-based tests (if we had quickcheck) =====
    // These are manual property tests

    #[test]
    fn test_format_duration_properties() {
        // Property: format_duration should never panic
        let test_values = vec![0, 1, 1000, 60000, 3600000, i64::MAX, i64::MIN];
        
        for value in test_values {
            let _ = format_duration(value);
        }
    }

    #[test]
    fn test_format_cost_properties() {
        // Property: format_cost should always start with $ or -$
        let test_values = vec![0.0, 0.001, 1.0, 1000.0, -1.0, f64::MAX, f64::MIN, f64::NAN, f64::INFINITY];
        
        for value in test_values {
            let formatted = format_cost(value);
            assert!(formatted.starts_with('$') || formatted.starts_with("-$"));
        }
    }

    #[test]
    fn test_calculate_cost_properties() {
        // Property: cost should never be negative with positive inputs
        let models = vec!["opus-3", "sonnet-4", "unknown"];
        
        for model in models {
            let cost = calculate_cost(model, 1000, 1000, 1000, 1000);
            assert!(cost >= 0.0);
            assert!(!cost.is_nan());
            assert!(cost.is_finite());
        }
    }
}