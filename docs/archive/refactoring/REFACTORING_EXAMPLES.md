# Detailed Refactoring Examples

## 1. Extract Magic Strings to Enums

### Current Code (agents.rs)
```rust
// Line 50: Magic strings in struct
pub status: String,  // 'pending', 'running', 'completed', 'failed', 'cancelled', 'scheduled', 'paused_usage_limit'

// Line 888: String literal usage
.unwrap_or_else(|_| "pending".to_string()),

// Line 2076: String comparison
if status != "running" {
```

### Refactored Code
```rust
// Create enum in src-tauri/src/agents/models.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Scheduled,
    #[serde(rename = "paused_usage_limit")]
    PausedUsageLimit,
}

impl Default for RunStatus {
    fn default() -> Self {
        RunStatus::Pending
    }
}

// Update struct
pub struct AgentRun {
    // ...
    pub status: RunStatus,
    // ...
}

// Update usage
.unwrap_or_else(|_| RunStatus::default()),

// Comparison
if status != RunStatus::Running {
```

## 2. Split Large execute_agent Function

### Current Code Structure (agents.rs lines ~850-1745)
```rust
#[tauri::command]
pub async fn execute_agent(
    app_handle: AppHandle,
    db: State<'_, AgentDb>,
    // ... many parameters
) -> Result<AgentRun, String> {
    // 850+ lines of mixed concerns:
    // - Database queries
    // - Directory creation
    // - Environment setup
    // - Process spawning
    // - Output streaming
    // - Error handling
}
```

### Refactored Code Structure
```rust
// src-tauri/src/agents/execution.rs
pub struct AgentExecutor {
    db: Arc<AgentDb>,
    process_registry: Arc<ProcessRegistry>,
    app_handle: AppHandle,
}

impl AgentExecutor {
    pub async fn execute(
        &self,
        agent_id: i64,
        task: String,
        project_path: String,
    ) -> Result<AgentRun, AgentError> {
        let agent = self.fetch_agent(agent_id).await?;
        let execution_context = self.prepare_execution_context(&agent, &task, &project_path)?;
        let process_handle = self.spawn_process(&execution_context).await?;
        let agent_run = self.create_agent_run(&agent, &execution_context, process_handle.pid)?;
        
        self.monitor_execution(process_handle, agent_run.clone()).await?;
        Ok(agent_run)
    }
    
    async fn fetch_agent(&self, agent_id: i64) -> Result<Agent, AgentError> {
        // Extract database query logic
    }
    
    fn prepare_execution_context(
        &self,
        agent: &Agent,
        task: &str,
        project_path: &str,
    ) -> Result<ExecutionContext, AgentError> {
        // Extract environment setup logic
    }
    
    async fn spawn_process(
        &self,
        context: &ExecutionContext,
    ) -> Result<ProcessHandle, AgentError> {
        // Extract process spawning logic
    }
    
    async fn monitor_execution(
        &self,
        process: ProcessHandle,
        agent_run: AgentRun,
    ) -> Result<(), AgentError> {
        // Extract monitoring logic
    }
}

// Tauri command becomes thin wrapper
#[tauri::command]
pub async fn execute_agent(
    app_handle: AppHandle,
    db: State<'_, AgentDb>,
    registry: State<'_, ProcessRegistryState>,
    agent_id: i64,
    task: String,
    project_path: String,
) -> Result<AgentRun, String> {
    let executor = AgentExecutor {
        db: Arc::new(db.inner().clone()),
        process_registry: Arc::new(registry.inner().clone()),
        app_handle,
    };
    
    executor.execute(agent_id, task, project_path)
        .await
        .map_err(|e| e.to_string())
}
```

## 3. Fix Duplicated Pricing Logic

### Current Code (duplicated in agents.rs and usage.rs)
```rust
// agents.rs lines 171-214
const OPUS_4_INPUT_PRICE: f64 = 15.0;
const OPUS_4_OUTPUT_PRICE: f64 = 75.0;
// ... more constants

let (input_price, output_price, cache_write_price, cache_read_price) =
    if model.contains("opus-4") || model.contains("claude-opus-4") {
        (OPUS_4_INPUT_PRICE, OPUS_4_OUTPUT_PRICE, ...)
    } else if model.contains("sonnet-4") || model.contains("claude-sonnet-4") {
        (SONNET_4_INPUT_PRICE, SONNET_4_OUTPUT_PRICE, ...)
    } else {
        // Default to sonnet pricing
    };
```

