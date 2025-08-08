# Security Documentation

## Implemented Security Features ✅

### Authentication Security

#### Session Fixation Prevention
- **Auto-invalidation**: Sessions older than 30 days are automatically invalidated on login
- **Transaction Safety**: Session cleanup is performed within login transaction for consistency
- **Selective Cleanup**: Only affects very old sessions, preserves recent active sessions
- **Implementation**: `starter/src/auth/services.rs:198-208`

#### Timing Attack Protection
- **Constant-Time Comparison**: Password verification uses constant-time comparison to prevent timing analysis
- **Dummy Hash Processing**: Non-existent users trigger dummy hash computation to maintain consistent timing
- **Comprehensive Coverage**: Protects both login attempts and password validation during registration
- **Implementation**: `starter/src/users/models.rs:333+` and `starter/src/auth/services.rs:177-188`

#### Enhanced Password Validation
- **Case-Insensitive Common Password Detection**: Prevents use of common passwords regardless of case variations
- **RFC-Compliant Email Validation**: 75+ validation rules covering edge cases and malformed inputs
- **Comprehensive Security Testing**: 9 security vulnerability tests covering edge cases
- **Unicode and Special Character Handling**: Proper validation of international characters and security bypass attempts
- **Implementation**: `starter/src/users/models.rs:218-333`

#### Database Security
- **Error Propagation**: Replaced unsafe `unwrap_or(0)` patterns with proper error handling
- **Transaction Race Condition Fixes**: Fixed soft-delete operations to prevent concurrent modification issues
- **Consistent RBAC Error Handling**: Standardized authorization error responses across all endpoints
- **Implementation**: `starter/src/users/services.rs:498+`

### Background Tasks Security

#### Input Validation and Sanitization
- **Task Type Validation**: Alphanumeric characters, underscores, and hyphens only (128 char limit)
- **Payload Size Limits**: 1MB maximum to prevent DoS attacks
- **Metadata Sanitization**: 64KB total metadata limit, 4KB per value, character restrictions
- **Scheduling Restrictions**: Maximum 1 year future, 1 hour past tolerance
- **Implementation**: `starter/src/tasks/types.rs:237-314`

#### SQL Injection Protection
- **Priority Parameter Filtering**: Strict enum validation prevents SQL injection via priority parameter
- **Parameterized Queries**: All database queries use proper parameter binding
- **Input Sanitization**: User inputs validated before database operations
- **Implementation**: `starter/src/tasks/api.rs:223-229`

#### Authorization and Access Control
- **RBAC Stats Endpoint Protection**: `/tasks/stats` requires moderator+ permissions
- **Task Ownership Validation**: Users can only access their own tasks unless admin/moderator
- **Task Type Registration**: Prevents creation of tasks for unregistered types
- **Implementation**: `starter/src/tasks/api.rs:284`

#### Race Condition Prevention
- **Optimistic Concurrency Control**: Task status updates use valid state transition validation
- **Atomic Status Updates**: Database constraints prevent invalid state transitions
- **Semaphore Error Handling**: Proper error handling prevents resource exhaustion
- **Implementation**: `starter/src/tasks/processor.rs:441-491`

#### Task Security Validation Constants
- **MAX_TASK_TYPE_LEN**: 128 characters
- **MAX_PAYLOAD_SIZE_BYTES**: 1MB
- **MAX_METADATA_VALUE_SIZE_BYTES**: 4KB
- **MAX_TOTAL_METADATA_SIZE_BYTES**: 64KB
- **MAX_SCHEDULE_FUTURE_DAYS**: 365 days
- **MAX_SCHEDULE_PAST_HOURS**: 1 hour

### Monitoring & Observability Security

#### Authorization Boundaries
- **Source Ownership Validation**: Users can only create events for sources they own or generic sources
- **Metric Name Authorization**: Users restricted to generic metrics or user-prefixed metric names  
- **System Privilege Enforcement**: System sources/metrics (`system-*`, `health-*`, `monitoring-*`) require moderator+ role
- **Dynamic Ownership Checking**: Username and user ID-based ownership pattern validation
- **Implementation**: `starter/src/monitoring/api.rs:19-141`

