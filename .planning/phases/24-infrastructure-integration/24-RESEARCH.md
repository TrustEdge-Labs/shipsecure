# Phase 24: Infrastructure Integration - Research

**Researched:** 2026-02-16
**Domain:** Production infrastructure deployment, monitoring agent integration, security hardening, container lifecycle management
**Confidence:** HIGH

## Summary

Phase 24 integrates all observability components into production infrastructure by deploying the DigitalOcean metrics agent, hardening Nginx to protect the /metrics endpoint, configuring Docker Compose for graceful shutdown and log rotation, extending systemd timeouts, and enabling JSON logging in production. This is a pure infrastructure phase with no code changes - all modifications are to Ansible playbooks, templates, and configuration files.

The phase follows the 12-factor app pattern established in earlier phases (LOG_FORMAT env var for logging, SHUTDOWN_TIMEOUT for graceful shutdown). Infrastructure changes use Ansible's declarative model with templates and idempotent tasks. The key technical challenge is coordinating timeout values across three layers: Docker stop_grace_period (90s), systemd TimeoutStopSec (95s to accommodate Docker's timeout plus buffer), and the application's SHUTDOWN_TIMEOUT (90s, configured in Phase 23).

**Primary recommendation:** Use Ansible template module for all configuration deployment, systemd drop-in files for service modifications, and DigitalOcean's official installation script for metrics agent deployment. Verify each change independently before combining them.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| INF-01 | DigitalOcean metrics agent installed via Ansible playbook | Official installation script via curl, systemd service verification, metrics appear in control panel within minutes |
| INF-02 | Nginx restricts /metrics endpoint to localhost only (deny all external) | Location block with allow 127.0.0.1/::1 and deny all directives, access_log off for metrics scraping |
| INF-03 | Docker Compose configured with STOPSIGNAL, stop_grace_period, and JSON log rotation | stop_signal: SIGTERM, stop_grace_period: 90s, logging driver json-file with max-size 10m and max-file 3 |
| INF-04 | systemd TimeoutStopSec extended to accommodate scan drain timeout | Drop-in file or direct service modification setting TimeoutStopSec=95s (Docker grace + buffer) |
| INF-05 | LOG_FORMAT=json set in production environment configuration | Ansible template for .env file includes LOG_FORMAT=json, follows 12-factor pattern |

</phase_requirements>

## Standard Stack

### Core Infrastructure

| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| Ansible | 2.15+ | Infrastructure automation and configuration management | Declarative infrastructure-as-code, idempotent operations, used throughout project |
| DigitalOcean Metrics Agent (do-agent) | Latest (stable) | System metrics collection (CPU, memory, disk) | Official DigitalOcean monitoring solution, free tier included, 14-day retention |
| Nginx | 1.18+ | Reverse proxy and /metrics endpoint access control | Already deployed, battle-tested access control directives |
| Docker Compose | v2.18.0+ | Container orchestration with graceful shutdown | Project standard, supports v2 features (stop_signal, stop_grace_period) |
| systemd | 247+ (Debian 11/Ubuntu 20.04+) | Service lifecycle management | Linux standard init system, manages Docker Compose via unit file |

### Supporting Tools

| Tool | Version | Purpose | When to Use |
|------|---------|---------|-------------|
| Ansible Template Module | Built-in | Deploy Jinja2 templates with variable substitution | All configuration file deployment (.env, docker-compose.yml, nginx conf) |
| Ansible systemd Module | Built-in | Manage systemd services (enable, start, daemon-reload) | Service management after configuration changes |
| systemd drop-in files | systemd built-in | Override service parameters without modifying main unit file | Safer than editing /etc/systemd/system/service files directly |

### Installation Commands

**DigitalOcean Metrics Agent:**
```bash
# Automated script (recommended for Ansible)
curl -sSL https://repos.insights.digitalocean.com/install.sh | sudo bash

# Verify installation
systemctl status do-agent
ps aux | grep do-agent
```

**No new Ansible collections required** - uses existing ansible.builtin modules (template, systemd, shell, uri).

## Architecture Patterns

### Recommended Project Structure

