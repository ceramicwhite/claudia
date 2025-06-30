# Execute Agent Function Breakdown

## Current State Analysis

**File**: `src-tauri/src/commands/agents.rs`  
**Lines**: 988-1745 (757 lines total!)  
**Cyclomatic Complexity**: ~40+

## Extraction Plan

### 1. Configuration Loading (Lines 1000-1013)
**Extract to**: `AgentRepository::find_by_id()`
```rust
async fn load_agent_config(db: &AgentDb, agent_id: i64) -> AgentResult<Agent> {
    let conn = acquire_db_connection(db)?;
    let repo = AgentRepository::new(&conn);
    repo.find_by_id(agent_id)?
        .ok_or_else(|| AgentError::AgentNotFound(agent_id))
}
```

### 2. Run Record Creation (Lines 1005-1013)
**Extract to**: `AgentRepository::create_run()`
```rust
async fn create_agent_run(
    db: &AgentDb,
    agent: &Agent,
    params: &ExecutionParams,
) -> AgentResult<i64> {
    let conn = acquire_db_connection(db)?;
    let repo = AgentRepository::new(&conn);
    
    let run = AgentRun::new(agent, params);
    repo.create_run(&run)
}
```

### 3. Sandbox Profile Creation (Lines 1016-1183)
**Extract to**: `SandboxService::create_profile()`
```rust
fn create_sandbox_profile(agent: &Agent, project_path: &str) -> Option<SandboxProfile> {
    if !agent.sandbox_enabled {
        return None;
    }
    
    let builder = SandboxProfileBuilder::new()
        .with_file_read(agent.enable_file_read, project_path)
        .with_file_write(agent.enable_file_write, project_path)
        .with_network(agent.enable_network)
        .with_system_defaults();
        
    Some(builder.build())
}
```

### 4. Claude Binary Testing (Lines 1188-1276)
**Extract to**: `ProcessService::validate_claude_binary()`
```rust
async fn validate_claude_binary(
    app: &AppHandle,
    task: &str,
    system_prompt: &str,
    model: &str,
) -> AgentResult<String> {
    let claude_path = find_claude_binary(app)?;
    
    // Test --version
    test_claude_version(&claude_path).await?;
    
    // Test actual command
    test_claude_command(&claude_path, task, system_prompt, model).await?;
    
    Ok(claude_path)
}
```

### 5. Command Building (Lines 1278-1419)
**Extract to**: `ProcessService::build_command()`
```rust
fn build_command(
    claude_path: &str,
    agent: &Agent,
    params: &ExecutionParams,
    sandbox_profile: Option<SandboxProfile>,
) -> AgentResult<Command> {
    if let Some(profile) = sandbox_profile {
        build_sandboxed_command(claude_path, agent, params, profile)
    } else {
        build_unsandboxed_command(claude_path, agent, params)
    }
}
```

### 6. Process Spawning (Lines 1421-1448)
**Extract to**: `ProcessService::spawn_process()`
```rust
async fn spawn_process(cmd: Command) -> AgentResult<ProcessHandle> {
    let mut child = cmd.spawn()
        .map_err(|e| AgentError::ProcessSpawnFailed(e.to_string()))?;
        
    let pid = child.id()
        .ok_or_else(|| AgentError::ProcessSpawnFailed("No PID".into()))?;
        
    let stdout = child.stdout.take()
        .ok_or_else(|| AgentError::ProcessSpawnFailed("No stdout".into()))?;
        
    let stderr = child.stderr.take()
        .ok_or_else(|| AgentError::ProcessSpawnFailed("No stderr".into()))?;
        
    Ok(ProcessHandle { child, pid, stdout, stderr })
}
```

### 7. Event Streaming Setup (Lines 1450-1587)
**Extract to**: `EventService::setup_streaming()`
```rust
async fn setup_event_streaming(
    app: &AppHandle,
    handle: ProcessHandle,
    run_id: i64,
) -> AgentResult<StreamingContext> {
    let session_id = Arc::new(Mutex::new(String::new()));
    let live_output = Arc::new(Mutex::new(String::new()));
    
    let stdout_task = spawn_stdout_reader(
        handle.stdout,
        run_id,
        app.clone(),
        session_id.clone(),
        live_output.clone(),
    );
    
    let stderr_task = spawn_stderr_reader(
        handle.stderr,
        run_id,
        app.clone(),
    );
    
    Ok(StreamingContext {
        session_id,
        live_output,
        stdout_task,
        stderr_task,
    })
}
```

