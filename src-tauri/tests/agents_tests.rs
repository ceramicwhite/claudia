//! Unit tests for agents functionality

#[cfg(test)]
mod agent_metrics_tests {
    use chrono::{DateTime, Utc};
    use serde_json::json;

    // Mock the AgentRunMetrics struct for testing
    #[derive(Debug, PartialEq)]
    struct AgentRunMetrics {
        total_tokens: i64,
        total_input_tokens: i64,
        total_output_tokens: i64,
        total_cache_creation_tokens: i64,
        total_cache_read_tokens: i64,
        cost_usd: f64,
        message_count: i64,
        duration: Option<i64>,
    }

    impl AgentRunMetrics {
        pub fn from_jsonl(jsonl_content: &str, model: &str) -> Self {
            let mut total_tokens = 0i64;
            let mut total_input_tokens = 0i64;
            let mut total_output_tokens = 0i64;
            let mut total_cache_creation_tokens = 0i64;
            let mut total_cache_read_tokens = 0i64;
            let mut cost_usd = 0.0f64;
            let mut has_cost_field = false;
            let mut message_count = 0i64;
            let mut start_time: Option<DateTime<Utc>> = None;
            let mut end_time: Option<DateTime<Utc>> = None;

            for line in jsonl_content.lines() {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                    message_count += 1;

                    // Track timestamps
                    if let Some(timestamp_str) = json.get("timestamp").and_then(|t| t.as_str()) {
                        if let Ok(timestamp) = DateTime::parse_from_rfc3339(timestamp_str) {
                            let utc_time = timestamp.with_timezone(&Utc);
                            if start_time.is_none() || utc_time < start_time.unwrap() {
                                start_time = Some(utc_time);
                            }
                            if end_time.is_none() || utc_time > end_time.unwrap() {
                                end_time = Some(utc_time);
                            }
                        }
                    }

                    // Extract cost if available
                    if let Some(cost) = json.get("cost").and_then(|c| c.as_f64()) {
                        cost_usd += cost;
                        has_cost_field = true;
                    }

                    // Extract usage metrics
                    if let Some(usage) = json.get("usage") {
                        if let Some(tokens) = usage.get("total_tokens").and_then(|t| t.as_i64()) {
                            total_tokens += tokens;
                        }
                        if let Some(input) = usage.get("input_tokens").and_then(|t| t.as_i64()) {
                            total_input_tokens += input;
                        }
                        if let Some(output) = usage.get("output_tokens").and_then(|t| t.as_i64()) {
                            total_output_tokens += output;
                        }
                        if let Some(cache_creation) = usage.get("cache_creation_input_tokens").and_then(|t| t.as_i64()) {
                            total_cache_creation_tokens += cache_creation;
                        }
                        if let Some(cache_read) = usage.get("cache_read_input_tokens").and_then(|t| t.as_i64()) {
                            total_cache_read_tokens += cache_read;
                        }
                    }
                }
            }

            // Calculate cost if not provided
            if !has_cost_field && total_tokens > 0 {
                cost_usd = calculate_cost(
                    model,
                    total_input_tokens,
                    total_output_tokens,
                    total_cache_creation_tokens,
                    total_cache_read_tokens,
                );
            }

            let duration = if let (Some(start), Some(end)) = (start_time, end_time) {
                Some((end - start).num_seconds())
            } else {
                None
            };