```
infrastructure/
├── tasks/
│   ├── metrics-agent.yml      # New: Install and verify DO metrics agent
│   ├── nginx.yml               # Modified: Add /metrics location block
│   ├── app.yml                 # Modified: Update .env template with LOG_FORMAT
│   └── systemd.yml             # Modified: Add drop-in or update TimeoutStopSec
├── templates/
│   ├── docker-compose.production.yml.j2  # Modified: Add STOPSIGNAL, stop_grace_period, logging
│   ├── trustedge.nginx.conf.j2          # Modified: Add /metrics location block
│   ├── trustedge.service.j2             # Modified: Update TimeoutStopSec to 95s
│   ├── trustedge.service.d/
│   │   └── override.conf.j2              # Alternative: Drop-in file for TimeoutStopSec
│   └── env.production.j2                # Modified: Add LOG_FORMAT=json
└── playbooks/
    ├── provision.yml            # Modified: Import metrics-agent tasks
    └── resume-app.yml          # Modified: Import metrics-agent tasks
```

### Pattern 1: Nginx Metrics Endpoint Security

**What:** Restrict Prometheus /metrics endpoint to localhost only while keeping other endpoints publicly accessible.

**When to use:** Protecting internal monitoring endpoints from external access.

**Example:**
```nginx
# Source: https://docs.nginx.com/nginx/admin-guide/security-controls/controlling-access-proxied-http/
# Add to trustedge.nginx.conf.j2 in the HTTPS server block

# Metrics endpoint — restricted to localhost for Prometheus scraping
location /metrics {
    proxy_pass http://127.0.0.1:3000/metrics;
    proxy_http_version 1.1;

    # Allow only localhost access
    allow 127.0.0.1;
    allow ::1;
    deny all;

    # Don't log every metrics scrape (high frequency)
    access_log off;

    # Standard proxy headers
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
}
```

**Key points:**
- Order matters: allow directives are evaluated before deny all
- Both IPv4 (127.0.0.1) and IPv6 (::1) localhost addresses must be allowed
- access_log off prevents high-frequency scraping from filling logs
- Test with `curl https://yourdomain.com/metrics` (should return 403) and `curl http://127.0.0.1:3000/metrics` from server (should return metrics)

### Pattern 2: Docker Compose Graceful Shutdown Configuration

**What:** Configure Docker containers to receive SIGTERM, wait for application shutdown, then send SIGKILL after grace period.

**When to use:** Applications with long-running operations that need time to complete during shutdown.

**Example:**
```yaml
# Source: https://oneuptime.com/blog/post/2026-01-16-docker-graceful-shutdown-signals/view
services:
  backend:
    image: ghcr.io/trustedge-labs/trustedge-backend:latest
    stop_signal: SIGTERM
    stop_grace_period: 90s  # Matches SHUTDOWN_TIMEOUT from Phase 23
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

**Key coordination points:**
- Application SHUTDOWN_TIMEOUT (Phase 23): 90s - how long app drains scans
- Docker stop_grace_period: 90s - how long Docker waits before SIGKILL
- systemd TimeoutStopSec: 95s - how long systemd waits before killing Docker Compose
- **Critical:** systemd timeout MUST be longer than Docker timeout (add 5s buffer)

**Why SIGTERM is correct:** Rust applications (and most well-behaved programs) handle SIGTERM for graceful shutdown. SIGINT is for interactive Ctrl+C. Docker sends SIGTERM by default, but explicitly configuring it ensures consistency.

### Pattern 3: systemd Drop-in Override Files

**What:** Modify systemd service parameters without editing the main unit file.

**When to use:** Changing service timeouts, adding environment variables, or overriding any service parameter while keeping the main unit file maintainable.

**Example:**
```ini
# Source: https://www.baeldung.com/linux/systemd-modify-config
# File: /etc/systemd/system/trustedge.service.d/override.conf
[Service]
# Extend timeout to accommodate Docker's 90s grace period plus buffer
# Default was 30s, which would kill docker-compose mid-shutdown
TimeoutStopSec=95s
```

**Ansible deployment:**
```yaml
# Source: https://sleeplessbeastie.eu/2020/02/24/how-to-modify-systemd-service-configuration/
- name: Create systemd drop-in directory
  file:
    path: /etc/systemd/system/trustedge.service.d
    state: directory
    mode: '0755'

