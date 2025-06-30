import useSWR from 'swr';
import { api, type AgentRun } from '@/lib/api';

interface UseAgentRunsOptions {
  refreshInterval?: number;
  revalidateOnFocus?: boolean;
  revalidateOnReconnect?: boolean;
}

/**
 * Custom hook for fetching agent runs with SWR
 * Provides caching, revalidation, and error handling
 */
export function useAgentRuns(agentId?: number, options: UseAgentRunsOptions = {}) {
  const {
    refreshInterval = 0,
    revalidateOnFocus = true,
    revalidateOnReconnect = true,
  } = options;

  const { data, error, isLoading, mutate } = useSWR<AgentRun[]>(
    agentId ? `agent-runs-${agentId}` : null,
    () => agentId ? api.getAgentRuns(agentId) : Promise.resolve([]),
    {
      refreshInterval,
      revalidateOnFocus,
      revalidateOnReconnect,
      dedupingInterval: 2000,
    }
  );

  return {
    runs: data ?? [],
    isLoading,
    isError: !!error,
    error,
    mutate,
  };
}

/**
 * Custom hook for fetching a single agent run
 */
export function useAgentRun(runId?: number) {
  const { data, error, isLoading, mutate } = useSWR<AgentRun>(
    runId ? `agent-run-${runId}` : null,
    () => runId ? api.getAgentRun(runId) : Promise.resolve(null),
    {
      revalidateOnFocus: true,
      revalidateOnReconnect: true,
      dedupingInterval: 2000,
    }
  );

  return {
    run: data,
    isLoading,
    isError: !!error,
    error,
    mutate,
  };
}

/**
 * Custom hook for fetching running sessions with polling
 */
export function useRunningSessions(pollingInterval = 5000) {
  const { data, error, isLoading, mutate } = useSWR<AgentRun[]>(
    'running-sessions',
    () => api.listRunningSessions(),
    {
      refreshInterval: pollingInterval,
      revalidateOnFocus: true,
      revalidateOnReconnect: true,
      dedupingInterval: 2000,
    }
  );

  return {
    sessions: data ?? [],
    isLoading,
    isError: !!error,
    error,
    mutate,
  };
}