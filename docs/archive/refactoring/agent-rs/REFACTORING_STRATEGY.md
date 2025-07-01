# Rust Refactoring Strategy for Tauri Commands

## Overview

This document outlines modern Rust patterns and best practices for refactoring the agents.rs module based on 2024 research and current ecosystem standards.

## 1. Error Handling Migration Strategy

### Current State
- Using string-based error handling
- Lack of structured error types
- Limited error context

### Migration to thiserror/anyhow

Based on 2024 best practices:

**Use `thiserror` for:**
- Command-level errors that need specific handling
- Library-like modules where callers need error details
- Custom error types with structured information

**Use `anyhow` for:**
- Application-level error propagation
- Quick prototyping (though consider migrating to explicit types later)
- Infrastructure errors where details aren't critical

**Implementation Pattern:**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Agent not found: {id}")]
    NotFound { id: String },
    
    #[error("Process spawn failed")]
    SpawnFailed(#[source] std::io::Error),
    
    #[error("Database operation failed")]
    Database(#[from] rusqlite::Error),
    
    #[error("Serialization failed")]
    Serialization(#[from] serde_json::Error),
}

// For Tauri commands - implement Serialize
impl serde::Serialize for AgentError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        // Structured error format for frontend
        #[derive(serde::Serialize)]
        #[serde(tag = "kind", content = "message")]
        enum ErrorKind {
            NotFound(String),
            SpawnFailed(String),
            Database(String),
            Serialization(String),
        }
        
        let kind = match self {
            Self::NotFound { id } => ErrorKind::NotFound(format!("Agent not found: {}", id)),
            Self::SpawnFailed(e) => ErrorKind::SpawnFailed(e.to_string()),
            Self::Database(e) => ErrorKind::Database(e.to_string()),
            Self::Serialization(e) => ErrorKind::Serialization(e.to_string()),
        };
        
        kind.serialize(serializer)
    }
}
```

## 2. Async Function Decomposition

### Current Issues
- Large monolithic async functions
- Mixed concerns (DB, process management, event handling)
- Difficult to test individual components

### Refactoring Strategy

**Use block expressions for scoped operations:**

```rust
async fn execute_agent(params: ExecuteParams) -> Result<ExecutionResult, AgentError> {
    // Database operation block
    let agent_config = {
        let db = get_db_connection().await?;
        db.fetch_agent_config(&params.agent_id).await?
        // db drops here, connection returned to pool
    };
    
    // Process spawn block
    let process_handle = {
        let mut builder = ProcessBuilder::new(&agent_config);
        builder.configure_environment()?;
        builder.spawn().await?
        // builder drops here
    };
    
    // Event handling setup
    setup_event_handlers(process_handle, params.session_id).await?;
    
    Ok(ExecutionResult { 
        process_id: process_handle.id(),
        session_id: params.session_id,
    })
}
```

**Extract service functions:**

```rust
// services/agent_service.rs
pub struct AgentService {
    db_pool: Arc<DbPool>,
    process_registry: Arc<ProcessRegistry>,
}

impl AgentService {
    pub async fn execute_agent(&self, params: ExecuteParams) -> Result<ExecutionResult, AgentError> {
        let config = self.fetch_config(&params.agent_id).await?;
        let handle = self.spawn_process(config, params).await?;
        self.register_process(handle).await?;
        Ok(ExecutionResult::from(handle))
    }
    
    async fn fetch_config(&self, agent_id: &str) -> Result<AgentConfig, AgentError> {
        // Focused function for config fetching
    }
    
    async fn spawn_process(&self, config: AgentConfig, params: ExecuteParams) -> Result<ProcessHandle, AgentError> {
        // Focused function for process spawning
    }
}
```

## 3. Database Connection Pooling

### Implementation with r2d2

```rust
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::{Pool, PooledConnection};

pub type DbPool = Pool<SqliteConnectionManager>;
pub type DbConnection = PooledConnection<SqliteConnectionManager>;