- name: Deploy systemd timeout override
  template:
    src: ../templates/trustedge.service.d/override.conf.j2
    dest: /etc/systemd/system/trustedge.service.d/override.conf
    mode: '0644'
  notify: Reload systemd daemon

# Handler
- name: Reload systemd daemon
  systemd:
    daemon_reexec: yes
```

**Alternative:** Modify trustedge.service.j2 directly and change TimeoutStopSec=30 to TimeoutStopSec=95s. This is simpler but less flexible if other overrides are needed later.

### Pattern 4: DigitalOcean Metrics Agent Installation

**What:** Install DO metrics agent via official script, verify service status, and confirm metrics appear in control panel.

**When to use:** Enabling DigitalOcean's infrastructure monitoring for droplets.

**Example:**
```yaml
# Source: https://docs.digitalocean.com/products/monitoring/how-to/install-metrics-agent/
- name: Check if DigitalOcean metrics agent is already installed
  command: systemctl status do-agent
  register: do_agent_status
  failed_when: false
  changed_when: false

- name: Install DigitalOcean metrics agent
  shell: |
    curl -sSL https://repos.insights.digitalocean.com/install.sh | sudo bash
  args:
    warn: false
  when: do_agent_status.rc != 0

- name: Verify metrics agent is running
  systemd:
    name: do-agent
    state: started
    enabled: yes

- name: Wait for metrics agent to start reporting
  pause:
    seconds: 30
    prompt: "Waiting for metrics agent to initialize and start reporting data"

- name: Display metrics agent status
  command: systemctl status do-agent
  changed_when: false
```

**What the agent collects:**
- CPU: Usage percentage, user/system time, iowait
- Memory: Used, free, cached, buffers, available
- Disk: I/O operations, read/write bytes, utilization
- Network: Inbound/outbound bytes, packets, errors

**No configuration required:** Agent auto-detects the droplet and reports to DigitalOcean's monitoring API using the droplet's internal metadata.

### Pattern 5: Environment File Template with LOG_FORMAT

**What:** Add LOG_FORMAT=json to production .env file while keeping it unset (defaulting to text) in development.

**When to use:** Following 12-factor app pattern for environment-specific configuration.

**Example:**
```jinja2
# File: templates/env.production.j2
# Source: Project's existing .env.example and Phase 19 decisions

# ========================================
# OBSERVABILITY CONFIGURATION
# ========================================

# Log output format (OPTIONAL)
# json = structured JSON output (for production log aggregation)
# Anything else or unset = plain text (for development)
# Default: text
LOG_FORMAT=json

# Logging level and filter (OPTIONAL)
# Production should use info level to balance visibility and noise
RUST_LOG=info,trustedge_audit=info

# Shutdown timeout (OPTIONAL)
# How long to wait for in-flight scans to complete before forced shutdown
# Default: 90s (accommodates longest SSL Labs scans)
SHUTDOWN_TIMEOUT=90
```

**Why this pattern:**
- Development uses text logs (unset LOG_FORMAT or LOG_FORMAT=text) for human readability
- Production uses JSON logs (LOG_FORMAT=json) for aggregation and parsing
- Ansible template includes LOG_FORMAT=json, deployed only to production
- Application code (Phase 19) already supports this via env var check

### Anti-Patterns to Avoid

- **Hardcoding timeouts:** Don't use different values across layers - coordinate Docker stop_grace_period, systemd TimeoutStopSec, and SHUTDOWN_TIMEOUT to avoid premature termination
- **Ignoring systemd buffer:** Don't set systemd TimeoutStopSec equal to Docker's grace period - add 5s buffer to ensure systemd waits for Docker's cleanup
- **Forgetting daemon-reload:** After modifying systemd unit files or drop-ins, always run `systemctl daemon-reload` or Ansible will apply stale configuration
- **Testing metrics endpoint externally first:** Test localhost access first (`curl http://127.0.0.1:3000/metrics` from server), verify it works, THEN test external denial
- **Editing running service files:** Use Ansible templates and drop-ins, not manual edits - manual changes are lost on next playbook run

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| System metrics collection | Custom CPU/memory/disk monitoring scripts | DigitalOcean metrics agent | Officially supported, free, integrated with DO control panel, 14-day retention, alerting |
| Nginx access control | Custom authentication middleware | Nginx allow/deny directives | Battle-tested, evaluated at network layer (faster), no application code needed |
| Docker log rotation | Logrotate scripts for container logs | json-file driver with max-size/max-file | Built into Docker, rotates atomically, works with docker logs command |
| systemd service modification | sed/awk editing /etc/systemd/system files | systemd drop-in override files | Safer, easier to version control, persists across package updates |
| Environment-specific config | Separate codebases or conditional logic | 12-factor app env vars + Ansible templates | Clean separation of code and config, auditable, rollback-friendly |

