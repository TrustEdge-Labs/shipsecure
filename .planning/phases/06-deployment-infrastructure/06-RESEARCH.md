# Phase 06: Deployment Infrastructure - Research

**Researched:** 2026-02-06
**Domain:** Infrastructure provisioning and production deployment
**Confidence:** HIGH

## Summary

Phase 06 provisions production infrastructure on DigitalOcean using Ansible for automated, idempotent server configuration. The architecture uses a single droplet running Docker Compose services (backend, frontend) with host-based Nginx reverse proxy for SSL termination and routing. PostgreSQL runs as a DigitalOcean Managed Database (external to the droplet). Certbot handles automatic SSL certificate provisioning and renewal. UFW firewall restricts access to SSH (non-standard port), HTTP, and HTTPS only.

**Key architectural decisions:** Host-based Nginx (not containerized) enables simpler Certbot SSL integration. Docker Compose manages application services with production overrides. Manual SSH + git pull deployment workflow (CI/CD deferred). Systemd service ensures Docker Compose auto-starts on boot.

**Primary recommendation:** Use `digitalocean.cloud` Ansible collection (official, actively maintained) instead of deprecated `community.digitalocean`. Configure Nginx with localhost proxy_pass to Docker services. Ensure CA certificate handling for DigitalOcean Managed PostgreSQL SSL connections. Test idempotence by running playbook twice.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Provisioning approach:**
- Ansible playbook for all server configuration (idempotent, re-runnable)
- Playbook lives in this repo under `deploy/` or `infrastructure/` directory
- Playbook handles droplet creation via DigitalOcean API / Ansible module
- Droplet size: s-4vcpu-8gb ($48/mo) — 4 vCPUs, 8GB RAM

**Service architecture:**
- Docker Compose for running all application services (using docker-compose.prod.yml from Phase 05)
- PostgreSQL on DigitalOcean Managed Database (not containerized in production)
- Base docker-compose.yml keeps Postgres for local dev; production override excludes it and uses DATABASE_URL for managed DB
- Deploy workflow: SSH in, git pull, docker compose build & up (manual, simple)

**Domain & SSL:**
- User has a domain ready — playbook takes domain as a variable
- DNS managed at registrar — user updates A records manually
- SSL via Certbot with Nginx plugin (automatic cert placement and renewal)
- Nginx runs on the host (not containerized) — simpler certbot integration, directly manages ports 80/443
- Docker Compose services expose on localhost ports, Nginx reverse proxies to them

**SSH & access:**
- SSH key-only authentication, password auth disabled
- Non-standard SSH port (configurable, e.g., 2222) to reduce noise
- Root login disabled
- Playbook creates a non-root 'deploy' user with sudo — app runs under this user

**Firewall:**
- UFW configured allowing only SSH (non-standard port), 80, and 443

**Logging:**
- Docker's default logging driver + journald
- No extra log tooling — viewable with `docker logs` and `journalctl`

**Monitoring:**
- Skipped for this phase — handle manually until there's traffic

### Claude's Discretion

- Exact Ansible role/task structure and directory layout
- Nginx configuration details (worker processes, buffer sizes, etc.)
- Certbot renewal cron/timer setup specifics
- Systemd service unit details for Docker Compose
- DigitalOcean region selection
- Exact non-standard SSH port number

### Deferred Ideas (OUT OF SCOPE)

- CI/CD pipeline (GitHub Actions) — future improvement after manual deploy workflow is validated
- Uptime monitoring (UptimeRobot, Betterstack, or custom) — add after there's real traffic
- Log aggregation / centralized logging — not needed at MVP scale

</user_constraints>

## Standard Stack

### Core Infrastructure Tools

| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| **Ansible** | 2.14+ core | Server provisioning & configuration automation | Industry standard for declarative infrastructure, idempotent by design |
| **digitalocean.cloud** | Latest | Official DigitalOcean Ansible collection | Replaces deprecated community.digitalocean, uses pydo SDK |
| **Docker Compose** | v2 | Multi-container orchestration | Already in use from Phase 05, lightweight alternative to K8s |
| **Nginx** | Latest stable | Reverse proxy & SSL termination | De facto standard for production reverse proxy |
| **Certbot** | Latest snap | Let's Encrypt SSL automation | Official ACME client, automatic renewal built-in |
| **UFW** | Default Ubuntu | Firewall management | Ubuntu's simplified iptables frontend |
| **systemd** | Default Ubuntu | Service management | Standard init system for Ubuntu |

### Supporting Components

| Component | Purpose | When to Use |
|-----------|---------|-------------|
| **Ansible Vault** | Encrypt API tokens, secrets | Always for production credentials |
| **DigitalOcean Managed PostgreSQL** | Production database | Already decided, provides automatic backups |
| **pydo** | Python DigitalOcean SDK | Required dependency for digitalocean.cloud collection |

### Installation

**Ansible collection:**
```bash
ansible-galaxy collection install digitalocean.cloud
```

**Python dependencies:**
```bash
pip3 install --user azure-core==1.26.1 boto3==1.28.53 pydo==0.1.7
```

**Certbot (on droplet via snap):**
```bash
sudo snap install --classic certbot
sudo ln -s /snap/bin/certbot /usr/local/bin/certbot
```

## Architecture Patterns

### Recommended Directory Structure

```
infrastructure/           # or deploy/ — your discretion
├── ansible.cfg          # Ansible configuration
├── inventory/
│   ├── production       # Production hosts inventory
│   └── group_vars/
│       └── all.yml      # Global variables (domain, region, etc.)
├── playbooks/
│   ├── provision.yml    # Main provisioning playbook
│   └── deploy.yml       # Application deployment playbook (optional)
├── roles/
│   ├── droplet/         # Droplet creation
│   │   └── tasks/
│   │       └── main.yml
│   ├── security/        # SSH, UFW, fail2ban (optional)
│   │   └── tasks/
│   │       └── main.yml
│   ├── docker/          # Docker & Docker Compose installation
│   │   └── tasks/
│   │       └── main.yml
│   ├── nginx/           # Nginx installation & config
│   │   ├── tasks/
│   │   │   └── main.yml
│   │   └── templates/
│   │       └── default.conf.j2
│   ├── certbot/         # SSL certificate provisioning
│   │   └── tasks/
│   │       └── main.yml
│   └── app/             # Application setup (git clone, env vars)
│       └── tasks/
│           └── main.yml
└── files/
    └── systemd/
        └── trustedge.service.j2
```

**Alternative:** Flat playbook structure without roles (acceptable for MVP simplicity)

### Pattern 1: DigitalOcean Droplet Creation (Idempotent)

**What:** Create droplet only if it doesn't exist, using unique name as idempotence key

**When to use:** Always for production infrastructure

**Example:**
```yaml
# Source: https://docs.digitalocean.com/reference/ansible/reference/
- name: Create DigitalOcean droplet
  digitalocean.cloud.droplet:
    state: present
    name: trustedge-prod
    size: s-4vcpu-8gb
    image: ubuntu-24-04-x64
    region: nyc3
    ssh_keys: "{{ ssh_key_ids }}"
    unique_name: true  # Ensures idempotence
    wait: true
    wait_timeout: 500
    oauth_token: "{{ do_api_token }}"
  register: droplet_result
```

**Key insight:** Setting `unique_name: true` makes droplet creation idempotent. Re-running won't create duplicates.

### Pattern 2: Nginx Reverse Proxy to Docker Localhost Ports

**What:** Host-based Nginx proxies HTTPS requests to Docker services on localhost ports

**When to use:** When SSL termination happens outside containers (Certbot integration)

