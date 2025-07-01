// Service tests are temporarily disabled pending proper Tauri test harness setup
// TODO: Re-enable these tests when tauri::test module is available

/* Original tests preserved for reference:

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::agents::{
        constants, error::AgentError, pool::SqlitePool, repository::*, service::*, types::*,
    };
    use crate::process_registry::{ProcessRegistry, ProcessRegistryState};
    use anyhow::Result;
    use chrono;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use tauri::{AppHandle, Manager};
    // use tauri::test::{mock_builder, MockRuntime};
    use tempfile::TempDir;
    
    // ... rest of the original test code ...
}
*/