**Key insight:** Infrastructure automation (Ansible) and system-level configurations (systemd, Docker, Nginx) are mature domains with well-established tools. Custom solutions introduce brittleness, maintenance burden, and edge cases that standard tools already handle.

## Common Pitfalls

### Pitfall 1: Timeout Misalignment Causing Premature Shutdown

**What goes wrong:** systemd kills docker-compose before containers finish draining, losing in-flight scans.

**Why it happens:** systemd's TimeoutStopSec (default or too low) expires while Docker is still waiting for container graceful shutdown (stop_grace_period). systemd sends SIGKILL to docker-compose, which force-kills containers.

**How to avoid:**
1. Set application SHUTDOWN_TIMEOUT to drain scans (90s - matches longest scan duration)
2. Set Docker stop_grace_period to match application timeout (90s)
3. Set systemd TimeoutStopSec to Docker timeout + buffer (95s minimum)

**Warning signs:**
- Logs show "Shutdown forced: N scans remaining after 90s" but systemd service stop takes < 90s
- Docker containers show exit code 137 (SIGKILL) instead of 0 (clean exit)
- systemd logs show "State 'stop-sigterm' timed out. Killing."

**Verification:**
```bash
# Check effective timeout values
systemctl show trustedge.service -p TimeoutStopUSec  # Should show 1min 35s (95s)
docker inspect trustedge-backend-1 | grep -A5 StopTimeout  # Should show 90
grep SHUTDOWN_TIMEOUT /opt/trustedge/.env  # Should show 90
```

### Pitfall 2: Metrics Endpoint Exposed to Public Internet

**What goes wrong:** Prometheus /metrics endpoint accessible externally, leaking infrastructure details (request rates, scan queue depth, error rates).

**Why it happens:** Forgetting to add access control for new endpoints, or Nginx configuration error allowing fallthrough to default backend proxy.

**How to avoid:**
- Add /metrics location block BEFORE the catch-all `location /` block (Nginx evaluates in order)
- Test external access returns 403: `curl -I https://yourdomain.com/metrics`
- Test localhost access returns metrics: `ssh deploy@host "curl http://127.0.0.1:3000/metrics"`
- Include verification step in Ansible playbook after Nginx reload

**Warning signs:**
- `curl https://yourdomain.com/metrics` returns metrics data (should return 403)
- No "access forbidden" logs in /var/log/nginx/error.log when external request hits /metrics

**Prevention in Ansible:**
```yaml
- name: Verify metrics endpoint is restricted externally
  uri:
    url: https://{{ domain_name }}/metrics
    status_code: 403
    validate_certs: yes
  delegate_to: localhost
  become: no

- name: Verify metrics endpoint works from localhost
  uri:
    url: http://127.0.0.1:3000/metrics
    status_code: 200
  register: metrics_check
  failed_when: "'# HELP' not in metrics_check.content"
```

### Pitfall 3: Forgetting daemon-reload After systemd Changes

**What goes wrong:** systemd continues using old configuration even after Ansible deploys new unit file or drop-in.

**Why it happens:** systemd caches unit file definitions. Changes to files on disk don't take effect until `systemctl daemon-reload` (or `daemon-reexec`) is called.

**How to avoid:**
- Use Ansible handlers to automatically reload after template changes
- Call `systemctl daemon-reload` before restarting services
- Verify effective configuration with `systemctl show` after deployment

**Example handler pattern:**
```yaml
- name: Deploy systemd service unit
  template:
    src: trustedge.service.j2
    dest: /etc/systemd/system/trustedge.service
  notify: Reload systemd daemon

- name: Deploy systemd drop-in override
  template:
    src: override.conf.j2
    dest: /etc/systemd/system/trustedge.service.d/override.conf
  notify: Reload systemd daemon

# Handler section
handlers:
  - name: Reload systemd daemon
    systemd:
      daemon_reload: yes
```

