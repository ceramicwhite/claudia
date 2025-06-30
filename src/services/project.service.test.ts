import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { z } from 'zod';

// Mock the tauri invoke function before importing anything that uses it
const mockInvoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: any[]) => mockInvoke(...args),
}));

import { ProjectService } from './project.service';
import { BaseService } from './base.service';
import { ClaudeError, ErrorCode, ValidationError, TauriError, AppError } from '@/lib/errors';
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
import { FilePathSchema, NonEmptyStringSchema } from '@/schemas/common';
import type {
  ClaudeSettings,
  ClaudeVersionStatus,
  ClaudeMdFile,
  FileEntry,
  ClaudeInstallation
} from '@/lib/api.types';

// Mock console methods to verify logging
const originalConsoleLog = console.log;
const originalConsoleError = console.error;
const mockConsoleLog = vi.fn();
const mockConsoleError = vi.fn();

describe('ProjectService', () => {
  let service: ProjectService;

  beforeEach(() => {
    vi.clearAllMocks();
    console.log = mockConsoleLog;
    console.error = mockConsoleError;
    service = new ProjectService();
  });

  afterEach(() => {
    console.log = originalConsoleLog;
    console.error = originalConsoleError;
  });

  describe('constructor', () => {
    it('should initialize with correct configuration', () => {
      expect(service).toBeInstanceOf(BaseService);
      expect(service['serviceName']).toBe('ProjectService');
      expect(service['retryConfig']).toEqual({
        maxRetries: 2,
        retryDelay: 1000,
      });
    });
  });

  describe('listProjects', () => {
    it('should return transformed projects list', async () => {
      const mockProjects = [
        {
          id: 'project-1',
          name: 'Test Project 1',
          path: '/path/to/project1',
          created_at: '2024-01-01T00:00:00Z',
          last_accessed: '2024-01-02T00:00:00Z',
          description: 'Test description',
          is_git_repo: true,
          remote_url: 'https://github.com/test/repo',
          branch: 'main',
          tags: ['tag1', 'tag2'],
          metadata: { custom: 'data' },
        },
        {
          id: 'project-2',
          name: 'Test Project 2',
          path: '/path/to/project2',
          created_at: '2024-01-03T00:00:00Z',
          last_accessed: null,
          description: null,
          is_git_repo: false,
          remote_url: null,
          branch: null,
          tags: [],
          metadata: {},
        },
      ];

      mockInvoke.mockResolvedValueOnce(mockProjects);

      const result = await service.listProjects();

      expect(mockInvoke).toHaveBeenCalledWith('list_projects', {});
      expect(result).toHaveLength(2);
      expect(result[0]).toMatchObject({
        id: 'project-1',
        name: 'Test Project 1',
        path: '/path/to/project1',
        tags: ['tag1', 'tag2'],
      });
      expect(result[1]).toMatchObject({
        id: 'project-2',
        name: 'Test Project 2',
        tags: [],
      });
    });

    it('should handle empty projects list', async () => {
      mockInvoke.mockResolvedValueOnce([]);

      const result = await service.listProjects();

      expect(result).toEqual([]);
    });

    it('should handle transformation errors', async () => {
      const invalidProject = {
        id: 'project-1',
        name: 'Test Project',
        // Missing required 'path' field
        created_at: '2024-01-01T00:00:00Z',
        is_git_repo: true,
      };

      mockInvoke.mockResolvedValueOnce([invalidProject]);

      await expect(service.listProjects()).rejects.toThrow(ValidationError);
    });
  });

  describe('getProjectWithStats', () => {
    it('should return project with statistics', async () => {
      const mockProjectWithStats: ProjectWithStats = {
        id: 'project-1',
        name: 'Test Project',
        path: '/path/to/project',
        created_at: '2024-01-01T00:00:00Z',
        last_accessed: '2024-01-02T00:00:00Z',
        description: 'Test description',
        is_git_repo: true,
        remote_url: 'https://github.com/test/repo',
        branch: 'main',
        tags: ['tag1'],
        metadata: {},
        stats: {
          total_sessions: 10,
          active_sessions: 2,
          total_tokens: 5000,
          last_session_date: '2024-01-02T00:00:00Z',
          average_session_duration: 3600,
          most_used_agent: 'default-agent',
        },
      };

      mockInvoke.mockResolvedValueOnce(mockProjectWithStats);

      const result = await service.getProjectWithStats('project-1');

      expect(mockInvoke).toHaveBeenCalledWith('get_project_with_stats', { projectId: 'project-1' });
      expect(result).toEqual(mockProjectWithStats);
      expect(result.stats.total_sessions).toBe(10);
    });

    it('should handle project not found', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Project not found'));

      await expect(service.getProjectWithStats('non-existent')).rejects.toThrow(TauriError);
    });
  });

  describe('createProject', () => {
    it('should create a new project', async () => {
      const createParams: CreateProjectParams = {
        name: 'New Project',
        path: '/path/to/new/project',
        description: 'A new test project',
        tags: ['new', 'test'],
      };

      const mockCreatedProject: Project = {
        id: 'new-project-id',
        name: 'New Project',
        path: '/path/to/new/project',
        created_at: '2024-01-01T00:00:00Z',
        last_accessed: null,
        description: 'A new test project',
        is_git_repo: false,
        remote_url: null,
        branch: null,
        tags: ['new', 'test'],
        metadata: {},
      };

      mockInvoke.mockResolvedValueOnce(mockCreatedProject);

      const result = await service.createProject(createParams);

      expect(mockInvoke).toHaveBeenCalledWith('create_project', createParams);
      // Check all fields except dates
      expect(result.id).toBe(mockCreatedProject.id);
      expect(result.name).toBe(mockCreatedProject.name);
      expect(result.path).toBe(mockCreatedProject.path);
      expect(result.description).toBe(mockCreatedProject.description);
      expect(result.tags).toEqual(mockCreatedProject.tags);
      // Check that created_at is a valid ISO date
      expect(result.created_at).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$/);
    });

    it('should validate input parameters', async () => {
      const invalidParams = {
        name: '', // Empty name should fail validation
        path: '/path/to/project',
      };

      await expect(service.createProject(invalidParams as CreateProjectParams)).rejects.toThrow(z.ZodError);
    });

    it('should handle creation errors', async () => {
      const createParams: CreateProjectParams = {
        name: 'New Project',
        path: '/invalid/path',
      };

      mockInvoke.mockRejectedValueOnce(new Error('Failed to create project'));

      await expect(service.createProject(createParams)).rejects.toThrow(TauriError);
    });
  });

  describe('updateProject', () => {
    it('should update an existing project', async () => {
      const updateParams: UpdateProjectParams = {
        name: 'Updated Name',
        description: 'Updated description',
        tags: ['updated', 'tags'],
      };

      const mockUpdatedProject: Project = {
        id: 'project-1',
        name: 'Updated Name',
        path: '/path/to/project',
        created_at: '2024-01-01T00:00:00Z',
        last_accessed: '2024-01-02T00:00:00Z',
        description: 'Updated description',
        is_git_repo: true,
        remote_url: null,
        branch: 'main',
        tags: ['updated', 'tags'],
        metadata: {},
      };

      mockInvoke.mockResolvedValueOnce(mockUpdatedProject);

      const result = await service.updateProject('project-1', updateParams);

      expect(mockInvoke).toHaveBeenCalledWith('update_project', { projectId: 'project-1', ...updateParams });
      expect(result.name).toBe('Updated Name');
      expect(result.tags).toEqual(['updated', 'tags']);
    });

    it('should allow partial updates', async () => {
      const partialUpdate: UpdateProjectParams = {
        description: 'Only updating description',
      };

      const mockProject: Project = {
        id: 'project-1',
        name: 'Original Name',
        path: '/path/to/project',
        created_at: '2024-01-01T00:00:00Z',
        last_accessed: null,
        description: 'Only updating description',
        is_git_repo: false,
        remote_url: null,
        branch: null,
        tags: [],
        metadata: {},
      };

      mockInvoke.mockResolvedValueOnce(mockProject);

      const result = await service.updateProject('project-1', partialUpdate);

      expect(mockInvoke).toHaveBeenCalledWith('update_project', { projectId: 'project-1', ...partialUpdate });
      expect(result.description).toBe('Only updating description');
    });
  });

  describe('deleteProject', () => {
    it('should delete a project', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);

      await service.deleteProject('project-1');

      expect(mockInvoke).toHaveBeenCalledWith('delete_project', { projectId: 'project-1' });
    });

    it('should handle deletion errors', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Cannot delete active project'));

      await expect(service.deleteProject('active-project')).rejects.toThrow(TauriError);
    });
  });

  describe('getClaudeSettings', () => {
    it('should return Claude settings', async () => {
      const mockSettings: ClaudeSettings = {
        theme: 'dark',
        autoSave: true,
        defaultProjectPath: '/home/user/projects',
        claudeBinaryPath: '/usr/local/bin/claude',
        enableTelemetry: false,
        customStyles: '.custom { color: red; }',
      };

      mockInvoke.mockResolvedValueOnce({ data: mockSettings });

      const result = await service.getClaudeSettings();

      expect(mockInvoke).toHaveBeenCalledWith('get_claude_settings', {});
      expect(result).toEqual(mockSettings);
    });

    it('should handle empty settings', async () => {
      mockInvoke.mockResolvedValueOnce({ data: {} });

      const result = await service.getClaudeSettings();

      expect(result).toEqual({});
    });
  });

  describe('saveClaudeSettings', () => {
    it('should save valid settings', async () => {
      const settings: ClaudeSettings = {
        theme: 'light',
        autoSave: false,
        enableTelemetry: true,
      };

      mockInvoke.mockResolvedValueOnce(undefined);

      await service.saveClaudeSettings(settings);

      expect(mockInvoke).toHaveBeenCalledWith('save_claude_settings', { settings });
    });

    it('should validate settings schema', async () => {
      const invalidSettings = {
        theme: 'invalid-theme', // Should be 'light', 'dark', or 'system'
        autoSave: 'not-boolean',
      };

      await expect(service.saveClaudeSettings(invalidSettings as any)).rejects.toThrow(z.ZodError);
    });
  });

  describe('checkClaudeVersion', () => {
    it('should return version status when Claude is installed', async () => {
      const mockStatus: ClaudeVersionStatus = {
        installed: true,
        version: 'claude-code 0.1.23',
        path: '/usr/local/bin/claude',
        error: null,
      };

      mockInvoke.mockResolvedValueOnce(mockStatus);

      const result = await service.checkClaudeVersion();

      expect(mockInvoke).toHaveBeenCalledWith('check_claude_version', {});
      expect(result).toEqual(mockStatus);
      expect(result.installed).toBe(true);
    });

    it('should handle Claude not installed error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Claude not found in PATH'));

      try {
        await service.checkClaudeVersion();
        expect(true).toBe(false); // Should not reach here
      } catch (error: any) {
        expect(error).toBeInstanceOf(ClaudeError);
        expect(error.code).toBe(ErrorCode.CLAUDE_NOT_INSTALLED);
        expect(error.message).toBe('Claude Code CLI is not installed or not in PATH');
      }
    });

    it('should handle other errors', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Permission denied'));

      await expect(service.checkClaudeVersion()).rejects.toThrow(TauriError);
    });
  });

  describe('findClaudeMdFiles', () => {
    it('should find CLAUDE.md files in project', async () => {
      const mockFiles: ClaudeMdFile[] = [
        {
          path: '/project/CLAUDE.md',
          relativePath: 'CLAUDE.md',
          type: 'project',
          exists: true,
          content: '# Project instructions',
        },
        {
          path: '/project/docs/CLAUDE.md',
          relativePath: 'docs/CLAUDE.md',
          type: 'project',
          exists: true,
        },
      ];

      mockInvoke.mockResolvedValueOnce(mockFiles);

      const result = await service.findClaudeMdFiles('/project');

      expect(mockInvoke).toHaveBeenCalledWith('find_claude_md_files', { projectPath: '/project' });
      expect(result).toHaveLength(2);
      expect(result[0].type).toBe('project');
    });

    it('should validate project path', async () => {
      await expect(service.findClaudeMdFiles('')).rejects.toThrow(z.ZodError);
    });
  });

  describe('readClaudeMdFile', () => {
    it('should read file content', async () => {
      const content = '# Claude Instructions\n\n- Follow these rules...';
      mockInvoke.mockResolvedValueOnce(content);

      const result = await service.readClaudeMdFile('/project/CLAUDE.md');

      expect(mockInvoke).toHaveBeenCalledWith('read_claude_md_file', { filePath: '/project/CLAUDE.md' });
      expect(result).toBe(content);
    });

    it('should handle file not found', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('File not found'));

      await expect(service.readClaudeMdFile('/nonexistent/CLAUDE.md')).rejects.toThrow(TauriError);
    });
  });

  describe('saveClaudeMdFile', () => {
    it('should save file content', async () => {
      const content = '# Updated Instructions';
      mockInvoke.mockResolvedValueOnce(undefined);

      await service.saveClaudeMdFile('/project/CLAUDE.md', content);

      expect(mockInvoke).toHaveBeenCalledWith('save_claude_md_file', {
        filePath: '/project/CLAUDE.md',
        content,
      });
    });

    it('should validate non-empty content', async () => {
      await expect(service.saveClaudeMdFile('/project/CLAUDE.md', '')).rejects.toThrow(z.ZodError);
    });

    it('should validate file path', async () => {
      await expect(service.saveClaudeMdFile('', 'content')).rejects.toThrow(z.ZodError);
    });
  });

  describe('getSystemPrompt', () => {
    it('should return system prompt content', async () => {
      const systemPrompt = 'You are Claude, an AI assistant...';
      mockInvoke.mockResolvedValueOnce(systemPrompt);

      const result = await service.getSystemPrompt();

      expect(mockInvoke).toHaveBeenCalledWith('get_system_prompt', {});
      expect(result).toBe(systemPrompt);
    });
  });

  describe('saveSystemPrompt', () => {
    it('should save system prompt', async () => {
      const content = 'Updated system prompt...';
      mockInvoke.mockResolvedValueOnce(undefined);

      await service.saveSystemPrompt(content);

      expect(mockInvoke).toHaveBeenCalledWith('save_system_prompt', { content });
    });

    it('should validate non-empty content', async () => {
      await expect(service.saveSystemPrompt('')).rejects.toThrow(z.ZodError);
    });
  });

  describe('listDirectoryContents', () => {
    it('should list directory contents', async () => {
      const mockFiles: FileEntry[] = [
        {
          name: 'file1.ts',
          path: '/project/src/file1.ts',
          is_directory: false,
          size: 1024,
          modified: 1704067200,
        },
        {
          name: 'components',
          path: '/project/src/components',
          is_directory: true,
          size: 0,
          modified: 1704067200,
        },
      ];

      mockInvoke.mockResolvedValueOnce(mockFiles);

      const result = await service.listDirectoryContents('/project/src');

      expect(mockInvoke).toHaveBeenCalledWith('list_directory_contents', { directoryPath: '/project/src' });
      expect(result).toHaveLength(2);
      expect(result[0].is_directory).toBe(false);
      expect(result[1].is_directory).toBe(true);
    });

    it('should validate directory path', async () => {
      await expect(service.listDirectoryContents('')).rejects.toThrow(z.ZodError);
    });
  });

  describe('searchFiles', () => {
    it('should search files with query', async () => {
      const mockResults: FileEntry[] = [
        {
          name: 'test.spec.ts',
          path: '/project/src/test.spec.ts',
          is_directory: false,
          size: 2048,
        },
        {
          name: 'test-utils.ts',
          path: '/project/src/utils/test-utils.ts',
          is_directory: false,
          size: 512,
        },
      ];

      mockInvoke.mockResolvedValueOnce(mockResults);

      const result = await service.searchFiles('/project', 'test');

      expect(mockInvoke).toHaveBeenCalledWith('search_files', { basePath: '/project', query: 'test' });
      expect(result).toHaveLength(2);
    });

    it('should validate inputs', async () => {
      await expect(service.searchFiles('', 'query')).rejects.toThrow(z.ZodError);
      await expect(service.searchFiles('/path', '')).rejects.toThrow(z.ZodError);
    });
  });

  describe('getClaudeBinaryPath', () => {
    it('should return binary path when set', async () => {
      mockInvoke.mockResolvedValueOnce('/custom/path/to/claude');

      const result = await service.getClaudeBinaryPath();

      expect(mockInvoke).toHaveBeenCalledWith('get_claude_binary_path', {});
      expect(result).toBe('/custom/path/to/claude');
    });

    it('should return null when not set', async () => {
      mockInvoke.mockResolvedValueOnce(null);

      const result = await service.getClaudeBinaryPath();

      expect(result).toBeNull();
    });
  });

  describe('setClaudeBinaryPath', () => {
    it('should set binary path', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);

      await service.setClaudeBinaryPath('/new/path/to/claude');

      expect(mockInvoke).toHaveBeenCalledWith('set_claude_binary_path', { path: '/new/path/to/claude' });
    });

    it('should validate path', async () => {
      await expect(service.setClaudeBinaryPath('')).rejects.toThrow(z.ZodError);
    });
  });

  describe('listClaudeInstallations', () => {
    it('should list all Claude installations', async () => {
      const mockInstallations: ClaudeInstallation[] = [
        {
          path: '/usr/local/bin/claude',
          version: '0.1.23',
          is_default: true,
          source: 'homebrew',
        },
        {
          path: '/home/user/.nvm/versions/node/v20.0.0/bin/claude',
          version: '0.1.22',
          is_default: false,
          source: 'npm',
        },
      ];

      mockInvoke.mockResolvedValueOnce(mockInstallations);

      const result = await service.listClaudeInstallations();

      expect(mockInvoke).toHaveBeenCalledWith('list_claude_installations', {});
      expect(result).toHaveLength(2);
      expect(result[0].source).toBe('homebrew');
    });

    it('should handle no installations found', async () => {
      mockInvoke.mockResolvedValueOnce([]);

      const result = await service.listClaudeInstallations();

      expect(result).toEqual([]);
    });
  });

  describe('captureUrlScreenshot', () => {
    it('should capture screenshot with default options', async () => {
      const screenshotPath = '/tmp/screenshot-123.png';
      mockInvoke.mockResolvedValueOnce(screenshotPath);

      const result = await service.captureUrlScreenshot('https://example.com');

      expect(mockInvoke).toHaveBeenCalledWith('capture_url_screenshot', {
        url: 'https://example.com',
        selector: null,
        fullPage: false,
      });
      expect(result).toBe(screenshotPath);
    });

    it('should capture screenshot with selector and full page', async () => {
      const screenshotPath = '/tmp/screenshot-456.png';
      mockInvoke.mockResolvedValueOnce(screenshotPath);

      const result = await service.captureUrlScreenshot('https://example.com', '#main-content', true);

      expect(mockInvoke).toHaveBeenCalledWith('capture_url_screenshot', {
        url: 'https://example.com',
        selector: '#main-content',
        fullPage: true,
      });
      expect(result).toBe(screenshotPath);
    });

    it('should validate URL format', async () => {
      await expect(service.captureUrlScreenshot('not-a-url')).rejects.toThrow(z.ZodError);
    });

    it('should treat empty string selector as null', async () => {
      // Empty string selector should be treated as null (no selector)
      const screenshotPath = '/tmp/screenshot-789.png';
      mockInvoke.mockResolvedValueOnce(screenshotPath);

      const result = await service.captureUrlScreenshot('https://example.com', '');

      expect(mockInvoke).toHaveBeenCalledWith('capture_url_screenshot', {
        url: 'https://example.com',
        selector: null, // Empty string is converted to null
        fullPage: false,
      });
      expect(result).toBe(screenshotPath);
    });
  });

  describe('cleanupScreenshotTempFiles', () => {
    it('should cleanup with default age', async () => {
      mockInvoke.mockResolvedValueOnce(5);

      const result = await service.cleanupScreenshotTempFiles();

      expect(mockInvoke).toHaveBeenCalledWith('cleanup_screenshot_temp_files', { olderThanMinutes: 60 });
      expect(result).toBe(5);
    });

    it('should cleanup with custom age', async () => {
      mockInvoke.mockResolvedValueOnce(10);

      const result = await service.cleanupScreenshotTempFiles(120);

      expect(mockInvoke).toHaveBeenCalledWith('cleanup_screenshot_temp_files', { olderThanMinutes: 120 });
      expect(result).toBe(10);
    });

    it('should validate positive integer', async () => {
      await expect(service.cleanupScreenshotTempFiles(-1)).rejects.toThrow(z.ZodError);
      await expect(service.cleanupScreenshotTempFiles(0)).rejects.toThrow(z.ZodError);
      await expect(service.cleanupScreenshotTempFiles(1.5)).rejects.toThrow(z.ZodError);
    });
  });

  describe('cached methods', () => {
    beforeEach(() => {
      vi.useFakeTimers();
    });

    afterEach(() => {
      vi.useRealTimers();
    });

    it('should cache checkClaudeVersion results', async () => {
      const mockStatus: ClaudeVersionStatus = {
        installed: true,
        version: '0.1.23',
        path: '/usr/local/bin/claude',
        error: null,
      };

      mockInvoke.mockResolvedValue(mockStatus);

      // First call
      const result1 = await service.checkClaudeVersionCached();
      expect(result1).toEqual(mockStatus);
      expect(mockInvoke).toHaveBeenCalledTimes(1);

      // Second call should use cache
      const result2 = await service.checkClaudeVersionCached();
      expect(result2).toEqual(mockStatus);
      expect(mockInvoke).toHaveBeenCalledTimes(1);

      // Advance time past TTL (5 minutes)
      vi.advanceTimersByTime(301000);

      // Third call should hit the service again
      const result3 = await service.checkClaudeVersionCached();
      expect(result3).toEqual(mockStatus);
      expect(mockInvoke).toHaveBeenCalledTimes(2);
    });

    it('should cache getClaudeSettings results', async () => {
      const mockSettings: ClaudeSettings = {
        theme: 'dark',
        autoSave: true,
      };

      mockInvoke.mockResolvedValue({ data: mockSettings });

      // First call
      const result1 = await service.getClaudeSettingsCached();
      expect(result1).toEqual(mockSettings);
      expect(mockInvoke).toHaveBeenCalledTimes(1);

      // Second call should use cache
      const result2 = await service.getClaudeSettingsCached();
      expect(result2).toEqual(mockSettings);
      expect(mockInvoke).toHaveBeenCalledTimes(1);

      // Advance time past TTL (1 minute)
      vi.advanceTimersByTime(61000);

      // Third call should hit the service again
      const result3 = await service.getClaudeSettingsCached();
      expect(result3).toEqual(mockSettings);
      expect(mockInvoke).toHaveBeenCalledTimes(2);
    });
  });

  describe('error handling edge cases', () => {
    it('should handle network errors with retry', async () => {
      const networkError = new AppError(ErrorCode.NETWORK_ERROR, 'Network failed');
      
      mockInvoke
        .mockRejectedValueOnce(networkError)
        .mockResolvedValueOnce([]);

      const result = await service.listProjects();

      expect(result).toEqual([]);
      expect(mockInvoke).toHaveBeenCalledTimes(2);
    });

    it('should handle validation errors without retry', async () => {
      const validationError = new AppError(ErrorCode.VALIDATION, 'Invalid data');
      
      mockInvoke.mockRejectedValueOnce(validationError);

      await expect(service.listProjects()).rejects.toThrow(AppError);
      expect(mockInvoke).toHaveBeenCalledTimes(1);
    });

    it('should handle malformed responses gracefully', async () => {
      // Test date format transformation
      const projectWithBadDates = {
        id: 'project-1',
        name: 'Test Project',
        path: '/path/to/project',
        created_at: 'invalid-date',
        last_accessed: '2024-13-45T99:99:99Z', // Invalid date
        is_git_repo: true,
        // Missing optional fields should be handled
      };

      mockInvoke.mockResolvedValueOnce([projectWithBadDates]);

      // Should throw validation error for invalid dates
      await expect(service.listProjects()).rejects.toThrow(ValidationError);
    });
  });

  describe('concurrent operations', () => {
    it('should handle multiple concurrent project operations', async () => {
      mockInvoke.mockImplementation((command) => {
        switch (command) {
          case 'list_projects':
            return Promise.resolve([]);
          case 'get_project_with_stats':
            return Promise.resolve({
              id: 'project-1',
              name: 'Test',
              path: '/test',
              created_at: '2024-01-01T00:00:00Z',
              last_accessed: null,
              description: null,
              is_git_repo: false,
              remote_url: null,
              branch: null,
              tags: [],
              metadata: {},
              stats: {
                total_sessions: 0,
                active_sessions: 0,
                total_tokens: 0,
                last_session_date: null,
                average_session_duration: null,
                most_used_agent: null,
              },
            });
          case 'get_claude_settings':
            return Promise.resolve({ data: {} });
          default:
            return Promise.resolve(null);
        }
      });

      const [projects, projectStats, settings] = await Promise.all([
        service.listProjects(),
        service.getProjectWithStats('project-1'),
        service.getClaudeSettings(),
      ]);

      expect(projects).toEqual([]);
      expect(projectStats.id).toBe('project-1');
      expect(settings).toEqual({});
      expect(mockInvoke).toHaveBeenCalledTimes(3);
    });
  });
});