# Redirect Testing Findings
Based on `net/url_request/url_request_unittest.cc`:

## Chromium Pattern
- **RedirectLoopTest**: Sets up a server that redirects A -> B -> A.
- **Assertion**: `ERR_TOO_MANY_REDIRECTS`.
- **Mechanism**: `URLRequestJob` follows redirect, increments count. `URLRequest` checks `kMaxRedirects` (usually 20).

## Chromenet Gap
- We have `URLRequest` redirect following, but:
    - We rely on `reqwest` or `hyper`? No, we implemented `HttpNetworkTransaction`.
    - `HttpNetworkTransaction` has a manual `do_loop`.
    - **Critical**: Does `HttpNetworkTransaction` actually *count* redirects?
    - Let's check `src/http/transaction.rs`. If it loops forever, that's a bug.
- **Action**: Verify loop detection in code. If missing, this is a **Critical Finding** to document and fix.

## Auth Stripping
- Chromium `RedirectInfo` logic removes `Authorization` header on cross-origin redirects.
- **Action**: Verify `chromenet` does this.

## Fragment Preservation
- Redirects should preserve `#fragment` if the new URL doesn't specify one.

# Plan
1.  Inspect `src/http/transaction.rs` for redirect counting.
2.  Write `tests/redirect_test.rs` to reproduce infinite loop (if exists).
3.  Fix it.