**Warning signs:**
- `systemctl show trustedge.service -p TimeoutStopUSec` shows old value after deployment
- Service behavior doesn't match updated configuration
- systemd warnings about "Configuration file changed on disk"

### Pitfall 4: Docker Log Driver Configuration Not Applied to Running Containers

**What goes wrong:** Existing containers continue writing unlimited logs even after docker-compose.yml updated with log rotation.

**Why it happens:** Log driver configuration is set at container creation time. Updating docker-compose.yml doesn't affect running containers.

**How to avoid:**
- After updating docker-compose.yml with logging configuration, recreate containers
- Use `docker compose up -d` which recreates containers when configuration changes
- Verify log rotation is active: `docker inspect trustedge-backend-1 | grep -A5 LogConfig`

**Warning signs:**
- JSON log files in /var/lib/docker/containers growing without bound
- `docker inspect` shows "LogConfig": { "Type": "json-file", "Config": {} } (no max-size)

**Verification after deployment:**
```bash
# Check container log configuration
docker inspect trustedge-backend-1 --format='{{.HostConfig.LogConfig}}'
# Should show: {json-file map[max-file:3 max-size:10m]}

# Check actual log file sizes aren't growing past 10MB
ls -lh /var/lib/docker/containers/*/trustedge-backend-1*-json.log
```

### Pitfall 5: DigitalOcean Metrics Agent Installation Script Idempotency

**What goes wrong:** Re-running Ansible playbook tries to reinstall metrics agent, potentially failing or showing changes on every run.

**Why it happens:** The curl | bash script isn't inherently idempotent - it installs packages even if already present.

**How to avoid:**
- Check if do-agent service exists before running installation script
- Use `register` and `when` conditions in Ansible to skip installation if already present
- Script itself handles reinstallation gracefully, but Ansible should show "ok" not "changed"

**Example idempotent task:**
```yaml
- name: Check if DigitalOcean metrics agent is already installed
  systemd:
    name: do-agent
  register: do_agent_check
  failed_when: false
  changed_when: false

- name: Install DigitalOcean metrics agent
  shell: curl -sSL https://repos.insights.digitalocean.com/install.sh | sudo bash
  when: do_agent_check.status.ActiveState is not defined or do_agent_check.status.ActiveState != "active"
```

**Warning signs:**
- Ansible shows "changed" for metrics agent installation on every playbook run
- Installation task output shows "already installed" but Ansible marks as changed

## Code Examples

### Complete Nginx Configuration with Metrics Protection

```nginx
# Source: Adapted from existing trustedge.nginx.conf.j2
# References: https://docs.nginx.com/nginx/admin-guide/security-controls/controlling-access-proxied-http/

server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name {{ domain_name }};

    # SSL certificates — managed by Certbot
    ssl_certificate /etc/letsencrypt/live/{{ domain_name }}/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/{{ domain_name }}/privkey.pem;

    # ... existing SSL configuration ...

    # Metrics endpoint — restricted to localhost for Prometheus scraping
    # MUST come before /api/ location to avoid catch-all matching
    location = /metrics {
        proxy_pass http://127.0.0.1:3000/metrics;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Connection "";

        # Access control: allow only localhost
        allow 127.0.0.1;
        allow ::1;
        deny all;

        # Don't log high-frequency metrics scraping
        access_log off;
    }

    # Backend API — existing configuration unchanged
    location /api/ {
        proxy_pass http://127.0.0.1:3000;
        # ... existing proxy configuration ...
    }

    # Frontend — existing configuration unchanged
    location / {
        proxy_pass http://127.0.0.1:3001;
        # ... existing proxy configuration ...
    }
}
```

### Complete Docker Compose Production Template

