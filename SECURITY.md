# Security Updates

## Recent Security Fixes

### Astro XSS Vulnerability (January 2026)

**Vulnerability:** CVE-2024-XXXXX - Reflected XSS via server islands feature in Astro  
**Affected Versions:** Astro <= 5.15.6  
**Fixed Version:** Astro 5.15.8  
**Severity:** High  

**Update Applied:**
- Updated `astro` from `^4.11.3` to `^5.15.8`
- Updated `@astrojs/react` from `^3.6.0` to `^4.0.0`
- Updated `@astrojs/tailwind` from `^5.1.0` to `^6.0.0`

**Impact:**
This vulnerability only affects the Web UI component when built in release mode. The API server and core process management functionality are not affected.

**Mitigation:**
If you are using an older build with the vulnerable Astro version:
1. Rebuild the project: `cargo build --release`
2. Restart the daemon with web UI: `opm daemon restore --api --webui`

## Reporting Security Issues

If you discover a security vulnerability in OPM, please report it by:
1. **DO NOT** open a public issue
2. Email the maintainer directly (check repository for contact)
3. Include detailed steps to reproduce
4. Allow reasonable time for a fix before public disclosure

## Security Best Practices

When using OPM with the Web UI/API server:

### Authentication
- **Always enable token authentication** in production environments
- Use strong, randomly generated tokens (min 32 characters)
- Rotate tokens regularly
- Never commit tokens to version control

```bash
# Generate a secure token
openssl rand -hex 32
```

### Network Configuration
- Default binding to `127.0.0.1` (localhost only) is secure for single-user systems
- For remote access, use a reverse proxy (nginx, caddy) with HTTPS
- Consider firewall rules to restrict access
- Use VPN for remote management when possible

### Configuration Example
```toml
[daemon.web]
ui = true
api = true
address = "127.0.0.1"  # Localhost only
port = 9876

[daemon.web.secure]
enabled = true
token = "<generated-secure-token>"
```

### Production Deployment
- Use HTTPS (via reverse proxy)
- Enable token authentication
- Restrict network access with firewall
- Run with least privilege user
- Keep dependencies updated
- Monitor access logs
- Use rate limiting on API endpoints

## Security Update Process

1. Security vulnerabilities are monitored in all dependencies
2. Updates are applied as soon as patches are available
3. Security releases are tagged and documented
4. Users are notified via GitHub releases and security advisories

## Dependency Scanning

We regularly scan dependencies for known vulnerabilities. Current tooling:
- `cargo audit` for Rust dependencies
- `npm audit` for Node.js/frontend dependencies
- GitHub Dependabot alerts
- Manual review of security advisories

To check for vulnerabilities yourself:
```bash
# Rust dependencies
cargo install cargo-audit
cargo audit

# Node.js dependencies (requires release build)
cd src/webui
npm audit
```

## Changelog

### 2026-01-11
- Fixed Astro reflected XSS vulnerability (CVE-2024-XXXXX)
- Updated Astro to v5.15.8
- Updated Astro integrations for compatibility

## References

- [Astro Security Advisory](https://github.com/withastro/astro/security/advisories)
- [OWASP Web Security Testing Guide](https://owasp.org/www-project-web-security-testing-guide/)
- [CWE-79: Cross-site Scripting](https://cwe.mitre.org/data/definitions/79.html)
