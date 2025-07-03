// Sandbox API types
export interface SandboxProfile {
  id?: number;
  name: string;
  description?: string;
  is_active: boolean;
  is_default: boolean;
  created_at: string;
  updated_at: string;
}

export interface SandboxRule {
  id?: number;
  profile_id: number;
  operation_type: string;
  pattern_type: string;
  pattern_value: string;
  enabled: boolean;
  platform_support?: string;
  created_at: string;
}

export interface PlatformCapabilities {
  os: string;
  sandboxing_supported: boolean;
  operations: OperationSupport[];
  notes: string[];
}

export interface OperationSupport {
  operation: string;
  support_level: string;
  description: string;
}

// Sandbox violation types
export interface SandboxViolation {
  id?: number;
  profile_id?: number;
  agent_id?: number;
  agent_run_id?: number;
  operation_type: string;
  pattern_value?: string;
  process_name?: string;
  pid?: number;
  denied_at: string;
}

export interface SandboxViolationStats {
  total: number;
  recent_24h: number;
  by_operation: Array<{
    operation: string;
    count: number;
  }>;
}

// Import/Export types
export interface SandboxProfileExport {
  version: number;
  exported_at: string;
  platform: string;
  profiles: SandboxProfileWithRules[];
}

export interface SandboxProfileWithRules {
  profile: SandboxProfile;
  rules: SandboxRule[];
}

export interface SandboxImportResult {
  profile_name: string;
  imported: boolean;
  reason?: string;
  new_name?: string;
}