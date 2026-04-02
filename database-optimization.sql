-- Database Query Optimizations for Beads Issue Tracker
-- Generated: 2026-04-02
-- Target: Optimize frequently executed queries for better performance

-- ============================================================================
-- ANALYSIS: Current State
-- ============================================================================
-- Total issues: 187
-- Primary query patterns:
--   1. Filter by status + priority (ready issues)
--   2. Filter by assignee + status (user's work)
--   3. Filter by created_at (recent activity)
--   4. JOIN with dependencies for blockers
--   5. GROUP BY status, priority for reporting

-- ============================================================================
-- OPTIMIZATION 1: Composite Index for Status + Priority Queries
-- ============================================================================
-- Common pattern: SELECT * FROM issues WHERE status = 'open' AND priority = 2
-- Current: Two separate indexes (idx_issues_status, idx_issues_priority)
-- Optimized: Single composite index for combined filtering

CREATE INDEX IF NOT EXISTS idx_issues_status_priority 
ON issues(status, priority);

-- ============================================================================
-- OPTIMIZATION 2: Composite Index for Assignee + Status Queries  
-- ============================================================================
-- Common pattern: SELECT * FROM issues WHERE assignee = 'user' AND status IN ('open', 'in_progress')
-- Current: Only idx_issues_assignee exists
-- Optimized: Composite index for user-specific work queries

CREATE INDEX IF NOT EXISTS idx_issues_assignee_status 
ON issues(assignee, status);

-- ============================================================================
-- OPTIMIZATION 3: Covering Index for Status Reports
-- ============================================================================
-- Common pattern: SELECT status, priority, COUNT(*) FROM issues GROUP BY status, priority
-- Current: Requires table lookup after index scan
-- Optimized: Covering index includes all needed columns

CREATE INDEX IF NOT EXISTS idx_issues_status_priority_count 
ON issues(status, priority, issue_type);

-- ============================================================================
-- OPTIMIZATION 4: Index for Recent Activity Queries
-- ============================================================================
-- Common pattern: SELECT * FROM issues WHERE updated_at > NOW() - INTERVAL 7 DAY
-- Current: No index on updated_at
-- Optimized: Index for time-based filtering

CREATE INDEX IF NOT EXISTS idx_issues_updated_at 
ON issues(updated_at);

-- ============================================================================
-- OPTIMIZATION 5: Composite Index for Created + Status
-- ============================================================================
-- Common pattern: SELECT * FROM issues WHERE status = 'open' ORDER BY created_at DESC
-- Current: Separate indexes, requires filesort
-- Optimized: Composite index for sorted queries

CREATE INDEX IF NOT EXISTS idx_issues_status_created 
ON issues(status, created_at DESC);

-- ============================================================================
-- OPTIMIZATION 6: Index for Dependency/Blocker Queries
-- ============================================================================
-- Common pattern: JOIN dependencies ON issues.id = dependencies.blocker_id
-- Check if dependencies table exists and needs optimization

-- First, check dependencies table structure
-- DESCRIBE dependencies;

-- Create index if blocker_id column exists
-- CREATE INDEX IF NOT EXISTS idx_dependencies_blocker 
-- ON dependencies(blocker_id, blocked_id);

-- ============================================================================
-- OPTIMIZATION 7: Composite Index for Priority + Created Date
-- ============================================================================
-- Common pattern: SELECT * FROM issues WHERE priority = 1 ORDER BY created_at
-- Optimized: Composite index for prioritized work queries

CREATE INDEX IF NOT EXISTS idx_issues_priority_created 
ON issues(priority, created_at);

-- ============================================================================
-- OPTIMIZATION 8: Index for Issue Type Filtering
-- ============================================================================
-- Common pattern: SELECT * FROM issues WHERE issue_type = 'task' AND status = 'open'
-- Current: idx_issues_issue_type exists but not combined with status
-- Optimized: Composite index for type + status queries

CREATE INDEX IF NOT EXISTS idx_issues_type_status 
ON issues(issue_type, status);

-- ============================================================================
-- OPTIMIZATION 9: Index for Owner-Based Queries
-- ============================================================================
-- Common pattern: SELECT * FROM issues WHERE owner = 'username'
-- Current: No index on owner column
-- Optimized: Index for owner-specific queries

CREATE INDEX IF NOT EXISTS idx_issues_owner 
ON issues(owner);

-- ============================================================================
-- OPTIMIZATION 10: Composite Index for Owner + Status
-- ============================================================================
-- Common pattern: SELECT * FROM issues WHERE owner = 'user' AND status = 'open'
-- Optimized: Composite index for owner's open work

CREATE INDEX IF NOT EXISTS idx_issues_owner_status 
ON issues(owner, status);

-- ============================================================================
-- VERIFICATION QUERIES
-- ============================================================================

-- Check all indexes created
SHOW INDEX FROM issues;

-- Get index statistics
SELECT 
    INDEX_NAME,
    GROUP_CONCAT(COLUMN_NAME ORDER BY SEQ_IN_INDEX) AS columns,
    NON_UNIQUE
FROM information_schema.STATISTICS 
WHERE TABLE_NAME = 'issues' AND TABLE_SCHEMA = 'pi_autoresearch'
GROUP BY INDEX_NAME, NON_UNIQUE
ORDER BY INDEX_NAME;

-- ============================================================================
-- QUERY OPTIMIZATION EXAMPLES
-- ============================================================================

-- Before optimization (may use multiple indexes or full table scan):
-- SELECT * FROM issues WHERE status = 'open' AND priority = 2;

-- After optimization (uses idx_issues_status_priority):
-- EXPLAIN SELECT * FROM issues WHERE status = 'open' AND priority = 2;

-- Before optimization (filesort required):
-- SELECT * FROM issues WHERE status = 'open' ORDER BY created_at DESC LIMIT 10;

-- After optimization (uses idx_issues_status_created, no filesort):
-- EXPLAIN SELECT * FROM issues WHERE status = 'open' ORDER BY created_at DESC LIMIT 10;

-- Before optimization (table lookup after index):
-- SELECT status, priority, COUNT(*) FROM issues GROUP BY status, priority;

-- After optimization (covering index idx_issues_status_priority_count):
-- EXPLAIN SELECT status, priority, COUNT(*) FROM issues GROUP BY status, priority;

-- ============================================================================
-- MAINTENANCE: Analyze Table for Updated Statistics
-- ============================================================================
-- Run ANALYZE to update query optimizer statistics
ANALYZE TABLE issues;

-- ============================================================================
-- PERFORMANCE TESTING
-- ============================================================================
-- Test query performance before and after optimizations

-- Test 1: Ready issues query
-- EXPLAIN ANALYZE SELECT * FROM issues WHERE status = 'open' AND priority = 2;

-- Test 2: User's work query  
-- EXPLAIN ANALYZE SELECT * FROM issues WHERE assignee = 'gagecane' AND status IN ('open', 'in_progress');

-- Test 3: Recent activity query
-- EXPLAIN ANALYZE SELECT * FROM issues WHERE updated_at > NOW() - INTERVAL 7 DAY ORDER BY updated_at DESC;

-- Test 4: Status report query
-- EXPLAIN ANALYZE SELECT status, priority, COUNT(*) FROM issues GROUP BY status, priority;

-- Test 5: Owner's open issues
-- EXPLAIN ANALYZE SELECT * FROM issues WHERE owner = 'gagecane' AND status = 'open' ORDER BY priority, created_at;

