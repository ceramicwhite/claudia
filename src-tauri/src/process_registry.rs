use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::process::Child;

/// Registry for tracking active agent processes by session ID
#[derive(Default)]
pub struct ProcessRegistry {
    processes: HashMap<String, Child>,           // session_id -> Child
    output_buffers: HashMap<String, Vec<String>>, // session_id -> output lines
}

impl ProcessRegistry {
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
            output_buffers: HashMap::new(),
        }
    }

    /// Register a new process
    pub fn register_process(&mut self, session_id: String, child: Child) {
        self.processes.insert(session_id.clone(), child);
        self.output_buffers.insert(session_id, Vec::new());
    }

    /// Unregister a process
    pub fn unregister_process(&mut self, session_id: &str) {
        self.processes.remove(session_id);
        self.output_buffers.remove(session_id);
    }

    /// Check if a process is alive
    pub fn is_process_alive(&self, session_id: &str) -> bool {
        self.processes.contains_key(session_id)
    }

    /// Kill a process
    pub fn kill_process(&mut self, session_id: &str) -> Result<(), anyhow::Error> {
        if let Some(mut child) = self.processes.remove(session_id) {
            child.start_kill()?;
            self.output_buffers.remove(session_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Process not found"))
        }
    }

    /// Get output buffer for a session
    pub fn get_output(&self, session_id: &str) -> Result<Vec<String>, anyhow::Error> {
        self.output_buffers
            .get(session_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Session not found"))
    }

    /// Add output line to buffer
    pub fn add_output(&mut self, session_id: &str, line: String) {
        if let Some(buffer) = self.output_buffers.get_mut(session_id) {
            buffer.push(line);
        }
    }

    /// Take child process (removes it from registry but keeps output buffer)
    pub fn take_child(&mut self, session_id: &str) -> Option<Child> {
        self.processes.remove(session_id)
    }
}

// Type alias for state management
pub type ProcessRegistryState = Arc<Mutex<ProcessRegistry>>;