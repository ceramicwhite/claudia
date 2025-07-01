/**
 * Constants for the Claudia application
 * This file contains all shared constants to avoid magic strings
 */

// Agent run status constants
export enum AgentRunStatus {
  PENDING = 'pending',
  RUNNING = 'running',
  COMPLETED = 'completed',
  FAILED = 'failed',
  CANCELLED = 'cancelled',
  SCHEDULED = 'scheduled',
  PAUSED_USAGE_LIMIT = 'paused_usage_limit'
}

// Model constants
export enum ClaudeModel {
  SONNET = 'sonnet',
  OPUS = 'opus',
  HAIKU = 'haiku'
}

// Event name constants
export const TAURI_EVENTS = {
  CLAUDE_OUTPUT: 'claude-output',
  CLAUDE_SESSION_SELECTED: 'claude-session-selected',
  CLAUDE_EXECUTION_CANCELLED: 'claude-execution-cancelled',
  CLAUDE_EXECUTION_COMPLETED: 'claude-execution-completed',
  CLAUDE_EXECUTION_STARTED: 'claude-execution-started',
  AGENT_OUTPUT: 'agent-output',
  AGENT_EXECUTION_COMPLETED: 'agent-execution-completed',
  AGENT_EXECUTION_FAILED: 'agent-execution-failed',
  AGENT_EXECUTION_CANCELLED: 'agent-execution-cancelled'
} as const;