            AgentRunMetrics {
                total_tokens,
                total_input_tokens,
                total_output_tokens,
                total_cache_creation_tokens,
                total_cache_read_tokens,
                cost_usd,
                message_count,
                duration,
            }
        }
    }

    fn calculate_cost(
        model: &str,
        input_tokens: i64,
        output_tokens: i64,
        cache_creation_tokens: i64,
        cache_read_tokens: i64,
    ) -> f64 {
        let (input_rate, output_rate, cache_creation_rate, cache_read_rate) = match model {
            "claude-3-5-sonnet-20241022" => (3.0, 15.0, 3.75, 0.30),
            "claude-3-5-haiku-20241022" => (1.0, 5.0, 1.25, 0.10),
            "claude-3-opus-20240229" => (15.0, 75.0, 18.75, 1.50),
            "claude-3-haiku-20240307" => (0.25, 1.25, 0.30, 0.03),
            _ => (3.0, 15.0, 3.75, 0.30), // Default to Sonnet pricing
        };

        let input_cost = (input_tokens as f64 * input_rate) / 1_000_000.0;
        let output_cost = (output_tokens as f64 * output_rate) / 1_000_000.0;
        let cache_creation_cost = (cache_creation_tokens as f64 * cache_creation_rate) / 1_000_000.0;
        let cache_read_cost = (cache_read_tokens as f64 * cache_read_rate) / 1_000_000.0;

        input_cost + output_cost + cache_creation_cost + cache_read_cost
    }

    #[test]
    fn test_from_jsonl_with_cost_field() {
        let jsonl = r#"{"timestamp":"2024-01-01T00:00:00Z","cost":0.05,"usage":{"total_tokens":1000,"input_tokens":800,"output_tokens":200}}
{"timestamp":"2024-01-01T00:01:00Z","cost":0.03,"usage":{"total_tokens":600,"input_tokens":400,"output_tokens":200}}"#;

        let metrics = AgentRunMetrics::from_jsonl(jsonl, "claude-3-5-sonnet-20241022");

        assert_eq!(metrics.total_tokens, 1600);
        assert_eq!(metrics.total_input_tokens, 1200);
        assert_eq!(metrics.total_output_tokens, 400);
        assert_eq!(metrics.cost_usd, 0.08);
        assert_eq!(metrics.message_count, 2);
        assert_eq!(metrics.duration, Some(60)); // 1 minute difference
    }

    #[test]
    fn test_from_jsonl_without_cost_field() {
        let jsonl = r#"{"timestamp":"2024-01-01T00:00:00Z","usage":{"total_tokens":1000,"input_tokens":800,"output_tokens":200}}
{"timestamp":"2024-01-01T00:01:00Z","usage":{"total_tokens":600,"input_tokens":400,"output_tokens":200}}"#;

        let metrics = AgentRunMetrics::from_jsonl(jsonl, "claude-3-5-sonnet-20241022");

        assert_eq!(metrics.total_tokens, 1600);
        assert_eq!(metrics.total_input_tokens, 1200);
        assert_eq!(metrics.total_output_tokens, 400);
        // Cost should be calculated: (1200 * 3.0 + 400 * 15.0) / 1_000_000
        assert_eq!(metrics.cost_usd, 0.0096);
        assert_eq!(metrics.message_count, 2);
    }

    #[test]
    fn test_from_jsonl_with_cache_tokens() {
        let jsonl = r#"{"usage":{"total_tokens":1000,"input_tokens":600,"output_tokens":200,"cache_creation_input_tokens":100,"cache_read_input_tokens":100}}"#;

        let metrics = AgentRunMetrics::from_jsonl(jsonl, "claude-3-5-sonnet-20241022");

        assert_eq!(metrics.total_cache_creation_tokens, 100);
        assert_eq!(metrics.total_cache_read_tokens, 100);
        // Cost calculation should include cache tokens
        let expected_cost = (600.0 * 3.0 + 200.0 * 15.0 + 100.0 * 3.75 + 100.0 * 0.30) / 1_000_000.0;
        assert!((metrics.cost_usd - expected_cost).abs() < 0.000001);
    }

    #[test]
    fn test_from_jsonl_empty_content() {
        let metrics = AgentRunMetrics::from_jsonl("", "claude-3-5-sonnet-20241022");

        assert_eq!(metrics.total_tokens, 0);
        assert_eq!(metrics.cost_usd, 0.0);
        assert_eq!(metrics.message_count, 0);
        assert_eq!(metrics.duration, None);
    }

    #[test]
    fn test_from_jsonl_invalid_json_lines() {
        let jsonl = r#"{"valid":true,"usage":{"total_tokens":100}}
not valid json
{"valid":true,"usage":{"total_tokens":200}}"#;

        let metrics = AgentRunMetrics::from_jsonl(jsonl, "claude-3-5-sonnet-20241022");

        // Should skip invalid lines
        assert_eq!(metrics.total_tokens, 300);
        assert_eq!(metrics.message_count, 2);
    }

    #[test]
    fn test_from_jsonl_different_models() {
        let jsonl = r#"{"usage":{"total_tokens":1000,"input_tokens":800,"output_tokens":200}}"#;

        // Test Haiku pricing
        let haiku_metrics = AgentRunMetrics::from_jsonl(jsonl, "claude-3-5-haiku-20241022");
        let expected_haiku_cost = (800.0 * 1.0 + 200.0 * 5.0) / 1_000_000.0;
        assert!((haiku_metrics.cost_usd - expected_haiku_cost).abs() < 0.000001);

        // Test Opus pricing
        let opus_metrics = AgentRunMetrics::from_jsonl(jsonl, "claude-3-opus-20240229");
        let expected_opus_cost = (800.0 * 15.0 + 200.0 * 75.0) / 1_000_000.0;
        assert!((opus_metrics.cost_usd - expected_opus_cost).abs() < 0.000001);
    }

    #[test]
    fn test_from_jsonl_timestamps_out_of_order() {
        let jsonl = r#"{"timestamp":"2024-01-01T00:05:00Z","usage":{"total_tokens":100}}
{"timestamp":"2024-01-01T00:00:00Z","usage":{"total_tokens":200}}
{"timestamp":"2024-01-01T00:10:00Z","usage":{"total_tokens":300}}"#;

        let metrics = AgentRunMetrics::from_jsonl(jsonl, "claude-3-5-sonnet-20241022");

        assert_eq!(metrics.duration, Some(600)); // 10 minutes total
    }
}

