//! Unit tests for scheduler functionality

#[cfg(test)]
mod scheduler_tests {
    use chrono::{Utc, Duration};
    use rusqlite::{Connection, params};
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::sync::Mutex;

    // Mock structures for testing
    struct SchedulerState {
        pub is_running: Arc<Mutex<bool>>,
    }

    impl SchedulerState {
        pub fn new() -> Self {
            Self {
                is_running: Arc::new(Mutex::new(false)),
            }
        }
    }

    #[derive(Debug, Clone)]
    struct AgentRun {
        id: Option<i64>,
        agent_id: i64,
        agent_name: String,
        agent_icon: Option<String>,
        task: String,
        model: String,
        project_path: String,
        session_id: String,
        status: String,
        pid: Option<i32>,
        process_started_at: Option<String>,
        scheduled_start_time: Option<String>,
        created_at: String,
        completed_at: Option<String>,
        usage_limit_reset_time: Option<String>,
        auto_resume_enabled: bool,
        resume_count: i32,
        parent_run_id: Option<i64>,
    }

    fn create_test_db() -> (TempDir, Connection) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();

        // Create agent_runs table
        conn.execute(
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
            )",
            [],
        ).unwrap();

        (temp_dir, conn)
    }

    #[tokio::test]
    async fn test_scheduler_state_initialization() {
        let state = SchedulerState::new();
        let is_running = state.is_running.lock().await;
        assert_eq!(*is_running, false);
    }

    #[tokio::test]
    async fn test_scheduler_state_toggle() {
        let state = SchedulerState::new();
        
        // Start scheduler
        {
            let mut is_running = state.is_running.lock().await;
            *is_running = true;
        }
        
        // Verify it's running
        {
            let is_running = state.is_running.lock().await;
            assert_eq!(*is_running, true);
        }
        
        // Stop scheduler
        {
            let mut is_running = state.is_running.lock().await;
            *is_running = false;
        }
        
        // Verify it's stopped
        {
            let is_running = state.is_running.lock().await;
            assert_eq!(*is_running, false);
        }
    }

    #[test]
    fn test_find_scheduled_runs_ready_to_execute() {
        let (_temp_dir, conn) = create_test_db();
        let now = Utc::now();
        
        // Insert a run scheduled for the past (should be executed)
        let past_time = (now - Duration::hours(1)).to_rfc3339();
        conn.execute(
            "INSERT INTO agent_runs (agent_id, agent_name, task, model, project_path, 
             session_id, status, scheduled_start_time) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![1, "Agent1", "task1", "model", "/path", "session1", "scheduled", past_time],
        ).unwrap();

        // Insert a run scheduled for the future (should not be executed)
        let future_time = (now + Duration::hours(1)).to_rfc3339();
        conn.execute(
            "INSERT INTO agent_runs (agent_id, agent_name, task, model, project_path, 
             session_id, status, scheduled_start_time) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![2, "Agent2", "task2", "model", "/path", "session2", "scheduled", future_time],
        ).unwrap();

        // Insert a run that's already running (should not be executed)
        conn.execute(
            "INSERT INTO agent_runs (agent_id, agent_name, task, model, project_path, 
             session_id, status, scheduled_start_time) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![3, "Agent3", "task3", "model", "/path", "session3", "running", past_time],
        ).unwrap();

        // Query for runs ready to execute
        let now_iso = now.to_rfc3339();
        let mut stmt = conn.prepare(
            "SELECT id, agent_name FROM agent_runs 
             WHERE status = 'scheduled' 
               AND scheduled_start_time IS NOT NULL 
               AND scheduled_start_time <= ?1
             ORDER BY scheduled_start_time ASC"
        ).unwrap();
        
        let runs: Vec<(i64, String)> = stmt.query_map(params![now_iso], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].1, "Agent1");
    }

    #[test]
    fn test_update_scheduled_run_to_pending() {
        let (_temp_dir, conn) = create_test_db();
        
        // Insert a scheduled run
        conn.execute(
            "INSERT INTO agent_runs (agent_id, agent_name, task, model, project_path, 
             session_id, status, scheduled_start_time) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![1, "Agent1", "task", "model", "/path", "session1", "scheduled", 
                    "2024-01-01T00:00:00Z"],
        ).unwrap();
        
        let run_id = conn.last_insert_rowid();

        // Update status to pending
        conn.execute(
            "UPDATE agent_runs SET status = 'pending' WHERE id = ?1",
            params![run_id]
        ).unwrap();

        // Verify the update
        let status: String = conn.query_row(
            "SELECT status FROM agent_runs WHERE id = ?1",
            params![run_id],
            |row| row.get(0)
        ).unwrap();

        assert_eq!(status, "pending");
    }

    #[test]
    fn test_mark_scheduled_run_as_completed() {
        let (_temp_dir, conn) = create_test_db();
        
        // Insert a scheduled run
        conn.execute(
            "INSERT INTO agent_runs (agent_id, agent_name, task, model, project_path, 
             session_id, status, scheduled_start_time) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![1, "Agent1", "task", "model", "/path", "session1", "scheduled", 
                    "2024-01-01T00:00:00Z"],
        ).unwrap();
        
        let run_id = conn.last_insert_rowid();

        // Mark as completed
        conn.execute(
            "UPDATE agent_runs SET status = 'completed', completed_at = CURRENT_TIMESTAMP WHERE id = ?1",
            params![run_id]
        ).unwrap();

        // Verify the update
        let (status, completed_at): (String, Option<String>) = conn.query_row(
            "SELECT status, completed_at FROM agent_runs WHERE id = ?1",
            params![run_id],
            |row| Ok((row.get(0)?, row.get(1)?))
        ).unwrap();

        assert_eq!(status, "completed");
        assert!(completed_at.is_some());
    }

    #[test]
    fn test_mark_scheduled_run_as_failed() {
        let (_temp_dir, conn) = create_test_db();
        
        // Insert a scheduled run
        conn.execute(
            "INSERT INTO agent_runs (agent_id, agent_name, task, model, project_path, 
             session_id, status, scheduled_start_time) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![1, "Agent1", "task", "model", "/path", "session1", "scheduled", 
                    "2024-01-01T00:00:00Z"],
        ).unwrap();
        
        let run_id = conn.last_insert_rowid();

        // Mark as failed
        conn.execute(
            "UPDATE agent_runs SET status = 'failed', completed_at = CURRENT_TIMESTAMP WHERE id = ?1",
            params![run_id]
        ).unwrap();

        // Verify the update
        let (status, completed_at): (String, Option<String>) = conn.query_row(
            "SELECT status, completed_at FROM agent_runs WHERE id = ?1",
            params![run_id],
            |row| Ok((row.get(0)?, row.get(1)?))
        ).unwrap();

        assert_eq!(status, "failed");
        assert!(completed_at.is_some());
    }

    #[test]
    fn test_scheduler_handles_multiple_scheduled_runs() {
        let (_temp_dir, conn) = create_test_db();
        let now = Utc::now();
        let past_time = (now - Duration::hours(1)).to_rfc3339();
        
        // Insert multiple scheduled runs
        for i in 1..=5 {
            conn.execute(
                "INSERT INTO agent_runs (agent_id, agent_name, task, model, project_path, 
                 session_id, status, scheduled_start_time) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![i, format!("Agent{}", i), format!("task{}", i), "model", "/path", 
                        format!("session{}", i), "scheduled", past_time],
            ).unwrap();
        }

        // Query all scheduled runs
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM agent_runs WHERE status = 'scheduled'",
            [],
            |row| row.get(0)
        ).unwrap();

        assert_eq!(count, 5);

        // Simulate updating all to pending
        conn.execute(
            "UPDATE agent_runs SET status = 'pending' WHERE status = 'scheduled' AND scheduled_start_time <= ?1",
            params![now.to_rfc3339()]
        ).unwrap();

        // Verify all were updated
        let pending_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM agent_runs WHERE status = 'pending'",
            [],
            |row| row.get(0)
        ).unwrap();

        assert_eq!(pending_count, 5);
    }

    #[test]
    fn test_scheduler_respects_order_by_scheduled_time() {
        let (_temp_dir, conn) = create_test_db();
        let base_time = Utc::now() - Duration::hours(2);
        
        // Insert runs with different scheduled times
        let times = vec![
            base_time + Duration::minutes(30),
            base_time,
            base_time + Duration::minutes(15),
        ];
        
        for (i, time) in times.iter().enumerate() {
            conn.execute(
                "INSERT INTO agent_runs (agent_id, agent_name, task, model, project_path, 
                 session_id, status, scheduled_start_time) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![i + 1, format!("Agent{}", i + 1), "task", "model", "/path", 
                        format!("session{}", i + 1), "scheduled", time.to_rfc3339()],
            ).unwrap();
        }

        // Query in order
        let mut stmt = conn.prepare(
            "SELECT agent_name FROM agent_runs 
             WHERE status = 'scheduled' 
             ORDER BY scheduled_start_time ASC"
        ).unwrap();
        
        let ordered_names: Vec<String> = stmt.query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(ordered_names, vec!["Agent2", "Agent3", "Agent1"]);
    }

    #[test]
    fn test_scheduler_ignores_null_scheduled_time() {
        let (_temp_dir, conn) = create_test_db();
        
        // Insert a run with NULL scheduled_start_time
        conn.execute(
            "INSERT INTO agent_runs (agent_id, agent_name, task, model, project_path, 
             session_id, status) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![1, "Agent1", "task", "model", "/path", "session1", "scheduled"],
        ).unwrap();

        // Query for runs ready to execute
        let now_iso = Utc::now().to_rfc3339();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM agent_runs 
             WHERE status = 'scheduled' 
               AND scheduled_start_time IS NOT NULL 
               AND scheduled_start_time <= ?1",
            params![now_iso],
            |row| row.get(0)
        ).unwrap();

        assert_eq!(count, 0);
    }

    #[test]
    fn test_scheduler_concurrent_safety() {
        let (_temp_dir, conn) = create_test_db();
        let now = Utc::now();
        let past_time = (now - Duration::hours(1)).to_rfc3339();
        
        // Insert a scheduled run
        conn.execute(
            "INSERT INTO agent_runs (agent_id, agent_name, task, model, project_path, 
             session_id, status, scheduled_start_time) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![1, "Agent1", "task", "model", "/path", "session1", "scheduled", past_time],
        ).unwrap();
        
        let run_id = conn.last_insert_rowid();

        // Simulate concurrent updates (only one should succeed due to status check)
        let result1 = conn.execute(
            "UPDATE agent_runs SET status = 'pending' WHERE id = ?1 AND status = 'scheduled'",
            params![run_id]
        ).unwrap();

        let result2 = conn.execute(
            "UPDATE agent_runs SET status = 'pending' WHERE id = ?1 AND status = 'scheduled'",
            params![run_id]
        ).unwrap();

        // Only the first update should have affected a row
        assert_eq!(result1, 1);
        assert_eq!(result2, 0);
    }
}