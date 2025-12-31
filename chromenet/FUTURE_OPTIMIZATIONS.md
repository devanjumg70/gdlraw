do # Future Enhancements

These items were identified during the Performance Optimization phase but were determined to be lower priority or outside current scope.

## 1. Cookie Import Optimization (Performance - P2)
**Location:** `src/cookies/browser.rs`
**Issue:** `import_from_browser()` currently executes `SELECT * FROM cookies` and filters on the client side (Rust).
**Impact:** Loads potentially thousands of irrelevant cookies into memory when importing a specific domain.
**Fix:** Push the filter down to the SQL query.
```sql
SELECT * FROM cookies WHERE host_key LIKE '%example.com'
```
**Effort:** ~2 hours.

## 2. Auth Password Zeroizing (Security - P3)
**Location:** `src/socket/authcache.rs`
**Issue:** Passwords in `BasicAuthEntry` and `DigestAuthSession` are stored as plain `String`.
**Impact:** Credentials could remain in memory after use if the memory is not explicitly scrubbed.
**Fix:** Use `zeroize::Zeroizing<String>` or `secrecy::Secret<String>` for password fields.
**Effort:** ~1 hour.
