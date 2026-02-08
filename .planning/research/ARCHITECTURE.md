# Architecture Research: DigitalOcean Deployment Topology

**Project:** TrustEdge Audit v1.1
**Domain:** Single-droplet deployment architecture
**Researched:** 2026-02-06
**Confidence:** HIGH (verified with official Docker documentation, DigitalOcean deployment guides, and nginx best practices)

## Executive Summary

For TrustEdge Audit's DigitalOcean deployment, a **hybrid native/Docker topology** provides the optimal balance of performance, security, and operational simplicity on a single droplet. PostgreSQL runs natively for maximum database performance, application services (Rust backend and Next.js frontend) run in Docker containers managed by docker-compose, and Nuclei scanners run as ephemeral Docker containers spawned by the backend. Nginx sits in front as the SSL-terminating reverse proxy, systemd manages the docker-compose stack, and UFW hardens the firewall. This topology avoids Docker-in-Docker complexity while maintaining isolation for application services and scanner containers.

## Recommended Service Topology

### Architecture Pattern: Hybrid Native + Docker Stack

```
┌─────────────────────────────────────────────────────────────────────┐
│                        DigitalOcean Droplet                         │
│                        (Ubuntu 24.04 LTS)                          │
│                                                                     │
│  Port 443 (HTTPS)                                                  │
│       │                                                            │
│       ▼                                                            │
│  ┌────────────────────────────────────────────────────┐           │
│  │  Nginx (Native, systemd-managed)                   │           │
│  │  - SSL termination (Let's Encrypt)                 │           │
│  │  - Reverse proxy                                   │           │
│  │  - Static file serving (optional)                  │           │
│  │  Port: 443 → 80 (external to internal)            │           │
│  └─────────────┬──────────────────┬───────────────────┘           │
│                │                  │                                │
│        /:3001 (frontend)    /api:3000 (backend)                   │
│                │                  │                                │
│                ▼                  ▼                                │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │         Docker Compose Stack (systemd-managed)              │  │
│  │                                                             │  │
│  │  ┌───────────────────────┐  ┌───────────────────────┐      │  │
│  │  │  Frontend Container   │  │  Backend Container    │      │  │
│  │  │  Next.js:3001        │  │  Rust/Axum:3000      │      │  │
│  │  │  - SSR/App Router    │  │  - HTTP API          │      │  │
│  │  │  - Static assets     │  │  - Scan orchestrator │      │  │
│  │  │  - Server Actions    │  │  - PDF generator     │      │  │
│  │  └───────────────────────┘  └──────────┬────────────┘      │  │
│  │                                        │                    │  │
│  │                                        │ docker run         │  │
│  │                                        │ (via socket)       │  │
│  └────────────────────────────────────────┼────────────────────┘  │
│                                           │                       │
│       ┌───────────────────────────────────┘                       │
│       │                                                           │
│       ▼                                                           │
│  /var/run/docker.sock (bind-mounted to backend container)        │
│       │                                                           │
│       ▼                                                           │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │         Ephemeral Scanner Containers (Docker)               │ │
│  │  ┌──────────────┐  ┌──────────────┐                        │ │
│  │  │ Nuclei       │  │ testssl.sh   │  Spawned via:          │ │
│  │  │ (--rm)       │  │ (--rm)       │  docker run --rm       │ │
│  │  │ CIS-hardened │  │ CIS-hardened │  --read-only           │ │
│  │  │ 120s timeout │  │ 180s timeout │  --cap-drop all        │ │
│  │  └──────────────┘  └──────────────┘  --memory 512M          │ │
│  │                                      --pids-limit 1000     │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                   │
│       ┌────────────────────────────────────┐                     │
│       │                                    │                     │
│       ▼                                    ▼                     │
│  ┌────────────────────┐        ┌─────────────────────┐          │
│  │  PostgreSQL 16     │←───────│  pgAdmin (optional) │          │
│  │  (Native)          │        │  Docker (port 5050) │          │
│  │  Port: 5432        │        └─────────────────────┘          │
│  │  Data: /var/lib/   │                                         │
│  │  postgresql/16/    │                                         │
│  └────────────────────┘                                         │
│                                                                   │
│  File System Layout:                                             │
│  /opt/trustedge/           # Application root                   │
│  ├── docker-compose.yml    # Stack definition                   │
│  ├── .env.production       # Secrets (DATABASE_URL, etc.)       │
│  ├── backend/              # Backend source (if building on-box)│
│  ├── frontend/             # Frontend source                    │
│  ├── reports/              # PDF reports (if saved to disk)     │
│  └── logs/                 # Application logs (optional)        │
│                                                                   │
│  /etc/nginx/                                                     │
│  ├── nginx.conf            # Main config                        │
│  ├── sites-available/                                           │
│  │   └── trustedge        # Site config                        │
│  └── sites-enabled/                                             │
│      └── trustedge → ../sites-available/trustedge              │
│                                                                   │
│  /etc/systemd/system/                                            │
│  └── trustedge.service     # docker-compose systemd unit        │
│                                                                   │
│  /etc/letsencrypt/         # SSL certificates                   │
│                                                                   │
└─────────────────────────────────────────────────────────────────────┘
```