**Example:**
```nginx
# Source: Nginx best practices for Docker localhost proxy
upstream backend {
    server 127.0.0.1:3000;
    keepalive 32;
}

upstream frontend {
    server 127.0.0.1:3001;
    keepalive 32;
}

server {
    listen 80;
    server_name {{ domain_name }};
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name {{ domain_name }};

    ssl_certificate /etc/letsencrypt/live/{{ domain_name }}/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/{{ domain_name }}/privkey.pem;

    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    location /api/ {
        proxy_pass http://backend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Connection "";
    }

    location / {
        proxy_pass http://frontend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Connection "";
    }
}
```

**Why localhost (127.0.0.1) not container names:** Nginx runs on host, containers expose ports on host via `ports:` mapping added to docker-compose.prod.yml (currently empty in Phase 05, needs update)

### Pattern 3: Docker Compose Production Override with Managed Database

**What:** Override base config to use external managed database instead of containerized Postgres

**When to use:** Production deployments with external database services

**Example update to docker-compose.prod.yml:**
```yaml
services:
  db:
    # Completely disable local Postgres in production
    deploy:
      replicas: 0

  backend:
    ports:
      - "127.0.0.1:3000:3000"  # Expose on localhost only for Nginx
    environment:
      DATABASE_URL: ${DATABASE_URL}  # DigitalOcean Managed DB connection string
      # ... other env vars

  frontend:
    ports:
      - "127.0.0.1:3001:3001"  # Expose on localhost only for Nginx
    # ... rest of config
```

**Production .env file:**
```bash
# DigitalOcean Managed PostgreSQL connection string
# Includes SSL mode requirement
DATABASE_URL=postgres://doadmin:password@db-host.db.ondigitalocean.com:25060/trustedge_prod?sslmode=require

# CA certificate path (downloaded from DigitalOcean console)
DATABASE_CA_CERT=/opt/trustedge/ca-certificate.crt
```

### Pattern 4: Certbot Nginx Plugin Auto-Configuration

**What:** Certbot automatically modifies Nginx config and sets up renewal

**When to use:** Always for Let's Encrypt SSL with Nginx

**Example:**
```yaml
# Source: https://certbot.eff.org/instructions?ws=nginx&os=snap
- name: Obtain SSL certificate with Nginx plugin
  command: >
    certbot --nginx
    --non-interactive
    --agree-tos
    --email {{ admin_email }}
    --domain {{ domain_name }}
    --redirect
  args:
    creates: /etc/letsencrypt/live/{{ domain_name }}/fullchain.pem
```

**Auto-renewal:** Snap installation includes systemd timer, test with:
```bash
sudo certbot renew --dry-run
```

### Pattern 5: Systemd Service for Docker Compose

**What:** Systemd unit manages Docker Compose lifecycle (start on boot, restart on failure)

**When to use:** Production deployments requiring auto-start and supervision

**Example:**
```ini
# /etc/systemd/system/trustedge.service
[Unit]
Description=TrustEdge Audit Application
Requires=docker.service
After=docker.service network-online.target
Wants=network-online.target

[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory=/opt/trustedge
ExecStart=/usr/bin/docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d
ExecStop=/usr/bin/docker compose -f docker-compose.yml -f docker-compose.prod.yml down
Restart=on-failure
User=deploy
Group=deploy

[Install]
WantedBy=multi-user.target
```

**Enable and start:**
```bash
sudo systemctl daemon-reload
sudo systemctl enable trustedge.service
sudo systemctl start trustedge.service
```

**Important:** Do NOT use `restart: always` or `restart: unless-stopped` in docker-compose.prod.yml when using systemd service. Use systemd's `Restart=on-failure` instead to avoid conflicts.

### Pattern 6: UFW Firewall Configuration for Non-Standard SSH

**What:** Allow only SSH (custom port), HTTP, HTTPS; deny all else

**When to use:** Always for production server hardening

