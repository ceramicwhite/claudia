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
                status TEXT NOT NULL,
                project_path TEXT NOT NULL,
                session_id TEXT NOT NULL,
                start_time TEXT,
                end_time TEXT,
                pid INTEGER,
                scheduled_start_time TEXT,
                parent_run_id INTEGER
            )",
        ).unwrap();

        (temp_dir, conn)
    }

    // Local helper function for parsing usage limit errors
    fn parse_usage_limit_error(output: &str) -> Option<DateTime<Utc>> {
        if output.contains("Usage limit reached") {
            if let Some(start) = output.find("resets at ") {
                let timestamp_str = &output[start + 10..];
                if let Some(end) = timestamp_str.find('.') {
                    if let Ok(timestamp) = timestamp_str[..end].parse::<i64>() {
                        return DateTime::from_timestamp(timestamp, 0);
                    }
                }
            }
        }
        None
    }

    #[test]
    fn test_malformed_timestamp_parsing() {
        let output_variations = vec![
            "Usage limit reached. Daily limit resets at -1234567890.",
            "Usage limit reached. Daily limit resets at 999999999999999999.",
            "Usage limit reached. Daily limit resets at 0.",
            "Usage limit reached. Daily limit resets at abc123.",
            "Usage limit reached. Daily limit resets at 1234.5678.",
            "Usage limit reached. Daily limit resets at .",
            "Usage limit reached. Daily limit resets at",
            "Usage limit reached.",
            "",
        ];

        for output in output_variations {
            let result = parse_usage_limit_error(output);
            // Most should return None due to invalid timestamp
            if output.contains("0.") {
                // Epoch 0 is valid (1970-01-01)
                assert!(result.is_some());
            } else if output.contains("999999999999999999") || output.contains("-1234567890") {
                // These timestamps are out of range
                assert!(result.is_none(), "Expected None for output: {}", output);
            }
        }
    }

    #[test]
    fn test_concurrent_run_updates() {
        let (_temp, conn) = create_test_db();

        // Insert test run
        conn.execute(
            "INSERT INTO agent_runs (agent_id, agent_name, task, model, status, project_path, session_id) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![1, "Test Agent", "Test Task", "claude-3-opus", "running", "/test", "session-1"],
        ).unwrap();

        let run_id = conn.last_insert_rowid();

        // Simulate concurrent updates
        let updates = vec![
            ("completed", Some("2024-01-01T00:00:01Z")),
            ("failed", Some("2024-01-01T00:00:02Z")),
            ("cancelled", Some("2024-01-01T00:00:03Z")),
        ];

        for (status, end_time) in updates {
            let result = conn.execute(
                "UPDATE agent_runs SET status = ?1, end_time = ?2 WHERE id = ?3",
                params![status, end_time, run_id],
            );
            assert!(result.is_ok());
        }

        // Verify final state
        let final_status: String = conn
            .query_row(
                "SELECT status FROM agent_runs WHERE id = ?1",
                params![run_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(final_status, "cancelled");
    }

    #[test]
    fn test_database_constraint_violations() {
        let (_temp, conn) = create_test_db();

        // Add unique constraint
        conn.execute(
            "CREATE UNIQUE INDEX idx_session_id ON agent_runs(session_id)",
            [],
        ).unwrap();

        // Insert first run
        conn.execute(
            "INSERT INTO agent_runs (agent_id, agent_name, task, model, status, project_path, session_id) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![1, "Test Agent", "Test Task", "claude-3-opus", "running", "/test", "session-1"],
        ).unwrap();

        // Try to insert duplicate session_id
        let result = conn.execute(
            "INSERT INTO agent_runs (agent_id, agent_name, task, model, status, project_path, session_id) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![2, "Test Agent 2", "Test Task 2", "claude-3-opus", "running", "/test2", "session-1"],
        );

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, SqliteError::SqliteFailure(_, _)));
        }
    }

    #[test]
    fn test_extreme_duration_calculations() {
        let test_cases = vec![
            // Normal case
            ("2024-01-01T00:00:00Z", "2024-01-01T00:01:00Z", 60_000),
            // Same time
            ("2024-01-01T00:00:00Z", "2024-01-01T00:00:00Z", 0),
            // Very long duration (1 year)
            ("2023-01-01T00:00:00Z", "2024-01-01T00:00:00Z", 31536000000),
        ];

        for (start, end, expected_ms) in test_cases {
            let start_time = DateTime::parse_from_rfc3339(start).unwrap().with_timezone(&Utc);
            let end_time = DateTime::parse_from_rfc3339(end).unwrap().with_timezone(&Utc);
            let duration = end_time.signed_duration_since(start_time);
            let duration_ms = duration.num_milliseconds();
            
            assert_eq!(duration_ms, expected_ms, 
                "Duration mismatch for {} -> {}", start, end);
        }
    }

    #[test]
    fn test_orphaned_processes() {
        let (_temp, conn) = create_test_db();

        // Insert runs with PIDs
        let pids = vec![99999, 88888, 77777];
        for (i, pid) in pids.iter().enumerate() {
            conn.execute(
                "INSERT INTO agent_runs (agent_id, agent_name, task, model, status, project_path, session_id, pid) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    1, 
                    "Test Agent", 
                    format!("Task {}", i), 
                    "claude-3-opus", 
                    "running", 
                    "/test", 
                    format!("session-{}", i),
                    pid
                ],
            ).unwrap();
        }

        // Simulate finding orphaned processes (PIDs that don't exist)
        let mut orphaned_count = 0;
        let mut stmt = conn.prepare("SELECT id, pid FROM agent_runs WHERE status = 'running' AND pid IS NOT NULL").unwrap();
        let runs = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i32>(1)?))
        }).unwrap();

        for run in runs {
            let (_id, pid) = run.unwrap();
            // These PIDs shouldn't exist
            #[cfg(unix)]
            {
                use nix::sys::signal::{self, Signal};
                use nix::unistd::Pid;
                
                let result = signal::kill(Pid::from_raw(pid), Signal::SIGTERM);
                if result.is_err() {
                    orphaned_count += 1;
                }
            }
            #[cfg(not(unix))]
            {
                // On non-Unix, assume all are orphaned for test
                orphaned_count += 1;
            }
        }

        assert_eq!(orphaned_count, 3);
    }

    #[test]
    fn test_migration_edge_cases() {
        let (_temp, conn) = create_test_db();

        // Add column that might be missing in old schemas
        let result = conn.execute("ALTER TABLE agent_runs ADD COLUMN new_field TEXT", []);
        assert!(result.is_ok());

        // Try adding it again - should fail but be handled gracefully
        let result = conn.execute("ALTER TABLE agent_runs ADD COLUMN new_field TEXT", []);
        assert!(result.is_err());
    }
}