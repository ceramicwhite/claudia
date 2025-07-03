/**
 * This is the main API barrel file that provides backward compatibility
 * while using the new modular structure.
 * 
 * All types and API methods are re-exported from their respective modules.
 */

// Re-export all types for backward compatibility
export * from './api/types';

// Import all API modules
import {
  projectsApi,
  claudeApi,
  agentsApi,
  sandboxApi,
  usageApi,
  checkpointApi,
  mcpApi,
  fileSystemApi,
  screenshotApi
} from './api/modules';

/**
 * API client for interacting with the Rust backend
 * 
 * This object provides backward compatibility by spreading all module methods
 * into a single api object, maintaining the same interface as before.
 */
export const api = {
  // Projects API
  ...projectsApi,
  
  // Claude API
  ...claudeApi,
  
  // Agents API
  ...agentsApi,
  
  // Sandbox API
  ...sandboxApi,
  
  // Usage API
  ...usageApi,
  
  // Checkpoint API
  ...checkpointApi,
  
  // MCP API
  ...mcpApi,
  
  // File System API
  ...fileSystemApi,
  
  // Screenshot API
  ...screenshotApi,
};