**Example:**
```yaml
# Source: https://www.digitalocean.com/community/tutorials/how-to-set-up-a-firewall-with-ufw-on-ubuntu
- name: Configure UFW firewall rules
  community.general.ufw:
    rule: "{{ item.rule }}"
    port: "{{ item.port }}"
    proto: "{{ item.proto | default('tcp') }}"
  loop:
    - { rule: 'allow', port: '{{ ssh_port }}', proto: 'tcp' }
    - { rule: 'allow', port: '80', proto: 'tcp' }
    - { rule: 'allow', port: '443', proto: 'tcp' }

- name: Set UFW default policies
  community.general.ufw:
    direction: "{{ item.direction }}"
    policy: "{{ item.policy }}"
  loop:
    - { direction: 'incoming', policy: 'deny' }
    - { direction: 'outgoing', policy: 'allow' }

- name: Enable UFW
  community.general.ufw:
    state: enabled
```

**Critical safety:** Verify SSH port is allowed BEFORE enabling UFW, or you'll be locked out. Use `ufw show added` to confirm.

### Pattern 7: SSH Hardening (Key-Only, Non-Root)

**What:** Disable password auth, disable root login, create deploy user

**When to use:** Always for production security

**Example:**
```yaml
- name: Create deploy user
  user:
    name: deploy
    groups: sudo,docker
    append: yes
    create_home: yes
    shell: /bin/bash

- name: Add SSH authorized key for deploy user
  authorized_key:
    user: deploy
    key: "{{ deploy_user_ssh_key }}"
    state: present

- name: Harden SSH configuration
  lineinfile:
    path: /etc/ssh/sshd_config
    regexp: "{{ item.regexp }}"
    line: "{{ item.line }}"
    validate: 'sshd -t -f %s'
  loop:
    - { regexp: '^#?Port ', line: 'Port {{ ssh_port }}' }
    - { regexp: '^#?PermitRootLogin ', line: 'PermitRootLogin no' }
    - { regexp: '^#?PasswordAuthentication ', line: 'PasswordAuthentication no' }
    - { regexp: '^#?PubkeyAuthentication ', line: 'PubkeyAuthentication yes' }
  notify: Restart SSH
```

**Test connection on new port BEFORE closing session:**
```bash
ssh -p {{ ssh_port }} deploy@<droplet-ip>
```

### Pattern 8: Ansible Vault for Secrets

**What:** Encrypt sensitive variables (API tokens, passwords) in version control

**When to use:** Always for production credentials

**Example:**
```bash
# Encrypt DigitalOcean API token
ansible-vault create infrastructure/inventory/group_vars/vault.yml
```

**vault.yml contents:**
```yaml
vault_do_api_token: "dop_v1_abc123..."
vault_db_password: "strong-random-password"
vault_admin_email: "admin@example.com"
```

**Reference in playbook:**
```yaml
---
# group_vars/all.yml
do_api_token: "{{ vault_do_api_token }}"
db_password: "{{ vault_db_password }}"
admin_email: "{{ vault_admin_email }}"
```

**Run playbook with vault:**
```bash
ansible-playbook -i inventory/production playbooks/provision.yml --ask-vault-pass
```

### Anti-Patterns to Avoid

- **Running Nginx in Docker with host-based Certbot:** Creates complexity with volume mounts and cert renewal. Use host Nginx OR containerize everything (Certbot included).
- **Using community.digitalocean collection:** Deprecated and removed from Ansible 13. Use `digitalocean.cloud` instead.
- **Mixing Docker restart policies with systemd:** Don't use `restart: always` in compose file when systemd manages the service. Pick one supervisor.
- **Hardcoding secrets in playbooks:** Always use Ansible Vault for production credentials.
- **Enabling UFW before verifying SSH rule:** Will lock you out. Always verify rules with `ufw show added` first.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| SSL certificate renewal | Custom cron scripts | Certbot with snap (includes systemd timer) | Handles edge cases, automatic Nginx reload, rate limit handling |
| Firewall management | Raw iptables rules | UFW | State management, idempotent rules, simpler syntax |
| Secret management in Ansible | Base64 encoding, .gitignore | Ansible Vault | AES256 encryption, integrated workflow, team collaboration |
| Process supervision | Custom systemd units from scratch | Docker Compose restart policies OR systemd generator | Battle-tested, handles dependency ordering, resource limits |
| Droplet provisioning | DigitalOcean API calls via curl/Python | digitalocean.cloud Ansible collection | Idempotence, state management, error handling |

