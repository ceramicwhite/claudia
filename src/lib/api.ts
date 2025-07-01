/**
 * API Module
 * 
 * This file now serves as a compatibility layer and type export hub.
 * The actual API methods have been moved to domain-specific services in src/services/
 * 
 * @deprecated Direct usage of the api object is deprecated. Use specific services instead:
 * - projectService for project operations
 * - sessionService for session operations
 * - claudeService for Claude process operations
 * - agentService for agent operations
 * - sandboxService for sandbox operations
 * - usageService for usage tracking
 * - checkpointService for checkpoint operations
 * - mcpService for MCP server operations
 */

import {
  projectService,
  sessionService,
  claudeService,
  agentService,
  sandboxService,
  usageService,
  checkpointService,
  mcpService
} from '@/services';

// Export all types
export * from './api.types';

/**
 * Legacy API object for backward compatibility
 * @deprecated Use specific services from @/services instead
 */
export const api = {
  // Project operations
  listProjects: projectService.listProjects.bind(projectService),
  getClaudeSettings: projectService.getClaudeSettings.bind(projectService),
  saveClaudeSettings: projectService.saveClaudeSettings.bind(projectService),
  checkClaudeVersion: projectService.checkClaudeVersion.bind(projectService),
  findClaudeMdFiles: projectService.findClaudeMdFiles.bind(projectService),
  readClaudeMdFile: projectService.readClaudeMdFile.bind(projectService),
  saveClaudeMdFile: projectService.saveClaudeMdFile.bind(projectService),
  getSystemPrompt: projectService.getSystemPrompt.bind(projectService),
  saveSystemPrompt: projectService.saveSystemPrompt.bind(projectService),
  listDirectoryContents: projectService.listDirectoryContents.bind(projectService),
  searchFiles: projectService.searchFiles.bind(projectService),
  getClaudeBinaryPath: projectService.getClaudeBinaryPath.bind(projectService),
  setClaudeBinaryPath: projectService.setClaudeBinaryPath.bind(projectService),
  listClaudeInstallations: projectService.listClaudeInstallations.bind(projectService),
  captureUrlScreenshot: projectService.captureUrlScreenshot.bind(projectService),
  cleanupScreenshotTempFiles: projectService.cleanupScreenshotTempFiles.bind(projectService),

  // Session operations
  getProjectSessions: sessionService.getProjectSessions.bind(sessionService),
  openNewSession: sessionService.openNewSession.bind(sessionService),
  loadSessionHistory: sessionService.loadSessionHistory.bind(sessionService),
  trackSessionMessages: sessionService.trackSessionMessages.bind(sessionService),

  // Claude operations
  executeClaudeCode: claudeService.executeClaudeCode.bind(claudeService),
  continueClaudeCode: claudeService.continueClaudeCode.bind(claudeService),
  resumeClaudeCode: claudeService.resumeClaudeCode.bind(claudeService),
  cancelClaudeExecution: claudeService.cancelClaudeExecution.bind(claudeService),

  // Agent operations
  listAgents: agentService.listAgents.bind(agentService),
  createAgent: agentService.createAgent.bind(agentService),
  updateAgent: agentService.updateAgent.bind(agentService),
  deleteAgent: agentService.deleteAgent.bind(agentService),
  getAgent: agentService.getAgent.bind(agentService),
  exportAgent: agentService.exportAgent.bind(agentService),
  importAgent: agentService.importAgent.bind(agentService),
  importAgentFromFile: agentService.importAgentFromFile.bind(agentService),
  fetchGitHubAgents: agentService.fetchGitHubAgents.bind(agentService),
  fetchGitHubAgentContent: agentService.fetchGitHubAgentContent.bind(agentService),
  importAgentFromGitHub: agentService.importAgentFromGitHub.bind(agentService),
  executeAgent: agentService.executeAgent.bind(agentService),
  createScheduledAgentRun: agentService.createScheduledAgentRun.bind(agentService),
  getScheduledAgentRuns: agentService.getScheduledAgentRuns.bind(agentService),
  cancelScheduledAgentRun: agentService.cancelScheduledAgentRun.bind(agentService),
  listAgentRuns: agentService.listAgentRuns.bind(agentService),
  getAgentRun: agentService.getAgentRun.bind(agentService),
  getAgentRunWithRealTimeMetrics: agentService.getAgentRunWithRealTimeMetrics.bind(agentService),
  listRunningAgentSessions: agentService.listRunningAgentSessions.bind(agentService),
  resumeAgent: agentService.resumeAgent.bind(agentService),
  listRunningAgentSessionsWithMetrics: agentService.listRunningAgentSessionsWithMetrics.bind(agentService),
  killAgentSession: agentService.killAgentSession.bind(agentService),
  getSessionStatus: agentService.getSessionStatus.bind(agentService),
  cleanupFinishedProcesses: agentService.cleanupFinishedProcesses.bind(agentService),
  getSessionOutput: agentService.getSessionOutput.bind(agentService),
  getAgentRunOutput: agentService.getAgentRunOutput.bind(agentService),
  getLiveSessionOutput: agentService.getLiveSessionOutput.bind(agentService),
  streamSessionOutput: agentService.streamSessionOutput.bind(agentService),

  // Sandbox operations
  listSandboxProfiles: sandboxService.listSandboxProfiles.bind(sandboxService),
  createSandboxProfile: sandboxService.createSandboxProfile.bind(sandboxService),
  updateSandboxProfile: sandboxService.updateSandboxProfile.bind(sandboxService),
  deleteSandboxProfile: sandboxService.deleteSandboxProfile.bind(sandboxService),
  getSandboxProfile: sandboxService.getSandboxProfile.bind(sandboxService),
  listSandboxRules: sandboxService.listSandboxRules.bind(sandboxService),
  createSandboxRule: sandboxService.createSandboxRule.bind(sandboxService),
  updateSandboxRule: sandboxService.updateSandboxRule.bind(sandboxService),
  deleteSandboxRule: sandboxService.deleteSandboxRule.bind(sandboxService),
  getPlatformCapabilities: sandboxService.getPlatformCapabilities.bind(sandboxService),
  testSandboxProfile: sandboxService.testSandboxProfile.bind(sandboxService),
  listSandboxViolations: sandboxService.listSandboxViolations.bind(sandboxService),
  logSandboxViolation: sandboxService.logSandboxViolation.bind(sandboxService),
  clearSandboxViolations: sandboxService.clearSandboxViolations.bind(sandboxService),
  getSandboxViolationStats: sandboxService.getSandboxViolationStats.bind(sandboxService),
  exportSandboxProfile: sandboxService.exportSandboxProfile.bind(sandboxService),
  exportAllSandboxProfiles: sandboxService.exportAllSandboxProfiles.bind(sandboxService),
  importSandboxProfiles: sandboxService.importSandboxProfiles.bind(sandboxService),

  // Usage operations
  getUsageStats: usageService.getUsageStats.bind(usageService),
  getUsageByDateRange: usageService.getUsageByDateRange.bind(usageService),
  getSessionStats: usageService.getSessionStats.bind(usageService),
  getUsageDetails: usageService.getUsageDetails.bind(usageService),

  // Checkpoint operations
  createCheckpoint: checkpointService.createCheckpoint.bind(checkpointService),
  restoreCheckpoint: checkpointService.restoreCheckpoint.bind(checkpointService),
  listCheckpoints: checkpointService.listCheckpoints.bind(checkpointService),
  forkFromCheckpoint: checkpointService.forkFromCheckpoint.bind(checkpointService),
  getSessionTimeline: checkpointService.getSessionTimeline.bind(checkpointService),
  updateCheckpointSettings: checkpointService.updateCheckpointSettings.bind(checkpointService),
  getCheckpointDiff: checkpointService.getCheckpointDiff.bind(checkpointService),
  trackCheckpointMessage: checkpointService.trackCheckpointMessage.bind(checkpointService),
  checkAutoCheckpoint: checkpointService.checkAutoCheckpoint.bind(checkpointService),
  cleanupOldCheckpoints: checkpointService.cleanupOldCheckpoints.bind(checkpointService),
  getCheckpointSettings: checkpointService.getCheckpointSettings.bind(checkpointService),
  clearCheckpointManager: checkpointService.clearCheckpointManager.bind(checkpointService),

  // MCP operations
  mcpAdd: mcpService.mcpAdd.bind(mcpService),
  mcpList: mcpService.mcpList.bind(mcpService),
  mcpGet: mcpService.mcpGet.bind(mcpService),
  mcpRemove: mcpService.mcpRemove.bind(mcpService),
  mcpAddJson: mcpService.mcpAddJson.bind(mcpService),
  mcpAddFromClaudeDesktop: mcpService.mcpAddFromClaudeDesktop.bind(mcpService),
  mcpServe: mcpService.mcpServe.bind(mcpService),
  mcpTestConnection: mcpService.mcpTestConnection.bind(mcpService),
  mcpResetProjectChoices: mcpService.mcpResetProjectChoices.bind(mcpService),
  mcpGetServerStatus: mcpService.mcpGetServerStatus.bind(mcpService),
  mcpReadProjectConfig: mcpService.mcpReadProjectConfig.bind(mcpService),
  mcpSaveProjectConfig: mcpService.mcpSaveProjectConfig.bind(mcpService),
};