// Command name constants
export const TAURI_COMMANDS = {
  // Project commands
  LIST_PROJECTS: 'list_projects',
  GET_PROJECT_SESSIONS: 'get_project_sessions',
  
  // Agent commands
  LIST_AGENTS: 'list_agents',
  CREATE_AGENT: 'create_agent',
  UPDATE_AGENT: 'update_agent',
  DELETE_AGENT: 'delete_agent',
  GET_AGENT: 'get_agent',
  EXPORT_AGENT: 'export_agent',
  IMPORT_AGENT: 'import_agent',
  IMPORT_AGENT_FROM_FILE: 'import_agent_from_file',
  EXECUTE_AGENT: 'execute_agent',
  CREATE_SCHEDULED_AGENT_RUN: 'create_scheduled_agent_run',
  GET_SCHEDULED_AGENT_RUNS: 'get_scheduled_agent_runs',
  CANCEL_SCHEDULED_AGENT_RUN: 'cancel_scheduled_agent_run',
  LIST_AGENT_RUNS: 'list_agent_runs',
  GET_AGENT_RUN: 'get_agent_run',
  GET_AGENT_RUN_WITH_REAL_TIME_METRICS: 'get_agent_run_with_real_time_metrics',
  LIST_RUNNING_SESSIONS: 'list_running_sessions',
  LIST_RUNNING_SESSIONS_WITH_METRICS: 'list_running_sessions_with_metrics',
  RESUME_AGENT: 'resume_agent',
  KILL_AGENT_SESSION: 'kill_agent_session',
  GET_SESSION_STATUS: 'get_session_status',
  CLEANUP_FINISHED_PROCESSES: 'cleanup_finished_processes',
  GET_SESSION_OUTPUT: 'get_session_output',
  GET_AGENT_RUN_OUTPUT: 'get_agent_run_output',
  GET_LIVE_SESSION_OUTPUT: 'get_live_session_output',
  STREAM_SESSION_OUTPUT: 'stream_session_output',
  FETCH_GITHUB_AGENTS: 'fetch_github_agents',
  FETCH_GITHUB_AGENT_CONTENT: 'fetch_github_agent_content',
  IMPORT_AGENT_FROM_GITHUB: 'import_agent_from_github',
  
  // Claude commands
  GET_CLAUDE_SETTINGS: 'get_claude_settings',
  SAVE_CLAUDE_SETTINGS: 'save_claude_settings',
  OPEN_NEW_SESSION: 'open_new_session',
  GET_SYSTEM_PROMPT: 'get_system_prompt',
  SAVE_SYSTEM_PROMPT: 'save_system_prompt',
  CHECK_CLAUDE_VERSION: 'check_claude_version',
  FIND_CLAUDE_MD_FILES: 'find_claude_md_files',
  READ_CLAUDE_MD_FILE: 'read_claude_md_file',
  SAVE_CLAUDE_MD_FILE: 'save_claude_md_file',
  LOAD_SESSION_HISTORY: 'load_session_history',
  EXECUTE_CLAUDE_CODE: 'execute_claude_code',
  CONTINUE_CLAUDE_CODE: 'continue_claude_code',
  RESUME_CLAUDE_CODE: 'resume_claude_code',
  CANCEL_CLAUDE_EXECUTION: 'cancel_claude_execution',
  LIST_DIRECTORY_CONTENTS: 'list_directory_contents',
  SEARCH_FILES: 'search_files',
  GET_CLAUDE_BINARY_PATH: 'get_claude_binary_path',
  SET_CLAUDE_BINARY_PATH: 'set_claude_binary_path',
  LIST_CLAUDE_INSTALLATIONS: 'list_claude_installations',
  
  // Sandbox commands
  LIST_SANDBOX_PROFILES: 'list_sandbox_profiles',
  CREATE_SANDBOX_PROFILE: 'create_sandbox_profile',
  UPDATE_SANDBOX_PROFILE: 'update_sandbox_profile',
  DELETE_SANDBOX_PROFILE: 'delete_sandbox_profile',
  GET_SANDBOX_PROFILE: 'get_sandbox_profile',
  LIST_SANDBOX_RULES: 'list_sandbox_rules',
  CREATE_SANDBOX_RULE: 'create_sandbox_rule',
  UPDATE_SANDBOX_RULE: 'update_sandbox_rule',
  DELETE_SANDBOX_RULE: 'delete_sandbox_rule',
  GET_PLATFORM_CAPABILITIES: 'get_platform_capabilities',
  TEST_SANDBOX_PROFILE: 'test_sandbox_profile',
  LIST_SANDBOX_VIOLATIONS: 'list_sandbox_violations',
  LOG_SANDBOX_VIOLATION: 'log_sandbox_violation',
  CLEAR_SANDBOX_VIOLATIONS: 'clear_sandbox_violations',
  GET_SANDBOX_VIOLATION_STATS: 'get_sandbox_violation_stats',
  EXPORT_SANDBOX_PROFILE: 'export_sandbox_profile',
  EXPORT_ALL_SANDBOX_PROFILES: 'export_all_sandbox_profiles',
  IMPORT_SANDBOX_PROFILES: 'import_sandbox_profiles',
  
  // Usage commands
  GET_USAGE_STATS: 'get_usage_stats',
  GET_USAGE_BY_DATE_RANGE: 'get_usage_by_date_range',
  GET_SESSION_STATS: 'get_session_stats',
  GET_USAGE_DETAILS: 'get_usage_details',
  
  // Checkpoint commands
  CREATE_CHECKPOINT: 'create_checkpoint',
  RESTORE_CHECKPOINT: 'restore_checkpoint',
  LIST_CHECKPOINTS: 'list_checkpoints',
  FORK_FROM_CHECKPOINT: 'fork_from_checkpoint',
  GET_SESSION_TIMELINE: 'get_session_timeline',
  UPDATE_CHECKPOINT_SETTINGS: 'update_checkpoint_settings',
  GET_CHECKPOINT_DIFF: 'get_checkpoint_diff',
  TRACK_CHECKPOINT_MESSAGE: 'track_checkpoint_message',
  CHECK_AUTO_CHECKPOINT: 'check_auto_checkpoint',
  CLEANUP_OLD_CHECKPOINTS: 'cleanup_old_checkpoints',
  GET_CHECKPOINT_SETTINGS: 'get_checkpoint_settings',
  CLEAR_CHECKPOINT_MANAGER: 'clear_checkpoint_manager',
  TRACK_SESSION_MESSAGES: 'track_session_messages',
  
  // MCP commands
  MCP_ADD: 'mcp_add',
  MCP_LIST: 'mcp_list',
  MCP_GET: 'mcp_get',
  MCP_REMOVE: 'mcp_remove',
  MCP_ADD_JSON: 'mcp_add_json',
  MCP_ADD_FROM_CLAUDE_DESKTOP: 'mcp_add_from_claude_desktop',
  MCP_SERVE: 'mcp_serve',
  MCP_TEST_CONNECTION: 'mcp_test_connection',
  MCP_RESET_PROJECT_CHOICES: 'mcp_reset_project_choices',
  MCP_GET_SERVER_STATUS: 'mcp_get_server_status',
  MCP_READ_PROJECT_CONFIG: 'mcp_read_project_config',
  MCP_SAVE_PROJECT_CONFIG: 'mcp_save_project_config',
  
  // Screenshot commands
  CAPTURE_URL_SCREENSHOT: 'capture_url_screenshot',
  CLEANUP_SCREENSHOT_TEMP_FILES: 'cleanup_screenshot_temp_files'
} as const;