**Key insight:** Infrastructure automation has mature tooling. Use it rather than scripting primitives.

## Common Pitfalls

### Pitfall 1: Lockout from SSH Configuration Changes

**What goes wrong:** Changing SSH port or disabling password auth without testing locks you out of the server.

**Why it happens:** SSH config validation happens locally, but connectivity issues only surface when reconnecting.

**How to avoid:**
1. Open a SECOND SSH session before making changes (keep first session open)
2. Test new SSH port connection in second session
3. If successful, then restart SSH service in first session
4. Keep first session open for 5 minutes as safety net

**Warning signs:** Unable to connect on new port immediately after sshd restart.

**Ansible safety:**
```yaml
- name: Validate SSH config before restarting
  command: sshd -t
  changed_when: false

- name: Test new SSH port (pause for manual verification)
  pause:
    prompt: "SSH config changed. Test connection on port {{ ssh_port }} in another terminal, then press ENTER"
```

### Pitfall 2: Docker Compose Port Conflicts with Nginx

**What goes wrong:** Nginx can't proxy to Docker services because ports aren't exposed on localhost.

**Why it happens:** Phase 05's docker-compose.prod.yml removes ALL port mappings (`ports: []`), including localhost bindings.

**How to avoid:** Update docker-compose.prod.yml to expose services on localhost only:
```yaml
backend:
  ports:
    - "127.0.0.1:3000:3000"  # NOT "3000:3000" (avoids external exposure)
frontend:
  ports:
    - "127.0.0.1:3001:3001"
```

**Warning signs:** Nginx returns 502 Bad Gateway. Check with `netstat -tlnp | grep 3000` — if no output, ports aren't exposed.

### Pitfall 3: DigitalOcean Managed Database SSL Certificate Verification

**What goes wrong:** Database connections fail with "self-signed certificate in chain" error.

**Why it happens:** DigitalOcean Managed Databases require SSL, but connection string alone doesn't provide CA certificate path.

**How to avoid:**
1. Download CA certificate from DigitalOcean console
2. Place in `/opt/trustedge/ca-certificate.crt` via Ansible
3. Configure connection string with `sslmode=require` AND set `PGSSLROOTCERT` environment variable

**Rust (sqlx) configuration:**
```rust
// In production DATABASE_URL, include sslmode parameter
// postgres://user:pass@host:25060/db?sslmode=require

// Set environment variable for CA cert path
std::env::set_var("PGSSLROOTCERT", "/opt/trustedge/ca-certificate.crt");
```

**Alternative:** Use `sslmode=require` without certificate verification (less secure but simpler for MVP).

**Warning signs:** Connection errors mentioning SSL, TLS, or certificate validation.

### Pitfall 4: Certbot Rate Limits

**What goes wrong:** Let's Encrypt temporarily blocks certificate issuance after too many failed attempts.

**Why it happens:** Main limit is 5 failed validations per account, per hostname, per hour.

**How to avoid:**
1. Use `--dry-run` flag to test Certbot configuration BEFORE real attempts
2. Verify DNS A record points to droplet IP before running Certbot
3. Ensure port 80 is open and Nginx is running (Certbot uses HTTP-01 challenge)

**Ansible idempotence check:**
```yaml
- name: Check if certificate already exists
  stat:
    path: /etc/letsencrypt/live/{{ domain_name }}/fullchain.pem
  register: cert_exists

- name: Obtain SSL certificate
  command: certbot --nginx ...
  when: not cert_exists.stat.exists
```

**Warning signs:** "too many failed authorizations recently" error message.

### Pitfall 5: Ansible Idempotence False Positives

**What goes wrong:** Playbook reports "changed" on every run even when nothing changed.

**Why it happens:** shell/command tasks always report changed status unless explicitly told otherwise.

