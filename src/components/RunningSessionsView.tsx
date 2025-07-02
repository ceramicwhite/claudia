import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Play, Clock, RefreshCw, ArrowLeft, Pause, ChevronDown, XCircle, AlertCircle, CheckCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Toast, ToastContainer } from '@/components/ui/toast';
import { SessionOutputViewer } from './SessionOutputViewer';
import { SessionCard } from './SessionCard';
import { api } from '@/lib/api';
import type { AgentRunWithMetrics } from '@/lib/api';
import { useOutputCache } from '@/lib/outputCache';

interface RunningSessionsViewProps {
  className?: string;
  showBackButton?: boolean;
  onBack?: () => void;
  onEditSession?: (session: AgentRunWithMetrics) => void;
}

export function RunningSessionsView({ className, showBackButton = false, onBack, onEditSession }: RunningSessionsViewProps) {
  const { allSessions } = useOutputCache();
  const [loading] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [selectedSession, setSelectedSession] = useState<AgentRunWithMetrics | null>(null);
  const [toast, setToast] = useState<{ message: string; type: "success" | "error" } | null>(null);
  
  // Separate sessions by status
  const scheduledSessions = allSessions.filter(s => s.status === 'scheduled');
  const runningSessions = allSessions.filter(s => s.status === 'running');
  const pausedSessions = allSessions.filter(s => s.status === 'paused_usage_limit');
  const failedSessions = allSessions.filter(s => s.status === 'failed');
  const completedSessions = allSessions.filter(s => s.status === 'completed');
  const cancelledSessions = allSessions.filter(s => s.status === 'cancelled');
  
  // Collapsible states for sections
  const [sectionsExpanded, setSectionsExpanded] = useState({
    scheduled: true,
    running: true,
    paused: true,
    failed: false,
    completed: false,
    cancelled: false
  });
  
  const toggleSection = (section: keyof typeof sectionsExpanded) => {
    setSectionsExpanded(prev => ({ ...prev, [section]: !prev[section] }));
  };

  // Remove the loadRunningSessions function as we're using cached data now

  const refreshSessions = async () => {
    setRefreshing(true);
    try {
      // First cleanup finished processes
      await api.cleanupFinishedProcesses();
      // The sessions will be refreshed automatically by the OutputCacheProvider
      setToast({ message: 'Running sessions list has been updated', type: 'success' });
    } catch (error) {
      console.error('Failed to refresh sessions:', error);
      setToast({ message: 'Failed to refresh sessions', type: 'error' });
    } finally {
      setRefreshing(false);
    }
  };

  const killSession = async (runId: number, agentName: string) => {
    try {
      const success = await api.killAgentSession(runId);
      if (success) {
        setToast({ message: `${agentName} session has been stopped`, type: 'success' });
        // The sessions will be refreshed automatically by the OutputCacheProvider
      } else {
        setToast({ message: 'Session may have already finished', type: 'error' });
      }
    } catch (error) {
      console.error('Failed to kill session:', error);
      setToast({ message: 'Failed to terminate session', type: 'error' });
    }
  };

  const handleResume = async (session: AgentRunWithMetrics) => {
    try {
      // Resume the agent using its session ID
      if (!session.id) {
        throw new Error('Session ID not found');
      }
      
      await api.resumeAgent(session.id);
      
      setToast({ message: `${session.agent_name} has been resumed`, type: 'success' });
      // The sessions will be refreshed automatically by the OutputCacheProvider
    } catch (error) {
      console.error('Failed to resume session:', error);
      setToast({ message: 'Failed to resume session', type: 'error' });
    }
  };

  const handleRetry = async (session: AgentRunWithMetrics) => {
    try {
      // Execute the agent with the same parameters
      await api.executeAgent(
        session.agent_id,
        session.project_path,
        session.task,
        session.model,
        session.auto_resume_enabled
      );
      
      setToast({ message: `${session.agent_name} has been retried`, type: 'success' });
      // The sessions will be refreshed automatically by the OutputCacheProvider
    } catch (error) {
      console.error('Failed to retry session:', error);
      setToast({ message: 'Failed to retry session', type: 'error' });
    }
  };

  const handleEdit = (session: AgentRunWithMetrics) => {
    if (onEditSession) {
      onEditSession(session);
    }
  };

  // No need for useEffect or polling - data is provided by OutputCacheProvider

  if (loading) {
    return (
      <div className={`flex items-center justify-center p-8 ${className}`}>
        <div className="flex items-center space-x-2">
          <RefreshCw className="h-4 w-4 animate-spin" />
          <span>Loading running sessions...</span>
        </div>
      </div>
    );
  }

  return (
    <div className={`space-y-4 ${className}`}>
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-2">
          {showBackButton && onBack && (
            <Button
              variant="ghost"
              size="icon"
              onClick={onBack}
              className="h-8 w-8"
            >
              <ArrowLeft className="h-4 w-4" />
            </Button>
          )}
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={refreshSessions}
          disabled={refreshing}
          className="flex items-center space-x-2"
        >
          <RefreshCw className={`h-4 w-4 ${refreshing ? 'animate-spin' : ''}`} />
          <span>Refresh</span>
        </Button>
      </div>

      {allSessions.length === 0 ? (
        <Card>
          <CardContent className="flex items-center justify-center p-8">
            <div className="text-center space-y-2">
              <Clock className="h-8 w-8 mx-auto text-muted-foreground" />
              <p className="text-muted-foreground">No agent sessions found</p>
            </div>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-4">
          {/* Scheduled Sessions Section */}
          <div className="space-y-3">
            <button
              onClick={() => toggleSection('scheduled')}
              className="flex items-center space-x-2 w-full hover:opacity-80 transition-opacity"
            >
              <Clock className="h-5 w-5 text-blue-600" />
              <h3 className="text-base font-medium text-muted-foreground">Scheduled ({scheduledSessions.length})</h3>
              <ChevronDown className={`h-4 w-4 text-muted-foreground transition-transform ${sectionsExpanded.scheduled ? '' : '-rotate-90'}`} />
            </button>
            {sectionsExpanded.scheduled && scheduledSessions.length > 0 && (
              <div className="space-y-3">
                {scheduledSessions.map((session, index) => (
                  <SessionCard
                    key={session.id}
                    session={session}
                    index={index}
                    onViewOutput={setSelectedSession}
                    onStop={killSession}
                    showStopButton={true}
                    statusConfig={{
                      bgColor: 'bg-blue-100',
                      iconColor: 'text-blue-600'
                    }}
                  />
                ))}
              </div>
            )}
          </div>
          
          {/* Running Sessions Section */}
          <div className="space-y-3">
            <button
              onClick={() => toggleSection('running')}
              className="flex items-center space-x-2 w-full hover:opacity-80 transition-opacity"
            >
              <Play className="h-5 w-5 text-green-600" />
              <h3 className="text-base font-medium text-muted-foreground">Running ({runningSessions.length})</h3>
              <ChevronDown className={`h-4 w-4 text-muted-foreground transition-transform ${sectionsExpanded.running ? '' : '-rotate-90'}`} />
            </button>
            {sectionsExpanded.running && runningSessions.length > 0 && (
              <div className="space-y-3">
                {runningSessions.map((session, index) => (
                  <SessionCard
                    key={session.id}
                    session={session}
                    index={index}
                    onViewOutput={setSelectedSession}
                    onStop={killSession}
                    showStopButton={true}
                    statusConfig={{
                      bgColor: 'bg-blue-100',
                      iconColor: 'text-blue-600'
                    }}
                  />
                ))}
              </div>
            )}
          </div>
          
          {/* Paused Sessions Section */}
          <div className="space-y-3">
            <button
              onClick={() => toggleSection('paused')}
              className="flex items-center space-x-2 w-full hover:opacity-80 transition-opacity"
            >
              <Pause className="h-5 w-5 text-orange-600" />
              <h3 className="text-base font-medium text-muted-foreground">Paused ({pausedSessions.length})</h3>
              <ChevronDown className={`h-4 w-4 text-muted-foreground transition-transform ${sectionsExpanded.paused ? '' : '-rotate-90'}`} />
            </button>
            {sectionsExpanded.paused && pausedSessions.length > 0 && (
              <div className="space-y-3">
                {pausedSessions.map((session, index) => (
                  <SessionCard
                    key={session.id}
                    session={session}
                    index={index}
                    onViewOutput={setSelectedSession}
                    onResume={handleResume}
                    showStopButton={false}
                    statusConfig={{
                      bgColor: 'bg-orange-100',
                      iconColor: 'text-orange-600'
                    }}
                  />
                ))}
              </div>
            )}
          </div>
          
          {/* Failed Sessions Section */}
          <div className="space-y-3">
            <button
              onClick={() => toggleSection('failed')}
              className="flex items-center space-x-2 w-full hover:opacity-80 transition-opacity"
            >
              <AlertCircle className="h-5 w-5 text-red-600" />
              <h3 className="text-base font-medium text-muted-foreground">Failed ({failedSessions.length})</h3>
              <ChevronDown className={`h-4 w-4 text-muted-foreground transition-transform ${sectionsExpanded.failed ? '' : '-rotate-90'}`} />
            </button>
            {sectionsExpanded.failed && failedSessions.length > 0 && (
              <div className="space-y-3">
                {failedSessions.map((session, index) => (
                  <SessionCard
                    key={session.id}
                    session={session}
                    index={index}
                    onViewOutput={setSelectedSession}
                    onRetry={handleRetry}
                    onEdit={handleEdit}
                    showStopButton={false}
                    statusConfig={{
                      bgColor: 'bg-red-100',
                      iconColor: 'text-red-600'
                    }}
                  />
                ))}
              </div>
            )}
          </div>
          
          {/* Completed Sessions Section */}
          <div className="space-y-3">
            <button
              onClick={() => toggleSection('completed')}
              className="flex items-center space-x-2 w-full hover:opacity-80 transition-opacity"
            >
              <CheckCircle className="h-5 w-5 text-green-600" />
              <h3 className="text-base font-medium text-muted-foreground">Completed ({completedSessions.length})</h3>
              <ChevronDown className={`h-4 w-4 text-muted-foreground transition-transform ${sectionsExpanded.completed ? '' : '-rotate-90'}`} />
            </button>
            {sectionsExpanded.completed && completedSessions.length > 0 && (
              <div className="space-y-3">
                {completedSessions.map((session, index) => (
                  <SessionCard
                    key={session.id}
                    session={session}
                    index={index}
                    onViewOutput={setSelectedSession}
                    showStopButton={false}
                    statusConfig={{
                      bgColor: 'bg-green-100',
                      iconColor: 'text-green-600'
                    }}
                  />
                ))}
              </div>
            )}
          </div>
          
          {/* Cancelled Sessions Section */}
          <div className="space-y-3">
            <button
              onClick={() => toggleSection('cancelled')}
              className="flex items-center space-x-2 w-full hover:opacity-80 transition-opacity"
            >
              <XCircle className="h-5 w-5 text-gray-600" />
              <h3 className="text-base font-medium text-muted-foreground">Cancelled ({cancelledSessions.length})</h3>
              <ChevronDown className={`h-4 w-4 text-muted-foreground transition-transform ${sectionsExpanded.cancelled ? '' : '-rotate-90'}`} />
            </button>
            {sectionsExpanded.cancelled && cancelledSessions.length > 0 && (
              <div className="space-y-3">
                {cancelledSessions.map((session, index) => (
                  <SessionCard
                    key={session.id}
                    session={session}
                    index={index}
                    onViewOutput={setSelectedSession}
                    onRetry={handleRetry}
                    onEdit={handleEdit}
                    showStopButton={false}
                    statusConfig={{
                      bgColor: 'bg-gray-100',
                      iconColor: 'text-gray-600'
                    }}
                  />
                ))}
              </div>
            )}
          </div>
        </div>
      )}

      {/* Session Output Viewer */}
      <AnimatePresence>
        {selectedSession && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 bg-background/80 backdrop-blur-sm z-50 flex items-center justify-center p-4"
          >
            <div className="w-full max-w-4xl h-full max-h-[90vh]">
              <SessionOutputViewer
                session={selectedSession}
                onClose={() => setSelectedSession(null)}
              />
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Toast Notification */}
      <ToastContainer>
        {toast && (
          <Toast
            message={toast.message}
            type={toast.type}
            onDismiss={() => setToast(null)}
          />
        )}
      </ToastContainer>
    </div>
  );
}