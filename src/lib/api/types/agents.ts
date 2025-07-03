// Agent API types
export interface Agent {
  id?: number;
  name: string;
  icon: string;
  system_prompt: string;
  default_task?: string;
  model: string;
  sandbox_enabled: boolean;
  enable_file_read: boolean;
  enable_file_write: boolean;
  enable_network: boolean;
  created_at: string;
  updated_at: string;
}

export interface AgentExport {
  version: number;
  exported_at: string;
  agent: {
    name: string;
    icon: string;
    system_prompt: string;
    default_task?: string;
    model: string;
    sandbox_enabled: boolean;
    enable_file_read: boolean;
    enable_file_write: boolean;
    enable_network: boolean;
  };
}

export interface GitHubAgentFile {
  name: string;
  path: string;
  download_url: string;
  size: number;
  sha: string;
}

export interface AgentRun {
  id?: number;
  agent_id: number;
  agent_name: string;
  agent_icon: string;
  task: string;
  model: string;
  project_path: string;
  session_id: string;
  status: string; // 'pending', 'running', 'completed', 'failed', 'cancelled', 'scheduled', 'paused_usage_limit'
  pid?: number;
  process_started_at?: string;
  scheduled_start_time?: string;
  created_at: string;
  completed_at?: string;
  usage_limit_reset_time?: string; // ISO 8601 datetime when usage limit resets
  auto_resume_enabled: boolean;
  resume_count: number;
  parent_run_id?: number; // ID of the original run if this is a resumed run
}

export interface AgentRunMetrics {
  duration_ms?: number;
  total_tokens?: number;
  cost_usd?: number;
  message_count?: number;
}

export interface AgentRunWithMetrics {
  id?: number;
  agent_id: number;
  agent_name: string;
  agent_icon: string;
  task: string;
  model: string;
  project_path: string;
  session_id: string;
  status: string; // 'pending', 'running', 'completed', 'failed', 'cancelled', 'scheduled', 'paused_usage_limit'
  pid?: number;
  process_started_at?: string;
  scheduled_start_time?: string;
  created_at: string;
  completed_at?: string;
  usage_limit_reset_time?: string; // ISO 8601 datetime when usage limit resets
  auto_resume_enabled: boolean;
  resume_count: number;
  parent_run_id?: number; // ID of the original run if this is a resumed run
  metrics?: AgentRunMetrics;
  output?: string; // Real-time JSONL content
}