import React from 'react';
import { motion } from 'framer-motion';
import { Bot, Eye, Square, Cpu, RefreshCw, DollarSign, Hash } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import type { AgentRun, AgentRunWithMetrics } from '@/lib/api';
import { cn } from '@/lib/utils';

interface SessionCardProps {
  session: AgentRun | AgentRunWithMetrics;
  index?: number;
  onViewOutput: (session: AgentRun | AgentRunWithMetrics) => void;
  onStop?: (runId: number, agentName: string) => void;
  showStopButton?: boolean;
  statusConfig: {
    bgColor: string;
    iconColor: string;
  };
}

function isAgentRunWithMetrics(session: AgentRun | AgentRunWithMetrics): session is AgentRunWithMetrics {
  return 'metrics' in session;
}

export const SessionCard: React.FC<SessionCardProps> = ({
  session,
  index = 0,
  onViewOutput,
  onStop,
  showStopButton = false,
  statusConfig
}) => {
  const formatDuration = (startTime: string) => {
    const start = new Date(startTime);
    const now = new Date();
    const durationMs = now.getTime() - start.getTime();
    const minutes = Math.floor(durationMs / (1000 * 60));
    const seconds = Math.floor((durationMs % (1000 * 60)) / 1000);
    return `${minutes}m ${seconds}s`;
  };
  
  const formatScheduledTime = (scheduledTime: string) => {
    const date = new Date(scheduledTime);
    const now = new Date();
    const diffMs = date.getTime() - now.getTime();
    
    if (diffMs < 0) {
      return 'Starting soon...';
    }
    
    const hours = Math.floor(diffMs / (1000 * 60 * 60));
    const minutes = Math.floor((diffMs % (1000 * 60 * 60)) / (1000 * 60));
    
    if (hours > 0) {
      return `in ${hours}h ${minutes}m`;
    }
    return `in ${minutes}m`;
  };

  const formatCurrency = (amount: number): string => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 2,
      maximumFractionDigits: 4
    }).format(amount);
  };

  const formatTokens = (tokens: number): string => {
    if (tokens >= 1000000) {
      return `${(tokens / 1000000).toFixed(1)}M`;
    }
    if (tokens >= 1000) {
      return `${(tokens / 1000).toFixed(1)}k`;
    }
    return tokens.toString();
  };

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'running':
        return <Badge variant="default" className="bg-green-100 text-green-800 border-green-200">Running</Badge>;
      case 'scheduled':
        return <Badge variant="secondary" className="bg-blue-100 text-blue-800 border-blue-200">Scheduled</Badge>;
      case 'pending':
        return <Badge variant="secondary">Pending</Badge>;
      case 'paused_usage_limit':
        return <Badge variant="outline" className="bg-orange-100 text-orange-800 border-orange-200">Usage Limit</Badge>;
      case 'cancelled':
        return <Badge variant="outline" className="bg-gray-100 text-gray-800 border-gray-200">Cancelled</Badge>;
      case 'failed':
        return <Badge variant="destructive" className="bg-red-100 text-red-800 border-red-200">Failed</Badge>;
      case 'completed':
        return <Badge variant="default" className="bg-gray-100 text-gray-800 border-gray-200">Completed</Badge>;
      default:
        return <Badge variant="outline">{status}</Badge>;
    }
  };

  const hasMetrics = isAgentRunWithMetrics(session) && session.metrics;

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      transition={{ duration: 0.2, delay: index * 0.05 }}
    >
      <Card className="hover:shadow-md transition-shadow">
        <CardHeader className="pb-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-3">
              <div className={cn("flex items-center justify-center w-8 h-8 rounded-full", statusConfig.bgColor)}>
                <Bot className={cn("h-5 w-5", statusConfig.iconColor)} />
              </div>
              <div>
                <CardTitle className="text-base">{session.agent_name}</CardTitle>
                <div className="flex items-center space-x-2 mt-1">
                  {getStatusBadge(session.status)}
                  {session.pid && (
                    <Badge variant="outline" className="text-xs">
                      <Cpu className="h-3 w-3 mr-1" />
                      PID {session.pid}
                    </Badge>
                  )}
                </div>
              </div>
            </div>
            <div className="flex items-center space-x-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => onViewOutput(session)}
                className="flex items-center space-x-2"
              >
                <Eye className="h-4 w-4" />
                <span>View Output</span>
              </Button>
              {showStopButton && onStop && session.id && (
                <Button
                  variant="destructive"
                  size="sm"
                  onClick={() => onStop(session.id!, session.agent_name)}
                  className="flex items-center space-x-2"
                >
                  <Square className="h-4 w-4" />
                  <span>Stop</span>
                </Button>
              )}
            </div>
          </div>
        </CardHeader>
        <CardContent className="pt-0">
          <div className="space-y-2">
            <div>
              <p className="text-sm text-muted-foreground">Task</p>
              <p className="text-sm font-medium truncate">{session.task}</p>
            </div>
            
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>
                <p className="text-muted-foreground">Model</p>
                <p className="font-medium">{session.model}</p>
              </div>
              <div>
                <p className="text-muted-foreground">
                  {session.status === 'scheduled' ? 'Starts' : 
                   session.status === 'paused_usage_limit' ? 'Resume Time' :
                   ['failed', 'cancelled'].includes(session.status) ? 
                     (session.status === 'failed' ? 'Failed At' : 'Cancelled At') :
                   'Duration'}
                </p>
                <p className="font-medium">
                  {session.status === 'scheduled' && session.scheduled_start_time
                    ? formatScheduledTime(session.scheduled_start_time)
                    : session.status === 'paused_usage_limit' && session.usage_limit_reset_time
                    ? new Date(session.usage_limit_reset_time).toLocaleString()
                    : ['failed', 'cancelled'].includes(session.status) && session.completed_at
                    ? new Date(session.completed_at).toLocaleString()
                    : session.process_started_at 
                    ? formatDuration(session.process_started_at)
                    : 'Unknown'
                  }
                </p>
              </div>
            </div>

            {/* Metrics Section - Cost and Tokens */}
            {hasMetrics && session.metrics && (session.metrics.cost_usd || session.metrics.total_tokens) && (
              <div className="grid grid-cols-2 gap-4 text-sm border-t pt-2">
                {session.metrics.cost_usd && (
                  <div>
                    <p className="text-muted-foreground flex items-center gap-1">
                      <DollarSign className="h-3 w-3" />
                      Cost
                    </p>
                    <p className="font-medium">{formatCurrency(session.metrics.cost_usd)}</p>
                  </div>
                )}
                {session.metrics.total_tokens && (
                  <div>
                    <p className="text-muted-foreground flex items-center gap-1">
                      <Hash className="h-3 w-3" />
                      Tokens
                    </p>
                    <p className="font-medium">{formatTokens(session.metrics.total_tokens)}</p>
                  </div>
                )}
              </div>
            )}
            
            <div>
              <p className="text-sm text-muted-foreground">Project Path</p>
              <p className="text-xs font-mono bg-muted px-2 py-1 rounded truncate">
                {session.project_path}
              </p>
            </div>

            {/* Auto-resume indicator */}
            {session.status === 'paused_usage_limit' && session.auto_resume_enabled && (
              <div className="flex items-center space-x-2 text-xs text-orange-600">
                <RefreshCw className="h-3 w-3" />
                <span>Auto-resume enabled</span>
              </div>
            )}
          </div>
        </CardContent>
      </Card>
    </motion.div>
  );
};