**How to avoid:**
```yaml
# Bad: Always reports changed
- name: Check Docker version
  command: docker --version

# Good: Uses changed_when
- name: Check Docker version
  command: docker --version
  register: docker_version
  changed_when: false

# Better: Use stat for file checks
- name: Check if Nuclei is installed
  stat:
    path: /usr/local/bin/nuclei
  register: nuclei_binary
```

**Verification:** Run playbook twice. Second run should show 0 changed tasks.

**Warning signs:** Playbook always shows changed tasks even when re-run immediately.

### Pitfall 6: Nuclei Binary PATH Resolution in Systemd Service

**What goes wrong:** Rust backend can't find Nuclei binary when run via systemd, but works when run manually.

**Why it happens:** Systemd services have minimal PATH (`/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin`), doesn't include user bin dirs.

**How to avoid:**
1. Install Nuclei to `/usr/local/bin/nuclei` (in system PATH)
2. OR set `NUCLEI_BINARY_PATH=/opt/nuclei/bin/nuclei` in .env
3. OR add `Environment="PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/opt/nuclei/bin"` to systemd unit

**Ansible task:**
```yaml
- name: Install Nuclei binary
  get_url:
    url: https://github.com/projectdiscovery/nuclei/releases/download/v3.1.0/nuclei_3.1.0_linux_amd64.zip
    dest: /tmp/nuclei.zip

- name: Extract and install Nuclei
  unarchive:
    src: /tmp/nuclei.zip
    dest: /usr/local/bin/
    remote_src: yes
    mode: '0755'
```

**Warning signs:** Scans work when testing manually (`sudo -u deploy /opt/trustedge/backend`) but fail when service starts.

### Pitfall 7: Docker Compose Restart Policy Conflicts

**What goes wrong:** Containers don't restart properly, or restart in unexpected ways.

**Why it happens:** Combining Docker's `restart: always` with systemd's `Restart=on-failure` creates conflicting supervision.

**How to avoid:** Choose ONE supervisor:
- **Option A (Recommended):** Use systemd service with `Restart=on-failure`, remove `restart:` from docker-compose.prod.yml
- **Option B:** Use `restart: unless-stopped` in compose file, don't create systemd service (less robust)

**Warning signs:** Containers restart when they shouldn't, or don't restart when expected.

## Code Examples

### Complete Ansible Playbook (Minimal)

```yaml
---
# playbooks/provision.yml
- name: Provision TrustEdge production infrastructure
  hosts: localhost
  gather_facts: false
  vars_files:
    - ../inventory/group_vars/vault.yml

  tasks:
    - name: Create DigitalOcean droplet
      digitalocean.cloud.droplet:
        state: present
        name: trustedge-prod
        size: s-4vcpu-8gb
        image: ubuntu-24-04-x64
        region: nyc3
        ssh_keys: "{{ ssh_key_ids }}"
        unique_name: true
        wait: true
        oauth_token: "{{ do_api_token }}"
      register: droplet

    - name: Add droplet to inventory
      add_host:
        name: "{{ droplet.data.droplet.networks.v4[0].ip_address }}"
        groups: production
        ansible_user: root

- name: Configure production server
  hosts: production
  become: yes
  vars_files:
    - ../inventory/group_vars/vault.yml

  tasks:
    # Security hardening
    - import_tasks: tasks/security.yml

    # Install Docker
    - import_tasks: tasks/docker.yml

    # Install Nginx
    - import_tasks: tasks/nginx.yml

    # Install Certbot & obtain SSL
    - import_tasks: tasks/certbot.yml

    # Deploy application
    - import_tasks: tasks/app.yml
```

### Docker Installation Tasks

