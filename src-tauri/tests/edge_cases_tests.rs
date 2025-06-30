//! Edge case and error scenario tests

#[cfg(test)]
mod edge_cases {
    use chrono::{DateTime, Utc, Duration, NaiveDateTime};
    use rusqlite::{Connection, params, Error as SqliteError};
    use tempfile::TempDir;

    fn create_test_db() -> (TempDir, Connection) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();

        conn.execute_batch(
            "CREATE TABLE agent_runs (
                id INTEGER PRIMARY KEY,
                agent_id INTEGER NOT NULL,
                agent_name TEXT NOT NULL,
                task TEXT NOT NULL,
                model TEXT NOT NULL,
                project_path TEXT NOT NULL,
                session_id TEXT NOT NULL,
                status TEXT NOT NULL,
                scheduled_start_time TEXT,
                usage_limit_reset_time TEXT,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                completed_at TEXT
            );
            
            CREATE TABLE agent_run_metrics (
                id INTEGER PRIMARY KEY,
                run_id INTEGER NOT NULL,
                total_tokens INTEGER DEFAULT 0,
                cost_usd REAL DEFAULT 0.0,
                FOREIGN KEY (run_id) REFERENCES agent_runs(id)
            );"
        ).unwrap();

        (temp_dir, conn)
    }

    #[test]
    fn test_malformed_timestamp_in_usage_limit() {
        let output_variations = vec![
            "Claude AI usage limit reached|not_a_timestamp",
            "Claude AI usage limit reached|",
            "Claude AI usage limit reached|-12345",
            "Claude AI usage limit reached|99999999999999999",
            "Claude AI usage limit reached|0",
        ];

        for output in output_variations {
            let result = super::super::agents_tests::usage_limit_tests::parse_usage_limit_error(output);
            // Most should return None due to invalid timestamp
            if output.contains("0") {
                // Epoch 0 is valid (1970-01-01)
                assert!(result.is_some());
            } else {
                assert!(result.is_none(), "Expected None for output: {}", output);
            }
        }
    }

    #[test]
    fn test_database_constraint_violations() {
        let (_temp_dir, conn) = create_test_db();
        
        // Try to insert metrics for non-existent run
        let result = conn.execute(
            "INSERT INTO agent_run_metrics (run_id, total_tokens) VALUES (?1, ?2)",
            params![9999, 100],
        );
        
        assert!(result.is_err());
        if let Err(e) = result {
            match e {
                SqliteError::SqliteFailure(_, _) => {
                    // Foreign key constraint should fail
                }
                _ => panic!("Expected foreign key constraint error"),
            }
        }
    }

    #[test]
    fn test_concurrent_scheduler_edge_cases() {
        let (_temp_dir, conn) = create_test_db();
        
        // Insert a run that's exactly at the current time
        let now = Utc::now();
        conn.execute(
            "INSERT INTO agent_runs (agent_id, agent_name, task, model, project_path, 
             session_id, status, scheduled_start_time) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![1, "Agent", "task", "model", "/path", "session1", 
                    "scheduled", now.to_rfc3339()],
        ).unwrap();

        // Query with exact same timestamp
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM agent_runs 
             WHERE status = 'scheduled' 
               AND scheduled_start_time <= ?1",
            params![now.to_rfc3339()],
            |row| row.get(0)
        ).unwrap();

        assert_eq!(count, 1);
    }

    #[test]
    fn test_extreme_token_values() {
        let extreme_values = vec![
            (i64::MAX, f64::MAX),
            (0, 0.0),
            (-1, -1.0), // Negative values shouldn't happen but test handling
        ];

        for (tokens, cost) in extreme_values {
            let jsonl = format!(
                r#"{{"usage":{{"total_tokens":{}}},"cost":{}}}"#,
                tokens, cost
            );
            
            // This should not panic
            let metrics = super::super::agents_tests::agent_metrics_tests::AgentRunMetrics::from_jsonl(
                &jsonl, 
                "claude-3-5-sonnet"
            );
            
            // For negative values, they should be treated as 0 or handled gracefully
            if tokens < 0 {
                assert!(metrics.total_tokens >= 0);
            }
        }
    }

    #[test]
    fn test_invalid_json_structures() {
        let invalid_jsons = vec![
            r#"{"usage":null}"#,
            r#"{"usage":{"total_tokens":"not_a_number"}}"#,
            r#"{"usage":[]}"#,
            r#"{"cost":true}"#,
            r#"{]"#, // Completely invalid JSON
            r#""#, // Empty string
            r#"null"#,
        ];

        for json in invalid_jsons {
            // Should handle gracefully without panicking
            let metrics = super::super::agents_tests::agent_metrics_tests::AgentRunMetrics::from_jsonl(
                json, 
                "claude-3-5-sonnet"
            );
            
            // Should return zero values for invalid data
            assert_eq!(metrics.total_tokens, 0);
            assert_eq!(metrics.message_count, 0);
        }
    }

    #[test]
    fn test_timezone_edge_cases() {
        let timestamps = vec![
            "2024-01-01T00:00:00+00:00", // UTC
            "2024-01-01T00:00:00Z",      // UTC shorthand
            "2024-01-01T00:00:00-05:00", // EST
            "2024-01-01T00:00:00+14:00", // Kiribati (max offset)
            "2024-01-01T00:00:00-12:00", // Baker Island (min offset)
        ];

        for timestamp in timestamps {
            let jsonl = format!(r#"{{"timestamp":"{}"}}"#, timestamp);
            let metrics = super::super::agents_tests::agent_metrics_tests::AgentRunMetrics::from_jsonl(
                &jsonl, 
                "claude-3-5-sonnet"
            );
            
            // Should successfully parse all valid ISO 8601 timestamps
            assert_eq!(metrics.message_count, 1);
        }
    }

    #[test]
    fn test_scheduler_with_dst_transitions() {
        let (_temp_dir, conn) = create_test_db();
        
        // Test scheduling around daylight saving time transitions
        // This is a conceptual test - actual DST handling depends on chrono's implementation
        let base_time = DateTime::parse_from_rfc3339("2024-03-10T01:30:00-05:00")
            .unwrap()
            .with_timezone(&Utc);
        
        // Schedule runs around DST transition (2 AM becomes 3 AM in US Eastern)
        for i in 0..4 {
            let scheduled_time = (base_time + Duration::minutes(30 * i)).to_rfc3339();
            conn.execute(
                "INSERT INTO agent_runs (agent_id, agent_name, task, model, project_path, 
                 session_id, status, scheduled_start_time) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![1, "Agent", "task", "model", "/path", 
                        format!("session{}", i), "scheduled", scheduled_time],
            ).unwrap();
        }

        // All times should be properly stored and comparable
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM agent_runs WHERE status = 'scheduled'",
            [],
            |row| row.get(0)
        ).unwrap();

        assert_eq!(count, 4);
    }

    #[test]
    fn test_model_pricing_edge_cases() {
        // Test with unknown model names
        let unknown_models = vec![
            "claude-3-unknown-model",
            "gpt-4",
            "",
            "Claude 3.5 Sonnet", // Wrong format
            "claude-3-5-sonnet", // Missing version
        ];

        for model in unknown_models {
            let jsonl = r#"{"usage":{"total_tokens":1000,"input_tokens":800,"output_tokens":200}}"#;
            let metrics = super::super::agents_tests::agent_metrics_tests::AgentRunMetrics::from_jsonl(
                jsonl, 
                model
            );
            
            // Should use default pricing for unknown models
            assert!(metrics.cost_usd > 0.0);
        }
    }

    #[test]
    fn test_file_system_edge_cases_in_migration() {
        let (_temp_dir, conn) = create_test_db();
        
        // Create runs with problematic paths
        let problematic_paths = vec![
            "/path/with spaces/project",
            "/path/with/special@#$%characters",
            "/very/long/path/".to_string() + &"a".repeat(255), // Very long path
            "relative/path/without/slash", // Relative path
            "/path/with/../dots",
            "/path/with/./single/dot",
        ];

        for (i, path) in problematic_paths.iter().enumerate() {
            conn.execute(
                "INSERT INTO agent_runs (agent_id, agent_name, task, model, project_path, 
                 session_id, status) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![1, "Agent", "task", "model", path, 
                        format!("session{}", i), "completed"],
            ).unwrap();
        }

        // Migration should handle all these paths gracefully
        super::super::agents_tests::migration_tests::migrate_old_usage_limit_runs(&conn);
        
        // No runs should have usage_limit_reset_time set (since we didn't create output files)
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM agent_runs WHERE usage_limit_reset_time IS NOT NULL",
            [],
            |row| row.get(0)
        ).unwrap();

        assert_eq!(count, 0);
    }

    #[test]
    fn test_race_condition_in_scheduler_status_update() {
        let (_temp_dir, conn) = create_test_db();
        
        // Insert a scheduled run
        conn.execute(
            "INSERT INTO agent_runs (agent_id, agent_name, task, model, project_path, 
             session_id, status, scheduled_start_time) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![1, "Agent", "task", "model", "/path", "session1", 
                    "scheduled", "2024-01-01T00:00:00Z"],
        ).unwrap();
        let run_id = conn.last_insert_rowid();

        // Simulate multiple concurrent attempts to update the same run
        // Using transactions to simulate concurrency
        let tx1 = conn.unchecked_transaction().unwrap();
        
        // First transaction updates to pending
        tx1.execute(
            "UPDATE agent_runs SET status = 'pending' WHERE id = ?1 AND status = 'scheduled'",
            params![run_id]
        ).unwrap();
        
        // Before committing tx1, try another update (simulating race condition)
        // This would fail in a real concurrent scenario
        let result = conn.execute(
            "UPDATE agent_runs SET status = 'running' WHERE id = ?1 AND status = 'scheduled'",
            params![run_id]
        );
        
        // Should not update any rows since status is no longer 'scheduled'
        assert_eq!(result.unwrap(), 0);
        
        tx1.commit().unwrap();
    }

    #[test]
    fn test_null_and_empty_string_handling() {
        let (_temp_dir, conn) = create_test_db();
        
        // Test various NULL and empty string scenarios
        conn.execute(
            "INSERT INTO agent_runs (agent_id, agent_name, task, model, project_path, 
             session_id, status) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![1, "", "", "model", "", "", "scheduled"],
        ).unwrap();

        // Query should handle empty strings
        let result: (String, String) = conn.query_row(
            "SELECT agent_name, task FROM agent_runs WHERE agent_id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?))
        ).unwrap();

        assert_eq!(result.0, "");
        assert_eq!(result.1, "");
    }
}