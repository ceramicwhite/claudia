#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::agents::{
        pool::{create_pool, init_pool_db},
        repository::AgentRepository,
        service::AgentService,
        types::{AgentCreate, AgentCreateBuilder, AgentId},
    };
    use rusqlite::Connection;
    use std::sync::Arc;
    use tempfile::tempdir;

    fn setup_test_db() -> Arc<SqlitePool> {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = create_pool(db_path).unwrap();
        init_pool_db(&pool).unwrap();
        Arc::new(pool)
    }

    #[test]
    fn test_agent_create_builder() {
        let agent = AgentCreateBuilder::new("Test Agent", "icon", "System prompt")
            .default_task("Default task")
            .model("gpt-4")
            .sandbox_enabled(true)
            .enable_file_read(true)
            .enable_file_write(false)
            .enable_network(true)
            .build()
            .unwrap();

        assert_eq!(agent.name, "Test Agent");
        assert_eq!(agent.icon, "icon");
        assert_eq!(agent.system_prompt, "System prompt");
        assert_eq!(agent.default_task, Some("Default task".to_string()));
        assert_eq!(agent.model, Some("gpt-4".to_string()));
        assert_eq!(agent.sandbox_enabled, Some(true));
        assert_eq!(agent.enable_file_read, Some(true));
        assert_eq!(agent.enable_file_write, Some(false));
        assert_eq!(agent.enable_network, Some(true));
    }

    #[test]
    fn test_agent_create_builder_validation() {
        // Empty name should fail
        let result = AgentCreateBuilder::new("", "icon", "System prompt").build();
        assert!(result.is_err());

        // Empty icon should fail
        let result = AgentCreateBuilder::new("Test", "", "System prompt").build();
        assert!(result.is_err());

        // Empty system prompt should fail
        let result = AgentCreateBuilder::new("Test", "icon", "").build();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_repository_create_agent() {
        let pool = setup_test_db();
        let repo = AgentRepository::new(pool);

        let agent_create = AgentCreateBuilder::new("Test Agent", "🤖", "Test system prompt")
            .default_task("Test task")
            .model("claude-3")
            .build()
            .unwrap();

        let agent = repo.create_agent(agent_create).await.unwrap();

        assert_eq!(agent.name, "Test Agent");
        assert_eq!(agent.icon, "🤖");
        assert_eq!(agent.system_prompt, "Test system prompt");
        assert!(agent.id > 0);
    }

    #[tokio::test]
    async fn test_repository_list_agents() {
        let pool = setup_test_db();
        let repo = AgentRepository::new(pool);

        // Create a few agents
        for i in 1..=3 {
            let agent_create = AgentCreateBuilder::new(
                &format!("Agent {}", i),
                "🤖",
                &format!("System prompt {}", i),
            )
            .build()
            .unwrap();
            repo.create_agent(agent_create).await.unwrap();
        }

        let agents = repo.list_agents().await.unwrap();
        assert_eq!(agents.len(), 3);
    }

    #[tokio::test]
    async fn test_repository_get_agent() {
        let pool = setup_test_db();
        let repo = AgentRepository::new(pool);

        let agent_create = AgentCreateBuilder::new("Test Agent", "🤖", "Test prompt")
            .build()
            .unwrap();
        let created = repo.create_agent(agent_create).await.unwrap();

        let found = repo.get_agent(created.id).await.unwrap();
        assert_eq!(found.id, created.id);
        assert_eq!(found.name, created.name);
    }

    #[tokio::test]
    async fn test_repository_update_agent() {
        let pool = setup_test_db();
        let repo = AgentRepository::new(pool);

        let agent_create = AgentCreateBuilder::new("Original", "🤖", "Original prompt")
            .build()
            .unwrap();
        let created = repo.create_agent(agent_create).await.unwrap();

        let update = AgentCreateBuilder::new("Updated", "🚀", "Updated prompt")
            .model("gpt-4")
            .build()
            .unwrap();

        let updated = repo.update_agent(created.id, update).await.unwrap();
        assert_eq!(updated.name, "Updated");
        assert_eq!(updated.icon, "🚀");
        assert_eq!(updated.system_prompt, "Updated prompt");
        assert_eq!(updated.model, Some("gpt-4".to_string()));
    }

    #[tokio::test]
    async fn test_repository_delete_agent() {
        let pool = setup_test_db();
        let repo = AgentRepository::new(pool);

        let agent_create = AgentCreateBuilder::new("To Delete", "🗑️", "Delete me")
            .build()
            .unwrap();
        let created = repo.create_agent(agent_create).await.unwrap();

        repo.delete_agent(created.id).await.unwrap();

        let result = repo.get_agent(created.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_newtype_id_safety() {
        let agent_id = AgentId(123);
        let run_id = RunId(123);

        // These should be different types and not interchangeable
        assert_eq!(agent_id.0, 123);
        assert_eq!(run_id.0, 123);

        // But they are different types at compile time
        // This would fail to compile:
        // let _: AgentId = run_id; // Error: mismatched types
    }

    #[tokio::test]
    async fn test_error_handling() {
        let pool = setup_test_db();
        let repo = AgentRepository::new(pool);

        // Test not found error
        let result = repo.get_agent(99999).await;
        assert!(matches!(result, Err(AgentError::NotFound(_))));

        // Test validation error through builder
        let result = AgentCreateBuilder::new("", "icon", "prompt").build();
        assert!(matches!(result, Err(AgentError::Validation(_))));
    }
}