### Service Responsibilities

| Service | Type | Port(s) | Purpose | Restart Policy |
|---------|------|---------|---------|----------------|
| Nginx | Native (systemd) | 443 (external), 80 (redirect) | SSL termination, reverse proxy | systemd `Restart=always` |
| PostgreSQL | Native (systemd) | 5432 (localhost only) | Persistent data store | systemd `Restart=always` |
| Backend | Docker (docker-compose) | 3000 (internal) | HTTP API, scan orchestrator | `restart: unless-stopped` |
| Frontend | Docker (docker-compose) | 3001 (internal) | Next.js SSR + static assets | `restart: unless-stopped` |
| Nuclei | Docker (ephemeral) | N/A | Security scanner (spawned per scan) | `--rm` (auto-remove) |
| testssl.sh | Docker (ephemeral) | N/A | TLS/SSL scanner (spawned per scan) | `--rm` (auto-remove) |

## Port Allocation and Internal Routing

### External (Internet → Droplet)

| Port | Protocol | Service | Purpose |
|------|----------|---------|---------|
| 443 | HTTPS | Nginx | Public HTTPS traffic (SSL-terminated) |
| 80 | HTTP | Nginx | Redirect to HTTPS |
| 22 | SSH | OpenSSH | Remote administration (restrict via UFW to specific IPs) |

### Internal (Localhost/Docker Network)

| Port | Service | Bound To | Accessible From |
|------|---------|----------|-----------------|
| 3000 | Backend | 127.0.0.1:3000 | Nginx only (proxy_pass) |
| 3001 | Frontend | 127.0.0.1:3001 | Nginx only (proxy_pass) |
| 5432 | PostgreSQL | 127.0.0.1:5432 | Backend container (via host network or db hostname) |

**Firewall (UFW) Rules:**
```bash
ufw default deny incoming
ufw default allow outgoing
ufw allow 22/tcp      # SSH (optionally limit to specific IPs)
ufw allow 80/tcp      # HTTP (redirect to HTTPS)
ufw allow 443/tcp     # HTTPS
ufw enable
```

## Request Flow Diagrams

### Frontend Request Flow

```
User Browser
    │ HTTPS (443)
    ▼
Nginx (SSL termination)
    │ HTTP (3001)
    ▼
Frontend Container (Next.js)
    │ Server Action (POST /api/...)
    ▼
Backend Container (Axum)
    │ SQL query
    ▼
PostgreSQL (native)
    │ Response
    ▼
Backend → Frontend → Nginx → Browser
```

### API Request Flow (Direct)

```
User Browser / API Client
    │ HTTPS POST /api/scans (443)
    ▼
Nginx (SSL termination)
    │ HTTP proxy_pass (3000)
    ▼
Backend Container (Axum)
    │ 1. Create scan record
    ▼
PostgreSQL (INSERT into scans)
    │ 2. Spawn scan job
    ▼
Backend spawns tokio task
    │ 3. Execute docker run
    ▼
Docker Socket (/var/run/docker.sock)
    │ 4. Create & run Nuclei container
    ▼
Nuclei Container (ephemeral)
    │ 5. Scan target URL
    │ 6. Output JSON to stdout
    │ 7. Container exits, auto-removed (--rm)
    ▼
Backend captures stdout
    │ 8. Parse JSON findings
    ▼
PostgreSQL (INSERT into findings)
    │ 9. Update scan status = 'completed'
    ▼
Backend → JSON response → Nginx → Browser
```

### WebSocket Consideration (Future)

Current architecture uses polling (`GET /api/scans/:id` every 2 seconds). If real-time scan progress is added later, Nginx must be configured to proxy WebSocket connections:

```nginx
location /api/ws {
    proxy_pass http://127.0.0.1:3000;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
}
```

## Docker Socket Access Pattern

### The Challenge

TrustEdge Audit's backend must spawn Docker containers (Nuclei, testssl.sh) for security scanning. This requires access to the Docker daemon socket (`/var/run/docker.sock`).