pub fn create_pool(db_path: &Path) -> Result<DbPool, r2d2::Error> {
    let manager = SqliteConnectionManager::file(db_path);
    Pool::builder()
        .max_size(10) // Adjust based on workload
        .min_idle(Some(1))
        .test_on_check_out(true)
        .build(manager)
}

// In Tauri setup
fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let db_path = app.path_resolver().app_data_dir()
                .unwrap()
                .join("agents.db");
            let pool = create_pool(&db_path)?;
            app.manage(pool);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// In commands
#[tauri::command]
async fn get_agent(
    pool: State<'_, DbPool>,
    agent_id: String,
) -> Result<Agent, AgentError> {
    let conn = pool.get()?;
    // Use connection
}
```

## 4. Module Organization

### Proposed Structure

```
src-tauri/src/
├── commands/
│   ├── mod.rs
│   ├── agents/
│   │   ├── mod.rs
│   │   ├── execute.rs
│   │   ├── list.rs
│   │   └── manage.rs
│   ├── claude.rs
│   ├── mcp.rs
│   └── sandbox.rs
├── services/
│   ├── mod.rs
│   ├── agent_service.rs
│   ├── process_service.rs
│   └── event_service.rs
├── repository/
│   ├── mod.rs
│   ├── agent_repository.rs
│   └── session_repository.rs
├── models/
│   ├── mod.rs
│   ├── agent.rs
│   └── session.rs
├── error.rs
└── main.rs
```

### Service Layer Pattern

```rust
// services/agent_service.rs
pub struct AgentService<R: AgentRepository> {
    repository: R,
    process_manager: Arc<ProcessManager>,
}

impl<R: AgentRepository> AgentService<R> {
    pub async fn create_agent(&self, request: CreateAgentRequest) -> Result<Agent, AgentError> {
        // Validate request
        request.validate()?;
        
        // Business logic
        let agent = Agent::new(request)?;
        
        // Persist
        self.repository.insert(&agent).await?;
        
        Ok(agent)
    }
}

// repository/agent_repository.rs
#[async_trait]
pub trait AgentRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Option<Agent>, RepositoryError>;
    async fn insert(&self, agent: &Agent) -> Result<(), RepositoryError>;
    async fn update(&self, agent: &Agent) -> Result<(), RepositoryError>;
    async fn delete(&self, id: &str) -> Result<(), RepositoryError>;
}

pub struct SqliteAgentRepository {
    pool: Arc<DbPool>,
}

#[async_trait]
impl AgentRepository for SqliteAgentRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Agent>, RepositoryError> {
        let conn = self.pool.get()?;
        // Implementation
    }
}
```

## 5. Type-Safe Builder Pattern

### For Complex Command Parameters

```rust
use std::marker::PhantomData;

// Type states
struct Required;
struct Optional;

pub struct AgentExecutionBuilder<T, U> {
    agent_id: Option<String>,
    session_id: Option<String>,
    env_vars: HashMap<String, String>,
    _phantom: PhantomData<(T, U)>,
}

impl AgentExecutionBuilder<Required, Required> {
    pub fn new() -> AgentExecutionBuilder<Optional, Optional> {
        AgentExecutionBuilder {
            agent_id: None,
            session_id: None,
            env_vars: HashMap::new(),
            _phantom: PhantomData,
        }
    }
}

impl<T, U> AgentExecutionBuilder<T, U> {
    pub fn agent_id(mut self, id: String) -> AgentExecutionBuilder<Required, U> {
        self.agent_id = Some(id);
        AgentExecutionBuilder {
            agent_id: self.agent_id,
            session_id: self.session_id,
            env_vars: self.env_vars,
            _phantom: PhantomData,
        }
    }
    