```yaml
# Source: Existing docker-compose.production.yml.j2 with Phase 24 additions
# References:
# - https://oneuptime.com/blog/post/2026-01-16-docker-graceful-shutdown-signals/view
# - https://docs.docker.com/engine/logging/drivers/json-file/

services:
  backend:
    image: ghcr.io/trustedge-labs/trustedge-backend:latest
    ports:
      - "127.0.0.1:3000:3000"
    env_file:
      - .env
    environment:
      DATABASE_URL: ${DATABASE_URL}
    restart: unless-stopped

    # Graceful shutdown configuration (Phase 24: INF-03)
    stop_signal: SIGTERM
    stop_grace_period: 90s  # Matches SHUTDOWN_TIMEOUT from Phase 23

    # Resource limits
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 2G

    # Log rotation (Phase 24: INF-03)
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"

  frontend:
    image: ghcr.io/trustedge-labs/trustedge-frontend:latest
    ports:
      - "127.0.0.1:3001:3001"
    environment:
      BACKEND_URL: http://backend:3000
      NEXT_PUBLIC_BACKEND_URL: ${NEXT_PUBLIC_BACKEND_URL}
    restart: unless-stopped
    depends_on:
      - backend

    # Graceful shutdown (frontend typically shuts down quickly)
    stop_signal: SIGTERM
    stop_grace_period: 10s

    # Resource limits
    deploy:
      resources:
        limits:
          cpus: '1.0'
          memory: 1G

    # Log rotation
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

### Complete systemd Service with Extended Timeout

```ini
# Source: Existing trustedge.service.j2 with Phase 24 modification
# Reference: https://www.freedesktop.org/software/systemd/man/latest/systemd.service.html

[Unit]
Description=TrustEdge Audit Application
Requires=docker.service
After=docker.service network-online.target
Wants=network-online.target

