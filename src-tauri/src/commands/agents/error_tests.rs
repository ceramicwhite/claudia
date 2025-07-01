#[cfg(test)]
mod tests {
    use super::super::error::AgentError;
    use std::io;

    // ===== Error Creation Tests =====
    
    #[test]
    fn test_create_database_error() {
        let sqlite_err = rusqlite::Error::QueryReturnedNoRows;
        let agent_err: AgentError = sqlite_err.into();
        assert!(matches!(agent_err, AgentError::Database(_)));
        assert_eq!(agent_err.to_string(), "Database error: Query returned no rows");
    }

    #[test]
    fn test_create_lock_error() {
        let err = AgentError::Lock("Failed to acquire mutex".to_string());
        assert!(matches!(err, AgentError::Lock(_)));
        assert_eq!(err.to_string(), "Lock error: Failed to acquire mutex");
    }

    #[test]
    fn test_create_agent_not_found_error() {
        let err = AgentError::AgentNotFound(42);
        assert!(matches!(err, AgentError::AgentNotFound(42)));
        assert_eq!(err.to_string(), "Agent not found: 42");
    }

    #[test]
    fn test_create_run_not_found_error() {
        let err = AgentError::RunNotFound(123);
        assert!(matches!(err, AgentError::RunNotFound(123)));
        assert_eq!(err.to_string(), "Run not found: 123");
    }

    #[test]
    fn test_create_process_error() {
        let err = AgentError::Process("Process terminated unexpectedly".to_string());
        assert!(matches!(err, AgentError::Process(_)));
        assert_eq!(err.to_string(), "Process error: Process terminated unexpectedly");
    }

    #[test]
    fn test_create_binary_not_found_error() {
        let err = AgentError::BinaryNotFound("claude".to_string());
        assert!(matches!(err, AgentError::BinaryNotFound(_)));
        assert_eq!(err.to_string(), "Binary not found: claude");
    }

