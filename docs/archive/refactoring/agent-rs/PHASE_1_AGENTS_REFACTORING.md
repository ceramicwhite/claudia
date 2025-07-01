# Phase 1: Agents.rs Refactoring Action Plan

## Immediate Actions (Day 1-2)

### 1. Create Error Types Module
**File**: `src-tauri/src/error.rs`

```rust
use thiserror::Error;
use serde::{Serialize, ser::SerializeStruct};

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Agent not found with ID: {0}")]
    AgentNotFound(i64),
    
    #[error("Agent run not found with ID: {0}")]
    RunNotFound(i64),
    
    #[error("Failed to spawn process: {0}")]
    ProcessSpawnFailed(String),
    
    #[error("Database operation failed")]
    Database(#[from] rusqlite::Error),
    
    #[error("Serialization failed")]
    Serialization(#[from] serde_json::Error),
    
    #[error("IO operation failed")]
    Io(#[from] std::io::Error),
    
    #[error("Sandbox configuration failed: {0}")]
    SandboxConfig(String),
    
    #[error("Process registry error: {0}")]
    ProcessRegistry(String),
    
    #[error("Claude binary not found: {0}")]
    ClaudeBinaryNotFound(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Usage limit exceeded")]
    UsageLimitExceeded,
}

// Implement Serialize for Tauri compatibility
impl Serialize for AgentError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("AgentError", 3)?;
        state.serialize_field("error", &true)?;
        state.serialize_field("message", &self.to_string())?;
        state.serialize_field("type", &format!("{:?}", self))?;
        state.end()
    }
}

pub type AgentResult<T> = Result<T, AgentError>;
```

### 2. Extract Constants and Enums
**File**: `src-tauri/src/models/constants.rs`

```rust
// Pricing constants
pub mod pricing {
    #[derive(Debug, Clone, Copy)]
    pub struct ModelPricing {
        pub input_per_million: f64,
        pub output_per_million: f64,
        pub cache_write_per_million: f64,
        pub cache_read_per_million: f64,
    }
    
    pub const OPUS_4_PRICING: ModelPricing = ModelPricing {
        input_per_million: 15.0,
        output_per_million: 75.0,
        cache_write_per_million: 18.75,
        cache_read_per_million: 1.50,
    };
    
    pub const SONNET_4_PRICING: ModelPricing = ModelPricing {
        input_per_million: 3.0,
        output_per_million: 15.0,
        cache_write_per_million: 3.75,
        cache_read_per_million: 0.30,
    };
    
    pub fn get_model_pricing(model: &str) -> ModelPricing {
        if model.contains("opus-4") || model.contains("claude-opus-4") {
            OPUS_4_PRICING
        } else {
            SONNET_4_PRICING // Default to Sonnet pricing
        }
    }
}

// Status enums
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

impl RunStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Scheduled => "scheduled",
            Self::PausedUsageLimit => "paused_usage_limit",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "running" => Some(Self::Running),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            "cancelled" => Some(Self::Cancelled),
            "scheduled" => Some(Self::Scheduled),
            "paused_usage_limit" => Some(Self::PausedUsageLimit),
            _ => None,
        }
    }
}

// Sandbox operation types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SandboxOperation {
    FileReadAll,
    FileReadMetadata,
    FileWriteAll,
    NetworkOutbound,
    SystemInfoRead,
}

impl SandboxOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FileReadAll => "file_read_all",
            Self::FileReadMetadata => "file_read_metadata",
            Self::FileWriteAll => "file_write_all",
            Self::NetworkOutbound => "network_outbound",
            Self::SystemInfoRead => "system_info_read",
        }
    }
}
```

### 3. Fix Critical unwrap() Calls

**Changes in agents.rs:**

```rust
// Line 1006 - Replace:
let conn = db.0.lock().map_err(|e| e.to_string())?;

// With:
let conn = db.0.lock()
    .map_err(|_| AgentError::Database(rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_BUSY),
        Some("Failed to acquire database lock".to_string())
    )))?;

// Line 1431 - Replace:
let pid = child.id().unwrap_or(0);

// With:
let pid = child.id()
    .ok_or_else(|| AgentError::ProcessSpawnFailed("Failed to get process ID".into()))?;

// Line 1446 - Replace:
let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
let stderr = child.stderr.take().ok_or("Failed to get stderr")?;

// With:
let stdout = child.stdout.take()
    .ok_or_else(|| AgentError::ProcessSpawnFailed("Failed to capture stdout".into()))?;
let stderr = child.stderr.take()
    .ok_or_else(|| AgentError::ProcessSpawnFailed("Failed to capture stderr".into()))?;
```