[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory={{ app_directory }}
EnvironmentFile={{ app_directory }}/.env
ExecStart=/usr/bin/docker compose up -d
ExecStop=/usr/bin/docker compose down
ExecReload=/usr/bin/docker compose pull && /usr/bin/docker compose up -d
TimeoutStartSec=120
# Phase 24: Extended to accommodate Docker's 90s grace period + buffer
# Previously: TimeoutStopSec=30
TimeoutStopSec=95s
Restart=on-failure
RestartSec=15
User={{ deploy_user }}
Group={{ deploy_user }}

[Install]
WantedBy=multi-user.target
```

**Alternative: Drop-in override file approach**

```ini
# File: templates/trustedge.service.d/override.conf.j2
# Deployed to: /etc/systemd/system/trustedge.service.d/override.conf
[Service]
TimeoutStopSec=95s
```

This approach leaves the main service file unchanged and only overrides the timeout parameter.

### Production Environment Template with LOG_FORMAT

```jinja2
# Source: Existing env.production.j2 with Phase 24 additions
# TrustEdge Audit — Production Environment
# Generated by Ansible — do not edit manually

# ========================================
# CORE CONFIGURATION
# ========================================

# PostgreSQL connection string (REQUIRED)
DATABASE_URL={{ database_url }}

# HTTP server port (REQUIRED)
PORT=3000

# Logging level and filter (REQUIRED)
RUST_LOG=info,trustedge_audit=info

# ========================================
# OBSERVABILITY CONFIGURATION (Phase 24)
# ========================================

# Log output format (Phase 24: INF-05)
# json = structured JSON output (for production log aggregation)
# Anything else or unset = plain text (for development)
LOG_FORMAT=json

# Shutdown timeout (Phase 23, documented for Phase 24 coordination)
# How long to wait for in-flight scans to complete before forced shutdown
# MUST match Docker stop_grace_period in docker-compose.yml
# Default: 90s
SHUTDOWN_TIMEOUT=90

# ========================================
# APPLICATION SETTINGS
# ========================================

# Base URL for this instance (REQUIRED)
TRUSTEDGE_BASE_URL=https://{{ domain_name }}

# Frontend URL (REQUIRED)
FRONTEND_URL=https://{{ domain_name }}

# Maximum concurrent scans (REQUIRED)
MAX_CONCURRENT_SCANS=10

# ========================================
# SCANNER CONFIGURATION (OPTIONAL)
# ========================================

# Nuclei binary path (OPTIONAL)
NUCLEI_BINARY_PATH=/usr/local/bin/nuclei

# testssl.sh binary path (OPTIONAL)
TESTSSL_BINARY_PATH=/usr/local/bin/testssl.sh

# ========================================
# THIRD-PARTY SERVICES (OPTIONAL)
# ========================================

{% if resend_api_key %}
# Resend API key for email delivery
RESEND_API_KEY={{ resend_api_key }}
{% endif %}

{% if stripe_secret_key %}
# Stripe secret key for payments
STRIPE_SECRET_KEY={{ stripe_secret_key }}
{% endif %}

{% if stripe_webhook_secret %}
# Stripe webhook secret for webhook validation
STRIPE_WEBHOOK_SECRET={{ stripe_webhook_secret }}
{% endif %}

# ========================================
# DOCKER COMPOSE PRODUCTION VARIABLES
# ========================================

# Frontend public backend URL (used by browser)
NEXT_PUBLIC_BACKEND_URL=https://{{ domain_name }}
```

### Ansible Task File for DigitalOcean Metrics Agent

```yaml
# File: infrastructure/tasks/metrics-agent.yml
# Purpose: Install and verify DigitalOcean metrics agent
# References:
# - https://docs.digitalocean.com/products/monitoring/how-to/install-metrics-agent/
# - https://github.com/digitalocean/do-agent

---
- name: Check if DigitalOcean metrics agent is already installed
  systemd:
    name: do-agent
  register: do_agent_check
  failed_when: false
  changed_when: false

- name: Display metrics agent installation status
  debug:
    msg: "DigitalOcean metrics agent is {{ 'already installed' if do_agent_check.status.ActiveState == 'active' else 'not installed' }}"

- name: Download and review installation script
  get_url:
    url: https://repos.insights.digitalocean.com/install.sh
    dest: /tmp/do-agent-install.sh
    mode: '0755'
  when: do_agent_check.status.ActiveState is not defined or do_agent_check.status.ActiveState != "active"

- name: Install DigitalOcean metrics agent
  shell: bash /tmp/do-agent-install.sh
  args:
    creates: /etc/systemd/system/do-agent.service
  when: do_agent_check.status.ActiveState is not defined or do_agent_check.status.ActiveState != "active"

- name: Clean up installation script
  file:
    path: /tmp/do-agent-install.sh
    state: absent

- name: Ensure metrics agent is running and enabled
  systemd:
    name: do-agent
    state: started
    enabled: yes

- name: Wait for metrics agent to start reporting
  pause:
    seconds: 30
    prompt: "Waiting for metrics agent to initialize and begin reporting data to DigitalOcean"

- name: Verify metrics agent status
  command: systemctl status do-agent
  register: agent_status
  changed_when: false

- name: Display metrics agent status
  debug:
    var: agent_status.stdout_lines

- name: Verify metrics agent process is running
  shell: ps aux | grep -v grep | grep do-agent
  register: agent_process
  changed_when: false

- name: Display agent process info
  debug:
    msg: "Metrics agent process: {{ agent_process.stdout }}"

- name: Remind about DigitalOcean control panel verification
  debug:
    msg: |
      DigitalOcean metrics agent installed successfully.
      Metrics should appear in the Graphs tab of the Droplet control panel within a few minutes.
      Log in to DigitalOcean and navigate to: Droplets > {{ inventory_hostname }} > Graphs
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual curl install.sh on server | Ansible-automated metrics agent deployment | 2022+ | Idempotent, version-controlled, repeatable deployments |
| Single 10-second timeout for all operations | Separate, coordinated timeouts per layer | Docker Compose v2 (2020+) | Applications can specify drain requirements without infrastructure fighting back |
| Text logs manually grepped | JSON logs with structured fields | Modern observability (2020+) | Log aggregation systems can parse, filter, and alert on structured data |
| Manual service file edits with sed/awk | systemd drop-in override.conf files | systemd 220+ (2015+) | Safer overrides, no package update conflicts |
| Docker Compose v1 (docker-compose) | Docker Compose v2 (docker compose plugin) | 2021-2023 transition | Better integration with Docker CLI, native Go implementation, active development |

**Deprecated/outdated:**
- Docker Compose v1 commands (docker-compose) - use docker compose (v2, plugin-based)
- Manual /etc/logrotate.d for Docker logs - use json-file driver rotation (built-in, atomic)
- Editing /etc/systemd/system/*.service directly - use drop-in files (safer)
- Running curl | bash without reviewing - download first, inspect, then execute (security best practice)

## Open Questions

1. **DigitalOcean Metrics Agent Configuration**
   - What we know: Agent auto-configures using droplet metadata, no config file needed
   - What's unclear: Whether we can customize collection intervals or add custom tags
   - Recommendation: Use defaults for Phase 24. If custom tags needed later, research /etc/do-agent/do-agent.yaml configuration

2. **Nginx /metrics Location Block Ordering**
   - What we know: Nginx evaluates location blocks in specific order (exact match, then prefix match)
   - What's unclear: Whether `location = /metrics` (exact) is needed or `location /metrics` (prefix) suffices
   - Recommendation: Use `location = /metrics` for exact matching to prevent /metrics/anything accidentally matching

3. **CI/CD Pipeline Integration**
   - What we know: Current pipeline uses SSH action to run `docker compose pull && up -d`
   - What's unclear: Whether pipeline should also trigger Ansible playbook run after code deployment
   - Recommendation: Keep current approach for Phase 24 (SSH action pulls and restarts containers). Future phase could add Ansible integration for config changes, but code deployments shouldn't require full playbook run.

4. **systemd Drop-in vs Direct Edit**
   - What we know: Drop-ins are safer but add directory/file overhead
   - What's unclear: For a single timeout change, is direct edit acceptable?
   - Recommendation: Use direct edit in trustedge.service.j2 for simplicity. Drop-ins are valuable for multiple overrides or when you can't modify the main file.

## Sources

### Primary (HIGH confidence)

- [DigitalOcean Metrics Agent Installation](https://docs.digitalocean.com/products/monitoring/how-to/install-metrics-agent/) - Official installation methods
- [DigitalOcean Monitoring Metrics](https://docs.digitalocean.com/products/monitoring/concepts/metrics/) - What metrics are collected
- [do-agent GitHub Repository](https://github.com/digitalocean/do-agent) - Agent source code and technical details
- [Docker Graceful Shutdown and Signal Handling (2026)](https://oneuptime.com/blog/post/2026-01-16-docker-graceful-shutdown-signals/view) - STOPSIGNAL and stop_grace_period
- [JSON File Logging Driver | Docker Docs](https://docs.docker.com/engine/logging/drivers/json-file/) - Log rotation configuration
- [systemd.service - freedesktop.org](https://www.freedesktop.org/software/systemd/man/latest/systemd.service.html) - TimeoutStopSec and service configuration
- [Nginx Access Control | NGINX Documentation](https://docs.nginx.com/nginx/admin-guide/security-controls/controlling-access-proxied-http/) - allow/deny directives
- [systemd Drop-in Files | Baeldung on Linux](https://www.baeldung.com/linux/systemd-modify-config) - Override.conf pattern

### Secondary (MEDIUM confidence)

- [Docker Log Rotation Configuration | SigNoz](https://signoz.io/blog/docker-log-rotation/) - Best practices verified against Docker docs
- [Using systemd Drop-in Units | Flatcar Container Linux](https://www.flatcar.org/docs/latest/setup/systemd/drop-in-units/) - Drop-in directory structure
- [How to Modify systemd Service Configuration](https://sleeplessbeastie.eu/2020/02/24/how-to-modify-systemd-service-configuration/) - Ansible deployment examples
- [Log Formatting in Production: Best Practices | Better Stack](https://betterstack.com/community/guides/logging/log-formatting/) - JSON logging standards
- [Ansible Template Module Documentation](https://docs.ansible.com/projects/ansible/latest/collections/ansible/builtin/template_module.html) - Jinja2 templating

### Tertiary (LOW confidence)

- Various community forum posts about Docker Compose graceful shutdown - used for pattern verification only
- GitHub issues discussing systemd timeout behavior - corroborated with official docs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Official documentation for all tools (DigitalOcean agent, Nginx, Docker Compose, systemd)
- Architecture: HIGH - Patterns verified against official docs and existing project structure
- Pitfalls: HIGH - Based on documented failure modes and official troubleshooting guides

**Research date:** 2026-02-16
**Valid until:** 90 days (2026-05-17) - Infrastructure tooling changes slowly, but monitor Docker Compose and systemd updates

**Phase dependencies:**
- Phase 23 must be complete (SHUTDOWN_TIMEOUT env var support, graceful shutdown implementation)
- Phases 19-22 should be complete (structured logging, metrics endpoint, health checks) to verify integration

**No new code changes required:** This is a pure infrastructure phase. All changes are configuration, templates, and Ansible tasks.
