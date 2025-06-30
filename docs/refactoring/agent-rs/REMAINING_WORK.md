# Remaining Work for Agents Module Refactoring

## Immediate Tasks (Before Merge)

### 1. Fix unwrap() Usage (9 instances)
```rust
// Location: execute.rs
- tx.send(output).unwrap();
+ if let Err(e) = tx.send(output) {
+     error!("Failed to send output: {}", e);
+ }

// Location: pool.rs
- pool.get().unwrap()
+ pool.get().map_err(|e| AgentError::Database(e.to_string()))?

// Location: repository.rs (4 instances)
- row.get("id").unwrap()
+ row.get("id").map_err(|e| AgentError::Database(e.to_string()))?
```

### 2. Remove Unused Imports
```bash
# Run this command to auto-fix:
cargo fix --allow-dirty

# Or manually remove:
- use tauri::Manager;
- use log::debug;
- use rusqlite::Connection;
```

### 3. Add Missing Tests
Create integration tests in `src-tauri/tests/agents_integration.rs`:
- Test full command flow
- Test error scenarios
- Test concurrent execution
- Test process cleanup

## Short-term Tasks (Next Sprint)

### 4. Add Documentation
```rust
/// Repository for agent database operations
/// 
/// Provides CRUD operations for agents and their runs,
/// managing database transactions and connection pooling.
pub struct AgentRepository { ... }
```

### 5. Implement Caching
- Add memory cache for frequently accessed agents
- Cache invalidation on updates
- TTL-based expiration

### 6. Performance Monitoring
- Add metrics collection
- Query performance tracking
- Connection pool monitoring

## Future Enhancements

### 7. Advanced Features
- Batch operations for agents
- Agent templates/presets
- Import/export functionality
- Version history for agents

### 8. Security Hardening
- Input sanitization
- Rate limiting
- Audit logging
- Permission system

### 9. Observability
- Structured logging
- Distributed tracing
- Performance metrics
- Error tracking

## Technical Debt

### 10. Query Optimization
```sql
-- Add indexes
CREATE INDEX idx_agent_runs_agent_id ON agent_runs(agent_id);
CREATE INDEX idx_agent_runs_status ON agent_runs(status);
CREATE INDEX idx_agent_runs_created_at ON agent_runs(created_at);
```

### 11. Error Recovery
- Implement retry logic
- Circuit breaker pattern
- Graceful degradation
- Fallback strategies

### 12. Testing Infrastructure
- Mock database for tests
- Test fixtures
- Property-based testing
- Stress testing

## Checklist for Completion

- [ ] All unwrap() calls replaced
- [ ] Unused imports removed
- [ ] Integration tests added
- [ ] Documentation complete
- [ ] Performance baseline established
- [ ] Migration guide updated
- [ ] Code review completed
- [ ] Manual testing done
- [ ] Frontend integration verified
- [ ] Deployment plan ready

## Time Estimates

| Task | Priority | Effort | Assignee |
|------|----------|--------|----------|
| Fix unwrap() | High | 2h | - |
| Remove imports | High | 30m | - |
| Integration tests | High | 4h | - |
| Documentation | Medium | 3h | - |
| Caching | Low | 8h | - |
| Performance | Low | 6h | - |

**Total immediate work**: ~6.5 hours
**Total planned work**: ~21.5 hours