### 4. Extract Helper Functions

**Create focused helper functions in agents.rs:**

```rust
// Extract sandbox rule creation
fn create_file_read_rules(project_path: &str) -> Vec<SandboxRule> {
    vec![
        SandboxRule {
            id: Some(1),
            profile_id: 0,
            operation_type: SandboxOperation::FileReadAll.as_str().to_string(),
            pattern_type: "subpath".to_string(),
            pattern_value: "{{PROJECT_PATH}}".to_string(),
            enabled: true,
            platform_support: Some(r#"["linux", "macos", "windows"]"#.to_string()),
            created_at: String::new(),
        },
        SandboxRule {
            id: Some(2),
            profile_id: 0,
            operation_type: SandboxOperation::FileReadAll.as_str().to_string(),
            pattern_type: "subpath".to_string(),
            pattern_value: "/usr/lib".to_string(),
            enabled: true,
            platform_support: Some(r#"["linux", "macos"]"#.to_string()),
            created_at: String::new(),
        },
        // ... rest of the rules
    ]
}

fn create_network_rules() -> Vec<SandboxRule> {
    vec![
        SandboxRule {
            id: Some(6),
            profile_id: 0,
            operation_type: SandboxOperation::NetworkOutbound.as_str().to_string(),
            pattern_type: "all".to_string(),
            pattern_value: "".to_string(),
            enabled: true,
            platform_support: Some(r#"["linux", "macos"]"#.to_string()),
            created_at: String::new(),
        }
    ]
}

fn create_system_default_rules() -> Vec<SandboxRule> {
    vec![
        // System binaries
        SandboxRule {
            id: Some(7),
            profile_id: 0,
            operation_type: SandboxOperation::FileReadAll.as_str().to_string(),
            pattern_type: "subpath".to_string(),
            pattern_value: "/usr/bin".to_string(),
            enabled: true,
            platform_support: Some(r#"["linux", "macos"]"#.to_string()),
            created_at: String::new(),
        },
        // ... rest of system rules
    ]
}

// Consolidate sandbox profile creation
fn create_agent_sandbox_profile(agent: &Agent, project_path: &str) -> Option<(String, Vec<SandboxRule>)> {
    if !agent.sandbox_enabled {
        info!("🔓 Agent '{}': Sandbox DISABLED", agent.name);
        return None;
    }

    info!(
        "🔒 Agent '{}': Sandbox enabled | File Read: {} | File Write: {} | Network: {}",
        agent.name, agent.enable_file_read, agent.enable_file_write, agent.enable_network
    );

    let mut rules = Vec::new();

    if agent.enable_file_read {
        rules.extend(create_file_read_rules(project_path));
    }

    if agent.enable_network {
        rules.extend(create_network_rules());
    }

    // Always add system defaults
    rules.extend(create_system_default_rules());

    Some(("Agent-specific".to_string(), rules))
}
```

### 5. Create Metric Calculation Service

**Extract from impl AgentRunMetrics:**

```rust
// src-tauri/src/services/metrics_service.rs
use crate::models::constants::pricing;

pub struct MetricsCalculator;

impl MetricsCalculator {
    pub fn calculate_cost(
        model: &str,
        input_tokens: i64,
        output_tokens: i64,
        cache_creation_tokens: i64,
        cache_read_tokens: i64,
    ) -> f64 {
        let pricing = pricing::get_model_pricing(model);
        
        let input_cost = (input_tokens as f64 / 1_000_000.0) * pricing.input_per_million;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * pricing.output_per_million;
        let cache_write_cost = (cache_creation_tokens as f64 / 1_000_000.0) * pricing.cache_write_per_million;
        let cache_read_cost = (cache_read_tokens as f64 / 1_000_000.0) * pricing.cache_read_per_million;
        
        input_cost + output_cost + cache_write_cost + cache_read_cost
    }
    
    pub fn parse_jsonl_metrics(jsonl_content: &str, model: &str) -> AgentRunMetrics {
        let mut metrics = MetricsData::default();
        
        for line in jsonl_content.lines() {
            if let Ok(json) = serde_json::from_str::<JsonValue>(line) {
                Self::process_json_entry(&json, &mut metrics);
            }
        }
        
        if !metrics.has_cost_field && metrics.total_tokens > 0 {
            metrics.cost_usd = Self::calculate_cost(
                model,
                metrics.total_input_tokens,
                metrics.total_output_tokens,
                metrics.total_cache_creation_tokens,
                metrics.total_cache_read_tokens,
            );
        }
        
        AgentRunMetrics {
            duration_ms: metrics.calculate_duration(),
            total_tokens: Some(metrics.total_tokens),
            cost_usd: Some(metrics.cost_usd),
            message_count: Some(metrics.message_count),
        }
    }
}
```

