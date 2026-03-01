# Contributing to ShipSecure

Thanks for your interest in contributing to ShipSecure. This document covers the process for reporting bugs, requesting features, and submitting code.

## Reporting Bugs

[Open a bug report](https://github.com/TrustEdge-Labs/shipsecure/issues/new?template=bug_report.md) with:

- What happened vs. what you expected
- Steps to reproduce
- Environment details (browser, OS)
- Scan ID or results token if relevant (no sensitive data)

## Requesting Features

[Open a feature request](https://github.com/TrustEdge-Labs/shipsecure/issues/new?template=feature_request.md) with:

- The problem you're trying to solve
- Your proposed solution
- Alternatives you've considered

## Submitting Code

### Before You Start

1. Check [open issues](https://github.com/TrustEdge-Labs/shipsecure/issues) to see if someone is already working on it
2. For non-trivial changes, open an issue first to discuss the approach
3. Fork the repository and create a branch from `main`

### Development Setup

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/shipsecure.git
cd shipsecure

# Backend
cp .env.example .env
# Edit .env with your configuration
cargo build
cargo run

# Frontend (separate terminal)
cd frontend
cp .env.example .env
# Edit .env with Clerk keys
npm install
npm run dev
```

See the [README](README.md) for full setup details including Docker and prerequisites.

### Code Standards

**Backend (Rust):**

- Run `cargo clippy` — no warnings
- Run `cargo fmt` — consistent formatting
- Add tests for new scanner logic

**Frontend (TypeScript/React):**

- Run `npm run lint` — ESLint passes
- Run `npm test` — Vitest passes
- Coverage thresholds: 80% lines / 80% functions / 75% branches (scoped to `components/**`)
- Use existing design tokens — don't hardcode colors or spacing

### Pull Request Process

1. Fill out the [PR template](.github/PULL_REQUEST_TEMPLATE.md)
2. Ensure CI passes (unit tests + E2E tests)
3. Keep PRs focused — one feature or fix per PR
4. Update documentation if you change API endpoints or env vars

### Branch Protection

`main` is protected:

- All CI checks must pass
- No force pushes
- No admin bypass

### Commit Messages

Use conventional commits:

```
feat: add CORS header scanner
fix: handle timeout in TLS scanner
docs: update API endpoint table
test: add E2E test for domain verification
```

## Adding Nuclei Templates

The easiest way to contribute is by adding vibe-code detection templates. See the [Extending ShipSecure](#extending-shipsecure) section in the README for the template format and examples.

Templates go in `templates/nuclei/` and follow standard [Nuclei template syntax](https://docs.projectdiscovery.io/templates/introduction).

## Security Vulnerabilities

Do **not** open public issues for security vulnerabilities. See [SECURITY.md](SECURITY.md) for the responsible disclosure process.

## License

By contributing, you agree that your contributions will be licensed under the [Mozilla Public License 2.0](LICENSE).
