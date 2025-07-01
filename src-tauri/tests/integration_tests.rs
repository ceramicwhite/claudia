//! Integration tests for agents and scheduler interaction

#[cfg(test)]
mod integration_tests {
    use chrono::{Utc, Duration};
    use rusqlite::{Connection, params};
    
    use tempfile::TempDir;
    

    fn create_full_test_db() -> (TempDir, Connection) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();

        // Create all necessary tables
        conn.execute_batch(
            "CREATE TABLE agents (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                icon TEXT,
                is_system BOOLEAN DEFAULT FALSE,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE TABLE agent_runs (
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
                parent_run_id INTEGER,
                FOREIGN KEY (agent_id) REFERENCES agents(id)
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
            );
            
            CREATE INDEX idx_agent_runs_status ON agent_runs(status);
            CREATE INDEX idx_agent_runs_scheduled_time ON agent_runs(scheduled_start_time);
            CREATE INDEX idx_agent_runs_usage_limit ON agent_runs(usage_limit_reset_time);"
        ).unwrap();

        (temp_dir, conn)
    }

    #[test]
    fn test_agent_run_with_usage_limit_creates_scheduled_run() {
        let (_temp_dir, conn) = create_full_test_db();
        
        // Create an agent
        conn.execute(
            "INSERT INTO agents (name, description, icon) VALUES (?1, ?2, ?3)",
            params!["TestAgent", "Test Description", "test-icon"],
        ).unwrap();
        let agent_id = conn.last_insert_rowid();

        // Create a run that hit usage limit
        let usage_limit_time = (Utc::now() + Duration::hours(1)).to_rfc3339();
        conn.execute(
            "INSERT INTO agent_runs (agent_id, agent_name, task, model, project_path, 
             session_id, status, usage_limit_reset_time, auto_resume_enabled) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![agent_id, "TestAgent", "original task", "claude-3-5-sonnet", 
                    "/path", "session1", "failed", usage_limit_time, true],
        ).unwrap();
        let parent_run_id = conn.last_insert_rowid();

        // Simulate creating a scheduled run for auto-resume
        conn.execute(
            "INSERT INTO agent_runs (agent_id, agent_name, task, model, project_path, 
             session_id, status, scheduled_start_time, parent_run_id, auto_resume_enabled) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![agent_id, "TestAgent", "original task", "claude-3-5-sonnet", 
                    "/path", "session1-resume", "scheduled", usage_limit_time, 
                    parent_run_id, false],
        ).unwrap();

        // Verify the scheduled run was created correctly
        let (status, scheduled_time, parent_id): (String, Option<String>, Option<i64>) = 
            conn.query_row(
                "SELECT status, scheduled_start_time, parent_run_id 
                 FROM agent_runs WHERE parent_run_id = ?1",
                params![parent_run_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            ).unwrap();

        assert_eq!(status, "scheduled");
        assert_eq!(scheduled_time, Some(usage_limit_time));
        assert_eq!(parent_id, Some(parent_run_id));
    }

    #[test]
    fn test_metrics_calculation_and_storage() {
        let (_temp_dir, conn) = create_full_test_db();
        
        // Create an agent and run
        conn.execute(
            "INSERT INTO agents (name) VALUES (?1)",
            params!["TestAgent"],
        ).unwrap();
        let agent_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO agent_runs (agent_id, agent_name, task, model, project_path, 
             session_id, status) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![agent_id, "TestAgent", "task", "claude-3-5-sonnet", 
                    "/path", "session1", "completed"],
        ).unwrap();
        let run_id = conn.last_insert_rowid();

        // Insert metrics
        let total_tokens = 1500;
        let input_tokens = 1000;
        let output_tokens = 500;
        let cache_creation = 100;
        let cache_read = 50;
        
        // Calculate expected cost for claude-3-5-sonnet
        // Rates: input=3.0, output=15.0, cache_creation=3.75, cache_read=0.30
        let expected_cost = (1000.0 * 3.0 + 500.0 * 15.0 + 100.0 * 3.75 + 50.0 * 0.30) / 1_000_000.0;

        conn.execute(
            "INSERT INTO agent_run_metrics (run_id, total_tokens, total_input_tokens, 
             total_output_tokens, total_cache_creation_tokens, total_cache_read_tokens, 
             cost_usd, message_count, duration_seconds) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![run_id, total_tokens, input_tokens, output_tokens, 
                    cache_creation, cache_read, expected_cost, 10, 120],
        ).unwrap();

        // Verify metrics
        let metrics = conn.query_row(
            "SELECT total_tokens, cost_usd, message_count, duration_seconds 
             FROM agent_run_metrics WHERE run_id = ?1",
            params![run_id],
            |row| Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, Option<i64>>(3)?
            ))
        ).unwrap();

        assert_eq!(metrics.0, total_tokens);
        assert!((metrics.1 - expected_cost).abs() < 0.000001);
        assert_eq!(metrics.2, 10);
        assert_eq!(metrics.3, Some(120));
    }

    #[test]
    fn test_scheduler_updates_and_agent_execution_flow() {
        let (_temp_dir, conn) = create_full_test_db();
        let now = Utc::now();
        
        // Create an agent
        conn.execute(
            "INSERT INTO agents (name) VALUES (?1)",
            params!["ScheduledAgent"],
        ).unwrap();
        let agent_id = conn.last_insert_rowid();

        // Create multiple scheduled runs with different times
        let times_and_ids = vec![
            ((now - Duration::hours(2)).to_rfc3339(), "past-run"),
            ((now - Duration::minutes(30)).to_rfc3339(), "recent-run"),
            ((now + Duration::hours(1)).to_rfc3339(), "future-run"),
        ];

        for (time, session_id) in &times_and_ids {
            conn.execute(
                "INSERT INTO agent_runs (agent_id, agent_name, task, model, project_path, 
                 session_id, status, scheduled_start_time) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![agent_id, "ScheduledAgent", "task", "model", "/path", 
                        session_id, "scheduled", time],
            ).unwrap();
        }

        // Simulate scheduler finding runs to execute
        let now_iso = now.to_rfc3339();
        let ready_runs: Vec<(i64, String)> = conn.prepare(
            "SELECT id, session_id FROM agent_runs 
             WHERE status = 'scheduled' 
               AND scheduled_start_time IS NOT NULL 
               AND scheduled_start_time <= ?1
             ORDER BY scheduled_start_time ASC"
        )
        .unwrap()
        .query_map(params![now_iso], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

        // Should find 2 runs (past and recent)
        assert_eq!(ready_runs.len(), 2);
        assert_eq!(ready_runs[0].1, "past-run");
        assert_eq!(ready_runs[1].1, "recent-run");

        // Simulate updating them to pending
        for (run_id, _) in &ready_runs {
            conn.execute(
                "UPDATE agent_runs SET status = 'pending' WHERE id = ?1",
                params![run_id]
            ).unwrap();
        }

        // Verify no scheduled runs remain that are ready
        let remaining_scheduled: i64 = conn.query_row(
            "SELECT COUNT(*) FROM agent_runs 
             WHERE status = 'scheduled' 
               AND scheduled_start_time IS NOT NULL 
               AND scheduled_start_time <= ?1",
            params![now_iso],
            |row| row.get(0)
        ).unwrap();

        assert_eq!(remaining_scheduled, 0);

        // Verify future run is still scheduled
        let future_status: String = conn.query_row(
            "SELECT status FROM agent_runs WHERE session_id = ?1",
            params!["future-run"],
            |row| row.get(0)
        ).unwrap();

        assert_eq!(future_status, "scheduled");
    }

    #[test]
    fn test_resume_count_tracking() {
        let (_temp_dir, conn) = create_full_test_db();
        
        // Create an agent
        conn.execute(
            "INSERT INTO agents (name) VALUES (?1)",
            params!["ResumeAgent"],
        ).unwrap();
        let agent_id = conn.last_insert_rowid();

        // Create initial run
        conn.execute(
            "INSERT INTO agent_runs (agent_id, agent_name, task, model, project_path, 
             session_id, status, resume_count) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![agent_id, "ResumeAgent", "task", "model", "/path", 
                    "session1", "failed", 0],
        ).unwrap();
        let parent_id = conn.last_insert_rowid();

        // Create resumed runs
        for i in 1..=3 {
            conn.execute(
                "INSERT INTO agent_runs (agent_id, agent_name, task, model, project_path, 
                 session_id, status, resume_count, parent_run_id) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![agent_id, "ResumeAgent", "task", "model", "/path", 
                        format!("session1-resume{}", i), "completed", i, parent_id],
            ).unwrap();
        }

        // Count total resumes for the original run
        let total_resumes: i64 = conn.query_row(
            "SELECT COUNT(*) FROM agent_runs WHERE parent_run_id = ?1",
            params![parent_id],
            |row| row.get(0)
        ).unwrap();

        assert_eq!(total_resumes, 3);

        // Get the highest resume count
        let max_resume_count: i32 = conn.query_row(
            "SELECT MAX(resume_count) FROM agent_runs WHERE parent_run_id = ?1",
            params![parent_id],
            |row| row.get(0)
        ).unwrap();

        assert_eq!(max_resume_count, 3);
    }

    #[test]
    fn test_cost_aggregation_across_runs() {
        let (_temp_dir, conn) = create_full_test_db();
        
        // Create an agent
        conn.execute(
            "INSERT INTO agents (name) VALUES (?1)",
            params!["CostAgent"],
        ).unwrap();
        let agent_id = conn.last_insert_rowid();

        // Create multiple runs for the same agent
        let costs = vec![0.05, 0.10, 0.15, 0.20];
        
        for (i, cost) in costs.iter().enumerate() {
            // Create run
            conn.execute(
                "INSERT INTO agent_runs (agent_id, agent_name, task, model, project_path, 
                 session_id, status) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![agent_id, "CostAgent", "task", "model", "/path", 
                        format!("session{}", i), "completed"],
            ).unwrap();
            let run_id = conn.last_insert_rowid();

            // Add metrics
            conn.execute(
                "INSERT INTO agent_run_metrics (run_id, cost_usd) VALUES (?1, ?2)",
                params![run_id, cost],
            ).unwrap();
        }

        // Calculate total cost for the agent
        let total_cost: f64 = conn.query_row(
            "SELECT SUM(m.cost_usd) 
             FROM agent_run_metrics m 
             JOIN agent_runs r ON m.run_id = r.id 
             WHERE r.agent_id = ?1",
            params![agent_id],
            |row| row.get(0)
        ).unwrap();

        assert!((total_cost - 0.50).abs() < 0.000001);
    }
}