import { BaseService, ServiceConfig } from './base.service';
import { z } from 'zod';
import { 
  ProjectSchema,
  ProjectWithStatsSchema,
  CreateProjectParamsSchema,
  UpdateProjectParamsSchema,
  transformProject,
  type Project,
  type ProjectWithStats,
  type CreateProjectParams,
  type UpdateProjectParams,
} from '@/schemas/project';
import { FilePathSchema, NonEmptyStringSchema, nullable } from '@/schemas/common';
import { ClaudeError, ErrorCode } from '@/lib/errors';
import type { 
  ClaudeSettings, 
  ClaudeVersionStatus,
  ClaudeMdFile,
  FileEntry,
  ClaudeInstallation
} from '@/lib/api.types';

// Schemas for Claude-specific types
const ClaudeSettingsSchema = z.object({
  theme: z.enum(['light', 'dark', 'system']).optional(),
  autoSave: z.boolean().optional(),
  defaultProjectPath: FilePathSchema.optional(),
  claudeBinaryPath: FilePathSchema.optional(),
  enableTelemetry: z.boolean().optional(),
  customStyles: z.string().optional(),
});

const ClaudeVersionStatusSchema = z.object({
  is_installed: z.boolean(),
  version: z.string().optional(),
  output: z.string(),
});

const ClaudeMdFileSchema = z.object({
  relative_path: z.string(),
  absolute_path: FilePathSchema,
  size: z.number(),
  modified: z.number(),
});

const FileEntrySchema = z.object({
  name: z.string(),
  path: FilePathSchema,
  is_directory: z.boolean(),
  size: z.number(),
  extension: z.string().optional(),
});

const ClaudeInstallationSchema = z.object({
  path: FilePathSchema,
  version: z.string().optional(),
  source: z.string(),
});

/**
 * Service for project-related operations with type-safe validation
 */
export class ProjectService extends BaseService {
  constructor(config?: Partial<ServiceConfig>) {
    super({ 
      serviceName: 'ProjectService',
      retryConfig: {
        maxRetries: 2,
        retryDelay: 1000,
      },
      ...config
    });
  }

  /**
   * Lists all projects in the ~/.claude/projects directory
   */
  async listProjects(): Promise<Project[]> {
    const result = await this.invoke<{}, any>(
      'list_projects',
      {}
    );
    // Validate and transform
    const validated = z.array(ProjectSchema).parse(result);
    return validated.map(transformProject);
  }

  /**
   * Get project with statistics
   */
  async getProjectWithStats(projectId: string): Promise<ProjectWithStats> {
    const result = await this.invoke<{ projectId: string }, any>(
      'get_project_with_stats',
      { projectId }
    );
    return ProjectWithStatsSchema.parse(result);
  }

  /**
   * Create a new project
   */
  async createProject(params: CreateProjectParams): Promise<Project> {
    // Validate params
    const validated = CreateProjectParamsSchema.parse(params);
    
    const result = await this.invoke<CreateProjectParams, any>(
      'create_project',
      validated
    );
    const parsed = ProjectSchema.parse(result);
    return transformProject(parsed);
  }

  /**
   * Update an existing project
   */
  async updateProject(projectId: string, params: UpdateProjectParams): Promise<Project> {
    // Validate params
    const validated = UpdateProjectParamsSchema.parse(params);
    
    const result = await this.invoke<{ projectId: string } & UpdateProjectParams, any>(
      'update_project',
      { projectId, ...validated }
    );
    const parsed = ProjectSchema.parse(result);
    return transformProject(parsed);
  }

  /**
   * Delete a project
   */
  async deleteProject(projectId: string): Promise<void> {
    return this.invokeVoid<{ projectId: string }>(
      'delete_project',
      { projectId }
    );
  }

  /**
   * Reads the Claude settings file
   */
  async getClaudeSettings(): Promise<ClaudeSettings> {
    const result = await this.invoke<{}, ClaudeSettings>(
      'get_claude_settings',
      {},
      ClaudeSettingsSchema
    );
    
    return result;
  }

  /**
   * Saves the Claude settings file
   */
  async saveClaudeSettings(settings: ClaudeSettings): Promise<void> {
    // Validate settings
    const validated = ClaudeSettingsSchema.parse(settings);
    
    await this.invokeVoid<{ settings: ClaudeSettings }>(
      'save_claude_settings',
      { settings: validated }
    );
  }

  /**
   * Checks if Claude Code is installed and gets its version
   */
  async checkClaudeVersion(): Promise<ClaudeVersionStatus> {
    try {
      return await this.invoke<{}, ClaudeVersionStatus>(
        'check_claude_version',
        {},
        ClaudeVersionStatusSchema
      );
    } catch (error) {
      // Handle specific Claude not installed error
      if (error instanceof Error && error.message.includes('not found')) {
        throw new ClaudeError(
          ErrorCode.CLAUDE_NOT_INSTALLED,
          'Claude Code CLI is not installed or not in PATH'
        );
      }
      throw error;
    }
  }