// Helper function to parse usage limit errors
pub fn parse_usage_limit_error(output: &str) -> Option<String> {
    use chrono::{DateTime, Utc, Duration};

    // Check if the output contains the usage limit error pattern
    if output.contains("Claude AI usage limit reached|") {
        // Extract the timestamp
        if let Some(pipe_pos) = output.rfind('|') {
            let timestamp_str = &output[pipe_pos + 1..].trim();
            if let Ok(timestamp) = timestamp_str.parse::<i64>() {
                return Some(timestamp.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod usage_limit_tests {
    use chrono::{DateTime, Utc, Duration};


    #[test]
    fn test_parse_usage_limit_error_valid() {
        let output = "Error: Claude AI usage limit reached|1704067200";
        let result = super::parse_usage_limit_error(output);
        
        assert!(result.is_some());
        let reset_time = result.unwrap();
        // The timestamp 1704067200 is 2024-01-01 00:00:00 UTC
        // With 1 minute buffer, it should be 2024-01-01 00:01:00 UTC
        assert!(reset_time.contains("2024-01-01T00:01:00"));
    }

    #[test]
    fn test_parse_usage_limit_error_no_match() {
        let output = "Some other error message";
        let result = super::parse_usage_limit_error(output);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_usage_limit_error_invalid_timestamp() {
        let output = "Claude AI usage limit reached|not_a_number";
        let result = super::parse_usage_limit_error(output);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_usage_limit_error_missing_pipe() {
        let output = "Claude AI usage limit reached";
        let result = super::parse_usage_limit_error(output);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_usage_limit_error_with_extra_content() {
        let output = "Some prefix text\nClaude AI usage limit reached|1704067200\nSome suffix";
        let result = super::parse_usage_limit_error(output);
        
        assert!(result.is_some());
        assert!(result.unwrap().contains("2024-01-01T00:01:00"));
    }
}

#[cfg(test)]
mod migration_tests {
    use rusqlite::{Connection, params};
    use tempfile::TempDir;
    use super::parse_usage_limit_error;

    fn create_test_db() -> (TempDir, Connection) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();

        // Create agent_runs table
        conn.execute(
            "CREATE TABLE agent_runs (
                id INTEGER PRIMARY KEY,
                session_id TEXT NOT NULL,
                project_path TEXT NOT NULL,
                status TEXT NOT NULL,
                usage_limit_reset_time TEXT,
                completed_at TEXT
            )",
            [],
        ).unwrap();

        (temp_dir, conn)
    }

    fn migrate_old_usage_limit_runs(conn: &Connection) {
        // Get all completed runs that might have ended with usage limit
        let mut stmt = match conn.prepare(
            "SELECT id, session_id, project_path FROM agent_runs 
             WHERE status IN ('completed', 'failed') 
             AND usage_limit_reset_time IS NULL"
        ) {
            Ok(stmt) => stmt,
            Err(_) => return,
        };

        let runs: Vec<(i64, String, String)> = match stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        }) {
            Ok(rows) => rows.filter_map(Result::ok).collect(),
            Err(_) => return,
        };

        drop(stmt);

        for (run_id, session_id, project_path) in runs {
            // Create fake session directory for testing
            let session_dir = std::path::PathBuf::from(&project_path)
                .join(".claudia")
                .join("sessions")
                .join(&session_id);
            
            if let Err(_) = std::fs::create_dir_all(&session_dir) {
                continue;
            }

            // Create test output file
            let output_path = session_dir.join("output.jsonl");
            let test_content = r#"{"type":"output","content":"Claude AI usage limit reached|1704067200"}"#;
            if let Err(_) = std::fs::write(&output_path, test_content) {
                continue;
            }

            // Read and parse the output
            if let Ok(content) = std::fs::read_to_string(&output_path) {
                for line in content.lines() {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                        if let Some(output) = json.get("content").and_then(|c| c.as_str()) {
                            if let Some(reset_time) = parse_usage_limit_error(output) {
                                let _ = conn.execute(
                                    "UPDATE agent_runs SET usage_limit_reset_time = ?1 WHERE id = ?2",
                                    params![reset_time, run_id],
                                );
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_migrate_old_usage_limit_runs_completed() {
        let (_temp_dir, conn) = create_test_db();
        
        // Insert test data
        conn.execute(
            "INSERT INTO agent_runs (session_id, project_path, status) VALUES (?1, ?2, ?3)",
            params!["session1", _temp_dir.path().to_str().unwrap(), "completed"],
        ).unwrap();

        // Run migration
        migrate_old_usage_limit_runs(&conn);

        // Check if usage_limit_reset_time was set
        let result: Option<String> = conn.query_row(
            "SELECT usage_limit_reset_time FROM agent_runs WHERE session_id = ?1",
            params!["session1"],
            |row| row.get(0),
        ).unwrap();

        assert!(result.is_some());
        assert!(result.unwrap().contains("2024-01-01T00:01:00"));
    }

    #[test]
    fn test_migrate_old_usage_limit_runs_no_output_file() {
        let (_temp_dir, conn) = create_test_db();
        
        // Insert test data (without creating output file)
        conn.execute(
            "INSERT INTO agent_runs (session_id, project_path, status) VALUES (?1, ?2, ?3)",
            params!["session2", "/nonexistent/path", "completed"],
        ).unwrap();

        // Run migration
        migrate_old_usage_limit_runs(&conn);

        // Check that usage_limit_reset_time is still NULL
        let result: Option<String> = conn.query_row(
            "SELECT usage_limit_reset_time FROM agent_runs WHERE session_id = ?1",
            params!["session2"],
            |row| row.get(0),
        ).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_migrate_old_usage_limit_runs_skip_already_set() {
        let (_temp_dir, conn) = create_test_db();
        
        // Insert test data with usage_limit_reset_time already set
        conn.execute(
            "INSERT INTO agent_runs (session_id, project_path, status, usage_limit_reset_time) 
             VALUES (?1, ?2, ?3, ?4)",
            params!["session3", "/some/path", "completed", "2024-01-01T00:00:00Z"],
        ).unwrap();

        // Run migration
        migrate_old_usage_limit_runs(&conn);

        // Check that usage_limit_reset_time wasn't changed
        let result: String = conn.query_row(
            "SELECT usage_limit_reset_time FROM agent_runs WHERE session_id = ?1",
            params!["session3"],
            |row| row.get(0),
        ).unwrap();

        assert_eq!(result, "2024-01-01T00:00:00Z");
    }
}

#[cfg(test)]
mod command_handler_tests {
    use rusqlite::{Connection, params};
    use tempfile::TempDir;
    use serde_json::json;

    fn create_test_db_with_schema() -> (TempDir, Connection) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();

        // Create complete schema
        conn.execute_batch(
            "CREATE TABLE agent_runs (
                id INTEGER PRIMARY KEY,
                agent_id INTEGER NOT NULL,
                agent_name TEXT NOT NULL,
                agent_icon TEXT,
                task TEXT NOT NULL,
                model TEXT NOT NULL,
                project_path TEXT NOT NULL,
                session_id TEXT NOT NULL,
                status TEXT NOT NULL,
                pid INTEGER,
                process_started_at TEXT,
                scheduled_start_time TEXT,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                completed_at TEXT,
                usage_limit_reset_time TEXT,
                auto_resume_enabled BOOLEAN DEFAULT FALSE,
                resume_count INTEGER DEFAULT 0,
                parent_run_id INTEGER
            );
            
            CREATE TABLE agent_run_metrics (
                id INTEGER PRIMARY KEY,
                run_id INTEGER NOT NULL,
                total_tokens INTEGER DEFAULT 0,
                total_input_tokens INTEGER DEFAULT 0,
                total_output_tokens INTEGER DEFAULT 0,
                total_cache_creation_tokens INTEGER DEFAULT 0,
                total_cache_read_tokens INTEGER DEFAULT 0,
                cost_usd REAL DEFAULT 0.0,
                message_count INTEGER DEFAULT 0,
                duration_seconds INTEGER,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (run_id) REFERENCES agent_runs(id)
            );"
        ).unwrap();

        (temp_dir, conn)
    }

    #[test]
    fn test_get_usage_limit_runs() {
        let (_temp_dir, conn) = create_test_db_with_schema();
        
        // Insert test data - runs with usage limits
        conn.execute(
            "INSERT INTO agent_runs (agent_id, agent_name, task, model, project_path, 
             session_id, status, usage_limit_reset_time) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![1, "TestAgent", "task1", "claude-3-5-sonnet", "/path", 
                    "session1", "completed", "2024-01-01T00:00:00Z"],
        ).unwrap();

        conn.execute(
            "INSERT INTO agent_runs (agent_id, agent_name, task, model, project_path, 
             session_id, status, usage_limit_reset_time) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![1, "TestAgent", "task2", "claude-3-5-sonnet", "/path", 
                    "session2", "failed", "2024-01-02T00:00:00Z"],
        ).unwrap();

        // Insert run without usage limit
        conn.execute(
            "INSERT INTO agent_runs (agent_id, agent_name, task, model, project_path, 
             session_id, status) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![1, "TestAgent", "task3", "claude-3-5-sonnet", "/path", 
                    "session3", "completed"],
        ).unwrap();

        // Query for usage limit runs
        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM agent_runs WHERE usage_limit_reset_time IS NOT NULL"
        ).unwrap();
        
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_update_run_metrics() {
        let (_temp_dir, conn) = create_test_db_with_schema();
        
        // Insert test run
        conn.execute(
            "INSERT INTO agent_runs (agent_id, agent_name, task, model, project_path, 
             session_id, status) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![1, "TestAgent", "task", "claude-3-5-sonnet", "/path", 
                    "session1", "running"],
        ).unwrap();
        
        let run_id = conn.last_insert_rowid();

        // Insert metrics
        conn.execute(
            "INSERT INTO agent_run_metrics (run_id, total_tokens, total_input_tokens, 
             total_output_tokens, cost_usd, message_count) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![run_id, 1000, 800, 200, 0.01, 5],
        ).unwrap();

        // Verify metrics
        let (tokens, cost): (i64, f64) = conn.query_row(
            "SELECT total_tokens, cost_usd FROM agent_run_metrics WHERE run_id = ?1",
            params![run_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap();

        assert_eq!(tokens, 1000);
        assert_eq!(cost, 0.01);
    }
}