#### Input Validation & DoS Protection
- **Comprehensive Field Limits**: Event types (50), sources (200), messages (10K), metric names (100 chars)
- **Tag Parsing Security**: Max 50 pairs, 100-char keys, 500-char values with character restrictions
- **Injection Prevention**: Alphanumeric + underscore/hyphen/dot only in tag keys
- **Duplicate Key Prevention**: Tag parsing prevents duplicate key attacks
- **Finite Value Validation**: Metrics must contain finite numbers (no NaN/infinity)
- **Implementation**: `starter/src/monitoring/models.rs:45-119`, `starter/src/monitoring/api.rs:152-235`

#### Database Security & Performance
- **Query Optimization**: Stats endpoint uses single CTE query instead of 6 separate queries
- **Transaction Integrity**: `SELECT FOR UPDATE` prevents race conditions in incident updates
- **Resource Protection**: Prometheus metrics limited to 10,000 items to prevent memory exhaustion
- **Error Message Security**: No UUID or sensitive data disclosure in error responses
- **From<String> Safety**: Enhanced enum conversion with critical error logging for data corruption detection
- **Implementation**: `starter/src/monitoring/services.rs:385-411`, `starter/src/monitoring/api.rs:686-722`

### Testing Coverage
- **182 Integration Tests**: Comprehensive test suite including security vulnerability tests
- **10 Monitoring Security Tests**: Dedicated security tests for input validation, authorization boundaries, error message security, resource protection, and transaction integrity
- **9 Tasks Security Tests**: Dedicated security vulnerability tests for tasks module covering SQL injection, DoS prevention, authorization bypass, race conditions, and input validation
- **Timing Attack Tests**: Specific tests to verify constant-time behavior
- **Session Fixation Tests**: Validates session cleanup behavior during login
- **RBAC Security Tests**: Complete role-based access control validation
- **Database Consistency Tests**: Transaction handling and error propagation validation
- **Authorization Edge Case Tests**: 28 tests covering ownership patterns and privilege escalation prevention

## Known Security Issues

### RUSTSEC-2023-0071: RSA Vulnerability in SQLx MySQL Dependency

**Status**: Acknowledged - Not affecting runtime security  
**Issue**: [SQLx GitHub Issue #2911](https://github.com/launchbadge/sqlx/issues/2911)  
**Advisory**: [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071)

#### Description
A Marvin Attack vulnerability exists in the `rsa` crate (version 0.9.8) that allows potential key recovery through timing side-channels. This vulnerability is present as a transitive dependency through `sqlx-mysql`.

#### Why This Doesn't Affect Our Application
1. **PostgreSQL Only**: This application exclusively uses PostgreSQL; no MySQL functionality is utilized at runtime
2. **Compile-Time Dependency**: The `sqlx-mysql` dependency is only included during compilation due to SQLx's macro system requirements
3. **No MySQL Code Paths**: No code paths in the application execute MySQL-related functionality where the RSA vulnerability could be exploited

#### Root Cause
SQLx's macro system requires all database driver dependencies to be available during compilation, even when only using a single database backend. This is a known limitation of the current SQLx architecture.

#### Mitigation
- Dependency is ignored in cargo audit configuration (`.cargo/audit.toml`)
- Issue is tracked upstream in SQLx repository
- Application security is not compromised as MySQL code is never executed
- Monitoring for SQLx updates that resolve this architectural issue

#### Verification
You can verify this doesn't affect runtime by checking:
1. Only PostgreSQL features are enabled in `Cargo.toml`
2. No MySQL connection strings or configurations exist
3. Application only connects to PostgreSQL databases

#### Future Resolution
This will be resolved when:
- SQLx fixes the macro system to only require actual used database drivers
- SQLx updates to a version of the `rsa` crate without this vulnerability
- Alternative database libraries are adopted that don't have this limitation