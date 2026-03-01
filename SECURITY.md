# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in ShipSecure, please report it responsibly. **Do not open a public GitHub issue.**

### How to Report

Email **security@shipsecure.ai** with:

- Description of the vulnerability
- Steps to reproduce
- Affected component (backend, frontend, scanning engine, infrastructure)
- Impact assessment (what an attacker could do)
- Any suggested fix (optional)

### What to Expect

- **Acknowledgment** within 48 hours
- **Initial assessment** within 5 business days
- **Fix timeline** communicated after assessment — critical issues are prioritized
- **Credit** in the fix commit and changelog (unless you prefer anonymity)

### Scope

The following are in scope:

- The ShipSecure application at shipsecure.ai
- The backend API (`/api/v1/*`)
- Authentication and authorization flows
- Scan result data exposure or leakage
- SSRF bypasses in the scan submission flow
- Rate limiting bypasses

The following are **out of scope**:

- Vulnerabilities in third-party dependencies with no demonstrated exploit path
- Denial-of-service attacks
- Social engineering
- Issues in the development environment only (not production)

### Safe Harbor

We will not pursue legal action against researchers who:

- Report vulnerabilities through the process above
- Avoid accessing or modifying other users' data
- Do not disrupt the service (no DoS, no data destruction)
- Allow reasonable time for a fix before public disclosure

## Supported Versions

| Version | Supported |
|---------|-----------|
| Latest (main branch) | Yes |
| Older releases | No |

ShipSecure is deployed continuously from `main`. Only the latest production deployment receives security fixes.