**Three approaches exist:**
1. **Docker-in-Docker (DinD):** Run Docker daemon inside backend container (complex, security risk, high overhead)
2. **Socket bind-mount:** Mount host's Docker socket into backend container (simple, security risk if container compromised)
3. **Native backend:** Run backend natively, call Docker directly (no isolation for backend, more complex deployment)

### Recommended: Socket Bind-Mount (Calculated Risk)

**Why this choice:**
- Backend container is trusted code (we control it)
- Simplifies deployment (docker-compose manages everything)
- Scanner containers are still isolated (CIS-hardened, ephemeral)
- Risk is acceptable for MVP (single-tenant, no untrusted code in backend)

**Implementation in docker-compose.yml:**

```yaml
services:
  backend:
    build: .
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro  # Read-only mount
    user: "1000:999"  # Non-root user, docker group (GID 999)
    environment:
      DATABASE_URL: postgres://trustedge:${DB_PASSWORD}@host.docker.internal:5432/trustedge_prod
```

**Security Mitigations:**

1. **Read-only socket mount:** Backend can spawn containers but cannot modify Docker daemon config
2. **Non-root user:** Backend runs as UID 1000 (non-root), member of docker group (GID 999)
3. **CIS-hardened scanner containers:** All spawned containers use 8 security flags:
   - `--rm` (auto-remove after exit)
   - `--read-only` (no filesystem writes)
   - `--cap-drop all` (drop all Linux capabilities)
   - `--user 1000:1000` (non-root inside container)
   - `--memory 512M` (memory limit)
   - `--pids-limit 1000` (process limit)
   - `--cpu-shares 512` (CPU throttling)
   - `--no-new-privileges` (prevent privilege escalation)
4. **Timeout enforcement:** All scanner executions have hard timeouts (120s Nuclei, 180s testssl)
5. **Network isolation:** Scanners only have outbound network access (no host network mode)

**Security Risk Assessment:**

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Backend container compromised → attacker gains Docker socket access | LOW (trusted code, no user input executed) | HIGH (full host control) | Non-root user, read-only mount, minimal attack surface in backend |
| Scanner container breakout | VERY LOW (hardened, ephemeral, no privileges) | MEDIUM (could access host filesystem) | CIS hardening, timeout enforcement, auto-removal |
| SSRF attack via target URL → scan internal services | MEDIUM (user-controlled input) | MEDIUM (could probe internal network) | URL validation, SSRF protection in backend |