```yaml
---
# tasks/docker.yml
- name: Install Docker prerequisites
  apt:
    name:
      - apt-transport-https
      - ca-certificates
      - curl
      - gnupg
      - lsb-release
    state: present
    update_cache: yes

- name: Add Docker GPG key
  apt_key:
    url: https://download.docker.com/linux/ubuntu/gpg
    state: present

- name: Add Docker repository
  apt_repository:
    repo: "deb [arch=amd64] https://download.docker.com/linux/ubuntu {{ ansible_distribution_release }} stable"
    state: present

- name: Install Docker
  apt:
    name:
      - docker-ce
      - docker-ce-cli
      - containerd.io
      - docker-compose-plugin
    state: present
    update_cache: yes

- name: Add deploy user to docker group
  user:
    name: deploy
    groups: docker
    append: yes

- name: Enable Docker service
  systemd:
    name: docker
    enabled: yes
    state: started
```

### Application Deployment Tasks

```yaml
---
# tasks/app.yml
- name: Create application directory
  file:
    path: /opt/trustedge
    state: directory
    owner: deploy
    group: deploy
    mode: '0755'

- name: Clone application repository
  git:
    repo: "{{ app_git_repo }}"
    dest: /opt/trustedge
    version: main
    force: yes
  become_user: deploy

- name: Create production .env file
  template:
    src: templates/env.production.j2
    dest: /opt/trustedge/.env
    owner: deploy
    group: deploy
    mode: '0600'

- name: Download DigitalOcean database CA certificate
  get_url:
    url: "{{ database_ca_cert_url }}"
    dest: /opt/trustedge/ca-certificate.crt
    owner: deploy
    group: deploy
    mode: '0644'

- name: Install Nuclei binary
  get_url:
    url: "https://github.com/projectdiscovery/nuclei/releases/download/v3.1.0/nuclei_3.1.0_linux_amd64.tar.gz"
    dest: /tmp/nuclei.tar.gz

- name: Extract Nuclei
  unarchive:
    src: /tmp/nuclei.tar.gz
    dest: /usr/local/bin/
    remote_src: yes
    creates: /usr/local/bin/nuclei

- name: Set Nuclei executable permissions
  file:
    path: /usr/local/bin/nuclei
    mode: '0755'

- name: Create systemd service
  template:
    src: templates/trustedge.service.j2
    dest: /etc/systemd/system/trustedge.service
    mode: '0644'
  notify:
    - Reload systemd
    - Restart trustedge

- name: Enable and start TrustEdge service
  systemd:
    name: trustedge
    enabled: yes
    state: started
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| community.digitalocean collection | digitalocean.cloud collection | Nov 2025 (archived) | Must migrate to official collection |
| Certbot manual renewal cron | Certbot snap with systemd timer | 2020+ | Automatic renewal out of box |
| Docker Compose v1 (docker-compose) | Docker Compose v2 (docker compose plugin) | 2022 | Plugin architecture, better integration |
| Manual iptables rules | UFW declarative rules | Ubuntu 8.04+ | Simpler, idempotent, less error-prone |
| Custom systemd service files | Docker Compose + systemd generator | 2023+ | Less boilerplate (but user decides) |

**Deprecated/outdated:**
- **community.digitalocean:** Archived Nov 2025, removed from Ansible 13. Use `digitalocean.cloud` instead.
- **docker-compose (Python binary):** Deprecated in favor of `docker compose` plugin (v2).
- **Certbot via apt:** Still works but snap is now recommended for latest version and auto-updates.

## Open Questions

1. **Exact s-4vcpu-8gb pricing for 2026**
   - What we know: General Purpose droplets with 8GB RAM start around $63/mo. Context says $48/mo but this may be outdated.
   - What's unclear: Current exact pricing for this specific tier.
   - Recommendation: Verify at https://www.digitalocean.com/pricing/droplets before provisioning. Pricing may vary by region.

2. **DigitalOcean Managed Database connection pooling**
   - What we know: Managed PostgreSQL provides connection pooling mode, requires SSL connections.
   - What's unclear: Whether default connection pool settings are sufficient for Axum backend with sqlx.
   - Recommendation: Start with default pool settings, monitor connection metrics after deploy.

3. **Nginx worker_processes and connection limits**
   - What we know: Default Nginx config works for most cases.
   - What's unclear: Optimal tuning for 4 vCPU droplet with expected MVP traffic.
   - Recommendation: Start with `worker_processes auto;` (default), tune later if needed.

4. **Nuclei templates directory in production**
   - What we know: Backend has `/app/templates` in Docker image, env var `TRUSTEDGE_TEMPLATES_DIR` allows override.
   - What's unclear: Whether to volume-mount custom templates or rely on baked-in templates.
   - Recommendation: Use baked-in templates from Docker image for MVP (simpler). Add volume mount later if custom templates needed.

5. **Database migration strategy on first deploy**
   - What we know: Migrations exist in migrations/ directory, sqlx handles migration running.
   - What's unclear: Whether to run migrations via separate task or let backend auto-migrate on startup.
   - Recommendation: Document in PLAN — likely manual `sqlx migrate run` in Ansible task before starting service for first deploy.

## Sources

### Primary (HIGH confidence)

- [DigitalOcean Ansible Collection Reference](https://docs.digitalocean.com/reference/ansible/reference/) - Official digitalocean.cloud collection docs
- [community.digitalocean GitHub](https://github.com/ansible-collections/community.digitalocean) - Deprecation notice and migration guidance
- [Certbot Official Instructions (Nginx/Snap)](https://certbot.eff.org/instructions?ws=nginx&os=snap) - Official installation and renewal setup
- [Docker Compose Documentation](https://docs.docker.com/compose/) - Official docs for v2 syntax and best practices
- [Ansible Roles Documentation](https://docs.ansible.com/projects/ansible/latest/playbook_guide/playbooks_reuse_roles.html) - Official role structure
- [ProjectDiscovery Nuclei Installation](https://docs.projectdiscovery.io/opensource/nuclei/install) - Official installation methods

### Secondary (MEDIUM confidence)

- [DigitalOcean: How to Set Up a Firewall with UFW](https://www.digitalocean.com/community/tutorials/how-to-set-up-a-firewall-with-ufw-on-ubuntu) - Verified UFW patterns
- [DigitalOcean: How to Use Ansible Vault](https://www.digitalocean.com/community/tutorials/how-to-use-vault-to-protect-sensitive-ansible-data) - Secrets management
- [DigitalOcean: How to Disable Root Login](https://www.digitalocean.com/community/tutorials/how-to-disable-root-login-on-ubuntu-20-04) - SSH hardening
- [Docker Docs: Start containers automatically](https://docs.docker.com/engine/containers/start-containers-automatically/) - Restart policies vs systemd
- [Spacelift: 50+ Ansible Best Practices](https://spacelift.io/blog/ansible-best-practices) - Community best practices compilation
- [Reintech: Automating SSL Renewal](https://reintech.io/blog/automating-ssl-renewal-nginx-certbot) - Certbot renewal patterns
- [BootVar: Docker Compose as systemd Service](https://bootvar.com/systemd-service-for-docker-compose/) - Systemd integration patterns

### Tertiary (LOW confidence)

- Web search results for 2026 pricing and recent updates (needs verification with official sources)
- Community discussions on DigitalOcean forums (anecdotal, verify before use)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Official sources verified, active collections identified
- Architecture: HIGH - Official docs for Nginx, Docker Compose, systemd patterns verified
- Pitfalls: MEDIUM-HIGH - Mix of official warnings and community experience, common issues documented

**Research date:** 2026-02-06
**Valid until:** 2026-04-06 (60 days - infrastructure tooling is relatively stable, but check for Ansible/Docker updates)

**Key verifications performed:**
- ✅ Confirmed community.digitalocean deprecation and digitalocean.cloud migration path
- ✅ Verified Certbot snap installation and auto-renewal mechanism
- ✅ Confirmed Docker Compose v2 syntax and override pattern
- ✅ Verified UFW configuration for non-standard SSH ports
- ✅ Confirmed DigitalOcean Managed Database SSL requirements
- ⚠️  Droplet pricing needs verification at deploy time
- ⚠️  Nuclei version (v3.1.0) should be checked for latest release
