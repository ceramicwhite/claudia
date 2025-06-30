# Comprehensive Refactoring Plan for Claudia Commands Module

## Executive Summary

This document outlines a detailed, phased approach to refactoring the Claudia commands module, with a primary focus on the agents.rs file. The refactoring addresses critical issues including:
- A 1700+ line agents.rs file with a 757-line execute_agent function
- String-based error handling throughout
- Mixed concerns and lack of separation
- No service layer or proper abstractions
- Direct database access without connection pooling
- Hardcoded values and magic strings

## 1. Dependency Analysis & Refactoring Order

### Module Dependencies Map
```
agents.rs
├── claude_binary (finding binary path)
├── sandbox/* (security profiles)
├── process/registry (process tracking)
├── Database (direct SQLite access)
├── Tauri (events, app handle)
└── External crates (tokio, reqwest, chrono)

claude.rs
├── process management (global state)
└── file system operations

mcp.rs
├── external process spawning
└── configuration management

sandbox.rs
├── platform-specific security
└── violation tracking
```

### Refactoring Order (to avoid breaking changes)
1. **Error types** - Create first, migrate incrementally
2. **Constants & Enums** - Extract magic values
3. **Repository layer** - Abstract database access
4. **Service layer** - Extract business logic
5. **Command refactoring** - Update Tauri commands last

## 2. Risk Assessment

### High Risk Areas
- **execute_agent function** (757 lines)
  - Critical path for agent execution
  - Handles subprocess lifecycle
  - Complex error scenarios
  - Database state management
  
- **Process management**
  - System resource cleanup
  - Zombie process prevention
  - Cross-platform compatibility

- **Sandbox integration**
  - Security-critical code
  - Platform-specific behavior
  - Fallback mechanisms

### Medium Risk Areas
- **Database migrations**
  - Data integrity concerns
  - Backward compatibility
  
- **Scheduled runs**
  - Timing-sensitive operations
  - State consistency

- **Usage tracking**
  - Billing calculations
  - Cost accuracy

### Low Risk Areas
- List/read operations
- Settings management
- Status checks

## 3. Phased Implementation Plan

### Phase 1: Safe Refactorings (Week 1)
**Goal**: Quick wins with minimal risk

#### 1.1 Extract Constants
```rust
// Create src-tauri/src/models/pricing.rs
pub mod pricing {
    pub const OPUS_4_INPUT_PRICE: f64 = 15.0;
    pub const OPUS_4_OUTPUT_PRICE: f64 = 75.0;
    pub const OPUS_4_CACHE_WRITE_PRICE: f64 = 18.75;
    pub const OPUS_4_CACHE_READ_PRICE: f64 = 1.50;
    
    pub const SONNET_4_INPUT_PRICE: f64 = 3.0;
    pub const SONNET_4_OUTPUT_PRICE: f64 = 15.0;
    pub const SONNET_4_CACHE_WRITE_PRICE: f64 = 3.75;
    pub const SONNET_4_CACHE_READ_PRICE: f64 = 0.30;
}
```

#### 1.2 Create Enums for Magic Strings
```rust
// Create src-tauri/src/models/status.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Scheduled,
    PausedUsageLimit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    FileReadAll,
    FileReadMetadata,
    FileWriteAll,
    NetworkOutbound,
    SystemInfoRead,
}
```

#### 1.3 Error Type Definition
```rust
// Create src-tauri/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Agent not found: {id}")]
    NotFound { id: i64 },
    
    #[error("Process spawn failed: {0}")]
    SpawnFailed(String),
    
    #[error("Database error")]
    Database(#[from] rusqlite::Error),
    
    #[error("Serialization error")]
    Serialization(#[from] serde_json::Error),
    
    #[error("IO error")]
    Io(#[from] std::io::Error),
    
    #[error("Sandbox error: {0}")]
    Sandbox(String),
    
    #[error("Process management error: {0}")]
    ProcessManagement(String),
}

// Implement Serialize for Tauri
impl serde::Serialize for AgentError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AgentError", 2)?;
        state.serialize_field("error", &self.to_string())?;
        state.serialize_field("type", &format!("{:?}", self))?;
        state.end()
    }
}
```

#### 1.4 Fix Unwrap Calls
Replace all `.unwrap()` with proper error handling:
```rust
// Before
let pid = child.id().unwrap_or(0);

// After
let pid = child.id().ok_or_else(|| AgentError::ProcessManagement("Failed to get process ID".into()))?;
```

### Phase 2: Structural Improvements (Week 2)

#### 2.1 Break Down execute_agent Function

