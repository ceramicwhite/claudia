-- Fix the status of resumed runs that completed successfully but were marked as paused_usage_limit
-- This query finds resumed runs (those with parent_run_id) that are marked as paused_usage_limit
-- but should be marked as completed based on their completion status

-- First, let's see what needs to be fixed (dry run)
SELECT 
    id, 
    agent_name, 
    status, 
    parent_run_id,
    created_at,
    completed_at,
    session_id
FROM agent_runs 
WHERE parent_run_id IS NOT NULL 
  AND status = 'paused_usage_limit'
  AND completed_at IS NOT NULL;

-- To fix the status, uncomment and run this UPDATE statement:
-- UPDATE agent_runs 
-- SET status = 'completed'
-- WHERE parent_run_id IS NOT NULL 
--   AND status = 'paused_usage_limit'
--   AND completed_at IS NOT NULL;

-- Specifically for run ID 75 that you mentioned:
UPDATE agent_runs 
SET status = 'completed'
WHERE id = 75;