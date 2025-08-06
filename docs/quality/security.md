# Security Documentation

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