## Day 3-4 Actions

### 6. Create Repository Layer

**File**: `src-tauri/src/repository/agent_repository.rs`

```rust
use crate::error::{AgentError, AgentResult};
use crate::models::{Agent, AgentRun, RunStatus};
use rusqlite::{params, Connection, OptionalExtension};

pub struct AgentRepository<'a> {
    conn: &'a Connection,
}

impl<'a> AgentRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }
    
    pub fn find_by_id(&self, id: i64) -> AgentResult<Option<Agent>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, icon, system_prompt, default_task, model, 
             sandbox_enabled, enable_file_read, enable_file_write, enable_network, 
             created_at, updated_at 
             FROM agents WHERE id = ?1"
        )?;
        
        let agent = stmt
            .query_row(params![id], |row| {
                Ok(Agent {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    icon: row.get(2)?,
                    system_prompt: row.get(3)?,
                    default_task: row.get(4)?,
                    model: row.get(5)?,
                    sandbox_enabled: row.get(6)?,
                    enable_file_read: row.get(7)?,
                    enable_file_write: row.get(8)?,
                    enable_network: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            })
            .optional()?;
            
        Ok(agent)
    }
    
    pub fn create_run(&self, run: &AgentRun) -> AgentResult<i64> {
        self.conn.execute(
            "INSERT INTO agent_runs (
                agent_id, agent_name, agent_icon, task, model, 
                project_path, session_id, status, auto_resume_enabled, resume_count
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                run.agent_id,
                run.agent_name,
                run.agent_icon,
                run.task,
                run.model,
                run.project_path,
                run.session_id,
                run.status,
                run.auto_resume_enabled,
                run.resume_count,
            ],
        )?;
        
        Ok(self.conn.last_insert_rowid())
    }
    
    pub fn update_run_status(&self, run_id: i64, status: RunStatus, pid: Option<u32>) -> AgentResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        
        self.conn.execute(
            "UPDATE agent_runs 
             SET status = ?1, pid = ?2, process_started_at = ?3 
             WHERE id = ?4",
            params![status.as_str(), pid.map(|p| p as i64), now, run_id],
        )?;
        
        Ok(())
    }
}
```

### 7. Extract Process Management

**File**: `src-tauri/src/services/process_service.rs`

```rust
use crate::error::{AgentError, AgentResult};
use std::process::Stdio;
use tokio::process::Command;

pub struct ProcessService {
    app_handle: AppHandle,
}

impl ProcessService {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }
    
    pub fn find_claude_binary(&self) -> AgentResult<String> {
        crate::claude_binary::find_claude_binary(&self.app_handle)
            .map_err(|e| AgentError::ClaudeBinaryNotFound(e))
    }
    
    pub async fn test_claude_binary(&self, claude_path: &str) -> AgentResult<()> {
        match Command::new(claude_path)
            .arg("--version")
            .output()
            .await
        {
            Ok(output) if output.status.success() => {
                info!("✅ Claude binary works: {}", 
                    String::from_utf8_lossy(&output.stdout).trim());
                Ok(())
            }
            Ok(output) => {
                warn!("⚠️ Claude binary failed: {}", output.status);
                Err(AgentError::ClaudeBinaryNotFound(
                    format!("Claude binary failed with status: {}", output.status)
                ))
            }
            Err(e) => {
                error!("❌ Failed to execute Claude binary: {}", e);
                Err(AgentError::ClaudeBinaryNotFound(e.to_string()))
            }
        }
    }
    
    pub fn build_claude_command(
        &self,
        claude_path: &str,
        task: &str,
        system_prompt: &str,
        model: &str,
        project_path: &str,
    ) -> Command {
        let mut cmd = create_command_with_env(claude_path);
        
        cmd.arg("-p")
            .arg(task)
            .arg("--system-prompt")
            .arg(system_prompt)
            .arg("--model")
            .arg(model)
            .arg("--output-format")
            .arg("stream-json")
            .arg("--verbose")
            .arg("--dangerously-skip-permissions")
            .current_dir(project_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
            
        cmd
    }
}
```

### 8. Create Event Streaming Service

**File**: `src-tauri/src/services/event_service.rs`