Create focused functions:
```rust
// src-tauri/src/services/agent_execution.rs
pub struct AgentExecutor {
    db: Arc<DbPool>,
    process_registry: Arc<ProcessRegistry>,
    sandbox_service: Arc<SandboxService>,
}

impl AgentExecutor {
    pub async fn execute(
        &self,
        params: ExecuteParams,
    ) -> Result<ExecutionResult, AgentError> {
        // 1. Load configuration
        let config = self.load_agent_config(params.agent_id).await?;
        
        // 2. Create execution context
        let context = self.create_execution_context(&config, &params)?;
        
        // 3. Setup sandbox if needed
        let sandbox_cmd = self.setup_sandbox(&config, &context).await?;
        
        // 4. Spawn process
        let process = self.spawn_process(sandbox_cmd, &context).await?;
        
        // 5. Setup monitoring
        let monitor = self.setup_monitoring(process, &context).await?;
        
        // 6. Register and return
        self.register_execution(monitor, &context).await?;
        
        Ok(ExecutionResult {
            run_id: context.run_id,
            session_id: monitor.session_id,
        })
    }
    
    async fn load_agent_config(&self, agent_id: i64) -> Result<AgentConfig, AgentError> {
        // Extract from current execute_agent
    }
    
    async fn setup_sandbox(
        &self,
        config: &AgentConfig,
        context: &ExecutionContext,
    ) -> Result<Command, AgentError> {
        // Extract sandbox setup logic
    }
}
```

#### 2.2 Extract Repository Pattern
```rust
// src-tauri/src/repository/agent_repository.rs
#[async_trait]
pub trait AgentRepository: Send + Sync {
    async fn find_by_id(&self, id: i64) -> Result<Option<Agent>, RepositoryError>;
    async fn create(&self, agent: &Agent) -> Result<i64, RepositoryError>;
    async fn update(&self, agent: &Agent) -> Result<(), RepositoryError>;
    async fn delete(&self, id: i64) -> Result<(), RepositoryError>;
    async fn list(&self) -> Result<Vec<Agent>, RepositoryError>;
}

pub struct SqliteAgentRepository {
    pool: Arc<DbPool>,
}

#[async_trait]
impl AgentRepository for SqliteAgentRepository {
    async fn find_by_id(&self, id: i64) -> Result<Option<Agent>, RepositoryError> {
        let conn = self.pool.get()?;
        // Implement query
    }
}
```

#### 2.3 Extract Sandbox Profile Builder
```rust
// src-tauri/src/services/sandbox_service.rs
pub struct SandboxProfileBuilder {
    rules: Vec<SandboxRule>,
}

impl SandboxProfileBuilder {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }
    
    pub fn with_file_read(mut self, enabled: bool, project_path: &Path) -> Self {
        if enabled {
            self.rules.extend(Self::file_read_rules(project_path));
        }
        self
    }
    
    pub fn with_network(mut self, enabled: bool) -> Self {
        if enabled {
            self.rules.push(SandboxRule::network_all());
        }
        self
    }
    
    pub fn build(self) -> Vec<SandboxRule> {
        self.rules
    }
    
    fn file_read_rules(project_path: &Path) -> Vec<SandboxRule> {
        vec![
            SandboxRule::file_read_subpath(project_path),
            SandboxRule::file_read_subpath("/usr/lib"),
            SandboxRule::file_read_subpath("/usr/local/lib"),
            // ... other rules
        ]
    }
}
```

### Phase 3: Pattern Migrations (Week 3)

#### 3.1 Implement Connection Pooling
```rust
// src-tauri/src/db/mod.rs
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub type DbPool = Pool<SqliteConnectionManager>;

pub fn create_pool(db_path: &Path) -> Result<DbPool, r2d2::Error> {
    let manager = SqliteConnectionManager::file(db_path);
    Pool::builder()
        .max_size(10)
        .min_idle(Some(1))
        .test_on_check_out(true)
        .build(manager)
}

// Update main.rs setup
.setup(|app| {
    let db_path = app.path_resolver()
        .app_data_dir()
        .unwrap()
        .join("agents.db");
    let pool = create_pool(&db_path)?;
    app.manage(pool);
    Ok(())
})
```

#### 3.2 Service Layer Implementation
```rust
// src-tauri/src/services/mod.rs
pub mod agent_service;
pub mod process_service;
pub mod sandbox_service;
pub mod event_service;

// src-tauri/src/services/agent_service.rs
pub struct AgentService {
    repository: Arc<dyn AgentRepository>,
    executor: Arc<AgentExecutor>,
    event_bus: Arc<EventBus>,
}

impl AgentService {
    pub async fn create_agent(&self, req: CreateAgentRequest) -> Result<Agent, AgentError> {
        // Validation
        req.validate()?;
        
        // Business logic
        let agent = Agent::from_request(req)?;
        
        // Persistence
        let id = self.repository.create(&agent).await?;
        
        // Event
        self.event_bus.publish(AgentCreated { id }).await;
        
        Ok(agent.with_id(id))
    }
    
    pub async fn execute_agent(&self, req: ExecuteAgentRequest) -> Result<ExecutionResult, AgentError> {
        // Delegate to executor
        self.executor.execute(req.into()).await
    }
}
```