    #[test]
    fn test_create_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let agent_err: AgentError = io_err.into();
        assert!(matches!(agent_err, AgentError::Io(_)));
        assert!(agent_err.to_string().contains("IO error:"));
    }

    #[test]
    fn test_create_invalid_status_error() {
        let err = AgentError::InvalidStatus("unknown_status".to_string());
        assert!(matches!(err, AgentError::InvalidStatus(_)));
        assert_eq!(err.to_string(), "Invalid status: unknown_status");
    }

    #[test]
    fn test_create_invalid_model_error() {
        let err = AgentError::InvalidModel("invalid-model-name".to_string());
        assert!(matches!(err, AgentError::InvalidModel(_)));
        assert_eq!(err.to_string(), "Invalid model: invalid-model-name");
    }

    #[test]
    fn test_create_sandbox_error() {
        let err = AgentError::Sandbox("Permission denied".to_string());
        assert!(matches!(err, AgentError::Sandbox(_)));
        assert_eq!(err.to_string(), "Sandbox error: Permission denied");
    }

    #[test]
    fn test_create_schedule_error() {
        let err = AgentError::Schedule("Invalid cron expression".to_string());
        assert!(matches!(err, AgentError::Schedule(_)));
        assert_eq!(err.to_string(), "Schedule error: Invalid cron expression");
    }

    #[test]
    fn test_create_parse_error() {
        let err = AgentError::Parse("Invalid JSON format".to_string());
        assert!(matches!(err, AgentError::Parse(_)));
        assert_eq!(err.to_string(), "Parse error: Invalid JSON format");
    }

    #[test]
    fn test_create_network_error() {
        let err = AgentError::Network("Connection timeout".to_string());
        assert!(matches!(err, AgentError::Network(_)));
        assert_eq!(err.to_string(), "Network error: Connection timeout");
    }

    #[test]
    fn test_create_serialization_error() {
        let json_err = serde_json::from_str::<i32>("not a number").unwrap_err();
        let agent_err: AgentError = json_err.into();
        assert!(matches!(agent_err, AgentError::Serialization(_)));
        assert!(agent_err.to_string().contains("Serialization error:"));
    }

    #[test]
    fn test_create_other_error() {
        let err = AgentError::Other("Unexpected error occurred".to_string());
        assert!(matches!(err, AgentError::Other(_)));
        assert_eq!(err.to_string(), "Other error: Unexpected error occurred");
    }

    // ===== Error Conversion Tests =====

    #[test]
    fn test_from_rusqlite_error() {
        let sqlite_errors = vec![
            rusqlite::Error::QueryReturnedNoRows,
            rusqlite::Error::InvalidPath("test.db".into()),
            rusqlite::Error::SqliteSingleThreadedMode,
        ];

        for sqlite_err in sqlite_errors {
            let agent_err: AgentError = sqlite_err.into();
            assert!(matches!(agent_err, AgentError::Database(_)));
            assert!(agent_err.to_string().starts_with("Database error:"));
        }
    }

    #[test]
    fn test_from_io_error() {
        let io_errors = vec![
            io::Error::new(io::ErrorKind::NotFound, "File not found"),
            io::Error::new(io::ErrorKind::PermissionDenied, "Access denied"),
            io::Error::new(io::ErrorKind::AlreadyExists, "File exists"),
        ];

        for io_err in io_errors {
            let agent_err: AgentError = io_err.into();
            assert!(matches!(agent_err, AgentError::Io(_)));
            assert!(agent_err.to_string().starts_with("IO error:"));
        }
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_errors = vec![
            serde_json::from_str::<i32>("not_a_number").unwrap_err(),
            serde_json::from_str::<Vec<String>>("{}").unwrap_err(),
        ];

        for json_err in json_errors {
            let agent_err: AgentError = json_err.into();
            assert!(matches!(agent_err, AgentError::Serialization(_)));
            assert!(agent_err.to_string().starts_with("Serialization error:"));
        }
    }

    #[test]
    fn test_into_string_conversion() {
        // Test individual conversions since AgentError doesn't implement Clone
        let err = AgentError::Lock("test".to_string());
        let err_string: String = err.into();
        assert_eq!(err_string, "Lock error: test");

        let err = AgentError::AgentNotFound(1);
        let err_string: String = err.into();
        assert_eq!(err_string, "Agent not found: 1");

        let err = AgentError::RunNotFound(2);
        let err_string: String = err.into();
        assert_eq!(err_string, "Run not found: 2");

        let err = AgentError::Process("test".to_string());
        let err_string: String = err.into();
        assert_eq!(err_string, "Process error: test");

        let err = AgentError::BinaryNotFound("test".to_string());
        let err_string: String = err.into();
        assert_eq!(err_string, "Binary not found: test");
    }

    // ===== Display Implementation Tests =====

    #[test]
    fn test_display_formatting() {
        struct TestCase {
            error: AgentError,
            expected_prefix: &'static str,
        }

        let test_cases = vec![
            TestCase {
                error: AgentError::Lock("mutex poisoned".to_string()),
                expected_prefix: "Lock error: mutex poisoned",
            },
            TestCase {
                error: AgentError::AgentNotFound(999),
                expected_prefix: "Agent not found: 999",
            },
            TestCase {
                error: AgentError::RunNotFound(555),
                expected_prefix: "Run not found: 555",
            },
            TestCase {
                error: AgentError::Process("failed to spawn".to_string()),
                expected_prefix: "Process error: failed to spawn",
            },
            TestCase {
                error: AgentError::BinaryNotFound("/usr/bin/claude".to_string()),
                expected_prefix: "Binary not found: /usr/bin/claude",
            },
            TestCase {
                error: AgentError::InvalidStatus("paused".to_string()),
                expected_prefix: "Invalid status: paused",
            },
            TestCase {
                error: AgentError::InvalidModel("gpt-5".to_string()),
                expected_prefix: "Invalid model: gpt-5",
            },
            TestCase {
                error: AgentError::Sandbox("violation detected".to_string()),
                expected_prefix: "Sandbox error: violation detected",
            },
            TestCase {
                error: AgentError::Schedule("*/5 * * * * *".to_string()),
                expected_prefix: "Schedule error: */5 * * * * *",
            },
            TestCase {
                error: AgentError::Parse("unexpected token".to_string()),
                expected_prefix: "Parse error: unexpected token",
            },
            TestCase {
                error: AgentError::Network("DNS resolution failed".to_string()),
                expected_prefix: "Network error: DNS resolution failed",
            },
            TestCase {
                error: AgentError::Other("unknown issue".to_string()),
                expected_prefix: "Other error: unknown issue",
            },
        ];

        for case in test_cases {
            assert_eq!(case.error.to_string(), case.expected_prefix);
        }
    }

    #[test]
    fn test_display_with_nested_errors() {
        // Test that nested errors are properly displayed
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "Cannot write to file");
        let agent_err: AgentError = io_err.into();
        let display_str = agent_err.to_string();
        assert!(display_str.contains("IO error:"));
        assert!(display_str.contains("Cannot write to file"));
    }

    // ===== Serialization Tests =====

    #[test]
    fn test_serialize_agent_error() {
        let errors = vec![
            AgentError::Lock("test lock".to_string()),
            AgentError::AgentNotFound(42),
            AgentError::RunNotFound(123),
            AgentError::Process("process error".to_string()),
            AgentError::BinaryNotFound("binary".to_string()),
            AgentError::InvalidStatus("status".to_string()),
            AgentError::InvalidModel("model".to_string()),
            AgentError::Sandbox("sandbox".to_string()),
            AgentError::Schedule("schedule".to_string()),
            AgentError::Parse("parse".to_string()),
            AgentError::Network("network".to_string()),
            AgentError::Other("other".to_string()),
        ];

        for err in errors {
            let serialized = serde_json::to_string(&err).unwrap();
            // AgentError serializes as a string containing the error message
            let expected = format!("\"{}\"", err.to_string());
            assert_eq!(serialized, expected);
        }
    }

    #[test]
    fn test_serialize_in_json_structure() {
        #[derive(serde::Serialize)]
        struct ErrorResponse {
            success: bool,
            error: AgentError,
        }

        let response = ErrorResponse {
            success: false,
            error: AgentError::AgentNotFound(404),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["error"], "Agent not found: 404");
    }

    // ===== Error Handling Patterns Tests =====

    #[test]
    fn test_result_type_usage() {
        fn operation_that_may_fail(should_fail: bool) -> Result<String, AgentError> {
            if should_fail {
                Err(AgentError::Other("Operation failed".to_string()))
            } else {
                Ok("Success".to_string())
            }
        }

        // Test success case
        let result = operation_that_may_fail(false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success");

        // Test failure case
        let result = operation_that_may_fail(true);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AgentError::Other(_)));
    }

    #[test]
    fn test_error_propagation() {
        fn inner_function() -> Result<(), AgentError> {
            Err(AgentError::Process("Inner error".to_string()))
        }

        fn outer_function() -> Result<String, AgentError> {
            inner_function()?;
            Ok("Never reached".to_string())
        }

        let result = outer_function();
        assert!(result.is_err());
        match result.unwrap_err() {
            AgentError::Process(msg) => assert_eq!(msg, "Inner error"),
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_error_recovery() {
        fn recoverable_operation(attempt: u32) -> Result<String, AgentError> {
            match attempt {
                1 => Err(AgentError::Network("Temporary failure".to_string())),
                2 => Err(AgentError::Network("Still failing".to_string())),
                3 => Ok("Success after retry".to_string()),
                _ => Err(AgentError::Other("Too many attempts".to_string())),
            }
        }

        // Simulate retry logic
        let mut result = Err(AgentError::Other("Initial".to_string()));
        for attempt in 1..=3 {
            result = recoverable_operation(attempt);
            if result.is_ok() {
                break;
            }
        }

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success after retry");
    }

    #[test]
    fn test_error_context_preservation() {
        // Test that errors preserve their context when converted
        let original_io_error = io::Error::new(
            io::ErrorKind::NotFound,
            "Specific file: /path/to/file.txt not found"
        );
        
        let agent_error: AgentError = original_io_error.into();
        let error_string = agent_error.to_string();
        
        assert!(error_string.contains("IO error:"));
        assert!(error_string.contains("/path/to/file.txt"));
    }

    #[test]
    fn test_error_chain_handling() {
        fn database_operation() -> Result<(), rusqlite::Error> {
            Err(rusqlite::Error::QueryReturnedNoRows)
        }

        fn service_operation() -> Result<String, AgentError> {
            database_operation()?;
            Ok("Success".to_string())
        }

        let result = service_operation();
        assert!(result.is_err());
        match result.unwrap_err() {
            AgentError::Database(err) => {
                assert_eq!(err.to_string(), "Query returned no rows");
            }
            _ => panic!("Expected Database error"),
        }
    }

    #[test]
    fn test_custom_error_creation_patterns() {
        // Test common patterns for creating errors with context
        fn validate_agent_name(name: &str) -> Result<(), AgentError> {
            if name.is_empty() {
                return Err(AgentError::Parse("Agent name cannot be empty".to_string()));
            }
            if name.len() > 100 {
                return Err(AgentError::Parse(format!("Agent name too long: {} characters", name.len())));
            }
            Ok(())
        }

        assert!(validate_agent_name("Valid Name").is_ok());
        
        let empty_result = validate_agent_name("");
        assert!(matches!(empty_result, Err(AgentError::Parse(_))));
        
        let long_name = "a".repeat(101);
        let long_result = validate_agent_name(&long_name);
        assert!(matches!(long_result, Err(AgentError::Parse(msg)) if msg.contains("101 characters")));
    }

    #[test]
    fn test_error_pattern_matching() {
        fn handle_agent_error(err: AgentError) -> String {
            match err {
                AgentError::AgentNotFound(id) => format!("Agent {} does not exist", id),
                AgentError::RunNotFound(id) => format!("Run {} does not exist", id),
                AgentError::Lock(_) => "System is busy, please try again".to_string(),
                AgentError::Network(_) => "Network issue, check your connection".to_string(),
                AgentError::Database(_) => "Database error, contact support".to_string(),
                _ => "An unexpected error occurred".to_string(),
            }
        }

        assert_eq!(
            handle_agent_error(AgentError::AgentNotFound(42)),
            "Agent 42 does not exist"
        );
        assert_eq!(
            handle_agent_error(AgentError::RunNotFound(99)),
            "Run 99 does not exist"
        );
        assert_eq!(
            handle_agent_error(AgentError::Lock("any".to_string())),
            "System is busy, please try again"
        );
    }

    #[test]
    fn test_error_equality_and_debug() {
        // Test Debug trait implementation
        let err = AgentError::AgentNotFound(123);
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("AgentNotFound"));
        assert!(debug_str.contains("123"));

        // Test that errors can be formatted in various ways
        let err = AgentError::Process("Test process error".to_string());
        let display = format!("{}", err);
        let debug = format!("{:?}", err);
        
        assert_eq!(display, "Process error: Test process error");
        assert!(debug.contains("Process"));
        assert!(debug.contains("Test process error"));
    }

    #[test]
    fn test_frontend_error_serialization() {
        // Test that errors serialize in a way the frontend expects
        #[derive(serde::Serialize)]
        struct ApiResponse<T> {
            success: bool,
            #[serde(skip_serializing_if = "Option::is_none")]
            data: Option<T>,
            #[serde(skip_serializing_if = "Option::is_none")]
            error: Option<String>,
        }

        impl<T> From<Result<T, AgentError>> for ApiResponse<T> {
            fn from(result: Result<T, AgentError>) -> Self {
                match result {
                    Ok(data) => ApiResponse {
                        success: true,
                        data: Some(data),
                        error: None,
                    },
                    Err(err) => ApiResponse {
                        success: false,
                        data: None,
                        error: Some(err.to_string()),
                    },
                }
            }
        }

        // Test successful response
        let success_result: Result<String, AgentError> = Ok("Data".to_string());
        let response: ApiResponse<String> = success_result.into();
        let json = serde_json::to_value(&response).unwrap();
        
        assert_eq!(json["success"], true);
        assert_eq!(json["data"], "Data");
        assert_eq!(json.get("error"), None);

        // Test error response
        let error_result: Result<String, AgentError> = Err(AgentError::AgentNotFound(404));
        let response: ApiResponse<String> = error_result.into();
        let json = serde_json::to_value(&response).unwrap();
        
        assert_eq!(json["success"], false);
        assert_eq!(json.get("data"), None);
        assert_eq!(json["error"], "Agent not found: 404");
    }
}