```rust
use crate::error::{AgentError, AgentResult};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{ChildStdout, ChildStderr};

pub struct EventStreamService {
    app_handle: AppHandle,
}

impl EventStreamService {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }
    
    pub async fn setup_output_streaming(
        &self,
        stdout: ChildStdout,
        stderr: ChildStderr,
        run_id: i64,
        output_handler: Arc<dyn OutputHandler>,
    ) -> AgentResult<StreamingHandle> {
        let stdout_task = self.spawn_stdout_reader(stdout, run_id, output_handler.clone());
        let stderr_task = self.spawn_stderr_reader(stderr, run_id, output_handler);
        
        Ok(StreamingHandle {
            stdout_task,
            stderr_task,
            run_id,
        })
    }
    
    fn spawn_stdout_reader(
        &self,
        stdout: ChildStdout,
        run_id: i64,
        handler: Arc<dyn OutputHandler>,
    ) -> JoinHandle<()> {
        let app_handle = self.app_handle.clone();
        
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            let mut line_count = 0;
            
            while let Ok(Some(line)) = reader.next_line().await {
                line_count += 1;
                
                // Process line
                if let Err(e) = handler.handle_output_line(&line, run_id).await {
                    error!("Failed to handle output line: {}", e);
                }
                
                // Emit to frontend
                let _ = app_handle.emit(&format!("agent-output:{}", run_id), &line);
            }
            
            info!("Finished reading stdout. Total lines: {}", line_count);
        })
    }
}

#[async_trait]
pub trait OutputHandler: Send + Sync {
    async fn handle_output_line(&self, line: &str, run_id: i64) -> AgentResult<()>;
}
```

## Day 5 Actions

### 9. Update execute_agent Function

**Refactored version using new services:**

```rust
pub async fn execute_agent(
    app: AppHandle,
    agent_id: i64,
    project_path: String,
    task: String,
    model: Option<String>,
    auto_resume_enabled: Option<bool>,
    db: State<'_, AgentDb>,
    registry: State<'_, ProcessRegistryState>,
) -> Result<i64, String> {
    execute_agent_impl(
        app,
        agent_id,
        project_path,
        task,
        model,
        auto_resume_enabled,
        db,
        registry,
    )
    .await
    .map_err(|e| e.to_string())
}

async fn execute_agent_impl(
    app: AppHandle,
    agent_id: i64,
    project_path: String,
    task: String,
    model: Option<String>,
    auto_resume_enabled: Option<bool>,
    db: State<'_, AgentDb>,
    registry: State<'_, ProcessRegistryState>,
) -> AgentResult<i64> {
    info!("Executing agent {} with task: {}", agent_id, task);
    
    // Step 1: Load agent configuration
    let agent = {
        let conn = db.0.lock()
            .map_err(|_| AgentError::Database(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_BUSY),
                Some("Failed to acquire database lock".to_string())
            )))?;
        
        let repo = AgentRepository::new(&conn);
        repo.find_by_id(agent_id)?
            .ok_or_else(|| AgentError::AgentNotFound(agent_id))?
    };
    
    let execution_model = model.unwrap_or(agent.model.clone());
    
    // Step 2: Create run record
    let run_id = {
        let conn = db.0.lock()
            .map_err(|_| AgentError::Database(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_BUSY),
                Some("Failed to acquire database lock".to_string())
            )))?;
            
        let repo = AgentRepository::new(&conn);
        
        let run = AgentRun {
            id: None,
            agent_id,
            agent_name: agent.name.clone(),
            agent_icon: agent.icon.clone(),
            task: task.clone(),
            model: execution_model.clone(),
            project_path: project_path.clone(),
            session_id: String::new(),
            status: RunStatus::Pending.as_str().to_string(),
            pid: None,
            process_started_at: None,
            scheduled_start_time: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
            usage_limit_reset_time: None,
            auto_resume_enabled: auto_resume_enabled.unwrap_or(false),
            resume_count: 0,
            parent_run_id: None,
        };
        
        repo.create_run(&run)?
    };
    
    // Step 3: Setup sandbox profile
    let sandbox_profile = create_agent_sandbox_profile(&agent, &project_path);
    
    // Step 4: Build and spawn process
    let process_service = ProcessService::new(app.clone());
    let claude_path = process_service.find_claude_binary()?;
    
    // Test Claude binary first
    process_service.test_claude_binary(&claude_path).await?;
    
    // Build command with or without sandbox
    let mut cmd = if let Some((profile_name, rules)) = sandbox_profile {
        build_sandboxed_command(&app, &agent, &claude_path, &task, &execution_model, &project_path, rules)?
    } else {
        warn!("🚨 Running agent '{}' WITHOUT SANDBOX - full system access!", agent.name);
        process_service.build_claude_command(&claude_path, &task, &agent.system_prompt, &execution_model, &project_path)
    };
    
    // Spawn the process
    info!("🚀 Spawning Claude process...");
    let mut child = cmd.spawn()
        .map_err(|e| AgentError::ProcessSpawnFailed(format!("Failed to spawn Claude: {}", e)))?;
    
    let pid = child.id()
        .ok_or_else(|| AgentError::ProcessSpawnFailed("Failed to get process ID".into()))?;
    
    info!("✅ Claude process spawned successfully with PID: {}", pid);
    
    // Step 5: Update database with running status
    {
        let conn = db.0.lock()
            .map_err(|_| AgentError::Database(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_BUSY),
                Some("Failed to acquire database lock".to_string())
            )))?;
            
        let repo = AgentRepository::new(&conn);
        repo.update_run_status(run_id, RunStatus::Running, Some(pid))?;
    }
    
    // Step 6: Setup event streaming
    let stdout = child.stdout.take()
        .ok_or_else(|| AgentError::ProcessSpawnFailed("Failed to capture stdout".into()))?;
    let stderr = child.stderr.take()
        .ok_or_else(|| AgentError::ProcessSpawnFailed("Failed to capture stderr".into()))?;
    
    let event_service = EventStreamService::new(app.clone());
    let output_handler = create_output_handler(app.clone(), run_id);
    
    let _streaming_handle = event_service
        .setup_output_streaming(stdout, stderr, run_id, output_handler)
        .await?;
    
    // Step 7: Register process
    registry.0
        .register_process(
            run_id,
            agent_id,
            agent.name,
            pid,
            project_path,
            task,
            execution_model,
            child,
        )
        .map_err(|e| AgentError::ProcessRegistry(format!("Failed to register process: {}", e)))?;
    
    info!("📋 Agent execution started successfully with run_id: {}", run_id);
    
    Ok(run_id)
}
```

