/// Claude command modules
pub mod checkpoints;
pub mod execution;
pub mod files;
pub mod projects;
pub mod sessions;
pub mod settings;
pub mod state;
pub mod types;
pub mod utils;
pub mod version;

// Re-export state
pub use state::ClaudeProcessState;

// Re-export types
pub use types::*;

// Re-export all command functions from projects
pub use projects::{get_project_sessions, list_projects};

// Re-export all command functions from sessions
pub use sessions::{load_session_history, open_new_session};

// Re-export all command functions from settings
pub use settings::{get_claude_settings, get_system_prompt, save_claude_settings, save_system_prompt};

// Re-export all command functions from version
pub use version::check_claude_version;

// Re-export all command functions from files
pub use files::{
    find_claude_md_files, list_directory_contents, read_claude_md_file, save_claude_md_file,
    search_files,
};

// Re-export all command functions from execution
pub use execution::{
    cancel_claude_execution, continue_claude_code, execute_claude_code, resume_claude_code,
};

// Re-export all command functions from checkpoints
pub use checkpoints::{
    check_auto_checkpoint, cleanup_old_checkpoints, clear_checkpoint_manager, create_checkpoint,
    fork_from_checkpoint, get_checkpoint_diff, get_checkpoint_settings, get_checkpoint_state_stats,
    get_recently_modified_files, get_session_timeline, list_checkpoints, restore_checkpoint,
    track_checkpoint_message, track_session_messages, update_checkpoint_settings,
};