#### 3.3 Event-Driven Architecture
```rust
// src-tauri/src/events/mod.rs
#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: Event) -> Result<(), EventError>;
}

pub struct EventBus {
    handlers: Vec<Box<dyn EventHandler>>,
}

impl EventBus {
    pub async fn publish(&self, event: impl Into<Event>) -> Result<(), EventError> {
        let event = event.into();
        for handler in &self.handlers {
            handler.handle(event.clone()).await?;
        }
        Ok(())
    }
}
```

### Phase 4: Type Safety Improvements (Week 4)

#### 4.1 Newtype Wrappers
```rust
// src-tauri/src/models/ids.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AgentId(i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RunId(i64);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(String);

impl AgentId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
    
    pub fn value(&self) -> i64 {
        self.0
    }
}
```

#### 4.2 Builder Pattern for Complex Operations
```rust
// src-tauri/src/builders/execution_builder.rs
pub struct ExecutionBuilder {
    agent_id: Option<AgentId>,
    project_path: Option<PathBuf>,
    task: Option<String>,
    model: Option<String>,
    sandbox_config: SandboxConfig,
}

impl ExecutionBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn agent(mut self, id: AgentId) -> Self {
        self.agent_id = Some(id);
        self
    }
    
    pub fn project(mut self, path: impl Into<PathBuf>) -> Self {
        self.project_path = Some(path.into());
        self
    }
    
    pub fn task(mut self, task: impl Into<String>) -> Self {
        self.task = Some(task.into());
        self
    }
    
    pub fn build(self) -> Result<ExecutionParams, BuilderError> {
        Ok(ExecutionParams {
            agent_id: self.agent_id.ok_or(BuilderError::MissingField("agent_id"))?,
            project_path: self.project_path.ok_or(BuilderError::MissingField("project_path"))?,
            task: self.task.ok_or(BuilderError::MissingField("task"))?,
            model: self.model,
            sandbox_config: self.sandbox_config,
        })
    }
}
```

## 4. Testing Strategy