**Alternative for Higher Security (Future):**
If backend is ever compromised, consider:
- Docker socket proxy (e.g., [Tecnativa/docker-socket-proxy](https://github.com/Tecnativa/docker-socket-proxy)) to restrict API surface
- Rootless Docker mode on host
- Separate scanner orchestration service (backend → message queue → scanner service → Docker)

## File System Layout

### Application Directory: `/opt/trustedge/`

Following Linux FHS conventions, third-party applications live in `/opt/`.

```
/opt/trustedge/
├── docker-compose.yml          # Stack definition
├── docker-compose.prod.yml     # Production overrides (optional)
├── .env.production             # Environment variables (SECRETS!)
│   # DATABASE_URL=postgres://trustedge:STRONG_PW@localhost:5432/trustedge_prod
│   # RESEND_API_KEY=re_...
│   # STRIPE_SECRET_KEY=sk_...
│   # TRUSTEDGE_BASE_URL=https://trustedge.audit
├── backend/
│   ├── Dockerfile
│   ├── Cargo.toml
│   ├── Cargo.lock
│   ├── src/
│   └── migrations/             # Database migrations (SQLx)
├── frontend/
│   ├── Dockerfile
│   ├── package.json
│   ├── package-lock.json
│   ├── next.config.ts
│   ├── app/                    # Next.js App Router
│   └── public/                 # Static assets
├── reports/                    # PDF reports (if saved to disk)
│   # NOTE: v1.0 generates PDFs in-memory for email, but may save here on failure
└── logs/                       # Application logs (optional)
    ├── backend.log
    └── frontend.log
```

**Permissions:**
```bash
sudo chown -R 1000:1000 /opt/trustedge
sudo chmod 700 /opt/trustedge/.env.production  # Secrets file
sudo chmod 755 /opt/trustedge
```

### Nginx Configuration: `/etc/nginx/`

```
/etc/nginx/
├── nginx.conf                  # Main config (usually untouched)
├── sites-available/
│   └── trustedge               # Site-specific config
└── sites-enabled/
    └── trustedge → ../sites-available/trustedge  # Symlink
```

**Example `/etc/nginx/sites-available/trustedge`:**

```nginx
# Redirect HTTP → HTTPS
server {
    listen 80;
    listen [::]:80;
    server_name trustedge.audit www.trustedge.audit;
    return 301 https://$server_name$request_uri;
}

# Main HTTPS server
server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name trustedge.audit www.trustedge.audit;

    # SSL certificates (Let's Encrypt via certbot)
    ssl_certificate /etc/letsencrypt/live/trustedge.audit/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/trustedge.audit/privkey.pem;

    # SSL configuration (Mozilla Intermediate)
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers 'ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:...';
    ssl_prefer_server_ciphers off;

    # Security headers
    add_header Strict-Transport-Security "max-age=63072000; includeSubDomains; preload" always;
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # Backend API
    location /api/ {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Timeouts (scans take 30s-3min)
        proxy_connect_timeout 10s;
        proxy_send_timeout 300s;
        proxy_read_timeout 300s;
    }

    # Frontend (everything else)
    location / {
        proxy_pass http://127.0.0.1:3001;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Timeouts
        proxy_connect_timeout 10s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # Optional: Serve static assets directly (if frontend exports static build)
    # location /_next/static/ {
    #     alias /opt/trustedge/frontend/.next/static/;
    #     expires 1y;
    #     add_header Cache-Control "public, immutable";
    # }
}
```

### Systemd Service: `/etc/systemd/system/trustedge.service`

```ini
[Unit]
Description=TrustEdge Audit Application Stack
Requires=docker.service postgresql.service
After=docker.service postgresql.service network-online.target
Wants=network-online.target

[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory=/opt/trustedge
ExecStart=/usr/bin/docker-compose up -d
ExecStop=/usr/bin/docker-compose down
TimeoutStartSec=300
TimeoutStopSec=15
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
```

**Key settings:**
- `Requires=postgresql.service`: PostgreSQL must be running before stack starts
- `Type=oneshot` + `RemainAfterExit=yes`: Service runs docker-compose command once, then systemd tracks it as active
- `Restart=on-failure`: Automatically restart if docker-compose fails
- `TimeoutStartSec=300`: Allow 5 minutes for pulling images and starting containers

### PostgreSQL Data: `/var/lib/postgresql/16/`

Native PostgreSQL installation stores data in standard location. Managed by PostgreSQL's own systemd service.

```
/var/lib/postgresql/16/main/
├── base/                       # Database files
├── pg_wal/                     # Write-ahead log
├── pg_stat/                    # Statistics
└── ...
```

**Backup strategy (recommended):**
```bash
# Daily backup via cron
0 2 * * * pg_dump -U postgres trustedge_prod | gzip > /opt/trustedge/backups/db-$(date +\%Y\%m\%d).sql.gz
```

### SSL Certificates: `/etc/letsencrypt/`

Managed by Certbot (Let's Encrypt client).

```
/etc/letsencrypt/
├── live/
│   └── trustedge.audit/
│       ├── fullchain.pem       # Certificate + chain
│       ├── privkey.pem         # Private key
│       ├── cert.pem            # Certificate only
│       └── chain.pem           # CA chain
└── renewal/
    └── trustedge.audit.conf    # Auto-renewal config
```

Certbot systemd timer (`certbot.timer`) runs twice daily to check for renewals. Nginx automatically reloads on certificate renewal.

## Deployment Build Order

Phases should be structured to address dependencies in this order:

### Phase 1: Base Infrastructure Setup
**What:** Provision droplet, harden OS, install prerequisites
**Tasks:**
1. Create DigitalOcean droplet (Ubuntu 24.04 LTS, 2 vCPU, 2GB RAM minimum)
2. SSH key authentication, disable password auth
3. Install: `docker`, `docker-compose`, `postgresql-16`, `nginx`, `certbot`, `ufw`
4. Enable UFW with ports 22, 80, 443
5. Configure unattended-upgrades for security patches

**Verification:** `docker --version`, `psql --version`, `nginx -v`, `ufw status`

### Phase 2: Database Setup
**What:** Initialize PostgreSQL, create database and user
**Tasks:**
1. Start PostgreSQL: `sudo systemctl enable --now postgresql`
2. Create database user and database:
   ```sql
   CREATE USER trustedge WITH PASSWORD 'STRONG_PASSWORD';
   CREATE DATABASE trustedge_prod OWNER trustedge;
   ```
3. Configure `pg_hba.conf` to allow localhost connections
4. Test connection: `psql -U trustedge -d trustedge_prod -h localhost`

**Verification:** Backend can connect via `DATABASE_URL`

### Phase 3: Application Deployment
**What:** Deploy backend and frontend as Docker containers
**Tasks:**
1. Create `/opt/trustedge/` directory structure
2. Copy `docker-compose.yml`, `backend/`, `frontend/` to droplet (via `rsync` or `git pull`)
3. Create `.env.production` with secrets:
   - `DATABASE_URL=postgres://trustedge:PW@localhost:5432/trustedge_prod`
   - `RESEND_API_KEY=...`
   - `STRIPE_SECRET_KEY=...`
   - `TRUSTEDGE_BASE_URL=https://trustedge.audit`
4. Build and start containers:
   ```bash
   cd /opt/trustedge
   docker-compose build
   docker-compose up -d
   ```
5. Run database migrations (SQLx):
   ```bash
   docker-compose exec backend trustedge_audit migrate
   ```

**Verification:** `curl http://localhost:3000/health`, `curl http://localhost:3001/`

### Phase 4: Nginx Reverse Proxy
**What:** Configure Nginx to proxy requests to backend/frontend
**Tasks:**
1. Create `/etc/nginx/sites-available/trustedge` (see config above)
2. Enable site: `sudo ln -s /etc/nginx/sites-available/trustedge /etc/nginx/sites-enabled/`
3. Disable default site: `sudo rm /etc/nginx/sites-enabled/default`
4. Test config: `sudo nginx -t`
5. Start Nginx: `sudo systemctl enable --now nginx`

**Verification:** `curl http://DROPLET_IP/` (should return frontend HTML)

### Phase 5: SSL Setup
**What:** Install Let's Encrypt SSL certificate
**Tasks:**
1. Point DNS A record `trustedge.audit` to droplet IP (wait for propagation)
2. Install certificate:
   ```bash
   sudo certbot --nginx -d trustedge.audit -d www.trustedge.audit
   ```
3. Certbot automatically updates Nginx config with SSL settings
4. Test auto-renewal: `sudo certbot renew --dry-run`

**Verification:** `https://trustedge.audit/` loads with valid certificate

### Phase 6: Systemd Service Management
**What:** Manage docker-compose stack via systemd
**Tasks:**
1. Create `/etc/systemd/system/trustedge.service` (see config above)
2. Reload systemd: `sudo systemctl daemon-reload`
3. Enable service: `sudo systemctl enable trustedge`
4. Test manual start/stop:
   ```bash
   sudo systemctl stop trustedge
   sudo systemctl start trustedge
   sudo systemctl status trustedge
   ```
5. Reboot droplet, verify services auto-start

**Verification:** After reboot, `https://trustedge.audit/` is accessible

### Phase 7: Monitoring and Logging
**What:** Set up basic monitoring and log aggregation
**Tasks:**
1. Configure docker-compose logging driver (json-file with rotation):
   ```yaml
   services:
     backend:
       logging:
         driver: "json-file"
         options:
           max-size: "10m"
           max-file: "3"
   ```
2. Set up log rotation for Nginx: `/etc/logrotate.d/nginx`
3. Optional: Install monitoring (Netdata, Prometheus, or DigitalOcean Monitoring Agent)
4. Set up alerting for disk space, memory, CPU

**Verification:** Logs are rotating, disk usage is stable

## Architectural Patterns

### Pattern 1: Database-as-Queue (Scan Orchestration)

**What:** Use PostgreSQL as job queue for scan orchestration instead of Redis/RabbitMQ

**When to use:** MVP with low-to-medium scan volume (<1000 scans/day)

**Trade-offs:**
- **Pros:** Simpler deployment (no additional service), ACID guarantees, easy debugging (SQL queries)
- **Cons:** Not optimized for high-throughput queues, polling overhead, potential lock contention at scale

**Implementation:**
```rust
// Backend polls for pending scan jobs
loop {
    let jobs = sqlx::query!("
        SELECT id FROM scan_jobs
        WHERE status = 'pending'
        ORDER BY created_at ASC
        LIMIT 5
        FOR UPDATE SKIP LOCKED
    ")
    .fetch_all(&pool).await?;

    for job in jobs {
        tokio::spawn(execute_scan(job.id, pool.clone()));
    }

    tokio::time::sleep(Duration::from_secs(2)).await;
}
```

**When to migrate:** When scan volume exceeds 1000/day or polling causes database CPU spikes

### Pattern 2: In-Process Worker Pool (Scan Concurrency)

**What:** Use tokio semaphore to limit concurrent scan executions within single backend process

**When to use:** Single-server deployment with predictable resource limits

**Trade-offs:**
- **Pros:** Simple, efficient (no inter-process communication), easy to tune (semaphore count)
- **Cons:** Single point of failure (backend crash = all scans fail), no horizontal scaling

**Implementation:**
```rust
// Limit to 5 concurrent scans
static SCAN_SEMAPHORE: Semaphore = Semaphore::const_new(5);

async fn execute_scan(scan_id: Uuid, pool: PgPool) -> Result<()> {
    let _permit = SCAN_SEMAPHORE.acquire().await?;
    // Spawn scanner containers, collect findings
    // Permit is released when function exits
}
```

**When to migrate:** When horizontal scaling is needed (multiple backend instances across droplets)

### Pattern 3: Ephemeral Scanner Containers (Isolation)

**What:** Spawn scanner containers per scan with `--rm` flag, auto-removed after execution

**When to use:** Always (for security and resource efficiency)

**Trade-offs:**
- **Pros:** Perfect isolation per scan, no container accumulation, no state leakage between scans
- **Cons:** Image pull overhead (mitigated by pre-pulling images), slight startup latency (200-500ms)

**Implementation:**
```rust
let args = vec![
    "run", "--rm",           // Auto-remove after exit
    "--read-only",           // No filesystem writes
    "--cap-drop", "all",     // Drop all capabilities
    "--user", "1000:1000",   // Non-root user
    "--memory", "512M",      // Memory limit
    "--pids-limit", "1000",  // Process limit
    "--cpu-shares", "512",   // CPU throttling
    "--no-new-privileges",   // Prevent privilege escalation
    "projectdiscovery/nuclei:latest",
    "-u", target_url,
    "-jsonl", "-silent",
];
Command::new("docker").args(args).output().await?
```

**When to optimize:** Pre-pull scanner images during deployment, consider keeping 1-2 warm containers

## Anti-Patterns

### Anti-Pattern 1: Docker-in-Docker (DinD)

**What people do:** Run Docker daemon inside backend container to spawn scanners

**Why it's wrong:**
- Requires privileged mode (`--privileged`), full security bypass
- Complex networking (nested NAT)
- High overhead (daemon inside daemon)
- Difficult debugging (logs nested two levels deep)

**Do this instead:** Bind-mount Docker socket from host (as recommended above)

### Anti-Pattern 2: Exposing PostgreSQL Port Publicly

**What people do:** Bind PostgreSQL to `0.0.0.0:5432` for "convenience"

**Why it's wrong:**
- Direct exposure to internet attacks (brute force, SQL injection via psql)
- No need for external access (backend is on same host)
- Violates least-privilege principle

**Do this instead:** Bind PostgreSQL to `127.0.0.1:5432`, firewall blocks external access

### Anti-Pattern 3: Running Backend/Frontend as Root

**What people do:** Deploy Docker containers with `user: root` for "simplicity"

**Why it's wrong:**
- Container breakout = root on host
- Violates least-privilege principle
- Unnecessary risk (applications don't need root)

**Do this instead:** Run as non-root user (UID 1000) in containers:
```dockerfile
# In Dockerfile
RUN useradd -m -u 1000 appuser
USER appuser
```

### Anti-Pattern 4: Storing Secrets in Git

**What people do:** Commit `.env` file with `DATABASE_URL`, `STRIPE_SECRET_KEY` to Git

**Why it's wrong:**
- Secrets leaked in Git history forever (even if file is deleted later)
- Public repos = instant compromise
- Private repos still vulnerable (any collaborator can see)

**Do this instead:**
- `.env.production` only on server, never committed
- Use `.env.example` with placeholder values in Git
- Consider secret management service (HashiCorp Vault, DigitalOcean Secrets) for larger teams

### Anti-Pattern 5: No Resource Limits on Scanner Containers

**What people do:** Spawn scanner containers without `--memory`, `--cpu-shares`, `--pids-limit`

**Why it's wrong:**
- Scanner gone rogue can consume all host resources
- OOM killer may kill PostgreSQL or backend instead
- DoS via malicious target URL triggering resource-intensive scan

**Do this instead:** Always set resource limits (as shown in Pattern 3)

## Scaling Considerations

### 0-100 Scans/Day (MVP: Single Droplet)

**Current architecture is optimal:**
- 2 vCPU, 2GB RAM DigitalOcean droplet ($18/month)
- PostgreSQL handles <10 concurrent scans easily
- 5 concurrent scanner containers fit in memory
- No bottlenecks expected

**Estimated costs:**
- Droplet: $18/month
- Bandwidth: ~10GB/month (free tier covers)
- **Total: ~$18/month**

### 100-1000 Scans/Day (Scale Up: Larger Droplet)

**Bottleneck:** CPU (scanner containers are CPU-intensive)

**Solution:** Upgrade droplet to 4 vCPU, 8GB RAM ($72/month)
- Increase `SCAN_SEMAPHORE` to 10 concurrent scans
- Add connection pooling to PostgreSQL (pgbouncer)
- No architecture changes required

**Estimated costs:**
- Droplet: $72/month
- **Total: ~$72/month**

### 1000-10000 Scans/Day (Scale Out: Multi-Droplet)

**Bottleneck:** Single backend process, database connection contention

**Solution:** Split into multiple droplets:
1. **Database droplet:** PostgreSQL + pgbouncer (dedicated)
2. **Backend droplet(s):** Multiple backend instances, round-robin load balancing
3. **Frontend droplet:** Nginx + Next.js (can colocate with load balancer)
4. **Load balancer:** DigitalOcean Load Balancer ($12/month)

**Architecture changes:**
- Migrate from database-as-queue to Redis-based job queue
- Add distributed locking (Redis) to prevent duplicate scan execution
- Horizontal scaling: 2-4 backend droplets behind load balancer

**Estimated costs:**
- DB droplet: $72/month (4 vCPU, 8GB)
- Backend droplets: $144/month (2 × $72)
- Frontend + LB droplet: $18/month
- Load Balancer: $12/month
- **Total: ~$246/month**

### 10000+ Scans/Day (Kubernetes or Managed Services)

**At this scale, consider:**
- DigitalOcean Kubernetes (DOKS) for orchestration
- Managed PostgreSQL (DigitalOcean Managed Database)
- Managed Redis (DigitalOcean Managed Cache)
- Object storage (DigitalOcean Spaces) for PDF reports
- CDN (Cloudflare) for static assets

**Architecture becomes microservices:**
- API gateway (Nginx Ingress or Traefik)
- Backend pods (auto-scaling 5-20 replicas)
- Scanner service (separate pods, auto-scaling)
- Background job processor (separate service)

## Integration Points

### External Services

| Service | Integration Pattern | Configuration | Notes |
|---------|---------------------|---------------|-------|
| PostgreSQL | Direct connection via `sqlx` | `DATABASE_URL` env var | Native on host, backend connects via localhost |
| Docker | Socket bind-mount + `Command::new("docker")` | `/var/run/docker.sock` mounted | Backend spawns containers via host Docker |
| Resend (Email) | HTTP API via `reqwest` | `RESEND_API_KEY` env var | Used for PDF report delivery |
| Stripe (Payments) | HTTP API via `stripe-rust` | `STRIPE_SECRET_KEY` env var | Webhook endpoint at `/api/payments/webhook` |
| SSL Labs API | HTTP API via `reqwest` | No auth, rate-limited | Used for TLS grading (testssl.sh alternative) |

### Internal Boundaries

| Boundary | Communication | Protocol | Notes |
|----------|---------------|----------|-------|
| Frontend ↔ Backend | HTTP API | REST (JSON) | `NEXT_PUBLIC_BACKEND_URL=http://localhost:3000` |
| Backend ↔ PostgreSQL | PostgreSQL wire protocol | SQL via `sqlx` | Connection pooling (max 20 connections) |
| Backend ↔ Docker | Unix socket | Docker API | Spawn containers, stream logs |
| Nginx ↔ Backend | HTTP reverse proxy | `proxy_pass` | Timeouts: 300s for scans |
| Nginx ↔ Frontend | HTTP reverse proxy | `proxy_pass` | Standard timeouts |

## Security Hardening Checklist

- [ ] UFW enabled with minimal ports (22, 80, 443)
- [ ] SSH password authentication disabled, key-only
- [ ] PostgreSQL bound to localhost only (`listen_addresses = 'localhost'`)
- [ ] Docker socket mounted read-only to backend container
- [ ] Backend/frontend containers run as non-root user (UID 1000)
- [ ] Scanner containers CIS-hardened (8 security flags)
- [ ] Secrets in `.env.production`, never committed to Git
- [ ] Nginx security headers (HSTS, X-Frame-Options, CSP)
- [ ] SSL/TLS A+ grade (Mozilla Intermediate profile)
- [ ] Unattended security updates enabled (`unattended-upgrades`)
- [ ] Log rotation configured (Nginx, Docker, PostgreSQL)
- [ ] Rate limiting on API endpoints (backend or Nginx level)
- [ ] SSRF protection in backend (URL validation, blocklist RFC 1918)
- [ ] Fail2ban or similar for SSH brute force protection (optional)

## Performance Optimization Checklist

- [ ] PostgreSQL connection pooling (max 20 connections)
- [ ] PostgreSQL indexes on frequently queried columns (`scans.id`, `findings.scan_id`)
- [ ] Docker images pre-pulled (`docker-compose pull` during deployment)
- [ ] Nginx gzip compression enabled for text assets
- [ ] Nginx caching for static assets (if Next.js static export used)
- [ ] Semaphore tuned for concurrent scans (start with 5, increase if CPU allows)
- [ ] Scanner container memory limits tuned (512M is conservative, may decrease)
- [ ] Database vacuuming scheduled (PostgreSQL autovacuum enabled by default)

## Comparison Matrix: Deployment Topologies

| Topology | PostgreSQL | App Containers | Scanner Execution | Complexity | Performance | Security |
|----------|------------|----------------|-------------------|------------|-------------|----------|
| **Hybrid (Recommended)** | Native | Docker | Docker (socket mount) | Medium | High (DB native) | Good (CIS-hardened scanners) |
| All Docker (docker-compose) | Docker | Docker | Docker-in-Docker (DinD) | Low | Medium (DB overhead) | Poor (requires privileged) |
| All Native | Native | Native binaries | Docker (native Docker CLI) | High | Highest (no overhead) | Best (no socket mount risk) |
| Kubernetes (overkill for MVP) | Managed DB | K8s Pods | K8s Jobs | Very High | High (auto-scaling) | Best (RBAC, network policies) |

**Recommendation:** **Hybrid topology** balances all factors for single-droplet MVP.

## Sources

### Docker Security and Socket Access
- [Docker Security - OWASP Cheat Sheet Series](https://cheatsheetseries.owasp.org/cheatsheets/Docker_Security_Cheat_Sheet.html) — Authoritative security best practices
- [Protect the Docker daemon socket | Docker Docs](https://docs.docker.com/engine/security/protect-access/) — Official guidance on socket access
- [Docker Security Best Practices | Better Stack Community](https://betterstack.com/community/guides/scaling-docker/docker-security-best-practices/) — Comprehensive hardening guide
- [Bind mounts | Docker Docs](https://docs.docker.com/engine/storage/bind-mounts/) — Official bind mount documentation

### DigitalOcean Deployment
- [Deploying multiple dockerized apps to a single DigitalOcean droplet using docker-compose contexts | Daniel Wachtel's Blog](https://danielwachtel.com/devops/deploying-multiple-dockerized-apps-digitalocean-docker-compose-contexts) — Multi-app deployment patterns
- [Deploying Your Web App to DigitalOcean with Docker Compose | Medirelay](https://medirelay.com/blog/127-deploy-webapp-digitalocean-docker-compose/) — Production deployment guide
- [How To Use the Docker 1-Click Install on DigitalOcean](https://www.digitalocean.com/community/tutorials/how-to-use-the-docker-1-click-install-on-digitalocean) — Droplet provisioning

### Nginx Reverse Proxy
- [How to Nginx Reverse Proxy with Docker Compose - Gcore](https://gcore.com/learning/reverse-proxy-with-docker-compose) — Nginx + Docker patterns
- [How to use Nginx with Docker Compose effectively with examples](https://geshan.com.np/blog/2024/03/nginx-docker-compose/) — Configuration examples
- [Deploying a Next.js App on HTTPS with Docker Using NGINX as a Reverse Proxy - Collabnix](https://collabnix.com/deploying-a-next-js-app-on-https-with-docker-using-nginx-as-a-reverse-proxy/) — Next.js-specific patterns

### PostgreSQL Performance
- [Docker vs Native PostgreSQL: Impact on Database Performance](https://secnep.com/docker-vs-native-postgresql-performance-comparison/) — Performance benchmarks
- [Benchmark PostgreSQL on all three systems: Docker versus native | ITNEXT](https://itnext.io/benchmark-postgresql-docker-versus-native-2dde6b5a8552) — Detailed comparison

### Systemd and Docker Compose
- [Running Docker Compose as a systemd Service: A Comprehensive Guide](https://bootvar.com/systemd-service-for-docker-compose/) — Systemd integration patterns
- [Start containers automatically | Docker Docs](https://docs.docker.com/engine/containers/start-containers-automatically/) — Restart policies
- [Docker compose as a systemd unit](https://gist.github.com/mosquito/b23e1c1e5723a7fd9e6568e5cf91180f) — Example service file

### Linux Filesystem Conventions
- [Understanding and Utilizing the Linux `/opt` Folder — linuxvox.com](https://linuxvox.com/blog/linux-opt-folder/) — FHS /opt best practices
- [What Is /Opt In Linux? (The Ultimate Guide) | Unixmen](https://www.unixmen.com/what-is-opt-in-linux-the-ultimate-guide/) — Directory structure conventions

---

*Architecture research for: TrustEdge Audit DigitalOcean Deployment*
*Researched: 2026-02-06*
*Confidence: HIGH (verified with official documentation and production deployment guides)*
