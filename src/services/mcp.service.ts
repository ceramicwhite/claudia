import { BaseService, ServiceConfig } from './base.service';
import { TAURI_COMMANDS, ERROR_MESSAGES } from '@/constants';
import type { 
  MCPServer,
  AddServerResult,
  ImportResult,
  ServerStatus,
  MCPProjectConfig
} from '@/lib/api.types';

/**
 * Service for MCP server operations
 */
export class MCPService extends BaseService {
  constructor(config?: Partial<ServiceConfig>) {
    super({
      serviceName: 'MCPService',
      enableLogging: true,
      ...config
    });
  }

  /**
   * Adds a new MCP server
   */
  async mcpAdd(
    name: string,
    transport: string,
    command?: string,
    args: string[] = [],
    env: Record<string, string> = {},
    url?: string,
    scope: string = "local"
  ): Promise<AddServerResult> {
    return this.invoke<{
      name: string;
      transport: string;
      command?: string;
      args: string[];
      env: Record<string, string>;
      url?: string;
      scope: string;
    }, AddServerResult>(
      TAURI_COMMANDS.MCP_ADD,
      { name, transport, command, args, env, url, scope },
      ERROR_MESSAGES.FAILED_TO_ADD_MCP_SERVER
    );
  }

  /**
   * Lists all configured MCP servers
   */
  async mcpList(): Promise<MCPServer[]> {
    return this.invokeNoArgs<MCPServer[]>(TAURI_COMMANDS.MCP_LIST, ERROR_MESSAGES.FAILED_TO_LIST_MCP_SERVERS);
  }

  /**
   * Gets details for a specific MCP server
   */
  async mcpGet(name: string): Promise<MCPServer> {
    return this.invoke<{ name: string }, MCPServer>(
      TAURI_COMMANDS.MCP_GET,
      { name },
      ERROR_MESSAGES.FAILED_TO_GET_MCP_SERVER
    );
  }

  /**
   * Removes an MCP server
   */
  async mcpRemove(name: string): Promise<string> {
    return this.invoke<{ name: string }, string>(
      TAURI_COMMANDS.MCP_REMOVE,
      { name },
      ERROR_MESSAGES.FAILED_TO_REMOVE_MCP_SERVER
    );
  }

  /**
   * Adds an MCP server from JSON configuration
   */
  async mcpAddJson(name: string, jsonConfig: string, scope: string = "local"): Promise<AddServerResult> {
    return this.invoke<{ name: string; jsonConfig: string; scope: string }, AddServerResult>(
      TAURI_COMMANDS.MCP_ADD_JSON,
      { name, jsonConfig, scope },
      ERROR_MESSAGES.FAILED_TO_ADD_MCP_JSON
    );
  }

  /**
   * Imports MCP servers from Claude Desktop
   */
  async mcpAddFromClaudeDesktop(scope: string = "local"): Promise<ImportResult> {
    return this.invoke<{ scope: string }, ImportResult>(
      TAURI_COMMANDS.MCP_ADD_FROM_CLAUDE_DESKTOP,
      { scope },
      ERROR_MESSAGES.FAILED_TO_IMPORT_FROM_CLAUDE_DESKTOP
    );
  }

  /**
   * Starts Claude Code as an MCP server
   */
  async mcpServe(): Promise<string> {
    return this.invokeNoArgs<string>(TAURI_COMMANDS.MCP_SERVE, ERROR_MESSAGES.FAILED_TO_START_MCP_SERVER);
  }

  /**
   * Tests connection to an MCP server
   */
  async mcpTestConnection(name: string): Promise<string> {
    return this.invoke<{ name: string }, string>(
      TAURI_COMMANDS.MCP_TEST_CONNECTION,
      { name },
      ERROR_MESSAGES.FAILED_TO_TEST_MCP_CONNECTION
    );
  }

  /**
   * Resets project-scoped server approval choices
   */
  async mcpResetProjectChoices(): Promise<string> {
    return this.invokeNoArgs<string>(TAURI_COMMANDS.MCP_RESET_PROJECT_CHOICES, ERROR_MESSAGES.FAILED_TO_RESET_PROJECT_CHOICES);
  }

  /**
   * Gets the status of MCP servers
   */
  async mcpGetServerStatus(): Promise<Record<string, ServerStatus>> {
    return this.invoke<undefined, Record<string, ServerStatus>>(
      TAURI_COMMANDS.MCP_GET_SERVER_STATUS,
      undefined,
      ERROR_MESSAGES.FAILED_TO_GET_SERVER_STATUS
    );
  }

  /**
   * Reads .mcp.json from the current project
   */
  async mcpReadProjectConfig(projectPath: string): Promise<MCPProjectConfig> {
    return this.invoke<{ projectPath: string }, MCPProjectConfig>(
      TAURI_COMMANDS.MCP_READ_PROJECT_CONFIG,
      { projectPath },
      ERROR_MESSAGES.FAILED_TO_READ_PROJECT_CONFIG
    );
  }

  /**
   * Saves .mcp.json to the current project
   */
  async mcpSaveProjectConfig(projectPath: string, config: MCPProjectConfig): Promise<string> {
    return this.invoke<{ projectPath: string; config: MCPProjectConfig }, string>(
      TAURI_COMMANDS.MCP_SAVE_PROJECT_CONFIG,
      { projectPath, config },
      ERROR_MESSAGES.FAILED_TO_SAVE_PROJECT_CONFIG
    );
  }
}

// Export singleton instance
export const mcpService = new MCPService();