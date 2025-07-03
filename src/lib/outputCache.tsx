import React, { createContext, useContext, useState, useCallback, useEffect, useRef } from 'react';
import { api, type AgentRunWithMetrics } from './api';

// Use the same message interface as AgentExecution for consistency
export interface ClaudeStreamMessage {
  type: "system" | "assistant" | "user" | "result";
  subtype?: string;
  message?: {
    content?: any[];
    usage?: {
      input_tokens: number;
      output_tokens: number;
    };
  };
  usage?: {
    input_tokens: number;
    output_tokens: number;
  };
  [key: string]: any;
}

interface CachedSessionOutput {
  output: string;
  messages: ClaudeStreamMessage[];
  lastUpdated: number;
  status: string;
}

interface OutputCacheContextType {
  getCachedOutput: (sessionId: number) => CachedSessionOutput | null;
  setCachedOutput: (sessionId: number, data: CachedSessionOutput) => void;
  updateSessionStatus: (sessionId: number, status: string) => void;
  clearCache: (sessionId?: number) => void;
  isPolling: boolean;
  startBackgroundPolling: () => void;
  stopBackgroundPolling: () => void;
  allSessions: AgentRunWithMetrics[];
  lastSessionsUpdate: number;
}

const OutputCacheContext = createContext<OutputCacheContextType | null>(null);

export function useOutputCache() {
  const context = useContext(OutputCacheContext);
  if (!context) {
    throw new Error('useOutputCache must be used within an OutputCacheProvider');
  }
  return context;
}

interface OutputCacheProviderProps {
  children: React.ReactNode;
}

export function OutputCacheProvider({ children }: OutputCacheProviderProps) {
  const [cache, setCache] = useState<Map<number, CachedSessionOutput>>(new Map());
  const [isPolling, setIsPolling] = useState(false);
  const [pollingInterval, setPollingInterval] = useState<NodeJS.Timeout | null>(null);
  const [allSessions, setAllSessions] = useState<AgentRunWithMetrics[]>([]);
  const [lastSessionsUpdate, setLastSessionsUpdate] = useState<number>(0);
  const isPollingStarted = useRef(false);

  const getCachedOutput = useCallback((sessionId: number): CachedSessionOutput | null => {
    return cache.get(sessionId) || null;
  }, [cache]);

  const setCachedOutput = useCallback((sessionId: number, data: CachedSessionOutput) => {
    setCache(prev => new Map(prev.set(sessionId, data)));
  }, []);

  const updateSessionStatus = useCallback((sessionId: number, status: string) => {
    setCache(prev => {
      const existing = prev.get(sessionId);
      if (existing) {
        const updated = new Map(prev);
        updated.set(sessionId, { ...existing, status });
        return updated;
      }
      return prev;
    });
  }, []);

  const clearCache = useCallback((sessionId?: number) => {
    if (sessionId) {
      setCache(prev => {
        const updated = new Map(prev);
        updated.delete(sessionId);
        return updated;
      });
    } else {
      setCache(new Map());
    }
  }, []);

  const parseOutput = useCallback((rawOutput: string): ClaudeStreamMessage[] => {
    if (!rawOutput) return [];

    const lines = rawOutput.split('\n').filter(line => line.trim());
    const parsedMessages: ClaudeStreamMessage[] = [];

    for (const line of lines) {
      try {
        const message = JSON.parse(line) as ClaudeStreamMessage;
        parsedMessages.push(message);
      } catch (err) {
        console.error("Failed to parse message:", err, line);
        // Add a fallback message for unparseable content
        parsedMessages.push({
          type: 'result',
          subtype: 'error',
          error: 'Failed to parse message',
          raw_content: line
        });
      }
    }

    return parsedMessages;
  }, []);

  const updateSessionCache = useCallback(async (sessionId: number, status: string) => {
    try {
      const rawOutput = await api.getSessionOutput(sessionId);
      const messages = parseOutput(rawOutput);
      
      setCachedOutput(sessionId, {
        output: rawOutput,
        messages,
        lastUpdated: Date.now(),
        status
      });
    } catch (error) {
      console.warn(`Failed to update cache for session ${sessionId}:`, error);
    }
  }, [parseOutput, setCachedOutput]);

  const pollRunningSessions = useCallback(async () => {
    try {
      // Fetch sessions with metrics in a single call
      const sessionsWithMetrics = await api.listRunningAgentSessionsWithMetrics();
      setAllSessions(sessionsWithMetrics);
      setLastSessionsUpdate(Date.now());
      
      // Only update output cache for running sessions
      const runningSessions = sessionsWithMetrics.filter(s => s.status === 'running');
      for (const session of runningSessions) {
        if (session.id) {
          // Only fetch output if we don't have recent cache
          const cached = cache.get(session.id);
          if (!cached || Date.now() - cached.lastUpdated > 10000) { // 10 seconds
            await updateSessionCache(session.id, session.status);
          }
        }
      }

      // Clean up cache for sessions that are no longer running
      const activeIds = new Set(sessionsWithMetrics.map(s => s.id).filter(Boolean));
      setCache(prev => {
        const updated = new Map();
        for (const [sessionId, data] of prev) {
          if (activeIds.has(sessionId)) {
            updated.set(sessionId, data);
          }
        }
        return updated;
      });
    } catch (error) {
      console.warn('Failed to poll running sessions:', error);
    }
  }, [updateSessionCache, cache]);

  const startBackgroundPolling = useCallback(() => {
    if (pollingInterval || isPollingStarted.current) return;

    isPollingStarted.current = true;
    setIsPolling(true);
    const interval = setInterval(pollRunningSessions, 5000); // Poll every 5 seconds
    setPollingInterval(interval);
    // Poll immediately after setting up interval
    pollRunningSessions();
  }, [pollingInterval, pollRunningSessions]);

  const stopBackgroundPolling = useCallback(() => {
    if (pollingInterval) {
      clearInterval(pollingInterval);
      setPollingInterval(null);
    }
    setIsPolling(false);
    isPollingStarted.current = false;
  }, [pollingInterval]);

  // Auto-start polling when provider mounts
  useEffect(() => {
    startBackgroundPolling();
    return () => {
      if (pollingInterval) {
        clearInterval(pollingInterval);
      }
    };
  }, []); // Empty deps - only run on mount/unmount

  const value: OutputCacheContextType = {
    getCachedOutput,
    setCachedOutput,
    updateSessionStatus,
    clearCache,
    isPolling,
    startBackgroundPolling,
    stopBackgroundPolling,
    allSessions,
    lastSessionsUpdate,
  };

  return (
    <OutputCacheContext.Provider value={value}>
      {children}
    </OutputCacheContext.Provider>
  );
}