### Refactored Code
```rust
// src-tauri/src/pricing/mod.rs
use once_cell::sync::Lazy;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct ModelPricing {
    pub input_per_million: f64,
    pub output_per_million: f64,
    pub cache_write_per_million: f64,
    pub cache_read_per_million: f64,
}

pub static MODEL_PRICING: Lazy<HashMap<&'static str, ModelPricing>> = Lazy::new(|| {
    let mut map = HashMap::new();
    
    map.insert("opus-4", ModelPricing {
        input_per_million: 15.0,
        output_per_million: 75.0,
        cache_write_per_million: 18.75,
        cache_read_per_million: 1.50,
    });
    
    map.insert("sonnet-4", ModelPricing {
        input_per_million: 3.0,
        output_per_million: 15.0,
        cache_write_per_million: 3.75,
        cache_read_per_million: 0.30,
    });
    
    map
});

pub fn get_model_pricing(model: &str) -> &ModelPricing {
    MODEL_PRICING
        .iter()
        .find(|(key, _)| model.contains(key))
        .map(|(_, pricing)| pricing)
        .unwrap_or(&MODEL_PRICING["sonnet-4"]) // Default
}

pub fn calculate_cost(
    model: &str,
    input_tokens: i64,
    output_tokens: i64,
    cache_creation_tokens: i64,
    cache_read_tokens: i64,
) -> f64 {
    let pricing = get_model_pricing(model);
    
    let tokens_to_millions = |tokens: i64| tokens as f64 / 1_000_000.0;
    
    tokens_to_millions(input_tokens) * pricing.input_per_million +
    tokens_to_millions(output_tokens) * pricing.output_per_million +
    tokens_to_millions(cache_creation_tokens) * pricing.cache_write_per_million +
    tokens_to_millions(cache_read_tokens) * pricing.cache_read_per_million
}
```

## 4. Replace Blocking I/O with Async

### Current Code (agents.rs)
```rust
// Line ~2160
std::fs::create_dir_all(&output_dir)
    .map_err(|e| format!("Failed to create output directory: {}", e))?;

// Line ~2170
let mut file = std::fs::OpenOptions::new()
    .create(true)
    .append(true)
    .open(&output_file)
    .map_err(|e| format!("Failed to open output file: {}", e))?;
```

### Refactored Code
```rust
use tokio::fs;
use tokio::io::AsyncWriteExt;

// Create directory asynchronously
fs::create_dir_all(&output_dir)
    .await
    .map_err(|e| format!("Failed to create output directory: {}", e))?;

// Open file asynchronously
let mut file = fs::OpenOptions::new()
    .create(true)
    .append(true)
    .open(&output_file)
    .await
    .map_err(|e| format!("Failed to open output file: {}", e))?;

// Write asynchronously
file.write_all(line.as_bytes())
    .await
    .map_err(|e| format!("Failed to write to output file: {}", e))?;
```

## 5. Implement Proper Error Handling

### Current Code Pattern
```rust
// Repeated throughout codebase
.map_err(|e| e.to_string())?
.map_err(|e| format!("Failed to {}: {}", action, e))?
```

### Refactored Code
```rust
// src-tauri/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Process error: {0}")]
    Process(String),
    
    #[error("Agent not found: {id}")]
    AgentNotFound { id: i64 },
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("GitHub API error: {0}")]
    GitHubApi(#[from] reqwest::Error),
}

// Implement conversion for Tauri commands
impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        err.to_string()
    }
}

// Usage in commands
#[tauri::command]
pub async fn get_agent(
    db: State<'_, AgentDb>,
    id: i64,
) -> Result<Agent, AppError> {
    let conn = db.0.lock().unwrap();
    
    conn.query_row(
        "SELECT * FROM agents WHERE id = ?1",
        params![id],
        |row| Agent::from_row(row),
    )
    .map_err(|_| AppError::AgentNotFound { id })
}
```

## 6. Fix Deep Nesting with Early Returns

### Current Code (agents.rs line ~120)
```rust
for line in jsonl_content.lines() {
    if let Ok(json) = serde_json::from_str::<JsonValue>(line) {
        message_count += 1;
        
        if let Some(timestamp_str) = json.get("timestamp").and_then(|t| t.as_str()) {
            if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(timestamp_str) {
                let utc_time = timestamp.with_timezone(&chrono::Utc);
                if start_time.is_none() || utc_time < start_time.unwrap() {
                    start_time = Some(utc_time);
                }
                // More nested logic...
            }
        }
    }
}
```