    pub fn session_id(mut self, id: String) -> AgentExecutionBuilder<T, Required> {
        self.session_id = Some(id);
        AgentExecutionBuilder {
            agent_id: self.agent_id,
            session_id: self.session_id,
            env_vars: self.env_vars,
            _phantom: PhantomData,
        }
    }
}

impl AgentExecutionBuilder<Required, Required> {
    pub fn build(self) -> ExecutionParams {
        ExecutionParams {
            agent_id: self.agent_id.unwrap(),
            session_id: self.session_id.unwrap(),
            env_vars: self.env_vars,
        }
    }
}
```

## 6. State Management Best Practices

### Using Type Aliases for Clarity

```rust
use std::sync::Mutex;
use tauri::async_runtime::Mutex as AsyncMutex;

// Clear type aliases
type SyncState<T> = Arc<Mutex<T>>;
type AsyncState<T> = Arc<AsyncMutex<T>>;

// Application state
pub struct AppStateInner {
    agents: HashMap<String, Agent>,
    active_processes: HashMap<String, ProcessHandle>,
}

pub type AppState = SyncState<AppStateInner>;

// For async operations
pub struct AsyncAppStateInner {
    event_queue: VecDeque<Event>,
}

pub type AsyncAppState = AsyncState<AsyncAppStateInner>;
```

## 7. Migration Steps

### Phase 1: Error Handling (Week 1)
1. Define error types with thiserror
2. Implement Serialize for Tauri compatibility
3. Update all Result types
4. Add error context with proper messages

### Phase 2: Connection Pooling (Week 1)
1. Add r2d2_sqlite dependency
2. Create pool initialization
3. Update all database access points
4. Test connection limits and performance

### Phase 3: Module Extraction (Week 2)
1. Create service layer structure
2. Extract agent operations to AgentService
3. Extract process management to ProcessService
4. Create repository traits and implementations

### Phase 4: Async Refactoring (Week 2-3)
1. Break down large async functions
2. Use block expressions for scoped resources
3. Implement proper cancellation handling
4. Add structured concurrency patterns

### Phase 5: Type Safety (Week 3)
1. Implement builder patterns for complex types
2. Add newtype wrappers for IDs
3. Use phantom types for state machines
4. Add const generic constraints where applicable

### Phase 6: Testing & Documentation (Week 4)
1. Unit tests for each service
2. Integration tests for commands
3. Document public APIs
4. Update examples

## 8. Backward Compatibility

To maintain compatibility during migration:

1. Keep existing command signatures
2. Use feature flags for new implementations
3. Provide migration utilities
4. Document breaking changes clearly

```rust
#[cfg(feature = "legacy")]
#[tauri::command]
async fn execute_agent_legacy(/* old signature */) -> Result<String, String> {
    // Adapter to new implementation
}

#[cfg(not(feature = "legacy"))]
#[tauri::command]
async fn execute_agent(/* new signature */) -> Result<ExecutionResult, AgentError> {
    // New implementation
}
```

## 9. Performance Considerations

1. **Connection Pool Sizing**: For SQLite, 5-10 connections is usually sufficient
2. **Async Boundaries**: Don't hold locks across await points
3. **Event Batching**: Batch events to reduce IPC overhead
4. **Resource Cleanup**: Use RAII patterns for guaranteed cleanup

## 10. Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::automock;
    
    #[automock]
    #[async_trait]
    trait TestRepository: AgentRepository {}
    
    #[tokio::test]
    async fn test_agent_execution() {
        let mut mock_repo = MockTestRepository::new();
        mock_repo.expect_find_by_id()
            .returning(|_| Ok(Some(Agent::default())));
            
        let service = AgentService::new(mock_repo);
        let result = service.execute_agent(/* params */).await;
        assert!(result.is_ok());
    }
}
```

## Conclusion

This refactoring strategy modernizes the codebase with:
- Type-safe error handling
- Efficient resource management
- Clear separation of concerns
- Testable architecture
- Performance optimizations

The migration can be done incrementally, maintaining backward compatibility while improving code quality and maintainability.