  /**
   * Finds all CLAUDE.md files in a project directory
   */
  async findClaudeMdFiles(projectPath: string): Promise<ClaudeMdFile[]> {
    // Validate path
    const validatedPath = FilePathSchema.parse(projectPath);
    
    return this.invoke<{ projectPath: string }, ClaudeMdFile[]>(
      'find_claude_md_files',
      { projectPath: validatedPath },
      z.array(ClaudeMdFileSchema)
    );
  }

  /**
   * Reads a specific CLAUDE.md file
   */
  async readClaudeMdFile(filePath: string): Promise<string> {
    // Validate path
    const validatedPath = FilePathSchema.parse(filePath);
    
    return this.invoke<{ filePath: string }, string>(
      'read_claude_md_file',
      { filePath: validatedPath },
      z.string()
    );
  }

  /**
   * Saves a specific CLAUDE.md file
   */
  async saveClaudeMdFile(filePath: string, content: string): Promise<void> {
    // Validate inputs
    const validatedPath = FilePathSchema.parse(filePath);
    const validatedContent = NonEmptyStringSchema.parse(content);
    
    await this.invokeVoid<{ filePath: string; content: string }>(
      'save_claude_md_file',
      { filePath: validatedPath, content: validatedContent }
    );
  }

  /**
   * Reads the CLAUDE.md system prompt file
   */
  async getSystemPrompt(): Promise<string> {
    return this.invoke<{}, string>(
      'get_system_prompt',
      {},
      z.string()
    );
  }

  /**
   * Saves the CLAUDE.md system prompt file
   */
  async saveSystemPrompt(content: string): Promise<void> {
    // Validate content
    const validatedContent = NonEmptyStringSchema.parse(content);
    
    await this.invokeVoid<{ content: string }>(
      'save_system_prompt',
      { content: validatedContent }
    );
  }

  /**
   * Lists files and directories in a given path
   */
  async listDirectoryContents(directoryPath: string): Promise<FileEntry[]> {
    // Validate path
    const validatedPath = FilePathSchema.parse(directoryPath);
    
    return this.invoke<{ directoryPath: string }, FileEntry[]>(
      'list_directory_contents',
      { directoryPath: validatedPath },
      z.array(FileEntrySchema)
    );
  }

  /**
   * Searches for files and directories matching a pattern
   */
  async searchFiles(basePath: string, query: string): Promise<FileEntry[]> {
    // Validate inputs
    const validatedPath = FilePathSchema.parse(basePath);
    const validatedQuery = NonEmptyStringSchema.parse(query);
    
    return this.invoke<{ basePath: string; query: string }, FileEntry[]>(
      'search_files',
      { basePath: validatedPath, query: validatedQuery },
      z.array(FileEntrySchema)
    );
  }

  /**
   * Get the stored Claude binary path from settings
   */
  async getClaudeBinaryPath(): Promise<string | null> {
    return this.invoke<{}, string | null>(
      'get_claude_binary_path',
      {},
      nullable(FilePathSchema)
    );
  }

  /**
   * Set the Claude binary path in settings
   */
  async setClaudeBinaryPath(path: string): Promise<void> {
    // Validate path
    const validatedPath = FilePathSchema.parse(path);
    
    await this.invokeVoid<{ path: string }>(
      'set_claude_binary_path',
      { path: validatedPath }
    );
  }

  /**
   * List all available Claude installations on the system
   */
  async listClaudeInstallations(): Promise<ClaudeInstallation[]> {
    return this.invoke<{}, ClaudeInstallation[]>(
      'list_claude_installations',
      {},
      z.array(ClaudeInstallationSchema)
    );
  }

  /**
   * Captures a screenshot of a specific region in the window
   */
  async captureUrlScreenshot(
    url: string,
    selector?: string | null,
    fullPage: boolean = false
  ): Promise<string> {
    // Validate inputs
    const validatedUrl = z.string().url().parse(url);
    const validatedSelector = selector ? NonEmptyStringSchema.parse(selector) : null;
    
    return this.invoke<{ url: string; selector: string | null; fullPage: boolean }, string>(
      'capture_url_screenshot',
      { 
        url: validatedUrl, 
        selector: validatedSelector, 
        fullPage 
      },
      FilePathSchema
    );
  }

  /**
   * Cleans up old screenshot files from the temporary directory
   */
  async cleanupScreenshotTempFiles(olderThanMinutes: number = 60): Promise<number> {
    // Validate input
    const validatedMinutes = z.number().int().positive().parse(olderThanMinutes);
    
    return this.invoke<{ olderThanMinutes: number }, number>(
      'cleanup_screenshot_temp_files',
      { olderThanMinutes: validatedMinutes },
      z.number().int().nonnegative()
    );
  }

  // Add cached version of frequently called methods
  checkClaudeVersionCached = this.createCachedMethod(
    this.checkClaudeVersion.bind(this),
    () => 'claude-version',
    300000 // 5 minutes
  );

  getClaudeSettingsCached = this.createCachedMethod(
    this.getClaudeSettings.bind(this),
    () => 'claude-settings',
    60000 // 1 minute
  );
}

// Export singleton instance
export const projectService = new ProjectService();