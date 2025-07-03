use std::sync::Arc;
use tokio::process::Child;
use tokio::sync::Mutex;

/// Global state to track current Claude process
pub struct ClaudeProcessState {
    pub current_process: Arc<Mutex<Option<Child>>>,
}

impl Default for ClaudeProcessState {
    fn default() -> Self {
        Self {
            current_process: Arc::new(Mutex::new(None)),
        }
    }
}