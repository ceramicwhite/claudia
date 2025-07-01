#[cfg(test)]
mod tests {
    use super::super::*;
    use r2d2::{Pool, CustomizeConnection};
    use r2d2_sqlite::SqliteConnectionManager;
    use rusqlite::Connection;
    use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}};
    use std::thread;
    use std::time::{Duration, Instant};
    use tempfile::TempDir;
    use std::path::Path;

    // Helper function to create a temporary database path
    fn temp_db_path() -> (TempDir, String) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db").to_string_lossy().to_string();
        (temp_dir, db_path)
    }

    // Helper function to create a test pool with custom configuration
    fn create_test_pool_with_config<P: AsRef<Path>>(
        path: P,
        max_size: u32,
        min_idle: Option<u32>,
    ) -> Result<SqlitePool, AgentError> {
        let manager = SqliteConnectionManager::file(path)
            .with_init(|c| {
                c.execute_batch("PRAGMA foreign_keys = ON")?;
                Ok(())
            });
        
        Pool::builder()
            .max_size(max_size)
            .min_idle(min_idle)
            .connection_timeout(Duration::from_secs(1))
            .build(manager)
            .map_err(|e| AgentError::Other(format!("Failed to create connection pool: {}", e)))
    }

    // Custom connection customizer for testing
    #[derive(Debug)]
    struct TestConnectionCustomizer {
        counter: Arc<AtomicUsize>,
    }

    impl CustomizeConnection<Connection, rusqlite::Error> for TestConnectionCustomizer {
        fn on_acquire(&self, _conn: &mut Connection) -> Result<(), rusqlite::Error> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn test_pool_initialization() {
        let (_temp_dir, db_path) = temp_db_path();
        
        let pool = create_pool(&db_path);
        assert!(pool.is_ok(), "Pool creation should succeed");
        
        let pool = pool.unwrap();
        assert_eq!(pool.max_size(), 10, "Max pool size should be 10");
        assert_eq!(pool.state().connections, 1, "Should have 1 connection initially");
    }

    #[test]
    fn test_pool_configuration() {
        let (_temp_dir, db_path) = temp_db_path();
        
        // Test custom configuration
        let pool = create_test_pool_with_config(&db_path, 5, Some(2));
        assert!(pool.is_ok(), "Pool creation with custom config should succeed");
        
        let pool = pool.unwrap();
        assert_eq!(pool.max_size(), 5, "Max pool size should be 5");
        
        // Force creation of idle connections
        let _conn1 = pool.get().unwrap();
        let _conn2 = pool.get().unwrap();
        drop(_conn1);
        drop(_conn2);
        
        // Give the pool time to adjust
        thread::sleep(Duration::from_millis(100));
        
        let state = pool.state();
        assert!(state.idle_connections >= 2, "Should maintain at least 2 idle connections");
    }

    #[test]
    fn test_connection_acquisition_and_release() {
        let (_temp_dir, db_path) = temp_db_path();
        let pool = create_pool(&db_path).unwrap();
        
        // Acquire connection
        let conn = pool.get();
        assert!(conn.is_ok(), "Should successfully acquire connection");
        
        let initial_state = pool.state();
        assert_eq!(initial_state.connections, 1, "Should have 1 connection in use");
        
        // Release connection
        drop(conn);
        
        // Give the pool time to reclaim the connection
        thread::sleep(Duration::from_millis(50));
        
        let final_state = pool.state();
        assert_eq!(final_state.idle_connections, 1, "Connection should be returned to idle pool");
    }

    #[test]
    fn test_pool_exhaustion_handling() {
        let (_temp_dir, db_path) = temp_db_path();
        let pool = create_test_pool_with_config(&db_path, 2, None).unwrap();
        
        // Acquire all connections
        let conn1 = pool.get().unwrap();
        let conn2 = pool.get().unwrap();
        
        // Try to acquire one more - should timeout
        let start = Instant::now();
        let result = pool.get();
        let elapsed = start.elapsed();
        
        assert!(result.is_err(), "Should fail to acquire connection when pool is exhausted");
        assert!(elapsed >= Duration::from_secs(1), "Should timeout after 1 second");
        
        // Release one connection
        drop(conn1);
        
        // Now we should be able to acquire again
        let conn3 = pool.get();
        assert!(conn3.is_ok(), "Should acquire connection after one is released");
    }

    #[test]
    fn test_concurrent_access_patterns() {
        let (_temp_dir, db_path) = temp_db_path();
        let pool = Arc::new(create_pool(&db_path).unwrap());
        let barrier = Arc::new(std::sync::Barrier::new(10));
        let success_count = Arc::new(AtomicUsize::new(0));
        
        let handles: Vec<_> = (0..10).map(|i| {
            let pool = Arc::clone(&pool);
            let barrier = Arc::clone(&barrier);
            let success_count = Arc::clone(&success_count);
            
            thread::spawn(move || {
                barrier.wait();
                
                // Each thread performs multiple operations
                for j in 0..5 {
                    match pool.get() {
                        Ok(conn) => {
                            // Simulate some work
                            let result: Result<i32, _> = conn.query_row(
                                "SELECT ?1 + ?2",
                                &[&i, &j],
                                |row| row.get(0)
                            );
                            
                            if result.is_ok() {
                                success_count.fetch_add(1, Ordering::SeqCst);
                            }
                            
                            // Hold connection briefly
                            thread::sleep(Duration::from_millis(10));
                        }
                        Err(_) => {
                            // Connection acquisition failed
                        }
                    }
                }
            })
        }).collect();
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        let total_successes = success_count.load(Ordering::SeqCst);
        assert!(total_successes > 0, "At least some operations should succeed");
        assert!(total_successes <= 50, "No more than 50 operations should succeed");
    }

    #[test]
    fn test_schema_initialization_fresh() {
        let (_temp_dir, db_path) = temp_db_path();
        let pool = create_pool(&db_path).unwrap();
        
        let result = init_pool_db(&pool);
        assert!(result.is_ok(), "Schema initialization should succeed");
        
        // Verify tables were created
        let conn = pool.get().unwrap();
        let tables: Vec<String> = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        
        assert!(tables.contains(&"agents".to_string()), "agents table should exist");
        assert!(tables.contains(&"agent_runs".to_string()), "agent_runs table should exist");
        assert!(tables.contains(&"jsonl_outputs".to_string()), "jsonl_outputs table should exist");
        assert!(tables.contains(&"sandbox_violations".to_string()), "sandbox_violations table should exist");
        assert!(tables.contains(&"app_settings".to_string()), "app_settings table should exist");
    }

    #[test]
    fn test_schema_initialization_idempotent() {
        let (_temp_dir, db_path) = temp_db_path();
        let pool = create_pool(&db_path).unwrap();
        
        // Initialize schema twice
        let result1 = init_pool_db(&pool);
        assert!(result1.is_ok(), "First schema initialization should succeed");
        
        let result2 = init_pool_db(&pool);
        assert!(result2.is_ok(), "Second schema initialization should succeed (idempotent)");
        
        // Verify no duplicate tables or issues
        let conn = pool.get().unwrap();
        let table_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='agents'",
            [],
            |row| row.get(0)
        ).unwrap();
        
        assert_eq!(table_count, 1, "Should have exactly one agents table");
    }

    #[test]
    fn test_schema_migration_handling() {
        let (_temp_dir, db_path) = temp_db_path();
        let pool = create_pool(&db_path).unwrap();
        
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
        drop(conn);
        
        // Run schema initialization - should add missing columns
        let result = init_pool_db(&pool);
        assert!(result.is_ok(), "Schema migration should succeed");
        
        // Verify new columns were added
        let conn = pool.get().unwrap();
        let columns: Vec<String> = conn.prepare("PRAGMA table_info(agent_runs)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        
        assert!(columns.contains(&"usage_limit_reset_time".to_string()), "usage_limit_reset_time column should exist");
        assert!(columns.contains(&"auto_resume_enabled".to_string()), "auto_resume_enabled column should exist");
        assert!(columns.contains(&"resume_count".to_string()), "resume_count column should exist");
        assert!(columns.contains(&"parent_run_id".to_string()), "parent_run_id column should exist");
    }

    #[test]
    fn test_foreign_keys_enabled() {
        let (_temp_dir, db_path) = temp_db_path();
        let pool = create_pool(&db_path).unwrap();
        
        let conn = pool.get().unwrap();
        let fk_enabled: i32 = conn.query_row(
            "PRAGMA foreign_keys",
            [],
            |row| row.get(0)
        ).unwrap();
        
        assert_eq!(fk_enabled, 1, "Foreign keys should be enabled");
    }

    #[test]
    fn test_connection_reuse_efficiency() {
        let (_temp_dir, db_path) = temp_db_path();
        let counter = Arc::new(AtomicUsize::new(0));
        
        // Create pool with connection customizer to track acquisitions
        let manager = SqliteConnectionManager::file(&db_path)
            .with_init(|c| {
                c.execute_batch("PRAGMA foreign_keys = ON")?;
                Ok(())
            });
        
        let pool = Pool::builder()
            .max_size(5)
            .min_idle(Some(2))
            .connection_customizer(Box::new(TestConnectionCustomizer {
                counter: Arc::clone(&counter),
            }))
            .build(manager)
            .unwrap();
        
        // Perform multiple operations
        for _ in 0..10 {
            let conn = pool.get().unwrap();
            conn.execute("SELECT 1", []).unwrap();
            drop(conn);
        }
        
        let acquisitions = counter.load(Ordering::SeqCst);
        assert!(acquisitions <= 5, "Should reuse connections, not create new ones for each operation");
    }

    #[test]
    fn test_pool_sizing_optimization() {
        let (_temp_dir, db_path) = temp_db_path();
        
        // Test various pool sizes
        let sizes = vec![1, 5, 10, 20];
        
        for size in sizes {
            let pool = create_test_pool_with_config(&db_path, size, Some(size / 2)).unwrap();
            assert_eq!(pool.max_size(), size, "Pool max size should be {}", size);
            
            // Verify pool can handle up to max_size connections
            let mut conns = vec![];
            for i in 0..size {
                match pool.get() {
                    Ok(conn) => conns.push(conn),
                    Err(_) => panic!("Should be able to acquire {} connections", i + 1),
                }
            }
            
            let state = pool.state();
            assert_eq!(state.connections as u32, size, "Should have {} connections", size);
        }
    }

    #[test]
    fn test_timeout_behavior() {
        let (_temp_dir, db_path) = temp_db_path();
        let pool = create_test_pool_with_config(&db_path, 1, None).unwrap();
        
        // Hold the only connection
        let _conn = pool.get().unwrap();
        
        // Measure timeout on second acquisition
        let start = Instant::now();
        let result = pool.get();
        let elapsed = start.elapsed();
        
        assert!(result.is_err(), "Should timeout when no connections available");
        assert!(elapsed >= Duration::from_secs(1), "Should respect timeout duration");
        assert!(elapsed < Duration::from_secs(2), "Should not wait too long");
    }

    #[test]
    fn test_database_connection_failure() {
        // Test with invalid path
        let pool = create_pool("/invalid/path/to/database.db");
        
        // Pool creation might succeed, but getting connection should fail
        if let Ok(pool) = pool {
            let conn = pool.get();
            assert!(conn.is_err(), "Should fail to get connection with invalid path");
        }
    }

    #[test]
    fn test_pool_configuration_errors() {
        let (_temp_dir, db_path) = temp_db_path();
        
        // Test with invalid configuration (0 max size)
        let result = create_test_pool_with_config(&db_path, 0, None);
        assert!(result.is_err(), "Should fail with 0 max size");
    }

    #[test]
    fn test_schema_initialization_failure_recovery() {
        let (_temp_dir, db_path) = temp_db_path();
        let pool = create_pool(&db_path).unwrap();
        
        // Corrupt the database by creating an invalid table
        let conn = pool.get().unwrap();
        conn.execute("CREATE TABLE agents (invalid_schema)", []).unwrap();
        drop(conn);
        
        // Try to initialize schema - should handle existing invalid table
        let result = init_pool_db(&pool);
        
        // The initialization might succeed or fail depending on exact error,
        // but it shouldn't panic
        match result {
            Ok(_) => {
                // If it succeeded, verify the schema is correct
                let conn = pool.get().unwrap();
                let columns: Vec<String> = conn.prepare("PRAGMA table_info(agents)")
                    .unwrap()
                    .query_map([], |row| row.get::<_, String>(1))
                    .unwrap()
                    .filter_map(Result::ok)
                    .collect();
                
                assert!(columns.contains(&"id".to_string()), "Should have proper schema");
            }
            Err(_) => {
                // Error is acceptable - at least it didn't panic
            }
        }
    }

    #[test]
    fn test_connection_leak_prevention() {
        let (_temp_dir, db_path) = temp_db_path();
        let pool = create_test_pool_with_config(&db_path, 3, None).unwrap();
        
        // Simulate potential leak by acquiring connections without explicit drop
        {
            let _conn1 = pool.get().unwrap();
            let _conn2 = pool.get().unwrap();
            // Connections should be automatically returned when going out of scope
        }
        
        thread::sleep(Duration::from_millis(100));
        
        // Verify connections were returned
        let state = pool.state();
        assert_eq!(state.idle_connections, 2, "Connections should be returned to pool");
        
        // Should be able to acquire connections again
        let conn = pool.get();
        assert!(conn.is_ok(), "Should be able to acquire connection after implicit drops");
    }

    #[test]
    fn test_transaction_handling() {
        let (_temp_dir, db_path) = temp_db_path();
        let pool = create_pool(&db_path).unwrap();
        init_pool_db(&pool).unwrap();
        
        // Test successful transaction
        {
            let conn = pool.get().unwrap();
            let tx = conn.unchecked_transaction().unwrap();
            
            tx.execute(
                "INSERT INTO app_settings (key, value) VALUES (?1, ?2)",
                &["test_key", "test_value"],
            ).unwrap();
            
            tx.commit().unwrap();
        }
        
        // Verify data was committed
        {
            let conn = pool.get().unwrap();
            let value: String = conn.query_row(
                "SELECT value FROM app_settings WHERE key = ?1",
                &["test_key"],
                |row| row.get(0),
            ).unwrap();
            
            assert_eq!(value, "test_value", "Transaction should have committed");
        }
        
        // Test rolled back transaction
        {
            let conn = pool.get().unwrap();
            let tx = conn.unchecked_transaction().unwrap();
            
            tx.execute(
                "INSERT INTO app_settings (key, value) VALUES (?1, ?2)",
                &["rollback_key", "rollback_value"],
            ).unwrap();
            
            // Explicitly rollback
            tx.rollback().unwrap();
        }
        
        // Verify data was not committed
        {
            let conn = pool.get().unwrap();
            let count: i32 = conn.query_row(
                "SELECT COUNT(*) FROM app_settings WHERE key = ?1",
                &["rollback_key"],
                |row| row.get(0),
            ).unwrap();
            
            assert_eq!(count, 0, "Transaction should have been rolled back");
        }
    }

    #[test]
    fn test_long_running_connection_behavior() {
        let (_temp_dir, db_path) = temp_db_path();
        let pool = create_pool(&db_path).unwrap();
        init_pool_db(&pool).unwrap();
        
        let start = Instant::now();
        
        // Hold connection for extended period
        let conn = pool.get().unwrap();
        
        // Perform operations over time
        for i in 0..5 {
            thread::sleep(Duration::from_millis(100));
            
            let result: Result<i32, _> = conn.query_row(
                "SELECT ?1",
                &[&i],
                |row| row.get(0)
            );
            
            assert!(result.is_ok(), "Long-running connection should remain valid");
        }
        
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(500), "Connection held for extended period");
        
        // Connection should still be valid
        let test_result = conn.execute("SELECT 1", []);
        assert!(test_result.is_ok(), "Connection should remain valid after extended use");
    }

    #[test]
    fn test_pool_cleanup_on_drop() {
        let (_temp_dir, db_path) = temp_db_path();
        
        // Create pool in a scope
        {
            let pool = create_pool(&db_path).unwrap();
            let _conn = pool.get().unwrap();
            // Pool and connection will be dropped here
        }
        
        // Create new pool to same database - should work
        let pool2 = create_pool(&db_path);
        assert!(pool2.is_ok(), "Should be able to create new pool after previous one is dropped");
    }

    #[test]
    fn test_concurrent_schema_initialization() {
        let (_temp_dir, db_path) = temp_db_path();
        let pool = Arc::new(create_pool(&db_path).unwrap());
        let barrier = Arc::new(std::sync::Barrier::new(3));
        
        let handles: Vec<_> = (0..3).map(|_| {
            let pool = Arc::clone(&pool);
            let barrier = Arc::clone(&barrier);
            
            thread::spawn(move || {
                barrier.wait();
                init_pool_db(&pool)
            })
        }).collect();
        
        let results: Vec<_> = handles.into_iter()
            .map(|h| h.join().unwrap())
            .collect();
        
        // All should succeed (idempotent operation)
        for result in results {
            assert!(result.is_ok(), "Concurrent schema initialization should succeed");
        }
        
        // Verify schema is correct
        let conn = pool.get().unwrap();
        let table_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
            [],
            |row| row.get(0),
        ).unwrap();
        
        assert!(table_count > 0, "Tables should be created");
    }

    #[test]
    fn test_stress_test_pool_under_load() {
        let (_temp_dir, db_path) = temp_db_path();
        let pool = Arc::new(create_pool(&db_path).unwrap());
        init_pool_db(&pool).unwrap();
        
        let operation_count = Arc::new(AtomicUsize::new(0));
        let error_count = Arc::new(AtomicUsize::new(0));
        let duration = Duration::from_secs(2);
        let start = Instant::now();
        
        let handles: Vec<_> = (0..20).map(|thread_id| {
            let pool = Arc::clone(&pool);
            let operation_count = Arc::clone(&operation_count);
            let error_count = Arc::clone(&error_count);
            
            thread::spawn(move || {
                while start.elapsed() < duration {
                    match pool.get() {
                        Ok(conn) => {
                            // Perform random operations
                            let ops = vec![
                                || conn.execute("SELECT 1", []).map(|_| ()),
                                || conn.query_row("SELECT COUNT(*) FROM agents", [], |row| row.get::<_, i32>(0)).map(|_| ()),
                                || {
                                    let tx = conn.unchecked_transaction()?;
                                    tx.execute("INSERT INTO app_settings (key, value) VALUES (?1, ?2)", 
                                              &[&format!("key_{}", thread_id), "value"])?;
                                    tx.commit()
                                },
                            ];
                            
                            for op in ops {
                                match op() {
                                    Ok(_) => operation_count.fetch_add(1, Ordering::SeqCst),
                                    Err(_) => error_count.fetch_add(1, Ordering::SeqCst),
                                };
                            }
                        }
                        Err(_) => {
                            error_count.fetch_add(1, Ordering::SeqCst);
                            thread::sleep(Duration::from_millis(10));
                        }
                    }
                }
            })
        }).collect();
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
        
        let total_operations = operation_count.load(Ordering::SeqCst);
        let total_errors = error_count.load(Ordering::SeqCst);
        
        println!("Stress test completed: {} operations, {} errors", total_operations, total_errors);
        
        assert!(total_operations > 100, "Should complete many operations under load");
        assert!(total_errors < total_operations / 10, "Error rate should be less than 10%");
    }
}