// Error messages
export const ERROR_MESSAGES = {
  FAILED_TO_LIST_PROJECTS: 'Failed to list projects',
  FAILED_TO_GET_PROJECT_SESSIONS: 'Failed to get project sessions',
  FAILED_TO_LIST_AGENTS: 'Failed to list agents',
  FAILED_TO_CREATE_AGENT: 'Failed to create agent',
  FAILED_TO_UPDATE_AGENT: 'Failed to update agent',
  FAILED_TO_DELETE_AGENT: 'Failed to delete agent',
  FAILED_TO_GET_AGENT: 'Failed to get agent',
  FAILED_TO_EXPORT_AGENT: 'Failed to export agent',
  FAILED_TO_IMPORT_AGENT: 'Failed to import agent',
  FAILED_TO_EXECUTE_AGENT: 'Failed to execute agent',
  FAILED_TO_CREATE_SCHEDULED_RUN: 'Failed to create scheduled agent run',
  FAILED_TO_GET_SCHEDULED_RUNS: 'Failed to get scheduled agent runs',
  FAILED_TO_CANCEL_SCHEDULED_RUN: 'Failed to cancel scheduled agent run',
  FAILED_TO_LIST_AGENT_RUNS: 'Failed to list agent runs',
  FAILED_TO_GET_AGENT_RUN: 'Failed to get agent run',
  FAILED_TO_GET_AGENT_RUN_WITH_METRICS: 'Failed to get agent run with real-time metrics',
  FAILED_TO_LIST_RUNNING_SESSIONS: 'Failed to list running agent sessions',
  FAILED_TO_RESUME_AGENT: 'Failed to resume agent',
  FAILED_TO_KILL_SESSION: 'Failed to kill agent session',
  FAILED_TO_GET_SESSION_STATUS: 'Failed to get session status',
  FAILED_TO_CLEANUP_PROCESSES: 'Failed to cleanup finished processes',
  FAILED_TO_GET_SESSION_OUTPUT: 'Failed to get session output',
  FAILED_TO_GET_AGENT_RUN_OUTPUT: 'Failed to get agent run output',
  FAILED_TO_GET_LIVE_OUTPUT: 'Failed to get live session output',
  FAILED_TO_START_STREAMING: 'Failed to start streaming session output',
  FAILED_TO_FETCH_GITHUB_AGENTS: 'Failed to fetch GitHub agents',
  FAILED_TO_FETCH_GITHUB_CONTENT: 'Failed to fetch GitHub agent content',
  FAILED_TO_IMPORT_FROM_GITHUB: 'Failed to import agent from GitHub',
  FAILED_TO_GET_CLAUDE_SETTINGS: 'Failed to get Claude settings',
  FAILED_TO_SAVE_CLAUDE_SETTINGS: 'Failed to save Claude settings',
  FAILED_TO_OPEN_NEW_SESSION: 'Failed to open new session',
  FAILED_TO_GET_SYSTEM_PROMPT: 'Failed to get system prompt',
  FAILED_TO_SAVE_SYSTEM_PROMPT: 'Failed to save system prompt',
  FAILED_TO_CHECK_CLAUDE_VERSION: 'Failed to check Claude version',
  FAILED_TO_FIND_CLAUDE_MD_FILES: 'Failed to find CLAUDE.md files',
  FAILED_TO_READ_CLAUDE_MD_FILE: 'Failed to read CLAUDE.md file',
  FAILED_TO_SAVE_CLAUDE_MD_FILE: 'Failed to save CLAUDE.md file',
  FAILED_TO_LIST_SANDBOX_PROFILES: 'Failed to list sandbox profiles',
  FAILED_TO_CREATE_SANDBOX_PROFILE: 'Failed to create sandbox profile',
  FAILED_TO_UPDATE_SANDBOX_PROFILE: 'Failed to update sandbox profile',
  FAILED_TO_DELETE_SANDBOX_PROFILE: 'Failed to delete sandbox profile',
  FAILED_TO_GET_SANDBOX_PROFILE: 'Failed to get sandbox profile',
  FAILED_TO_LIST_SANDBOX_RULES: 'Failed to list sandbox rules',
  FAILED_TO_CREATE_SANDBOX_RULE: 'Failed to create sandbox rule',
  FAILED_TO_UPDATE_SANDBOX_RULE: 'Failed to update sandbox rule',
  FAILED_TO_DELETE_SANDBOX_RULE: 'Failed to delete sandbox rule',
  FAILED_TO_GET_PLATFORM_CAPABILITIES: 'Failed to get platform capabilities',
  FAILED_TO_TEST_SANDBOX_PROFILE: 'Failed to test sandbox profile',
  FAILED_TO_LIST_SANDBOX_VIOLATIONS: 'Failed to list sandbox violations',
  FAILED_TO_LOG_SANDBOX_VIOLATION: 'Failed to log sandbox violation',
  FAILED_TO_CLEAR_SANDBOX_VIOLATIONS: 'Failed to clear sandbox violations',
  FAILED_TO_GET_SANDBOX_VIOLATION_STATS: 'Failed to get sandbox violation stats',
  FAILED_TO_EXPORT_SANDBOX_PROFILE: 'Failed to export sandbox profile',
  FAILED_TO_EXPORT_ALL_SANDBOX_PROFILES: 'Failed to export all sandbox profiles',
  FAILED_TO_IMPORT_SANDBOX_PROFILES: 'Failed to import sandbox profiles',
  FAILED_TO_GET_USAGE_STATS: 'Failed to get usage stats',
  FAILED_TO_GET_USAGE_BY_DATE_RANGE: 'Failed to get usage by date range',
  FAILED_TO_GET_SESSION_STATS: 'Failed to get session stats',
  FAILED_TO_GET_USAGE_DETAILS: 'Failed to get usage details',
  FAILED_TO_TRACK_CHECKPOINT_MESSAGE: 'Failed to track checkpoint message',
  FAILED_TO_CHECK_AUTO_CHECKPOINT: 'Failed to check auto checkpoint',
  FAILED_TO_CLEANUP_OLD_CHECKPOINTS: 'Failed to cleanup old checkpoints',
  FAILED_TO_GET_CHECKPOINT_SETTINGS: 'Failed to get checkpoint settings',
  FAILED_TO_CLEAR_CHECKPOINT_MANAGER: 'Failed to clear checkpoint manager',
  FAILED_TO_GET_CHECKPOINT_DIFF: 'Failed to get checkpoint diff',
  FAILED_TO_ADD_MCP_SERVER: 'Failed to add MCP server',
  FAILED_TO_LIST_MCP_SERVERS: 'Failed to list MCP servers',
  FAILED_TO_GET_MCP_SERVER: 'Failed to get MCP server',
  FAILED_TO_REMOVE_MCP_SERVER: 'Failed to remove MCP server',
  FAILED_TO_ADD_MCP_JSON: 'Failed to add MCP server from JSON',
  FAILED_TO_IMPORT_FROM_CLAUDE_DESKTOP: 'Failed to import from Claude Desktop',
  FAILED_TO_START_MCP_SERVER: 'Failed to start MCP server',
  FAILED_TO_TEST_MCP_CONNECTION: 'Failed to test MCP connection',
  FAILED_TO_RESET_PROJECT_CHOICES: 'Failed to reset project choices',
  FAILED_TO_GET_SERVER_STATUS: 'Failed to get server status',
  FAILED_TO_READ_PROJECT_CONFIG: 'Failed to read project MCP config',
  FAILED_TO_SAVE_PROJECT_CONFIG: 'Failed to save project MCP config',
  FAILED_TO_GET_CLAUDE_BINARY_PATH: 'Failed to get Claude binary path',
  FAILED_TO_SET_CLAUDE_BINARY_PATH: 'Failed to set Claude binary path',
  FAILED_TO_CLEANUP_SCREENSHOT_FILES: 'Failed to cleanup screenshot files',
  FAILED_TO_LIST_CLAUDE_INSTALLATIONS: 'Failed to list Claude installations',
  UNKNOWN_ERROR: 'Unknown error'
} as const;

// Priority levels
export enum Priority {
  HIGH = 'high',
  MEDIUM = 'medium', 
  LOW = 'low'
}

// Todo status
export enum TodoStatus {
  COMPLETED = 'completed',
  IN_PROGRESS = 'in_progress',
  PENDING = 'pending'
}