### 10. Add Basic Tests

**File**: `src-tauri/src/commands/agents/tests.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::constants::pricing;
    
    #[test]
    fn test_model_pricing() {
        let opus_pricing = pricing::get_model_pricing("claude-opus-4");
        assert_eq!(opus_pricing.input_per_million, 15.0);
        assert_eq!(opus_pricing.output_per_million, 75.0);
        
        let sonnet_pricing = pricing::get_model_pricing("claude-sonnet-4");
        assert_eq!(sonnet_pricing.input_per_million, 3.0);
        assert_eq!(sonnet_pricing.output_per_million, 15.0);
    }
    
    #[test]
    fn test_run_status_conversion() {
        assert_eq!(RunStatus::Pending.as_str(), "pending");
        assert_eq!(RunStatus::from_str("running"), Some(RunStatus::Running));
        assert_eq!(RunStatus::from_str("invalid"), None);
    }
    
    #[test]
    fn test_cost_calculation() {
        let cost = MetricsCalculator::calculate_cost(
            "claude-opus-4",
            1_000_000,  // 1M input tokens
            500_000,    // 500K output tokens
            100_000,    // 100K cache write
            50_000,     // 50K cache read
        );
        
        // Expected: 15 + 37.5 + 1.875 + 0.075 = 54.45
        assert!((cost - 54.45).abs() < 0.01);
    }
    
    #[tokio::test]
    async fn test_error_serialization() {
        let error = AgentError::AgentNotFound(123);
        let serialized = serde_json::to_string(&error).unwrap();
        assert!(serialized.contains("\"error\":true"));
        assert!(serialized.contains("Agent not found with ID: 123"));
    }
}
```

## Migration Checklist

### Day 1-2 Checklist
- [ ] Create error.rs with AgentError type
- [ ] Create models/constants.rs with pricing and enums
- [ ] Replace all `.unwrap()` calls with proper error handling
- [ ] Extract sandbox rule creation helper functions
- [ ] Create metrics calculation service

### Day 3-4 Checklist
- [ ] Create repository layer for database operations
- [ ] Extract process management to ProcessService
- [ ] Create event streaming service
- [ ] Begin refactoring execute_agent function

### Day 5 Checklist
- [ ] Complete execute_agent refactoring
- [ ] Add basic unit tests
- [ ] Update imports and module structure
- [ ] Test all affected commands
- [ ] Document new patterns

## Success Criteria

1. **Code Quality**
   - execute_agent reduced from 757 lines to ~150 lines
   - No more unwrap() calls in critical paths
   - Clear separation of concerns

2. **Functionality**
   - All existing features work correctly
   - Better error messages for users
   - No performance regression

3. **Maintainability**
   - New code has tests
   - Clear module boundaries
   - Consistent error handling

## Rollback Plan

If issues arise:
1. Git stash current changes
2. Revert to previous commit
3. Apply fixes incrementally
4. Test each change thoroughly

Keep the old code commented out until new implementation is stable.