### 8. Process Monitoring (Lines 1588-1745)
**Extract to**: `ProcessMonitor::start_monitoring()`
```rust
async fn start_process_monitoring(
    app: &AppHandle,
    db: &AgentDb,
    registry: &ProcessRegistry,
    run_id: i64,
    process_info: ProcessInfo,
    streaming_context: StreamingContext,
) -> AgentResult<()> {
    // Register process
    registry.register_process(process_info)?;
    
    // Spawn monitoring task
    tokio::spawn(monitor_process_lifecycle(
        app.clone(),
        db.clone(),
        run_id,
        streaming_context,
    ));
    
    Ok(())
}
```

## Refactored execute_agent Function

After extraction, the function becomes:

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
    // Create services
    let services = ExecutionServices {
        process_service: ProcessService::new(app.clone()),
        sandbox_service: SandboxService::new(),
        event_service: EventService::new(app.clone()),
        monitor: ProcessMonitor::new(app.clone(), db.inner().clone()),
    };
    
    // Build execution parameters
    let params = ExecutionParams {
        agent_id,
        project_path,
        task,
        model,
        auto_resume_enabled: auto_resume_enabled.unwrap_or(false),
    };
    
    // Execute with proper error handling
    execute_agent_with_services(services, params, db.inner(), registry.inner())
        .await
        .map_err(|e| e.to_string())
}

async fn execute_agent_with_services(
    services: ExecutionServices,
    params: ExecutionParams,
    db: &AgentDb,
    registry: &ProcessRegistry,
) -> AgentResult<i64> {
    // Step 1: Load configuration
    let agent = load_agent_config(db, params.agent_id).await?;
    
    // Step 2: Create run record
    let run_id = create_agent_run(db, &agent, &params).await?;
    
    // Step 3: Create sandbox profile
    let sandbox_profile = services.sandbox_service
        .create_profile(&agent, &params.project_path);
    
    // Step 4: Validate and build command
    let claude_path = services.process_service
        .validate_claude_binary(&params.task, &agent.system_prompt, &params.model)
        .await?;
        
    let cmd = services.process_service
        .build_command(&claude_path, &agent, &params, sandbox_profile)?;
    
    // Step 5: Spawn process
    let process_handle = services.process_service
        .spawn_process(cmd)
        .await?;
    
    // Step 6: Update database status
    update_run_status(db, run_id, RunStatus::Running, Some(process_handle.pid)).await?;
    
    // Step 7: Setup event streaming
    let streaming_context = services.event_service
        .setup_streaming(process_handle, run_id)
        .await?;
    
    // Step 8: Start monitoring
    let process_info = ProcessInfo::new(run_id, &agent, &params, process_handle.pid);
    services.monitor
        .start_monitoring(registry, run_id, process_info, streaming_context)
        .await?;
    
    Ok(run_id)
}
```

## Benefits of Refactoring

1. **Reduced Complexity**: From 757 lines to ~50 lines
2. **Testability**: Each extracted function can be unit tested
3. **Reusability**: Services can be used by other commands
4. **Maintainability**: Clear separation of concerns
5. **Error Handling**: Consistent error types throughout
6. **Debugging**: Easier to locate and fix issues

## Testing Strategy

Each extracted component should have tests:

```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_load_agent_config() {
        let db = create_test_db();
        let agent = create_test_agent(&db);
        
        let loaded = load_agent_config(&db, agent.id).await;
        assert!(loaded.is_ok());
        assert_eq!(loaded.unwrap().name, agent.name);
    }
    
    #[tokio::test] 
    async fn test_sandbox_profile_creation() {
        let agent = Agent {
            sandbox_enabled: true,
            enable_file_read: true,
            enable_file_write: false,
            enable_network: true,
            ..Default::default()
        };
        
        let profile = create_sandbox_profile(&agent, "/test/path");
        assert!(profile.is_some());
        
        let rules = profile.unwrap().rules;
        assert!(rules.iter().any(|r| r.operation_type == "file_read_all"));
        assert!(rules.iter().any(|r| r.operation_type == "network_outbound"));
        assert!(!rules.iter().any(|r| r.operation_type == "file_write_all"));
    }
}
```