### Unit Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    
    #[tokio::test]
    async fn test_agent_creation() {
        // Arrange
        let mut mock_repo = MockAgentRepository::new();
        mock_repo.expect_create()
            .returning(|_| Ok(1));
            
        let service = AgentService::new(Arc::new(mock_repo));
        
        // Act
        let result = service.create_agent(CreateAgentRequest {
            name: "Test Agent".into(),
            // ...
        }).await;
        
        // Assert
        assert!(result.is_ok());
    }
}
```

### Integration Testing
```rust
#[cfg(test)]
mod integration_tests {
    #[tokio::test]
    async fn test_full_agent_execution() {
        // Use in-memory SQLite
        let pool = create_test_pool();
        
        // Create test agent
        let agent = create_test_agent(&pool).await;
        
        // Execute with mock Claude binary
        let result = execute_with_mock(&pool, agent.id).await;
        
        assert!(result.is_ok());
    }
}
```

## 5. Migration Guide

### For Each Phase:

1. **Create feature flag**
   ```toml
   [features]
   refactored-agents = []
   ```

2. **Implement side-by-side**
   ```rust
   #[cfg(feature = "refactored-agents")]
   pub use new_implementation::*;
   
   #[cfg(not(feature = "refactored-agents"))]
   pub use legacy_implementation::*;
   ```

3. **Test both paths**
4. **Gradual rollout**
5. **Remove legacy code**

## 6. Specific execute_agent Refactoring

### Current State Analysis
- **Lines**: 757 (988-1745)
- **Responsibilities**: 12+ different concerns
- **Cyclomatic complexity**: ~40+
- **Test coverage**: 0%

### Target State
- **Lines**: ~50-100 (orchestration only)
- **Responsibilities**: 1 (orchestration)
- **Cyclomatic complexity**: <10
- **Test coverage**: >80%

### Extraction Plan

#### Step 1: Extract Configuration Loading (50 lines → service)
```rust
async fn load_agent_config(&self, agent_id: AgentId) -> Result<AgentConfig, AgentError> {
    self.agent_repository
        .find_by_id(agent_id)
        .await?
        .ok_or_else(|| AgentError::NotFound { id: agent_id })
}
```

#### Step 2: Extract Sandbox Profile Creation (200+ lines → service)
```rust
async fn create_sandbox_profile(&self, agent: &Agent, project_path: &Path) -> Result<Option<SandboxProfile>, AgentError> {
    if !agent.sandbox_enabled {
        return Ok(None);
    }
    
    let profile = SandboxProfileBuilder::new()
        .with_file_read(agent.enable_file_read, project_path)
        .with_file_write(agent.enable_file_write, project_path)
        .with_network(agent.enable_network)
        .with_system_defaults()
        .build()?;
        
    Ok(Some(profile))
}
```

#### Step 3: Extract Process Spawning (100+ lines → service)
```rust
async fn spawn_claude_process(&self, config: ProcessConfig) -> Result<Child, AgentError> {
    let claude_path = self.find_claude_binary()?;
    
    let mut cmd = Command::new(&claude_path);
    cmd.args(&config.args)
       .current_dir(&config.working_dir)
       .env_clear()
       .envs(&config.env_vars)
       .stdin(Stdio::null())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());
       
    cmd.spawn()
       .map_err(|e| AgentError::SpawnFailed(e.to_string()))
}
```

#### Step 4: Extract Event Streaming (200+ lines → service)
```rust
async fn setup_event_streaming(&self, mut child: Child, run_id: RunId) -> Result<StreamHandle, AgentError> {
    let stdout = child.stdout.take().ok_or_else(|| AgentError::ProcessManagement("No stdout".into()))?;
    let stderr = child.stderr.take().ok_or_else(|| AgentError::ProcessManagement("No stderr".into()))?;
    
    let handle = StreamHandle::new(run_id);
    
    // Spawn stdout reader
    let stdout_handle = self.spawn_output_reader(stdout, run_id, StreamType::Stdout);
    
    // Spawn stderr reader
    let stderr_handle = self.spawn_output_reader(stderr, run_id, StreamType::Stderr);
    
    handle.set_readers(stdout_handle, stderr_handle);
    Ok(handle)
}
```

#### Step 5: Refactored execute_agent
```rust
pub async fn execute_agent(
    app: AppHandle,
    agent_id: i64,
    project_path: String,
    task: String,
    model: Option<String>,
    auto_resume_enabled: Option<bool>,
    services: State<'_, Services>,
) -> Result<i64, String> {
    let params = ExecutionBuilder::new()
        .agent(AgentId::new(agent_id))
        .project(project_path)
        .task(task)
        .model(model)
        .auto_resume(auto_resume_enabled.unwrap_or(false))
        .build()
        .map_err(|e| e.to_string())?;
        
    services
        .agent_service
        .execute_agent(params)
        .await
        .map(|result| result.run_id.value())
        .map_err(|e| e.to_string())
}
```

## 7. Performance Considerations

### Connection Pool Tuning
- SQLite optimal pool size: 5-10 connections
- Enable WAL mode for better concurrency
- Use prepared statements for repeated queries

### Async Optimization
- Don't hold locks across await points
- Use tokio::select! for concurrent operations
- Batch database operations where possible

### Memory Management
- Stream large outputs instead of buffering
- Use Arc<str> for shared string data
- Implement backpressure for event streams

## 8. Rollback Strategy

### For Each Phase:
1. **Feature flags** - Easy on/off switch
2. **Database migrations** - Keep backward compatible
3. **API versioning** - Support old and new formats
4. **Monitoring** - Track errors and performance
5. **Quick revert** - Git tags for each phase

### Emergency Procedures:
```bash
# Revert to previous version
git checkout v1.x.x-pre-refactor
cargo build --release

# Disable new features
echo 'refactored-agents = false' >> Config.toml

# Roll back database
sqlite3 agents.db < rollback_script.sql
```

## 9. Success Metrics

### Code Quality
- [ ] Reduce execute_agent to <100 lines
- [ ] Achieve <10 cyclomatic complexity per function
- [ ] Remove all unwrap() calls
- [ ] 80%+ test coverage

### Performance
- [ ] No regression in execution time
- [ ] Reduced memory usage
- [ ] Better error messages
- [ ] Faster startup time

### Maintainability
- [ ] Clear separation of concerns
- [ ] Documented public APIs
- [ ] Consistent error handling
- [ ] Modular architecture

## 10. Timeline

### Week 1: Foundation
- Day 1-2: Error types and constants
- Day 3-4: Fix unwrap calls
- Day 5: Initial testing setup

### Week 2: Core Refactoring
- Day 1-2: Break down execute_agent
- Day 3-4: Repository pattern
- Day 5: Service layer basics

### Week 3: Advanced Patterns
- Day 1-2: Connection pooling
- Day 3-4: Event system
- Day 5: Integration testing

### Week 4: Polish
- Day 1-2: Type safety improvements
- Day 3-4: Documentation
- Day 5: Performance tuning

### Week 5: Deployment
- Day 1-2: Final testing
- Day 3: Staged rollout
- Day 4-5: Monitor and adjust

## Conclusion

This comprehensive refactoring plan addresses all major issues in the codebase while maintaining a pragmatic, phased approach. By following this plan, the codebase will become more maintainable, testable, and performant, while reducing technical debt and improving developer experience.