### Refactored Code
```rust
for line in jsonl_content.lines() {
    let json = match serde_json::from_str::<JsonValue>(line) {
        Ok(json) => json,
        Err(_) => continue,
    };
    
    message_count += 1;
    
    if let Some(utc_time) = extract_timestamp(&json) {
        update_time_bounds(&mut start_time, &mut end_time, utc_time);
    }
    
    if let Some(usage) = extract_usage(&json) {
        update_token_counts(&mut total_tokens, &usage);
    }
    
    if let Some(cost) = json.get("cost").and_then(|c| c.as_f64()) {
        cost_usd += cost;
        has_cost_field = true;
    }
}

fn extract_timestamp(json: &JsonValue) -> Option<DateTime<Utc>> {
    let timestamp_str = json.get("timestamp")?.as_str()?;
    let timestamp = DateTime::parse_from_rfc3339(timestamp_str).ok()?;
    Some(timestamp.with_timezone(&Utc))
}

fn update_time_bounds(
    start_time: &mut Option<DateTime<Utc>>,
    end_time: &mut Option<DateTime<Utc>>,
    current_time: DateTime<Utc>,
) {
    *start_time = Some(start_time.map_or(current_time, |t| t.min(current_time)));
    *end_time = Some(end_time.map_or(current_time, |t| t.max(current_time)));
}
```

## 7. Implement Service Layer Pattern

### Current Architecture
```rust
// Commands directly handle all logic
#[tauri::command]
pub async fn create_agent(
    db: State<'_, AgentDb>,
    name: String,
    // ... parameters
) -> Result<Agent, String> {
    // Direct database access
    // Direct validation
    // Direct business logic
}
```

### Refactored Architecture
```rust
// src-tauri/src/services/agent_service.rs
#[async_trait]
pub trait AgentService: Send + Sync {
    async fn create(&self, request: CreateAgentRequest) -> Result<Agent, AppError>;
    async fn update(&self, id: i64, request: UpdateAgentRequest) -> Result<Agent, AppError>;
    async fn delete(&self, id: i64) -> Result<(), AppError>;
    async fn find_by_id(&self, id: i64) -> Result<Agent, AppError>;
    async fn list(&self, filter: AgentFilter) -> Result<Vec<Agent>, AppError>;
    async fn execute(&self, id: i64, task: String) -> Result<AgentRun, AppError>;
}

pub struct AgentServiceImpl {
    repository: Arc<dyn AgentRepository>,
    executor: Arc<dyn AgentExecutor>,
    validator: Arc<dyn AgentValidator>,
}

impl AgentService for AgentServiceImpl {
    async fn create(&self, request: CreateAgentRequest) -> Result<Agent, AppError> {
        // Validate request
        self.validator.validate_create(&request)?;
        
        // Apply business rules
        let agent = Agent::from_request(request)?;
        
        // Persist
        self.repository.create(agent).await
    }
    
    // ... other implementations
}

// Thin Tauri command wrapper
#[tauri::command]
pub async fn create_agent(
    service: State<'_, Arc<dyn AgentService>>,
    name: String,
    icon: String,
    system_prompt: String,
    // ... other parameters
) -> Result<Agent, String> {
    let request = CreateAgentRequest {
        name,
        icon,
        system_prompt,
        // ... map parameters
    };
    
    service.create(request)
        .await
        .map_err(|e| e.to_string())
}
```

## 8. Add Connection Pooling

### Current Code
```rust
pub struct AgentDb(pub Mutex<Connection>);
```

### Refactored Code
```rust
// src-tauri/src/db/pool.rs
use deadpool_sqlite::{Config, Pool, Runtime};

pub struct DatabasePool {
    pool: Pool,
}

impl DatabasePool {
    pub fn new(database_path: &Path) -> Result<Self, AppError> {
        let cfg = Config {
            path: database_path.to_string_lossy().into(),
            pool: Some(deadpool::managed::PoolConfig {
                max_size: 16,
                timeouts: deadpool::managed::Timeouts {
                    wait: Some(Duration::from_secs(5)),
                    ..Default::default()
                },
                ..Default::default()
            }),
        };
        
        let pool = cfg.create_pool(Runtime::Tokio1)?;
        Ok(Self { pool })
    }
    
    pub async fn with_connection<F, R>(&self, f: F) -> Result<R, AppError>
    where
        F: FnOnce(&Connection) -> Result<R, AppError>,
    {
        let conn = self.pool.get().await?;
        f(&conn)
    }
}

// Usage
let agents = db_pool.with_connection(|conn| {
    let mut stmt = conn.prepare("SELECT * FROM agents")?;
    // ... query logic
}).await?;
```